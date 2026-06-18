# cc-truenas-drivetemp

`cc-truenas-drivetemp` is a Rust CoolerControl plugin that exposes TrueNAS drive temperatures as a native read-only CoolerControl device.

The plugin polls TrueNAS with the current JSON-RPC WebSocket API and serves CoolerControl's device gRPC API over a Unix domain socket. CoolerControl then sees a virtual device named `TrueNAS Drive Temperature` with temperature channels for each drive plus aggregate channels for max, min, and average drive temperature.

TrueNAS only refreshes disk temperatures every 5 minutes, while CoolerControl polls plugin device status frequently. The plugin therefore keeps an in-memory cache and answers CoolerControl status requests from that cache.

## Channels

The device exposes:

- `drive_sda`, `drive_nvme0n1`, etc. for individual drives.
- `aggregate_max` for the hottest selected drive.
- `aggregate_min` for the coolest selected drive.
- `aggregate_avg` for the arithmetic average.

## Requirements

- Linux host running CoolerControl.
- Rust 1.88 or newer to build from source.
- `protobuf-compiler` / `protoc`.
- Network access from the CoolerControl host to TrueNAS.
- A TrueNAS user-linked API key.
- TrueNAS roles:
  - `REPORTING_READ` for `disk.temperatures`.
  - `DISK_READ` if you want automatic disk discovery through `disk.query`.

Use `wss://.../api/current` with valid TLS whenever possible. TrueNAS revokes API keys used over insecure transport.

## Build And Test

```bash
cargo test
cargo build --release
```

## Manual TrueNAS Test

Before deploying, you can run a live test against your TrueNAS instance:

Create `.env` from example:

```bash
cp .env.example .env
```

Edit `.env`:

```bash
TRUENAS_URL=wss://truenas.example.lan/api/current
TRUENAS_USERNAME=your-truenas-username
TRUENAS_API_KEY=your-truenas-api-key
TRUENAS_VERIFY_TLS=true
TRUENAS_TIMEOUT_SECONDS=20
TRUENAS_DISKS=
```

Then run:

```bash
cargo test manual_query_truenas_temperatures_from_dotenv -- --ignored --nocapture
```

Leave `TRUENAS_DISKS` empty to test automatic disk discovery through `disk.query`. Set it to a comma-separated list like `sda,sdb,nvme0n1` to skip discovery and test only those disks.

## Local Run

To quickly setup and run the plugin manually, you can run:

```bash
cp config.example.toml config.toml
printf '%s\n' 'your-truenas-api-key' > truenas-api-key
chmod 600 truenas-api-key
cargo run -- --config ./config.toml --socket /tmp/cc-truenas-drivetemp.sock
```

## Install Into CoolerControl

To easily install the plugin into CoolerControl, you can run the provided install script:

```bash
sudo sh scripts/install.sh
```

Temporarily write your API key to a local file:

```bash
printf '%s\n' 'your-truenas-api-key' > truenas-api-key
```

Install the API key so the unprivileged CoolerControl plugin user can read it:

```bash
sudo install -m 0600 -o cc-plugin-user -g cc-plugin-user \
  ./truenas-api-key \
  /etc/coolercontrol/plugins/cc-truenas-drivetemp/truenas-api-key
```

Edit the config:

```bash
sudo nano /etc/coolercontrol/plugins/cc-truenas-drivetemp/config.toml
```

Restart CoolerControl:

```bash
sudo systemctl restart coolercontrold
```

Check logs:

```bash
journalctl -u cc-plugin-cc-truenas-drivetemp -f
```

If everything went well, it should now be up and running!

## Configuration

`config.example.toml` contains every supported option.

| Key | Meaning |
| --- | --- |
| `poll_interval_seconds` | Poll interval. Default is `300`. |
| `disks` | Explicit disk names such as `["sda", "nvme0n1"]`. If set, the plugin polls exactly these names and does not need disk discovery. |
| `exclude_disks` | Disk names to skip when `disks` is empty and the plugin discovers disks through `disk.query`. |
| `failsafe_aggregate_max` | On polling failure, update/create `aggregate_max` with this Celsius value. Omit it to keep the previous cached temperatures. |
| `temp_min` / `temp_max` | Profile range metadata reported to CoolerControl. |
| `truenas.url` | WebSocket API URL, usually `wss://host/api/current`. |
| `truenas.username` | Username associated with the API key. Required for newer TrueNAS login flows. |
| `truenas.api_key_file` | File containing the raw API key. Relative paths resolve from the config file directory. |
| `truenas.api_key_env` | Environment variable containing the raw API key. Defaults to `TRUENAS_API_KEY`. |
| `truenas.api_key` | Inline API key. Works, but avoid it for long-lived configs. |
| `truenas.verify_tls` | Verify TLS certificates for `wss://` connections. Set `false` only for controlled test systems. |
| `truenas.timeout_seconds` | Timeout for TrueNAS connection and JSON-RPC calls. |

Disk selection has two modes. Leave `disks` empty to discover disks through TrueNAS and optionally skip names with `exclude_disks`. Set `disks` to poll exactly those names; in that mode `exclude_disks` is ignored.

## Failure Behavior

If a poll succeeds but TrueNAS reports no temperature for a disk, that disk is omitted from the current status and the plugin logs a warning.

If the whole poll fails and `failsafe_aggregate_max` is set, the plugin updates or creates `aggregate_max` with that value. The example config uses `70.0`, which is intentionally conservative for fan-control use.

If `failsafe_aggregate_max` is omitted, failed polls keep the previous cached status.
