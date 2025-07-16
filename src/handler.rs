use log::{debug, error};
use serde::Serialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::*;
use crate::types::*;

/// Handles the TCP connection by processing an incoming request and sending a response.
///
/// This function is called after a new client connection is accepted.
/// It performs the following steps:
/// 1. Reads the data from the TCP stream;
/// 2. Attempts to deserialize it into a `Request`;
/// 3. Calls `form_response` to process the request and generate a `Response`;
/// 4. Serializes the response and writes it back to the same stream.
///
/// # Parameters
/// - `stream`: The TCP stream representing the client connection.
/// - `metrics`: A shared thread-safe pointer to the global `Metrics` instance.
///   This is passed to the `form_response` function without modification.
pub async fn handle_connection(mut stream: TcpStream, metrics: Arc<Mutex<Metrics>>) {
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
    debug!(
        "Received request: {}",
        serde_json::to_string(&request).unwrap()
    );

    send_response(stream, form_response(request, metrics).await).await;
}

async fn send_response<T: Serialize>(mut stream: TcpStream, resp: T) {
    let data = match serde_json::to_vec(&resp) {
        Ok(v) => v,
        Err(e) => {
            error!("Sending failed - couldn't serialize the provided response (how?): {e}");
            return;
        }
    };
    debug!(
        "Sending response: {}",
        serde_json::to_string(&resp).unwrap()
    );
    if let Err(e) = stream.write_all(&data).await {
        error!("Sending failed: {e}");
        return;
    };
    debug!("Response sent.")
}
