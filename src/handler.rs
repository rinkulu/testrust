use log::{debug, error, info};
use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::*;
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
            send_response(
                stream,
                ErrorResponse {
                    request_id: None,
                    status: Status::Error,
                    error: "request is not a valid JSON".to_string(),
                },
            )
            .await;
            return;
        }
    };
    // then try deserializing it into Request
    let request = match serde_json::from_value::<Request>(json_data) {
        Ok(v) => v,
        Err(e) => {
            debug!("Received data is not a valid request: {e}");
            send_response(
                stream,
                ErrorResponse {
                    request_id: None,
                    status: Status::Error,
                    error: e.to_string(),
                },
            )
            .await;
            return;
        }
    };

    info!("Received request: {request:?}");
    send_response(stream, form_response(request).await).await;
}

async fn send_response<T: Serialize + std::fmt::Debug>(mut stream: TcpStream, resp: T) {
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
