use anyhow::{Result, anyhow};
use chrono::Utc;
use log::{debug, error, info};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::types::*;

pub async fn handle_connection(mut stream: TcpStream) {
    let mut buf = Vec::new();
    if let Err(e) = stream.read_to_end(&mut buf).await {
        error!("Failed to receive data: {e}");
        return;
    }

    // first, check if the input is a valid JSON
    let json_data = match serde_json::from_slice::<Value>(&buf) {
        Ok(v) => v,
        Err(e) => {
            debug!("Received data is not a valid JSON: {e}");
            send_error(stream, None, "request is not a valid JSON").await;
            return;
        }
    };
    // then try deserializing it into Request
    let request = match serde_json::from_value::<Request>(json_data) {
        Ok(v) => v,
        Err(e) => {
            debug!("Received data is not a valid request: {e}");
            send_error(stream, None, e.to_string()).await;
            return;
        }
    };

    info!("Received request: {request:?}");

    let uuid = request.request_id;
    match process_request(request).await {
        Ok(v) => send_response(stream, uuid, v).await,
        Err(e) => send_error(stream, Some(uuid), e.to_string()).await,
    }
}

async fn process_request(request: Request) -> Result<Value> {
    match request.command.to_lowercase().as_str() {
        "ping" => Ok(json!("pong")),
        "echo" => Ok(request.payload.unwrap()),
        "time" => {
            let time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            Ok(json!({"time": time}))
        }
        "calculate" => process_command_calculate(request).await,
        unknown => Err(anyhow!("unknown command: {unknown}")),
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

async fn send_response<V: Into<Value>>(stream: TcpStream, uuid: Uuid, response: V) {
    let resp = Response {
        request_id: uuid,
        status: Status::Ok,
        response: response.into(),
    };
    _send(stream, resp).await;
}

async fn send_error<E: Into<String>>(stream: TcpStream, uuid: Option<Uuid>, error: E) {
    let resp = ErrorResponse {
        request_id: uuid,
        status: Status::Error,
        error: error.into(),
    };
    _send(stream, resp).await;
}

async fn _send<T: Serialize + std::fmt::Debug>(mut stream: TcpStream, resp: T) {
    let data = match serde_json::to_vec(&resp) {
        Ok(v) => v,
        Err(e) => {
            error!("Sending failed - couldn't serialize the provided response (how?): {e}");
            return;
        }
    };
    info!("Sending response: {resp:?}");
    if let Err(e) = stream.write_all(&data).await {
        error!("Sending failed: {e}");
        return;
    };
}
