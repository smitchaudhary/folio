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

#[derive(Debug, Default, Serialize)]
pub struct EditItemRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusResult {
    pub item: folio_core::Item,
    pub moved_to_archive: bool,
    pub moved_to_inbox: bool,
    pub overflow: Vec<folio_core::Item>,
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

    pub fn from_env() -> Self {
        let url = std::env::var("FOLIO_SERVER")
            .ok()
            .or_else(load_client_json)
            .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
        Self::new(url)
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

    pub fn change_status(
        &self,
        id: &str,
        status: folio_core::Status,
    ) -> Result<ChangeStatusResult, ClientError> {
        let body = serde_json::json!({ "status": status.as_string() }).to_string();
        let (resp_status, resp_body) = self.post(&format!("/api/items/{id}/status"), &body)?;
        if resp_status == 200 {
            serde_json::from_str(&resp_body).map_err(|e| ClientError::Parse(e.to_string()))
        } else {
            Err(self.error_from_status(resp_status, &resp_body))
        }
    }

    pub fn edit_item(
        &self,
        id: &str,
        request: EditItemRequest,
    ) -> Result<folio_core::Item, ClientError> {
        let body =
            serde_json::to_string(&request).map_err(|e| ClientError::Parse(e.to_string()))?;
        let (status, resp_body) = self.patch(&format!("/api/items/{id}"), &body)?;
        if status == 200 {
            parse_item_response(&resp_body)
        } else {
            Err(self.error_from_status(status, &resp_body))
        }
    }

    pub fn delete_item(&self, id: &str) -> Result<(), ClientError> {
        let (status, body) = self.delete(&format!("/api/items/{id}"))?;
        if status == 204 {
            Ok(())
        } else {
            Err(self.error_from_status(status, &body))
        }
    }

    pub fn toggle_kind(&self, id: &str) -> Result<folio_core::Item, ClientError> {
        let (status, body) = self.post(&format!("/api/items/{id}/kind"), "")?;
        if status == 200 {
            parse_item_response(&body)
        } else {
            Err(self.error_from_status(status, &body))
        }
    }

    pub fn get_config(&self) -> Result<folio_core::Config, ClientError> {
        let (status, body) = self.get("/api/config")?;
        self.check_status(200, status, &body)?;
        serde_json::from_str(&body).map_err(|e| ClientError::Parse(e.to_string()))
    }

    pub fn set_config(
        &self,
        config: &folio_core::Config,
    ) -> Result<folio_core::Config, ClientError> {
        let body = serde_json::to_string(config).map_err(|e| ClientError::Parse(e.to_string()))?;
        let (status, resp_body) = self.put("/api/config", &body)?;
        if status == 200 {
            serde_json::from_str(&resp_body).map_err(|e| ClientError::Parse(e.to_string()))
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
        self.send_with_body("POST", path, body)
    }

    fn patch(&self, path: &str, body: &str) -> Result<(u16, String), ClientError> {
        self.send_with_body("PATCH", path, body)
    }

    fn put(&self, path: &str, body: &str) -> Result<(u16, String), ClientError> {
        self.send_with_body("PUT", path, body)
    }

    fn send_with_body(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<(u16, String), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let builder = match method {
            "POST" => self.agent.post(&url),
            "PATCH" => self.agent.patch(&url),
            "PUT" => self.agent.put(&url),
            _ => unreachable!("unsupported body method: {method}"),
        };
        let mut resp = builder
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

    fn delete(&self, path: &str) -> Result<(u16, String), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let mut resp = self
            .agent
            .delete(&url)
            .call()
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

fn parse_item_response(body: &str) -> Result<folio_core::Item, ClientError> {
    #[derive(Deserialize)]
    struct ItemResponse {
        item: folio_core::Item,
    }
    serde_json::from_str::<ItemResponse>(body)
        .map(|r| r.item)
        .map_err(|e| ClientError::Parse(e.to_string()))
}

fn load_client_json() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".folio").join("client.json");
    let content = std::fs::read_to_string(path).ok()?;
    let config: ServerConfig = serde_json::from_str(&content).ok()?;
    Some(config.server_url)
}

#[derive(Deserialize)]
struct ServerConfig {
    server_url: String,
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
