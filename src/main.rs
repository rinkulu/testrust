use chrono::Utc;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod types;
use types::*;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:7878").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();

    let request: Request = serde_json::from_slice(&buf).unwrap();
    let mut resp = Response {
        uuid: request.uuid.clone(),
        status: Status::Ok,
        response: Value::Null,
    };

    match request.command.to_lowercase().as_str() {
        "ping" => resp.response = json!("pong"),
        "echo" => resp.response = request.payload.unwrap(),
        "time" => {
            resp.response = json!(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        }
        "calculate" => {
            (resp.status, resp.response) = process_command_calculate(request).await;
        }
        _ => {
            resp.status = Status::Error;
            resp.response = json!("unknown command");
        }
    }

    println!("{:?}", serde_json::to_string(&resp));
    stream
        .write_all(&serde_json::to_vec(&resp).unwrap())
        .await
        .unwrap();
}

async fn process_command_calculate(req: Request) -> (Status, Value) {
    let value = match req.payload {
        Some(v) => v,
        None => return (Status::Error, json!("payload not provided")),
    };
    let payload = match serde_json::from_value::<CalculationPayload>(value) {
        Ok(v) => v,
        Err(e) => {
            return (Status::Error, json!(e.to_string()));
        }
    };

    let status = Status::Ok;
    let result = match payload.operation {
        Operation::Add => payload.a + payload.b,
        Operation::Subtract => payload.a - payload.b,
        Operation::Multiply => payload.a * payload.b,
        Operation::Divide => payload.a / payload.b,
    };

    (status, json!(result))
}
