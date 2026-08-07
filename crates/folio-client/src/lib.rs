use folio_core::View;
use serde::Deserialize;

pub struct Client {
    base_url: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            agent: ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .new_agent(),
        }
    }

    pub fn health(&self) -> Result<(), ClientError> {
        let (status, _) = self.get("/api/health")?;
        if status == 200 {
            Ok(())
        } else {
            Err(ClientError::UnexpectedStatus(status))
        }
    }

    pub fn list_items(&self, view: View) -> Result<Vec<folio_core::Item>, ClientError> {
        let (status, body) = self.get(&format!("/api/items?view={}", view.as_string()))?;
        self.check_status(status, &body)?;
        serde_json::from_str(&body).map_err(|e| ClientError::Parse(e.to_string()))
    }

    fn get(&self, path: &str) -> Result<(u16, String), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let mut resp = self
            .agent
            .get(&url)
            .call()
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| ClientError::Parse(e.to_string()))?;
        Ok((status, body))
    }

    fn check_status(&self, status: u16, body: &str) -> Result<(), ClientError> {
        if status == 200 {
            Ok(())
        } else if let Ok(envelope) = serde_json::from_str::<ErrorEnvelope>(body) {
            Err(ClientError::Server {
                code: envelope.error.code,
                message: envelope.error.message,
            })
        } else {
            Err(ClientError::UnexpectedStatus(status))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[derive(Debug)]
pub enum ClientError {
    /// Network-level failure: server unreachable, DNS, connection refused, etc.
    Transport(String),
    /// The server responded with an error envelope: code + message.
    Server { code: String, message: String },
    /// The server responded, but with an unexpected status code.
    UnexpectedStatus(u16),
    /// The response body could not be parsed.
    Parse(String),
}
