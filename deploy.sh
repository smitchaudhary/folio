#!/bin/bash
# Folio server deploy script.
# Run on the VPS as root:  ssh vps-ts "./deploy.sh"
# Pulls the latest code from GitHub, rebuilds the server only,
# restarts the service, and verifies it is alive.
set -euo pipefail

REPO=/home/webuser/folio
HEALTH_URL=http://100.88.189.63:8787/api/health

# 1. Rollback point: save the binary currently in use.
cp "$REPO/target/release/folio-server" "$REPO/target/release/folio-server.bak"

# 2. Pull latest code + rebuild (as webuser, server only).
su - webuser -c "cd $REPO && git pull --ff-only && ~/.cargo/bin/cargo build --release -p folio-server"

# 3. Restart the service (needs root).
systemctl restart folio

# 4. Verify it is alive before declaring success.
sleep 2
curl -fsS "$HEALTH_URL" && echo "deploy OK"
