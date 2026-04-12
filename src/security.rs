use chrono::{DateTime, Duration, Local};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use serde::{Deserialize, Serialize};

use crate::{commands::init::InitArgs, config};

const KEYCHAIN_SERVICE: &str = "workday_cli";
const KEYCHAIN_ACCOUNT: &str = "tokens";

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

pub fn store_credentials(args: &InitArgs, token_response: TokenResponse) -> Result<(), String> {
    let expiration_time = Local::now() + Duration::hours(1);

    let key_store = KeyStore {
        token_response,
        expiration_time,
        client_secret: args.client_secret.clone(),
    };

    let json =
        serde_json::to_string(&key_store).map_err(|e| format!("Failed to serialize token: {e}"))?;

    // Delete existing entry if present (ignore errors if it doesn't exist)
    let _ = delete_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);

    set_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT, json.as_bytes())
        .map_err(|e| format!("Failed to store credentials in keychain: {e}"))
}

pub fn get_credentials() -> Result<KeyStore, String> {
    let bytes = get_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|_| "Credentials not found. Run 'init' first.".to_string())?;

    let json = String::from_utf8(bytes)
        .map_err(|e| format!("Failed to read credentials from keychain: {e}"))?;

    let key_store: KeyStore =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse credentials: {e}"))?;

    let time_remaining = key_store.expiration_time - Local::now();
    if time_remaining < Duration::minutes(1) {
        let refreshed = refresh_token(&key_store)?;
        update_credentials(key_store.client_secret, refreshed)?;
        return get_stored_credentials();
    }

    Ok(key_store)
}

fn get_stored_credentials() -> Result<KeyStore, String> {
    let bytes = get_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .map_err(|_| "Credentials not found. Run 'init' first.".to_string())?;

    let json = String::from_utf8(bytes)
        .map_err(|e| format!("Failed to read credentials from keychain: {e}"))?;

    serde_json::from_str::<KeyStore>(&json).map_err(|e| format!("Failed to parse credentials: {e}"))
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
    // Delete existing entry
    let _ = delete_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT);

    let expiration_time = Local::now() + Duration::hours(1);

    let key_store = KeyStore {
        token_response,
        expiration_time,
        client_secret,
    };

    let json =
        serde_json::to_string(&key_store).map_err(|e| format!("Failed to serialize token: {e}"))?;

    set_generic_password(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT, json.as_bytes())
        .map_err(|e| format!("Failed to update credentials in keychain: {e}"))
}
