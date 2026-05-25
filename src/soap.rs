use crate::config::{Config, load_config};
use crate::security::{KeyStore, get_credentials};
use std::io::Read;

pub struct SoapClient {
    config: Config,
    credentials: KeyStore,
}

impl SoapClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            config: load_config()?,
            credentials: get_credentials()?,
        })
    }

    /// Send a SOAP request to the specified application and version
    ///
    /// # Arguments
    /// * `application` - The SOAP application name (e.g., "Human_Resources")
    /// * `version` - The SOAP service version (e.g., "v42.0")
    /// * `request` - The complete SOAP XML request body
    ///
    /// # Returns
    /// The SOAP response as a string
    pub fn post(&self, application: &str, version: &str, request: &str) -> Result<String, String> {
        let url = format!(
            "https://{}/ccx/service/{}/{}/{}",
            self.config.host, self.config.tenant, application, version
        );

        match ureq::post(&url)
            .set("Content-Type", "text/xml")
            .set(
                "Authorization",
                &format!(
                    "{} {}",
                    self.credentials.token_response.token_type,
                    self.credentials.token_response.access_token
                ),
            )
            .send_string(request)
        {
            Ok(response) => response
                .into_string()
                .map_err(|e| format!("Failed to read SOAP response: {e}")),
            Err(ureq::Error::Status(_, response)) => {
                let body = response
                    .into_string()
                    .unwrap_or_else(|_| "Unable to read fault body".to_string());
                Err(body)
            }
            Err(e) => Err(format!("SOAP request failed: {e}")),
        }
    }

    /// Send a SOAP request and return the response as bytes
    ///
    /// # Arguments
    /// * `application` - The SOAP application name
    /// * `version` - The SOAP service version
    /// * `request` - The complete SOAP XML request body
    ///
    /// # Returns
    /// The SOAP response as a byte vector
    pub fn post_bytes(
        &self,
        application: &str,
        version: &str,
        request: &str,
    ) -> Result<Vec<u8>, String> {
        let url = format!(
            "https://{}/ccx/service/{}/{}/{}",
            self.config.host, self.config.tenant, application, version
        );

        let response = ureq::post(&url)
            .set("Content-Type", "text/xml")
            .set(
                "Authorization",
                &format!(
                    "{} {}",
                    self.credentials.token_response.token_type,
                    self.credentials.token_response.access_token
                ),
            )
            .send_string(request)
            .map_err(|e| format!("SOAP request failed: {e}"))?;

        let mut bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to read SOAP response: {e}"))?;
        Ok(bytes)
    }
}
