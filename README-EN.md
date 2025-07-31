This ReadMe is also available in [Russian](README.md)

---

## Asynchronous JSON API server in Rust with Tokio and Serde JSON

### Description

After startup, the server begins listening on local TCP port 7878. It accepts JSON requests consisting of a request identifier in UUID format, a command name, and optional data. The server response is identified by the same ID and contains the request status (`Ok`/`Error`) along with either the command execution result or an error message.

Request structure:
```js
{
    "request_id": "UUID_string",
    "command": "command_name",
    "payload": { ... }  // optional data
}
```

A successful response will have the following structure:
```js
{
    "request_id": "UUID_string",
    "status": "ok",
    "response": { ... } // the command execution result
}
```

In case of error, the response structure changes:
```js
{
    "request_id": "UUID_string",
    "status": "error",
    "error": "error_description"
}
```

### Supported commands

#### `ping`

Returns `{"response": "pong"}`.

#### `echo`

Returns the content of the request's `payload` field:
```js
// request
{
    ...
    "payload": {
        "number": 5,
        "nothing": null
    }
}

// response
{
    ...
    "response": {
        "number": 5,
        "nothing": null
    }
}
```

#### `time`

Returns current UTC time in RFC 3339 format:
```js
{
    ...
    "response": {
        "time": "2025-07-16 17:45:45Z"
    }
}
```

#### `calculate`

Required `payload` field with the folllowing format:
```js
{
    ...
    "payload": {
        "operation": "add|subtract|multiply|divide",
        "a": number,
        "b": number
    }
}
```
Returns the result of the specified `operation` on operands `a` and `b`:
```js
// request
{
    ...
    "payload": {
        "operation": "add",
        "a": 5,
        "b": 3
    }
}

// response
{
    ...
    "response": {
        "result": 8.0
    }
}
```

#### `batch`

Requires `payload` field. The content must be an array of JSON objects, each being a separate valid request.
This command is used to combine multiple commands into a single request. Server response will be an array of results for each command:
```js
// request
{
    "request_id": "batch_id",
    "command": "batch",
    "payload": [
        {
            "request_id": "id1",
            "command": "ping"
        },
        {
            "request_id": "id2",
            "command": "time"
        },
        {
            "request_id": "id3",
            "command": "calculate",
            "payload": "invalid_payload"
        }
    ]
}

// response
{
    "request_id": "batch_id",
    "status": "ok",
    "response": [
        {
            "request_id": "id1",
            "status": "ok",
            "response": "pong"
        },
        {
            "request_id": "id2",
            "status": "ok",
            "response": {
                "time": "2025-07-16 17:45:45Z"
            }
        },
        {
            "request_id": "id3",
            "status": "error",
            "error": "invalid type: string \"invalid_payload\", expected struct CalculationPayload"
        }
    ]
}
```

### Installation

Download the executable from the [latest release](https://github.com/rinkulu/testrust/releases/latest).

### Launch options

- `-d`/`--debug` - flag to enable debug logging;

- `-l <FILE>`/`--log-file <FILE>` - specifies the log file.
Default value: `default.log`
