use anyhow::{Result, anyhow};
use chrono::Utc;
use log::debug;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

use crate::types::*;

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
