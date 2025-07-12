use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// Requests

#[derive(Deserialize, Debug)]
pub struct Request {
    pub request_id: Uuid,
    pub command: String,
    pub payload: Option<Value>,
}

#[derive(Deserialize)]
pub struct CalculationPayload {
    pub operation: Operation,
    pub a: f64,
    pub b: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

// Responses

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Ok(OkResponse),
    Err(ErrorResponse),
}

#[derive(Serialize, Debug)]
pub struct OkResponse {
    pub request_id: Uuid,
    pub status: Status,
    pub response: Value,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub request_id: Option<Uuid>,
    pub status: Status,
    pub error: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}
