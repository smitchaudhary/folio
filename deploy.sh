#!/bin/bash
# Folio server deploy script (jj-native — ~/folio is a colocated jj repo).
# Run on the VPS as root:  ssh vps-ts "./deploy.sh"
set -euo pipefail

REPO=/home/webuser/folio
HEALTH_URL=http://100.88.189.63:8787/api/health

# 1. Rollback point: save the binary currently in use.
cp "$REPO/target/release/folio-server" "$REPO/target/release/folio-server.bak"

# 2. Pull latest code with jj, move the working copy onto the remote main,
#    build the server only.
su - webuser -c "cd $REPO && export PATH=\$HOME/.local/bin:\$PATH && \
  jj pull && \
  jj new main@origin && \
  ~/.cargo/bin/cargo build --release -p folio-server"

# 3. Restart the service (needs root).
systemctl restart folio

# 4. Verify it is alive before declaring success.
sleep 2
curl -fsS "$HEALTH_URL" && echo "deploy OK"
