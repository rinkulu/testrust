use anyhow::{Result, anyhow};
use chrono::Utc;
use log::debug;
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
    if request.command != Command::Batch {
        start = Some(std::time::Instant::now());
    }

    let uuid = request.request_id;
    let command = request.command.clone();
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
            guard.update(&command, duration);
            *guard.command_counts.get(&command).unwrap()
        };
        debug!(
            "Processed command {} in {duration}ms, total number of commands of this type processed: {count}",
            serde_plain::to_string(&command).unwrap()
        );
    };
    response
}

async fn process_command(request: Request, metrics: Arc<Mutex<Metrics>>) -> Result<Value> {
    match request.command {
        Command::Ping => Ok(json!("pong")),
        Command::Echo => Ok(request.payload.unwrap_or_default()),
        Command::Time => {
            let time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            Ok(json!({"time": time}))
        }
        Command::Calculate => process_command_calculate(request).await,
        Command::Batch => {
            let batch: Vec<Request> = serde_json::from_value(request.payload.unwrap())?;
            let mut result: Vec<Response> = Vec::new();
            for item in batch {
                result.push(Box::pin(form_response(item, metrics.clone())).await);
            }
            Ok(json!(result))
        }
    }
}

async fn process_command_calculate(req: Request) -> Result<Value> {
    let payload = match req.payload {
        Some(v) => serde_json::from_value::<CalculationPayload>(v)?,
        None => return Err(anyhow!("missing `payload` field")),
    };

    let result = match payload.operation {
        Operation::Add => payload.a + payload.b,
        Operation::Subtract => payload.a - payload.b,
        Operation::Multiply => payload.a * payload.b,
        Operation::Divide => {
            if payload.b == 0.0 {
                return Err(anyhow!("division by zero"));
            }
            payload.a / payload.b
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

    fn build_request(command: Command, payload: Option<Value>) -> Request {
        Request {
            request_id: Uuid::new_v4(),
            command: command,
            payload: payload,
        }
    }

    #[tokio::test]
    async fn test_command_ping() {
        let metrics = build_metrics();
        let req = build_request(Command::Ping, None);
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
        let req = build_request(Command::Time, None);
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

        let req = build_request(Command::Echo, Some(json!("hello")));
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
            (json!({"operation": "add", "a": 0.1, "b": 0.2}), 0.1 + 0.2),
            (json!({"operation": "subtract", "a": 21, "b": 9}), 12 as f64),
            (json!({"operation": "multiply", "a": 6.0, "b": -8}), 6.0 * -8 as f64),
            (json!({"operation": "divide", "a": 22, "b": 7}), 22 as f64 / 7 as f64),
        ]);

        let metrics = build_metrics();

        for item in test_data {
            let req = build_request(Command::Calculate, Some(json!(item.0)));
            let resp = form_response(req, metrics.clone()).await;
            match resp {
                Response::Ok(v) => {
                    assert!(matches!(v.status, Status::Ok));
                    assert_eq!(v.response, json!({"result": item.1}));
                }
                Response::Err(_) => panic!("Expected OK response"),
            }
        }

        let req = build_request(
            Command::Calculate,
            Some(json!({"operation": "divide", "a": 5, "b": 0})),
        );
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

        let test_data = Vec::from([
            (build_request(Command::Ping, None), json!("pong")),
            (
                build_request(Command::Echo, Some(json!({"key": "value"}))),
                json!({"key": "value"}),
            ),
            (
                build_request(
                    Command::Calculate,
                    Some(json!({"operation": "divide", "a": 3.5, "b": -1.05})),
                ),
                json!({"result": 3.5 / -1.05}),
            ),
        ]);

        let req = build_request(
            Command::Batch,
            Some(json!(
                test_data
                    .iter()
                    .map(|(f, _)| json!(f))
                    .collect::<Vec<Value>>()
            )),
        );
        let resp = form_response(req, metrics.clone()).await;

        match resp {
            Response::Err(_) => panic!("Expected OK response"),
            Response::Ok(resp) => {
                for (i, item) in resp.response.as_array().unwrap().iter().enumerate() {
                    let content = OkResponse::deserialize(item).unwrap();
                    assert_eq!(content.request_id, test_data[i].0.request_id);
                    assert!(matches!(content.status, Status::Ok));
                    assert_eq!(content.response, test_data[i].1);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_calculation_payload() {
        #[rustfmt::skip]
        let test_data = Vec::from([
            None,
            Some(Value::Null),
            Some(json!(["only", "objects", "are", "allowed"])),
            Some(json!({"operation": "this operation doesn't exist", "a": 6.0, "b": -8})),
            Some(json!({"operation": "add", "first": 1, "second": 2})),
            Some(json!({"operation": "divide", "a": 22, "b": 0})),
        ]);

        let metrics = build_metrics();
        for item in test_data {
            let req = build_request(Command::Calculate, item);
            match form_response(req, metrics.clone()).await {
                Response::Ok(_) => panic!("Expected Error response"),
                Response::Err(content) => assert!(matches!(content.status, Status::Error)),
            }
        }
    }
}
