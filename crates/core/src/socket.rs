use std::path::PathBuf;

const UNIX_PREFIX: &str = "unix://";
const TCP_PREFIX: &str = "tcp://";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocketEndpoint {
    Unix(PathBuf),
    Tcp(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocketEndpointParseError {
    message: String,
}

impl SocketEndpoint {
    pub fn parse(value: &str) -> Result<Self, SocketEndpointParseError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(SocketEndpointParseError::new(
                "socket endpoint must not be empty",
            ));
        }

        if let Some(path) = value.strip_prefix(UNIX_PREFIX) {
            if path.is_empty() {
                return Err(SocketEndpointParseError::new(
                    "unix socket endpoint must include a path",
                ));
            }
            if !path.starts_with('/') {
                return Err(SocketEndpointParseError::new(
                    "unix socket endpoint must use unix:///absolute/path.sock form",
                ));
            }
            return Ok(Self::Unix(PathBuf::from(path)));
        }

        if let Some(address) = value.strip_prefix(TCP_PREFIX) {
            validate_tcp_address(address)?;
            return Ok(Self::Tcp(address.to_string()));
        }

        Err(SocketEndpointParseError::new(
            "socket endpoint must use unix:///path.sock form, or tcp://host:port on non-Unix platforms",
        ))
    }

    pub fn as_endpoint(&self) -> String {
        match self {
            Self::Unix(path) => format!("unix://{}", path.display()),
            Self::Tcp(address) => format!("{TCP_PREFIX}{address}"),
        }
    }
}

impl SocketEndpointParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SocketEndpointParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SocketEndpointParseError {}

fn validate_tcp_address(address: &str) -> Result<(), SocketEndpointParseError> {
    if address.trim().is_empty() {
        return Err(SocketEndpointParseError::new(
            "tcp socket endpoint must include host:port",
        ));
    }
    if !address.contains(':') {
        return Err(SocketEndpointParseError::new(
            "tcp socket endpoint must include a port",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unix_endpoint() {
        let endpoint = SocketEndpoint::parse("unix:///tmp/tauri-dev.sock").unwrap();
        assert_eq!(
            endpoint,
            SocketEndpoint::Unix(PathBuf::from("/tmp/tauri-dev.sock"))
        );
        assert_eq!(endpoint.as_endpoint(), "unix:///tmp/tauri-dev.sock");
    }

    #[test]
    fn rejects_bare_tcp_endpoint() {
        let error = SocketEndpoint::parse("127.0.0.1:3901").unwrap_err();
        assert!(error.to_string().contains("unix:///path.sock"));
    }
}
