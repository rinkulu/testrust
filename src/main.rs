use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod types;
use types::*;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("localhost:7878").await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Couldn't start the server: {e}");
            return;
        }
    };
    loop {
        let (socket, _) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Couldn't accept an incoming connection: {e}");
                continue;
            }
        };
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buf = Vec::new();
    if let Err(e) = stream.read_to_end(&mut buf).await {
        eprintln!("Reading data failed with error: {e}");
        return;
    }

    let request: Request = serde_json::from_slice(&buf).unwrap();
    let uuid = request.request_id.clone();
    match request.command.to_lowercase().as_str() {
        "ping" => send_response(stream, uuid, "pong").await,
        "echo" => send_response(stream, uuid, request.payload).await,
        "time" => {
            let time = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            send_response(stream, uuid, json!({"time": time})).await;
        }
        "calculate" => process_command_calculate(stream, request).await,
        _ => send_error(stream, Some(uuid), "unknown command").await,
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

async fn send_response<V: Into<Value>>(stream: TcpStream, uuid: String, response: V) {
    let resp = Response {
        request_id: uuid,
        status: Status::Ok,
        response: response.into()
    };
    _send(stream, resp).await;
}

async fn send_error<E: Into<String>>(stream: TcpStream, uuid: Option<String>, error: E) {
    let resp = ErrorResponse {
        request_id: uuid,
        status: Status::Error,
        error: error.into()
    };
    _send(stream, resp).await;
}

async fn _send<T: Serialize>(mut stream: TcpStream, resp: T) {
    let data = match serde_json::to_vec(&resp) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Sending failed - couldn't serialize the provided response: {e}");
            return;
        }
    };
    println!("{:?}", serde_json::to_string(&resp));
    if let Err(e) = stream.write_all(&data).await {
        eprintln!("Sending failed: {e}");
        return;
    };
}
