use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// Requests

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub request_id: Uuid,
    pub command: Command,
    pub payload: Option<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct CalculationPayload {
    pub operation: Operation,
    pub a: f64,
    pub b: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Ping,
    Echo,
    Time,
    Calculate,
    Batch
}

// Responses

#[derive(Serialize)]
#[serde(untagged)]
pub enum Response {
    Ok(OkResponse),
    Err(ErrorResponse),
}

#[derive(Serialize)]
pub struct OkResponse {
    pub request_id: Uuid,
    pub status: Status,
    pub response: Value,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub request_id: Option<Uuid>,
    pub status: Status,
    pub error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}
