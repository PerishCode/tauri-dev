//! Inspect IPC bridge: connect to a sidecar's inspect socket and exchange a
//! single line-JSON request/response pair.
//!
//! Wire format (one line per direction):
//!   request:  `{"event":"...","payload":<json>}\n`
//!   response: `{"ok":true,"data":<json>}\n` or `{"ok":false,"error":"..."}\n`

use crate::socket::SocketEndpoint;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct InspectRequest {
    pub event: String,
    pub payload: Value,
}

#[derive(Clone, Debug)]
pub enum InspectResponse {
    Ok(Value),
    Err(String),
}

pub fn send(
    endpoint: &SocketEndpoint,
    request: &InspectRequest,
    timeout: Option<Duration>,
) -> Result<InspectResponse, String> {
    let mut line = serde_json::to_string(&serde_json::json!({
        "event": request.event,
        "payload": request.payload,
    }))
    .map_err(|err| err.to_string())?;
    line.push('\n');

    let raw = match endpoint {
        SocketEndpoint::Unix(path) => unix_round_trip(path, &line, timeout)?,
        SocketEndpoint::Tcp(address) => tcp_round_trip(address, &line, timeout)?,
    };
    parse_response(&raw)
}

fn parse_response(text: &str) -> Result<InspectResponse, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("inspect endpoint returned empty response".to_string());
    }
    let value: Value = serde_json::from_str(trimmed).map_err(|err| err.to_string())?;
    let ok = value.get("ok").and_then(Value::as_bool).unwrap_or(false);
    if ok {
        Ok(InspectResponse::Ok(
            value.get("data").cloned().unwrap_or(Value::Null),
        ))
    } else {
        let error = value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("inspect endpoint returned ok=false")
            .to_string();
        Ok(InspectResponse::Err(error))
    }
}

#[cfg(unix)]
fn unix_round_trip(
    path: &std::path::PathBuf,
    line: &str,
    timeout: Option<Duration>,
) -> Result<String, String> {
    let mut stream = UnixStream::connect(path).map_err(|err| err.to_string())?;
    if let Some(timeout) = timeout {
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
    }
    stream
        .write_all(line.as_bytes())
        .map_err(|err| err.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|err| err.to_string())?;
    Ok(response)
}

#[cfg(not(unix))]
fn unix_round_trip(
    _path: &std::path::PathBuf,
    _line: &str,
    _timeout: Option<Duration>,
) -> Result<String, String> {
    Err("unix inspect transport is not available on this platform".to_string())
}

fn tcp_round_trip(address: &str, line: &str, timeout: Option<Duration>) -> Result<String, String> {
    let mut stream = TcpStream::connect(address).map_err(|err| err.to_string())?;
    if let Some(timeout) = timeout {
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
    }
    stream
        .write_all(line.as_bytes())
        .map_err(|err| err.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|err| err.to_string())?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ok_response() {
        let parsed = parse_response("{\"ok\":true,\"data\":{\"answer\":42}}").unwrap();
        match parsed {
            InspectResponse::Ok(value) => {
                assert_eq!(value.get("answer").and_then(Value::as_i64), Some(42));
            }
            other => panic!("expected ok response, got {other:?}"),
        }
    }

    #[test]
    fn parses_error_response() {
        let parsed = parse_response("{\"ok\":false,\"error\":\"boom\"}").unwrap();
        match parsed {
            InspectResponse::Err(message) => assert_eq!(message, "boom"),
            other => panic!("expected error response, got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_response() {
        let err = parse_response("").unwrap_err();
        assert!(err.contains("empty"));
    }
}
