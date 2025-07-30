use anyhow::{Result, anyhow};
use chrono::Utc;
use log::info;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

use crate::types::*;

/// Processes a deserialized request, updates the performance metrics,
/// and returns a formed response object.
///
/// # Parameters:
/// - `request`: The deseriazized request to process.
/// - `metrics`: A shared thread-safe pointer to the global `Metrics` instance.
///
/// # Returns:
/// A formed `Response` object representing either a successful result or an error.
pub async fn form_response(request: Request, metrics: Arc<Mutex<Metrics>>) -> Response {
    let mut start = None;
    if !matches!(request.command, Command::Batch(_)) {
        start = Some(std::time::Instant::now());
    }

    let uuid = request.request_id;
    let command_kind = request.command.kind();
    let response = match process_command(request, metrics.clone()).await {
        Ok(v) => Response::Ok(OkResponse {
            request_id: uuid,
            status: Status::Ok,
            response: v,
        }),
        Err(e) => Response::Err(ErrorResponse {
            request_id: Some(uuid),
            status: Status::Error,
            error: e.to_string(),
        }),
    };

    if let Some(s) = start {
        let duration = s.elapsed().as_micros() as f64 / 1000.0;
        let count = {
            let mut guard = metrics.lock().unwrap();
            guard.update(command_kind, duration);
            *guard.command_counts.get(&command_kind).unwrap()
        };
        info!(
            "Processed command {:?} in {}ms, total number of commands of this type processed: {}",
            command_kind, duration, count
        );
    };
    response
}

async fn process_command(request: Request, metrics: Arc<Mutex<Metrics>>) -> Result<Value> {
    match request.command {
        Command::Ping => Ok(json!("pong")),
        Command::Echo(payload) => Ok(payload),
        Command::Time => {
            let time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            Ok(json!({"time": time}))
        }
        Command::Calculate { operation, a, b } => process_command_calculate(operation, a, b).await,
        Command::Batch(batch) => {
            let mut result: Vec<Response> = Vec::new();
            for item in batch {
                result.push(Box::pin(form_response(item, metrics.clone())).await);
            }
            Ok(json!(result))
        }
    }
}

async fn process_command_calculate(operation: Operation, a: f64, b: f64) -> Result<Value> {
    let result = match operation {
        Operation::Add => a + b,
        Operation::Subtract => a - b,
        Operation::Multiply => a * b,
        Operation::Divide => {
            if b == 0.0 {
                return Err(anyhow!("division by zero"));
            }
            a / b
        }
    };

    Ok(json!({"result": result}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use uuid::Uuid;

    fn build_metrics() -> Arc<Mutex<Metrics>> {
        Arc::new(Mutex::new(Metrics::default()))
    }

    fn build_request(command: Command) -> Request {
        Request {
            request_id: Uuid::new_v4(),
            command: command,
        }
    }

    #[tokio::test]
    async fn test_command_ping() {
        let metrics = build_metrics();
        let req = build_request(Command::Ping);
        let resp = form_response(req, metrics.clone()).await;
        match resp {
            Response::Ok(v) => {
                assert!(matches!(v.status, Status::Ok));
                assert_eq!(v.response, json!("pong"));
            }
            Response::Err(_) => panic!("Expected OK response"),
        }
    }

    #[tokio::test]
    async fn test_command_time() {
        use chrono::DateTime;

        let metrics = build_metrics();
        let time = Utc::now()
            .with_nanosecond(0)
            .expect("This shouldn't ever panic");
        let req = build_request(Command::Time);
        let resp = form_response(req, metrics.clone()).await;
        match resp {
            Response::Ok(v) => {
                assert!(matches!(v.status, Status::Ok));
                let resp_str = v
                    .response
                    .get("time")
                    .and_then(|v| v.as_str())
                    .expect("Missing `time` field in the response");
                let parsed = DateTime::parse_from_rfc3339(resp_str)
                    .expect("Invalid format")
                    .with_timezone(&Utc);
                println!("{}", parsed);
                println!("{}", time);
                assert!((parsed - time).as_seconds_f32() >= 0.0);
                assert!((parsed - time).as_seconds_f32() < 2.0);
            }
            Response::Err(_) => panic!("Expected OK response"),
        }
    }

    #[tokio::test]
    async fn test_command_echo() {
        let metrics = build_metrics();

        let req = build_request(Command::Echo(json!("hello")));
        let resp = form_response(req, metrics.clone()).await;
        match resp {
            Response::Ok(v) => {
                assert!(matches!(v.status, Status::Ok));
                assert_eq!(v.response, json!("hello"));
            }
            Response::Err(_) => panic!("Expected OK response"),
        }
    }

    #[tokio::test]
    async fn test_command_calculate() {
        #[rustfmt::skip]
        let test_data = Vec::from([
            (Operation::Add, 0.1, 0.2, 0.1 + 0.2),
            (Operation::Subtract, 21.0, 9.0, 21.0 - 9.0),
            (Operation::Multiply, 6.0, -8.0, 6.0 * -8.0),
            (Operation::Divide, 22.0, 7.0, 22.0 / 7.0),
        ]);

        let metrics = build_metrics();

        for item in test_data {
            let req = build_request(Command::Calculate {
                operation: item.0,
                a: item.1,
                b: item.2,
            });
            let resp = form_response(req, metrics.clone()).await;
            match resp {
                Response::Ok(v) => {
                    assert!(matches!(v.status, Status::Ok));
                    assert_eq!(v.response, json!({"result": item.3}));
                }
                Response::Err(_) => panic!("Expected OK response"),
            }
        }

        let req = build_request(Command::Calculate {
            operation: Operation::Divide,
            a: 5.0,
            b: 0.0,
        });
        let resp = form_response(req, metrics.clone()).await;
        match resp {
            Response::Err(e) => assert!(matches!(e.status, Status::Error)),
            Response::Ok(_) => panic!("Expected Error response"),
        }
    }

    #[tokio::test]
    async fn test_command_batch() {
        use serde::Deserialize;
        let metrics = build_metrics();

        let test_requests = Vec::from([
            build_request(Command::Ping),
            build_request(Command::Echo(json!({"key": "value"}))),
            build_request(Command::Calculate {
                operation: Operation::Divide,
                a: 3.5,
                b: -1.05,
            }),
        ]);
        let expected_responses = [
            json!("pong"),
            json!({"key": "value"}),
            json!({"result": 3.5 / -1.05}),
        ];

        let req = build_request(Command::Batch(test_requests.clone()));
        let resp = form_response(req, metrics.clone()).await;

        match resp {
            Response::Err(_) => panic!("Expected OK response"),
            Response::Ok(resp) => {
                for (i, item) in resp.response.as_array().unwrap().iter().enumerate() {
                    let content = OkResponse::deserialize(item).unwrap();
                    assert_eq!(content.request_id, test_requests[i].request_id);
                    assert!(matches!(content.status, Status::Ok));
                    assert_eq!(content.response, expected_responses[i]);
                }
            }
        }
    }
}
