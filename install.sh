#!/bin/bash

set -e

BINARY_NAME="solaris"
INSTALL_PATH="/usr/local/bin/$BINARY_NAME"
SERVICE_PATH="/etc/systemd/system/$BINARY_NAME.service"
SERVICE_USER=$(whoami)

build() {
    echo "Building $BINARY_NAME..."
    cargo build --release
}

install_binary() {
    echo "Installing binary to $INSTALL_PATH..."
    sudo cp "target/release/$BINARY_NAME" "$INSTALL_PATH"
    sudo chmod +x "$INSTALL_PATH"
}

create_service() {
    echo "Creating systemd service..."
    sudo tee "$SERVICE_PATH" > /dev/null <<SERVICE
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
SERVICE
    sudo systemctl daemon-reload
    sudo systemctl enable "$BINARY_NAME"
}

case "${1:-install}" in
    install)
        build
        install_binary
        create_service
        sudo systemctl start "$BINARY_NAME"
        echo "Done. Check status with: systemctl status $BINARY_NAME"
        ;;
    update)
        build
        install_binary
        sudo systemctl restart "$BINARY_NAME"
        echo "Updated. Check status with: systemctl status $BINARY_NAME"
        ;;
    *)
        echo "Usage: $0 [install|update]"
        exit 1
        ;;
esac
