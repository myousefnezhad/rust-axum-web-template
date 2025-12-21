.PHONY: run_app

APP_DIR := app
BIN_NAME := app

run_app:
	cd $(APP_DIR) && cargo run --bin $(BIN_NAME)
