use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

use serde_json::{Value, json};
use uuid::Uuid;

fn wait_for_server() {
    for _ in 0..10 {
        if TcpStream::connect("localhost:7878").is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("Server didn't start in time");
}

#[test]
fn test_ping() {
    let mut server = Command::new("cargo")
        .args(["run", "--", "--debug", "--log-file", "test.log"])
        .spawn()
        .unwrap();

    // the server needs some time to start
    wait_for_server();
    let mut stream = TcpStream::connect("localhost:7878").expect("Couldn't connect to the server");

    let uuid = Uuid::new_v4();
    let request = json!({
        "request_id": uuid,
        "command": "ping"
    });
    let data = &serde_json::to_vec(&request)
        .expect("This should never happen: couldn't serialize the request");
    stream.write_all(data).expect("Couldn't send the request");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("Couldn't shut down the write of the connection");

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .expect("Couldn't read the response");

    let response: Value =
        serde_json::from_slice(&buf).expect("Couldn't deserialize the data received");
    assert_eq!(response["request_id"], uuid.to_string());
    assert_eq!(response["status"], "ok");
    assert_eq!(response["response"], "pong");

    // this is SIGKILL, so no graceful shutdown... oh well.
    server.kill().expect("This should never happen: couldn't kill the server");
}
