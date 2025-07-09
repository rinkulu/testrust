use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    pub request_id: Uuid,
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
    pub request_id: Option<Uuid>,
    pub status: Status,
    pub error: String,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub request_id: Uuid,
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
