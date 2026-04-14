use std::fmt;

pub enum ZeroClawError {
    Http(reqwest::Error),
    #[allow(clippy::box_collection)]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),
    Json(serde_json::Error),
    Api {
        code: i32,
        message: String,
    },
    Config(String),
    #[allow(dead_code)]
    Signing(String),
    RateLimit,
    PlanRequired {
        required: String,
        current: String,
    },
    WsDisconnected,
    Io(String),
    Other(anyhow::Error),
}

impl fmt::Display for ZeroClawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeroClawError::Http(e) => {
                write!(
                    f,
                    "HTTP request failed: {}. Check your network connection.",
                    e
                )
            }
            ZeroClawError::WebSocket(e) => {
                write!(
                    f,
                    "WebSocket error: {}. Connection may have been interrupted.",
                    e
                )
            }
            ZeroClawError::Json(e) => {
                write!(
                    f,
                    "Failed to parse JSON response: {}. The API may have returned invalid data.",
                    e
                )
            }
            ZeroClawError::Api { code, message } => {
                write!(
                    f,
                    "API error (code {}): {}. See https://docs.ave.ai for error codes.",
                    code, message
                )
            }
            ZeroClawError::Config(msg) => {
                write!(
                    f,
                    "Configuration error: {}. Please check your environment variables.",
                    msg
                )
            }
            ZeroClawError::Signing(msg) => {
                write!(
                    f,
                    "Transaction signing failed: {}. Check your private key or mnemonic.",
                    msg
                )
            }
            ZeroClawError::RateLimit => {
                write!(f, "Rate limit exceeded. Upgrade your API plan at https://cloud.ave.ai to increase RPM (Free: 10, Normal: 60, Pro: 300).")
            }
            ZeroClawError::PlanRequired { required, current } => {
                write!(f, "Your current plan ({}) does not support this feature. Required: {}. Upgrade at https://cloud.ave.ai.", current, required)
            }
            ZeroClawError::WsDisconnected => {
                write!(
                    f,
                    "WebSocket disconnected unexpectedly. Will attempt to reconnect."
                )
            }
            ZeroClawError::Io(msg) => {
                write!(f, "IO error: {}", msg)
            }
            ZeroClawError::Other(e) => {
                write!(f, "Unexpected error: {}. Please report this issue.", e)
            }
        }
    }
}

impl fmt::Debug for ZeroClawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeroClawError::Http(e) => write!(f, "Http({})", e),
            ZeroClawError::WebSocket(e) => write!(f, "WebSocket({})", e),
            ZeroClawError::Json(e) => write!(f, "Json({})", e),
            ZeroClawError::Api { code, message } => {
                write!(f, "Api {{ code: {}, message: {} }}", code, message)
            }
            ZeroClawError::Config(msg) => write!(f, "Config({})", msg),
            ZeroClawError::Signing(msg) => write!(f, "Signing({})", msg),
            ZeroClawError::RateLimit => write!(f, "RateLimit"),
            ZeroClawError::PlanRequired { required, current } => {
                write!(
                    f,
                    "PlanRequired {{ required: {}, current: {} }}",
                    required, current
                )
            }
            ZeroClawError::WsDisconnected => write!(f, "WsDisconnected"),
            ZeroClawError::Io(msg) => write!(f, "Io({})", msg),
            ZeroClawError::Other(e) => write!(f, "Other({})", e),
        }
    }
}

impl std::error::Error for ZeroClawError {}

impl From<reqwest::Error> for ZeroClawError {
    fn from(err: reqwest::Error) -> Self {
        ZeroClawError::Http(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ZeroClawError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        ZeroClawError::WebSocket(Box::new(err))
    }
}

impl From<serde_json::Error> for ZeroClawError {
    fn from(err: serde_json::Error) -> Self {
        ZeroClawError::Json(err)
    }
}

impl From<anyhow::Error> for ZeroClawError {
    fn from(err: anyhow::Error) -> Self {
        ZeroClawError::Other(err)
    }
}
