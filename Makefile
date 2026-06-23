BINARY_NAME := solaris
INSTALL_PATH := /usr/local/bin/$(BINARY_NAME)
SERVICE_PATH := /etc/systemd/system/$(BINARY_NAME).service
SERVICE_USER := $(or $(SUDO_USER),$(shell whoami))

.PHONY: all build release install update uninstall run test clean help

all: build

build:
	cargo build

release:
	cargo build --release

install: release
	sudo install -m755 target/release/$(BINARY_NAME) $(INSTALL_PATH)
	printf '%s\n' \
		'[Unit]' \
		'Description=Solaris filesystem automation daemon' \
		'After=default.target' \
		'' \
		'[Service]' \
		'ExecStart=$(INSTALL_PATH)' \
		'Restart=on-failure' \
		'RestartSec=5' \
		'User=$(SERVICE_USER)' \
		'Type=forking' \
		'PIDFile=/tmp/solaris.pid' \
		'' \
		'[Install]' \
		'WantedBy=default.target' \
		| sudo tee $(SERVICE_PATH) > /dev/null
	sudo systemctl daemon-reload
	sudo systemctl enable $(BINARY_NAME)
	sudo systemctl start $(BINARY_NAME)
	@echo "Done. Check status with: systemctl status $(BINARY_NAME)"

update: release
	sudo install -m755 target/release/$(BINARY_NAME) $(INSTALL_PATH)
	sudo systemctl restart $(BINARY_NAME)
	@echo "Updated. Check status with: systemctl status $(BINARY_NAME)"

uninstall:
	sudo systemctl stop $(BINARY_NAME) 2>/dev/null || true
	sudo systemctl disable $(BINARY_NAME) 2>/dev/null || true
	sudo rm -f $(INSTALL_PATH) $(SERVICE_PATH)
	sudo systemctl daemon-reload
	@echo "$(BINARY_NAME) removed."

run: build
	cargo run -- $(ARGS)

test:
	cargo test

clean:
	cargo clean

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  all       Build the project (default)"
	@echo "  build     Build the debug binary"
	@echo "  release   Build the release binary"
	@echo "  install   Build release and install binary + systemd service"
	@echo "  update    Build release and update binary, restart service"
	@echo "  uninstall Remove the binary, service, and config"
	@echo "  run       Run the binary (pass args: make run ARGS='--help')"
	@echo "  test      Run all tests"
	@echo "  clean     Remove build artifacts"
