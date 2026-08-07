use folio_core::View;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AddItemRequest {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

impl AddItemRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            item_type: None,
            author: None,
            link: None,
            note: None,
            kind: None,
        }
    }
}

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
        let (status, body) = self.get("/api/health")?;
        self.check_status(200, status, &body)
    }

    pub fn list_items(&self, view: View) -> Result<Vec<folio_core::Item>, ClientError> {
        let (status, body) = self.get(&format!("/api/items?view={}", view.as_string()))?;
        self.check_status(200, status, &body)?;
        serde_json::from_str(&body).map_err(|e| ClientError::Parse(e.to_string()))
    }

    pub fn add_item(
        &self,
        request: AddItemRequest,
    ) -> Result<(folio_core::Item, Vec<folio_core::Item>), ClientError> {
        let body =
            serde_json::to_string(&request).map_err(|e| ClientError::Parse(e.to_string()))?;
        let (status, resp_body) = self.post("/api/items", &body)?;

        if status == 201 {
            #[derive(Deserialize)]
            struct AddResponse {
                item: folio_core::Item,
                overflow: Vec<folio_core::Item>,
            }
            let parsed: AddResponse =
                serde_json::from_str(&resp_body).map_err(|e| ClientError::Parse(e.to_string()))?;
            Ok((parsed.item, parsed.overflow))
        } else {
            Err(self.error_from_status(status, &resp_body))
        }
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

    fn post(&self, path: &str, body: &str) -> Result<(u16, String), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let mut resp = self
            .agent
            .post(&url)
            .content_type("application/json")
            .send(body)
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| ClientError::Parse(e.to_string()))?;
        Ok((status, body))
    }

    fn check_status(&self, expected: u16, status: u16, body: &str) -> Result<(), ClientError> {
        if status == expected {
            Ok(())
        } else {
            Err(self.error_from_status(status, body))
        }
    }

    fn error_from_status(&self, status: u16, body: &str) -> ClientError {
        if let Ok(envelope) = serde_json::from_str::<ErrorEnvelope>(body) {
            if envelope.error.code == "inbox_full" {
                ClientError::InboxFull
            } else {
                ClientError::Server {
                    code: envelope.error.code,
                    message: envelope.error.message,
                }
            }
        } else {
            ClientError::UnexpectedStatus(status)
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
    Transport(String),
    InboxFull,
    Server { code: String, message: String },
    UnexpectedStatus(u16),
    Parse(String),
}
