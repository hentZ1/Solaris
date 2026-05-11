#!/bin/bash

set -e

BINARY_NAME="solaris"
INSTALL_PATH="/usr/local/bin/$BINARY_NAME"
SERVICE_PATH="/etc/systemd/system/$BINARY_NAME.service"
SERVICE_USER=$(whoami)

echo "Building $BINARY_NAME..."
cargo build --release

echo "Installing binary to $INSTALL_PATH..."
sudo cp "target/release/$BINARY_NAME" "$INSTALL_PATH"
sudo chmod +x "$INSTALL_PATH"

echo "Creating systemd service..."
sudo tee "$SERVICE_PATH" >/dev/null <<EOF
[Unit]
Description=Solaris filesystem automation daemon
After=default.target

[Service]
ExecStart=$INSTALL_PATH
Restart=on-failure
RestartSec=5
User=$SERVICE_USER

[Install]
WantedBy=default.target
EOF

echo "Enabling and starting service..."
sudo systemctl daemon-reload
sudo systemctl enable "$BINARY_NAME"
sudo systemctl start "$BINARY_NAME"

echo "Done. Check status with: systemctl status $BINARY_NAME"
