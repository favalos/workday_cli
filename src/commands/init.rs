use std::fs;

use clap::Args;
use tiny_http::{Response, Server, SslConfig};

use crate::config::{self, Config};
use crate::security;

const CALLBACK_URL: &str = "https://localhost:8889/callback";

#[derive(Args)]
pub struct InitArgs {
    ///Workday Authentication URL
    #[arg(long)]
    pub auth_url: String,

    #[arg(long)]
    pub token_url: String,

    /// Workday API client ID
    #[arg(long)]
    pub client_id: String,

    /// Workday API client secret
    #[arg(long)]
    pub client_secret: String,

    /// Workday environment (e.g. sandbox, production)
    #[arg(long)]
    pub environment: String,
}

fn extract_host(url: &str) -> Result<(&str, &str), String> {
    let without_scheme = url
        .strip_prefix("https://")
        .ok_or("Invalid URL: missing http(s):// scheme")?;

    let (host, path) = without_scheme
        .split_once('/')
        .ok_or("Invalid URL: no path after host")?;

    let tenant = path
        .split('/')
        .rev()
        .nth(1)
        .ok_or("Invalid URL: cannot extract tenant from path")?;

    Ok((host, tenant))
}

fn build_and_save_config(args: &InitArgs) -> Result<(), String> {
    let host = extract_host(&args.token_url)?;

    let cfg = Config {
        host: host.0.to_string(),
        tenant: host.1.to_string(),
        client_id: args.client_id.clone(),
        token_url: args.token_url.clone(),
        environment: args.environment.clone(),
    };

    config::save_config(&cfg)
}

fn exchange_code(args: &InitArgs, code: &str) -> Result<security::TokenResponse, String> {
    let config = config::load_config().expect("Config does not exist.");

    let response = ureq::post(&config.token_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&format!(
            "client_id={}&client_secret={}&grant_type=authorization_code&code={}",
            args.client_id, args.client_secret, code
        ))
        .map_err(|e| format!("Token request failed: {e}"))?;

    let body = response
        .into_string()
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    serde_json::from_str::<security::TokenResponse>(&body)
        .map_err(|e| format!("Failed to parse token response: {e}\nBody: {body}"))
}

fn setup_tls() -> Result<SslConfig, String> {
    let dir = config::config_path()?;
    let cert_path = dir.join("localhost.pem");
    let key_path = dir.join("localhost-key.pem");

    if cert_path.exists() && key_path.exists() {
        let cert = fs::read(&cert_path).map_err(|e| format!("Failed to read certificate: {e}"))?;
        let key = fs::read(&key_path).map_err(|e| format!("Failed to read private key: {e}"))?;
        return Ok(SslConfig {
            certificate: cert,
            private_key: key,
        });
    }

    return Err(
        "mkcert is not installed. Install it with: brew install mkcert\n\
             Then run: mkcert -install"
            .to_string(),
    );
}

pub fn execute(args: &InitArgs) {
    let ssl_config = match setup_tls() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    match build_and_save_config(args) {
        Ok(()) => println!("Config saved to ~/.w-cli/config.json"),
        Err(e) => eprintln!("{e}"),
    }

    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code",
        args.auth_url, args.client_id, CALLBACK_URL
    );

    println!("Opening browser for authentication...");
    if let Err(e) = open::that(&auth_url) {
        eprintln!("Failed to open browser: {e}");
        eprintln!("Please open this URL manually:\n  {auth_url}");
    }

    println!("Waiting for callback on {CALLBACK_URL}...");
    let server = match Server::https("127.0.0.1:8889", ssl_config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to start HTTPS server on port 8889: {e}");
            return;
        }
    };

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if !url.starts_with("/callback") {
            let _ = request.respond(Response::from_string("Not found").with_status_code(404));
            continue;
        }

        let code = url.split('?').nth(1).and_then(|query| {
            query
                .split('&')
                .find(|param| param.starts_with("code="))
                .map(|param| param.trim_start_matches("code=").to_string())
        });

        let html = "<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
        let response = Response::from_string(html).with_header(
            "Content-Type: text/html"
                .parse::<tiny_http::Header>()
                .unwrap(),
        );
        let _ = request.respond(response);

        match code {
            Some(code) => {
                println!("Received authorization code, exchanging for tokens...");
                match exchange_code(args, &code) {
                    Ok(token) => {
                        println!("Token type:    {}", token.token_type);
                        match security::store_credentials(args, token) {
                            Ok(()) => println!("Tokens stored in credential store."),
                            Err(e) => eprintln!("{e}"),
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to exchange code for tokens: {e}");
                    }
                }
            }
            None => {
                eprintln!("No authorization code found in callback");
            }
        }

        break;
    }
}
