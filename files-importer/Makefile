INSTALL_DIR=~/.config/mircmd/plugins/mircmd/chemistry-files-importer

.PHONY: init
init:
	rustup target add wasm32-wasip2

.PHONY: build
build:
	cargo component build --target=wasm32-wasip2 --release

.PHONY: install
install:
	rm -rf $(INSTALL_DIR)
	mkdir -p $(INSTALL_DIR)
	cp manifest.yaml $(INSTALL_DIR)/
	cp target/wasm32-wasip2/release/files_importer.wasm $(INSTALL_DIR)/plugin.wasm

.PHONY: clean
clean:
	cargo clean
	rm -f src/bindings.rs
