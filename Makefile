.PHONY: run_app

APP_DIR := app
BIN_NAME := app

run:
	cd $(APP_DIR) && cargo run --bin $(BIN_NAME)
clean:
	cd $(APP_DIR) && cargo clean
build:
	cd $(APP_DIR) && cargo build --bin $(BIN_NAME)
