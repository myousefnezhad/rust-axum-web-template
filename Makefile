.PHONY: run_app

APP_DIR := app
WEB_NAME := web
MCP_NAME := mcp
ALI_NAME := agent_cli

clean:
	cd $(APP_DIR) && cargo clean
run_agent:
	cd $(APP_DIR) && cargo run --bin $(ALI_NAME)
build_agent:
	cd $(APP_DIR) && cargo build --bin $(ALI_NAME)
run_web:
	cd $(APP_DIR) && cargo run --bin $(WEB_NAME)
build_web:
	cd $(APP_DIR) && cargo build --bin $(WEB_NAME)
run_mcp:
	cd $(APP_DIR) && cargo run --bin $(MCP_NAME)
build_mcp:
	cd $(APP_DIR) && cargo build --bin $(MCP_NAME)
