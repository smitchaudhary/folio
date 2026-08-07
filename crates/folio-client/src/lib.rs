pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn health(&self) -> Result<(), ClientError> {
        let url = format!("{}/api/health", self.base_url);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        if resp.status() == 200 {
            Ok(())
        } else {
            Err(ClientError::UnexpectedStatus(resp.status().as_u16()))
        }
    }
}

#[derive(Debug)]
pub enum ClientError {
    Transport(String),
    UnexpectedStatus(u16),
}
