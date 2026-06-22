use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Duration, Local};
use keyring_core::api::CredentialStoreApi;
use keyring_core::{CredentialStore, Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};

use crate::{commands::init::InitArgs, config};

const CREDENTIAL_SERVICE: &str = "workday_cli";
const CREDENTIAL_ACCOUNT: &str = "tokens";
const MISSING_CREDENTIALS_MESSAGE: &str = "Credentials not found. Run 'init' first.";

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyStore {
    pub token_response: TokenResponse,
    pub expiration_time: DateTime<Local>,
    pub client_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CredentialBackendError {
    Missing,
    Unavailable(String),
}

impl fmt::Display for CredentialBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialBackendError::Missing => f.write_str(MISSING_CREDENTIALS_MESSAGE),
            CredentialBackendError::Unavailable(message) => f.write_str(message),
        }
    }
}

trait CredentialBackend {
    fn set(&self, value: &str) -> Result<(), CredentialBackendError>;
    fn get(&self) -> Result<String, CredentialBackendError>;
}

struct SystemCredentialBackend;

struct SystemStore {
    name: &'static str,
    store: Arc<CredentialStore>,
}

impl SystemStore {
    fn entry(&self) -> Result<Entry, KeyringError> {
        self.store
            .build(CREDENTIAL_SERVICE, CREDENTIAL_ACCOUNT, None)
    }
}

impl CredentialBackend for SystemCredentialBackend {
    fn set(&self, value: &str) -> Result<(), CredentialBackendError> {
        let (stores, mut failures) = system_stores();

        for store in stores {
            match store.entry().and_then(|entry| entry.set_password(value)) {
                Ok(()) => return Ok(()),
                Err(error) => failures.push(format!("{}: {error}", store.name)),
            }
        }

        Err(CredentialBackendError::Unavailable(format!(
            "No credential store could save credentials. {}",
            format_failures(&failures)
        )))
    }

    fn get(&self) -> Result<String, CredentialBackendError> {
        let (stores, init_failures) = system_stores();
        let mut read_failures = Vec::new();
        let mut saw_missing = false;

        for store in stores {
            match store.entry().and_then(|entry| entry.get_password()) {
                Ok(value) => return Ok(value),
                Err(KeyringError::NoEntry) => saw_missing = true,
                Err(error) => read_failures.push(format!("{}: {error}", store.name)),
            }
        }

        if init_failures.is_empty() && read_failures.is_empty() && saw_missing {
            return Err(CredentialBackendError::Missing);
        }

        let mut failures = init_failures;
        failures.extend(read_failures);

        Err(CredentialBackendError::Unavailable(format!(
            "No credential store could read credentials. {}",
            format_failures(&failures)
        )))
    }
}

fn system_stores() -> (Vec<SystemStore>, Vec<String>) {
    let mut stores = Vec::new();
    let mut failures = Vec::new();

    add_platform_stores(&mut stores, &mut failures);

    if stores.is_empty() && failures.is_empty() {
        failures.push("unsupported platform".to_string());
    }

    (stores, failures)
}

fn push_store<T>(
    stores: &mut Vec<SystemStore>,
    failures: &mut Vec<String>,
    name: &'static str,
    store: keyring_core::Result<Arc<T>>,
) where
    T: CredentialStoreApi + Send + Sync + 'static,
{
    match store {
        Ok(store) => stores.push(SystemStore { name, store }),
        Err(error) => failures.push(format!("{name}: {error}")),
    }
}

#[cfg(target_os = "macos")]
fn add_platform_stores(stores: &mut Vec<SystemStore>, failures: &mut Vec<String>) {
    push_store(
        stores,
        failures,
        "macOS Keychain",
        apple_native_keyring_store::keychain::Store::new(),
    );
}

#[cfg(target_os = "windows")]
fn add_platform_stores(stores: &mut Vec<SystemStore>, failures: &mut Vec<String>) {
    push_store(
        stores,
        failures,
        "Windows Credential Manager",
        windows_native_keyring_store::Store::new(),
    );
}

#[cfg(target_os = "linux")]
fn add_platform_stores(stores: &mut Vec<SystemStore>, failures: &mut Vec<String>) {
    push_store(
        stores,
        failures,
        "Secret Service",
        zbus_secret_service_keyring_store::Store::new(),
    );
    push_store(
        stores,
        failures,
        "Linux keyutils",
        linux_keyutils_keyring_store::Store::new(),
    );
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn add_platform_stores(_stores: &mut Vec<SystemStore>, failures: &mut Vec<String>) {
    failures.push("no credential backend is configured for this platform".to_string());
}

fn format_failures(failures: &[String]) -> String {
    if failures.is_empty() {
        "No credential stores were available.".to_string()
    } else {
        failures.join("; ")
    }
}

pub fn store_credentials(args: &InitArgs, token_response: TokenResponse) -> Result<(), String> {
    store_credentials_with_backend(&SystemCredentialBackend, args, token_response)
}

fn store_credentials_with_backend<B: CredentialBackend>(
    backend: &B,
    args: &InitArgs,
    token_response: TokenResponse,
) -> Result<(), String> {
    let key_store = KeyStore {
        token_response,
        expiration_time: Local::now() + Duration::hours(1),
        client_secret: args.client_secret.clone(),
    };

    save_key_store(backend, &key_store, "store")
}

pub fn get_credentials() -> Result<KeyStore, String> {
    get_credentials_with_backend(&SystemCredentialBackend)
}

fn get_credentials_with_backend<B: CredentialBackend>(backend: &B) -> Result<KeyStore, String> {
    let key_store = load_key_store(backend)?;

    let time_remaining = key_store.expiration_time - Local::now();
    if time_remaining < Duration::minutes(1) {
        let refreshed = refresh_token(&key_store)?;
        update_credentials_with_backend(backend, key_store.client_secret, refreshed)?;
        return load_key_store(backend);
    }

    Ok(key_store)
}

fn load_key_store<B: CredentialBackend>(backend: &B) -> Result<KeyStore, String> {
    let json = match backend.get() {
        Ok(value) => value,
        Err(CredentialBackendError::Missing) => return Err(MISSING_CREDENTIALS_MESSAGE.to_string()),
        Err(error) => {
            return Err(format!(
                "Failed to read credentials from credential store: {error}"
            ));
        }
    };

    serde_json::from_str::<KeyStore>(&json).map_err(|e| format!("Failed to parse credentials: {e}"))
}

fn save_key_store<B: CredentialBackend>(
    backend: &B,
    key_store: &KeyStore,
    action: &str,
) -> Result<(), String> {
    let json =
        serde_json::to_string(key_store).map_err(|e| format!("Failed to serialize token: {e}"))?;

    backend
        .set(&json)
        .map_err(|e| format!("Failed to {action} credentials in credential store: {e}"))
}

fn refresh_token(key_store: &KeyStore) -> Result<TokenResponse, String> {
    let cfg = config::load_config()?;

    let response = ureq::post(&cfg.token_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!(
            "client_id={}&client_secret={}&grant_type=refresh_token&refresh_token={}",
            cfg.client_id, key_store.client_secret, key_store.token_response.refresh_token
        ))
        .map_err(|e| format!("Token refresh failed: {e}"))?;

    let body = response
        .into_string()
        .map_err(|e| format!("Failed to read refresh response: {e}"))?;

    serde_json::from_str::<TokenResponse>(&body)
        .map_err(|e| format!("Failed to parse refresh response: {e}\nBody: {body}"))
}

pub fn update_credentials(
    client_secret: String,
    token_response: TokenResponse,
) -> Result<(), String> {
    update_credentials_with_backend(&SystemCredentialBackend, client_secret, token_response)
}

fn update_credentials_with_backend<B: CredentialBackend>(
    backend: &B,
    client_secret: String,
    token_response: TokenResponse,
) -> Result<(), String> {
    let key_store = KeyStore {
        token_response,
        expiration_time: Local::now() + Duration::hours(1),
        client_secret,
    };

    save_key_store(backend, &key_store, "update")
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct FakeCredentialBackend {
        stored: Mutex<Option<String>>,
        get_error: Mutex<Option<CredentialBackendError>>,
        set_error: Mutex<Option<CredentialBackendError>>,
    }

    impl FakeCredentialBackend {
        fn set_stored(&self, value: &str) {
            *self.stored.lock().unwrap() = Some(value.to_string());
        }

        fn fail_next_get(&self, error: CredentialBackendError) {
            *self.get_error.lock().unwrap() = Some(error);
        }

        fn fail_next_set(&self, error: CredentialBackendError) {
            *self.set_error.lock().unwrap() = Some(error);
        }
    }

    impl CredentialBackend for FakeCredentialBackend {
        fn set(&self, value: &str) -> Result<(), CredentialBackendError> {
            if let Some(error) = self.set_error.lock().unwrap().take() {
                return Err(error);
            }
            *self.stored.lock().unwrap() = Some(value.to_string());
            Ok(())
        }

        fn get(&self) -> Result<String, CredentialBackendError> {
            if let Some(error) = self.get_error.lock().unwrap().take() {
                return Err(error);
            }
            self.stored
                .lock()
                .unwrap()
                .clone()
                .ok_or(CredentialBackendError::Missing)
        }
    }

    fn init_args(client_secret: &str) -> InitArgs {
        InitArgs {
            auth_url: "https://example.com/tenant/authorize".to_string(),
            token_url: "https://example.com/tenant/token".to_string(),
            client_id: "client-id".to_string(),
            client_secret: client_secret.to_string(),
            environment: "sandbox".to_string(),
        }
    }

    fn token(access_token: &str, refresh_token: &str) -> TokenResponse {
        TokenResponse {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            token_type: "Bearer".to_string(),
        }
    }

    #[test]
    fn store_and_get_credentials_round_trip() {
        let backend = FakeCredentialBackend::default();

        store_credentials_with_backend(
            &backend,
            &init_args("secret-1"),
            token("access-1", "refresh-1"),
        )
        .unwrap();
        let stored = get_credentials_with_backend(&backend).unwrap();

        assert_eq!(stored.client_secret, "secret-1");
        assert_eq!(stored.token_response.access_token, "access-1");
        assert_eq!(stored.token_response.refresh_token, "refresh-1");
        assert_eq!(stored.token_response.token_type, "Bearer");
        assert!(stored.expiration_time > Local::now());
    }

    #[test]
    fn update_credentials_replaces_existing_value() {
        let backend = FakeCredentialBackend::default();

        store_credentials_with_backend(
            &backend,
            &init_args("secret-1"),
            token("access-1", "refresh-1"),
        )
        .unwrap();
        update_credentials_with_backend(
            &backend,
            "secret-2".to_string(),
            token("access-2", "refresh-2"),
        )
        .unwrap();
        let stored = get_credentials_with_backend(&backend).unwrap();

        assert_eq!(stored.client_secret, "secret-2");
        assert_eq!(stored.token_response.access_token, "access-2");
        assert_eq!(stored.token_response.refresh_token, "refresh-2");
    }

    #[test]
    fn missing_credentials_keep_existing_user_message() {
        let backend = FakeCredentialBackend::default();

        let error = get_credentials_with_backend(&backend).unwrap_err();

        assert_eq!(error, MISSING_CREDENTIALS_MESSAGE);
    }

    #[test]
    fn malformed_stored_json_is_reported() {
        let backend = FakeCredentialBackend::default();
        backend.set_stored("not json");

        let error = get_credentials_with_backend(&backend).unwrap_err();

        assert!(error.starts_with("Failed to parse credentials:"));
    }

    #[test]
    fn set_backend_failures_include_action_context() {
        let backend = FakeCredentialBackend::default();
        backend.fail_next_set(CredentialBackendError::Unavailable(
            "backend down".to_string(),
        ));

        let error = store_credentials_with_backend(
            &backend,
            &init_args("secret"),
            token("access", "refresh"),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "Failed to store credentials in credential store: backend down"
        );
    }

    #[test]
    fn get_backend_failures_include_read_context() {
        let backend = FakeCredentialBackend::default();
        backend.fail_next_get(CredentialBackendError::Unavailable(
            "backend down".to_string(),
        ));

        let error = get_credentials_with_backend(&backend).unwrap_err();

        assert_eq!(
            error,
            "Failed to read credentials from credential store: backend down"
        );
    }
}
