use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// A structure representing a valid request to the server.
///
/// The server expects JSON requests that can be deserialized into this structure.
/// Each request must contain:
/// - a unique identifier in UUIDv4 format in the `request_id` field;
/// - a string in the `command` field, which must be one of the commands supported by the server.
/// This field determines the action the server takes and the response it returns.
///
/// Optionally, the request may also contain a `payload` field.
/// This field is required for some commands and may be omitted for others.
#[derive(Serialize, Deserialize)]
pub struct Request {
    /// A unique request identifier.
    pub request_id: Uuid,

    /// A command specifying the action the server is requested to perform.
    pub command: Command,

    /// Optional data provided for some of the commands.
    pub payload: Option<Value>,
}

/// A structure representing the `payload` content of a `Request` for the `Calculate` command.
///
/// This structure contains two operands and an arithmetic operation to perform on them.
/// The server will perform the specified `operation` using the values of `a` and `b`,
/// and return the result as a floating-point number.
#[derive(Serialize, Deserialize)]
pub struct CalculationPayload {
    /// The arithmetic operation to perform.
    pub operation: Operation,

    /// The first operand.
    pub a: f64,

    /// The second operand.
    pub b: f64,
}

/// An enumeration of supported arithmetic operations.
///
/// This enum represents the possible values of the `operation` field
/// in a `CalculationPayload`.
///
/// The operation values are (de)serialized in lowercase, e.g., `"add"`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// An enumeration of all of the commands supported by the server.
///
/// This enum represents the possible values of the `command` field in a `Request`.
/// Each variant corresponds to a specific action the server can perform,
/// and results in different `response` content in an `OkResponse`.
///
/// The command values are (de)serialized in lowercase, e.g., `"ping"`.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    /// The server will return the string "pong".
    Ping,

    /// The server will return the content of the original request's `payload` field
    /// without modifying it. If the `payload` field was omitted, the `response` will be `null`.
    Echo,

    /// The server will return the current UTC time in RFC 3339 format.
    Time,

    /// Requires the presence of the `payload` field in the request.
    /// The `payload` must be deserializable into a `CalculationPayload` structure.
    ///
    /// The server will return a JSON object in the format `{"result": <f64>}`,
    /// where `<f64>` is the result of the calculation as a floating-point number.
    Calculate,

    /// Requires the presence of the `payload` field in the request.
    /// The `payload` must be an array of objects, each one of which
    /// can be deserialized into a separate `Request`.
    ///
    /// The server will return an array of `Response` structures,
    /// one for each `Request` provided in the `payload`.
    Batch,
}

/// An enumeration representing the possible server responses to a request.
///
/// This enum can be either:
/// - `Ok`, containing a successful `OkResponse`,
/// - `Err`, containing an error `ErrorResponse`.
///
/// It is (de)serialized as an untagged enum, meaning that the JSON structure itself
/// determines which variant is used.2
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ok(OkResponse),
    Err(ErrorResponse),
}

/// A structure representing a successful response from the server.
///
/// This response is returned when a request is processed without errors.
/// It includes the `request_id` of the original request,
/// a `status` field indicating success,
/// and the actual result in the `response` field.
///
/// The format of the `response` field depends on the command that was executed.
/// See the `Command` documentation for more information.
#[derive(Serialize, Deserialize)]
pub struct OkResponse {
    /// The identifier of the original request without modification.
    pub request_id: Uuid,

    /// The status of the response. Always `Status::Ok` for this type.
    pub status: Status,

    /// The result of the executed command.
    /// Its content depends on the specific command;
    /// see the `Command` documentation for more information.
    pub response: Value,
}

/// A structure representing an error response from the server.
///
/// This response is returned when an error occurs while processing a request.
/// It includes:
/// - the `request_id` of the original request, if provided.
///   This is an optional field; it can be `null` if the request was not a valid JSON
///   or could not be successfully deserialized into a `Request` structure.
///   Note that partial requests (e.g., including `request_id` but missing `command`)
///   will also result in the `request_id` of the response being `null`.
/// - a `status` field indicating error;
/// - an `error` field with the description of the error that occurred.
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The identifier of the original request without modification.
    /// If the request could not be deserialized into a `Request` structure,
    /// this field will be `null`.
    pub request_id: Option<Uuid>,

    /// The status of the response. Always `Status::Error` for this type.
    pub status: Status,

    /// A description of the error that occurred.
    pub error: String,
}

/// An enumeration representing the status of a server response.
///
/// This enum indicates whether a response corresponds to the successful type (`OkResponse`)
/// or the error type (`ErrorReponse`).
///
/// The variants are (de)serialized in lowercase, e.g., `"ok"` or `"error"`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}

/// A structure for collecting performance metrics per command.
///
/// This structure tracks the number of times each command has been processed (`command_counts`),
/// as well as the minimum (`processing_time_min`), maximum (`processing_time_max`)
/// and average (`processing_time_avg`) processing times (in milliseconds) for each command.
#[derive(Default)]
pub struct Metrics {
    /// The count of how many times each command has been processed.
    pub command_counts: HashMap<Command, usize>,

    /// The minimum processing time (in ms) observed for each command.
    pub processing_time_min: HashMap<Command, f64>,

    /// The maximum processing time (in ms) observed for each command.
    pub processing_time_avg: HashMap<Command, f64>,

    /// The average processing time (in ms) observed for each command.
    pub processing_time_max: HashMap<Command, f64>,
}

impl Metrics {
    /// Updates the metrics with a new processing duration for a given command.
    ///
    /// This increments the count, updates the minimum and maximum times if needed,
    /// and recalculates the average processing time.
    ///
    /// # Parameters
    /// - `command`: The command that has been processed.
    /// - `duration`: The processing time in milliseconds for this command execution.
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
