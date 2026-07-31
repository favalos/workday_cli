# Workday CLI

Rust CLI tool for interacting with the Workday API. Uses OAuth 2.0 authorization code flow with browser-based authentication.

## Build & Run

```bash
cargo build
cargo run -- <command>
```

## Project Structure

```
src/
  main.rs              # Entry point, CLI definition (clap derive)
  config.rs            # Config struct, save/load from ~/.w-cli/config.json
  security.rs          # Keychain credential storage, token refresh logic
  commands/
    mod.rs             # Module declarations
    init.rs            # OAuth init flow: browser auth, HTTPS callback, token exchange
    worker.rs          # Worker API subcommands (details, direct-reports, time-off)
```

## Key Dependencies

- `clap` (derive) — CLI argument parsing
- `tiny_http` (ssl-rustls) — Local HTTPS callback server on port 8889
- `ureq` — Synchronous HTTP client for API calls
- `security-framework` — macOS Keychain for credential storage
- `serde` / `serde_json` — Serialization
- `chrono` — Token expiration tracking

## Architecture Notes

- **Rust 2024 edition** — `gen` is a reserved keyword; avoid using it as a variable name
- **No `url` crate** — URL parsing uses plain string operations (`strip_prefix`, `split`)
- **TLS via mkcert** — Locally-trusted certs stored in `~/.w-cli/`. Requires `brew install mkcert && mkcert -install`
- **Token auto-refresh** — `get_credentials()` in security.rs refreshes tokens when expiration < 1 minute
- **Config stored at** `~/.w-cli/config.json` with host, tenant, client_id, token_url, environment
- **Credentials stored in macOS Keychain** under service `workday_cli`, account `tokens`

## Commands

```bash
# Initialize with OAuth credentials
workday_cli init --auth-url <URL> --token-url <URL> --client-id <ID> --client-secret <SECRET> --environment <ENV>

# Worker subcommands
workday_cli worker <WID> details
workday_cli worker <WID> direct-reports
workday_cli worker <WID> time-off
```
