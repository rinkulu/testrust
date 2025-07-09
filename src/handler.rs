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

    match request.command.to_lowercase().as_str() {
        "ping" => send_response(stream, request.request_id, "pong").await,
        "echo" => send_response(stream, request.request_id, request.payload).await,
        "time" => {
            let time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            send_response(stream, request.request_id, json!({"time": time})).await;
        }
        "calculate" => process_command_calculate(stream, request).await,
        unknown => {
            debug!("Received request contains unknown command: {unknown}");
            send_error(stream, Some(request.request_id), "unknown command").await;
        }
    }
}

async fn process_command_calculate(stream: TcpStream, req: Request) {
    let value = match req.payload {
        Some(v) => v,
        None => {
            send_error(stream, Some(req.request_id), "payload not included").await;
            return;
        }
    };
    let payload = match serde_json::from_value::<CalculationPayload>(value) {
        Ok(v) => v,
        Err(e) => {
            send_error(stream, Some(req.request_id), e.to_string()).await;
            return;
        }
    };

    let result = match payload.operation {
        Operation::Add => payload.a + payload.b,
        Operation::Subtract => payload.a - payload.b,
        Operation::Multiply => payload.a * payload.b,
        Operation::Divide => {
            if payload.b == 0.0 {
                send_error(stream, Some(req.request_id), "division by zero").await;
                return;
            }
            payload.a / payload.b
        }
    };
    send_response(stream, req.request_id, json!({"result": result})).await;
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
