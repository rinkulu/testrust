use anyhow::{Result, anyhow};
use chrono::Utc;
use serde_json::{Value, json};

use crate::types::*;

pub async fn form_response(request: Request) -> Response {
    let uuid = request.request_id;
    match process_command(request).await {
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
    }
}

async fn process_command(request: Request) -> Result<Value> {
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
                result.push(Box::pin(form_response(item)).await);
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
