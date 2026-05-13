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

#### `payslips` — Get worker payslips

```bash
workday_cli worker payslips <WID> [LIMIT]
workday_cli worker payslips me        # defaults to 1 payslip
workday_cli worker payslips me 5
```

#### `history` — Get worker history events

```bash
workday_cli worker history <WID> [LIMIT]
workday_cli worker history me         # defaults to 5 events
workday_cli worker history me 20
```

---

### `revenue` — Query revenue data

#### `customer` — Search for a customer by name

```bash
workday_cli revenue customer <NAME>
```

#### `invoices` — Get invoices for a customer

```bash
workday_cli revenue invoices <CUSTOMER_ID> [LIMIT]
workday_cli revenue invoices abc123       # defaults to 1 invoice
workday_cli revenue invoices abc123 25
```

#### `invoice-print` — Get print runs for an invoice

```bash
workday_cli revenue invoice-print <INVOICE_ID> [LIMIT]
workday_cli revenue invoice-print inv123       # defaults to 1
workday_cli revenue invoice-print inv123 5
```

#### `invoice-pdf` — Download an invoice PDF by ID

```bash
workday_cli revenue invoice-pdf <PDF_ID>
workday_cli revenue invoice-pdf <PDF_ID> --output ~/Downloads/invoice.pdf
```

Saves to `invoice_<ID>.pdf` in the current directory by default.

---

### `integrations` — Query integration event data

All integration subcommands query integration events using WQL (Workday Query Language).

#### Status Mapping

When using the `--status` filter, use the status ID from this mapping:

| Status Name | Status ID |
|---|---|
| Completed | `d8b0bcd8446c11de98360015c5e6daf6` |
| Completed With Errors | `d8b0c264446c11de98360015c5e6daf6` |
| Completed with Warnings | `d8b0c34a446c11de98360015c5e6daf6` |
| Failed | `d8b0bdbe446c11de98360015c5e6daf6` |

#### `events` — Get all integration events from the last X days

```bash
workday_cli integrations events 0      # Today's events
workday_cli integrations events 7      # Last 7 days
workday_cli integrations events 30     # Last 30 days
```

#### `events-by-status` — Get summarized events by status for the last X days

Returns count of events grouped by `integrationSystem` and `status`.

```bash
workday_cli integrations events-by-status 0
workday_cli integrations events-by-status 7
```

#### `events-by-month` — Get summarized events for a specific month

- **For month 0 (current)**: Returns events from start of current month to current moment
- **For other months**: Returns events for the entire month (1 = last month, 2 = two months ago, etc.)

Grouped by `integrationSystem` and `status`.

```bash
# Current month
workday_cli integrations events-by-month 0

# Last month
workday_cli integrations events-by-month 1

# Fetch last 6 months (makes 6 separate requests)
workday_cli integrations events-by-month 0 --range 6
```

**With optional status filter:**

```bash
# Filter by status
workday_cli integrations events-by-month 1 --status "'d8b0bcd8446c11de98360015c5e6daf6'"

# Multiple statuses
workday_cli integrations events-by-month 1 --status "'d8b0bcd8446c11de98360015c5e6daf6','d8b0c264446c11de98360015c5e6daf6'"
```

**Response format:**

Each monthly result includes `month`, `year`, and `data` fields:

```json
{
  "month": 4,
  "year": 2026,
  "data": [
    {
      "integrationSystem": {
        "descriptor": "AR2 Migrator System",
        "id": "55e80cd04c5e1000113f191f48530000"
      },
      "status": {
        "descriptor": "Completed",
        "id": "d8b0bcd8446c11de98360015c5e6daf6"
      },
      "count()": "1"
    }
  ]
}
```

---

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
