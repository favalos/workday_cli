use crate::config::{Config, load_config};
use crate::security::{KeyStore, get_credentials};

pub struct ApiClient {
    config: Config,
    credentials: KeyStore,
}

impl ApiClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            config: load_config()?,
            credentials: get_credentials()?,
        })
    }

    pub fn get(&self, path: &str) -> Result<String, String> {
        let url = format!(
            "https://{}/api/common/v1/{}/{}",
            self.config.host, self.config.tenant, path
        );

        let response = ureq::get(&url)
            .set(
                "Authorization",
                &format!(
                    "{} {}",
                    self.credentials.token_response.token_type,
                    self.credentials.token_response.access_token
                ),
            )
            .call()
            .map_err(|e| format!("Request failed: {e}"))?;

        response
            .into_string()
            .map_err(|e| format!("Failed to read response: {e}"))
    }
}
