#!/usr/bin/env sh
set -eu

PLUGINS_DIR="${PLUGINS_DIR:-/etc/coolercontrol/plugins}"
SERVICE_ID="cc-truenas-drivetemp"
EXECUTABLE="cc-truenas-drivetemp"
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

cd "$REPO_DIR"
cargo build --release

sudo mkdir -p "${PLUGINS_DIR}/${SERVICE_ID}"
sudo install -m755 "target/release/${EXECUTABLE}" "${PLUGINS_DIR}/${SERVICE_ID}/"
sudo install -m644 "plugin-files/manifest.toml" "${PLUGINS_DIR}/${SERVICE_ID}/"

if [ ! -f "${PLUGINS_DIR}/${SERVICE_ID}/config.toml" ]; then
  sudo install -m640 "config.example.toml" "${PLUGINS_DIR}/${SERVICE_ID}/config.toml"
fi

if id cc-plugin-user >/dev/null 2>&1; then
  sudo chown cc-plugin-user:cc-plugin-user "${PLUGINS_DIR}/${SERVICE_ID}/config.toml"
fi

cat <<EOF
Installed ${SERVICE_ID} to:
  ${PLUGINS_DIR}/${SERVICE_ID}

Next steps:
  1. Put your API key in:
     ${PLUGINS_DIR}/${SERVICE_ID}/truenas-api-key
  2. Edit:
     ${PLUGINS_DIR}/${SERVICE_ID}/config.toml
  3. Restart CoolerControl:
     sudo systemctl restart coolercontrold
EOF
