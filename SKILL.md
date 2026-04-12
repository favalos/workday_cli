# Workday CLI — Skills & Usage Guide

## Commands

### `worker` — Query worker data

All worker subcommands accept a WID (Worker ID) or `me` to reference the currently authenticated user.

#### `details` — Get worker profile

```bash
workday_cli worker details <WID>
workday_cli worker details me
```

#### `direct-reports` — Get a worker's direct reports

```bash
workday_cli worker direct-reports <WID>
workday_cli worker direct-reports me
```

#### `time-off` — Get a worker's time-off information

```bash
workday_cli worker time-off <WID>
workday_cli worker time-off me
```

#### `search-worker` — Search workers by name

```bash
workday_cli worker search-worker <NAME>
```

---

## Troubleshooting

### "Config not found. Run 'init' first."

You haven't initialized the CLI yet. Run the `init` command with your Workday credentials:

```bash
workday_cli init --auth-url <URL> --token-url <URL> --client-id <ID> --client-secret <SECRET> --environment <ENV>
```

Config is stored at `~/.w-cli/config.json`.

### "Credentials not found. Run 'init' first."

The macOS Keychain has no stored tokens. Re-run `workday_cli init` to authenticate and store credentials.

### "mkcert is not installed"

The HTTPS callback server requires locally-trusted certificates. Install mkcert and set up the local CA:

```bash
brew install mkcert
mkcert -install
```

Then re-run `workday_cli init`. Certificates will be generated automatically in `~/.w-cli/`.

### "Failed to start HTTPS server on port 8889"

Another process is already using port 8889. Find and stop it:

```bash
lsof -i :8889
kill <PID>
```

### "Token refresh failed" / "Failed to parse refresh response"

The refresh token has expired or been revoked. Re-run `workday_cli init` to re-authenticate.

### "Request failed" on worker commands

Possible causes:
- **Expired session** — credentials may have fully expired. Re-run `workday_cli init`.
- **Wrong environment** — verify the `environment` value in `~/.w-cli/config.json` matches your Workday tenant.
- **Network issues** — ensure you can reach your Workday host: `curl -I https://<host>`.

### Browser doesn't open during init

If the browser fails to launch automatically, the CLI prints the auth URL to the terminal. Copy and open it manually.

### Certificate errors in the browser

If the browser shows a certificate warning on `https://localhost:8889/callback`, the local CA is not installed. Run:

```bash
mkcert -install
```

Then delete the old certs and re-init:

```bash
rm ~/.w-cli/localhost.pem ~/.w-cli/localhost-key.pem
workday_cli init ...
```

### Keychain access prompts

macOS may prompt you to allow `workday_cli` access to the Keychain. Click "Always Allow" to avoid repeated prompts.
