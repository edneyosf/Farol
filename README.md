# Farol

A simple Rust script that checks your current public IP address and updates (or creates) a Cloudflare `A` DNS record, working as a lightweight DDNS client.

## Usage

### 1. Create a Cloudflare API token

Go to **My Profile → API Tokens → Create Token** in Cloudflare and use the **Edit zone DNS** template, restricting it to the zone (domain) you want to manage.

### 2. Get your Zone ID

In the Cloudflare dashboard, open your domain page. The **Zone ID** is shown in the right sidebar under **Overview**.

### 3. Build

```bash
cargo build --release
```

The binary will be generated at:

```bash
target/release/Farol
```

### 4. Run

You can configure the script using environment variables:

```bash
export CF_API_TOKEN="your_token_here"
export CF_ZONE_ID="your_zone_id"
export CF_RECORD_NAME="home.yourdomain.com"

./target/release/Farol
```

Or use the equivalent command-line flags:

```bash
./target/release/Farol \
  --api-token "your_token_here" \
  --zone-id "your_zone_id" \
  --record-name "home.yourdomain.com" \
  --record-type A \
  --ttl 1
```

Use `--proxied` if you want the record to go through Cloudflare’s proxy (orange cloud). Without this flag, the record will remain **DNS only**.

### 5. Automate it

You can run the script periodically with cron. For example, to check your IP every 5 minutes:

```cron
*/5 * * * * CF_API_TOKEN=xxx CF_ZONE_ID=xxx CF_RECORD_NAME=home.yourdomain.com /path/to/Farol >> /var/log/farol.log 2>&1
```

The script only sends a `PATCH` request to Cloudflare when the IP has actually changed, so it is safe to run frequently.

## How it works

1. Queries `https://api.ipify.org` to determine the current public IP address.
2. Checks Cloudflare for an existing DNS record matching the configured name and type.
3. If the record exists and already points to the current IP, nothing is changed.
4. If the record exists but is outdated, it is updated with a `PATCH` request.
5. If the record does not exist, it is created with a `POST` request.

## Notes about `Cargo.toml`

This build environment uses an older Rust toolchain (`1.75`), so the `Cargo.toml` pins some transitive dependencies (`idna_adapter` and `openssl-sys`) to versions compatible with it.

If you are building with a newer Rust version (`1.80+`), you can safely remove those pinned dependency lines.
