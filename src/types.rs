use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Ping,
    Echo,
    Time,
    Calculate,
    Batch,
}

// Responses

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ok(OkResponse),
    Err(ErrorResponse),
}

#[derive(Serialize, Deserialize)]
pub struct OkResponse {
    pub request_id: Uuid,
    pub status: Status,
    pub response: Value,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub request_id: Option<Uuid>,
    pub status: Status,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}

// Metrics

#[derive(Default)]
pub struct Metrics {
    pub command_counts: HashMap<Command, usize>,
    pub processing_time_min: HashMap<Command, f64>,
    pub processing_time_avg: HashMap<Command, f64>,
    pub processing_time_max: HashMap<Command, f64>,
}

impl Metrics {
    pub fn update(&mut self, command: &Command, duration: f64) {
        *self.command_counts.entry(command.clone()).or_default() += 1;

        self.processing_time_min
            .entry(command.clone())
            .and_modify(|old| *old = old.min(duration))
            .or_insert(duration);

        self.processing_time_max
            .entry(command.clone())
            .and_modify(|old| *old = old.max(duration))
            .or_insert(duration);

        let count = *self.command_counts.get(command).unwrap();
        self.processing_time_avg
            .entry(command.clone())
            .and_modify(|old| *old = (*old * (count - 1) as f64 + duration) / count as f64)
            .or_insert(duration);
    }
}
