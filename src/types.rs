use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct Request {
    pub request_id: String,
    pub command: String,
    pub payload: Option<Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    // in case of invalid JSON or missing "request_id" field, we won't be able copy it from the request
    pub request_id: Option<String>,
    pub status: Status,
    pub error: String
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub request_id: String,
    pub status: Status,
    pub response: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Deserialize)]
pub struct CalculationPayload {
    pub operation: Operation,
    pub a: f64,
    pub b: f64,
}
