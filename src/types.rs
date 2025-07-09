use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct Request {
    pub uuid: String,
    pub command: String,
    pub payload: Option<Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub uuid: String,
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
