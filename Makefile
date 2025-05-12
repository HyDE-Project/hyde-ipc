BIN_NAME = hyde-ipc
REL_TGT = target/release/$(BIN_NAME)
DBG_TGT = target/debug/$(BIN_NAME)
BIN_DIR = ./bin
INSTALL_DIR = /usr/bin

.PHONY: release debug clean install
.DEFAULT_GOAL := release

release: clean
	@cargo build --release
	@mkdir -p $(BIN_DIR)
	@cp $(REL_TGT) $(BIN_DIR)/$(BIN_NAME)
	@echo "Built release -> $(BIN_DIR)/$(BIN_NAME)"

debug: clean
	@cargo build
	@mkdir -p $(BIN_DIR)
	@cp $(DBG_TGT) $(BIN_DIR)/debug-$(BIN_NAME)
	@echo "Built debug -> $(BIN_DIR)/debug-$(BIN_NAME)"

clean:
	@cargo clean
	@rm -rf $(BIN_DIR)
	@echo "Cleaned build artifacts and $(BIN_DIR)"

install: release
	@echo "Installing $(BIN_DIR)/$(BIN_NAME) to $(INSTALL_DIR)..."
	@sudo cp $(BIN_DIR)/$(BIN_NAME) $(INSTALL_DIR)/$(BIN_NAME)
	@echo "Installed."
	@$(MAKE) clean


