INSTALL_DIR=~/.config/mircmd/plugins/mircmd/chemistry-molecular-visualizer

.PHONY: init
init:
	cd web && npm install
	cargo install wasm-pack
	cargo install wasm-bindgen-cli --version 0.2.108
	rustup target add wasm32-unknown-unknown

.PHONY: build
build:
	wasm-pack build --target web --release
	cd web && npm run build

.PHONY: install
install:
	rm -rf $(INSTALL_DIR)
	mkdir -p $(INSTALL_DIR)
	cp manifest.yaml $(INSTALL_DIR)/
	cp web/dist/plugin.js $(INSTALL_DIR)/
	cp pkg/molecular_visualizer.js $(INSTALL_DIR)/
	cp pkg/molecular_visualizer_bg.wasm $(INSTALL_DIR)/

.PHONY: clean
clean:
	rm -rf pkg
	rm -rf target
	rm -rf web/node_modules
	rm -rf web/dist
	rm -f *.png

.PHONY: font-atlas
font-atlas:
	cargo run --bin debug_font_atlas --features debug-tools --release
