.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: init
init: ## Install dependencies
	@cargo install wasm-pack
	@rustup target add wasm32-unknown-unknown
	@cd molecular-visualizer/web && npm install

.PHONY: build
build:
	@cd cartesian-editor && npm run build
	@cargo component build --target=wasm32-wasip2 --release
	@mkdir -p dist/chemistry-files-importer \
		&& cp target/wasm32-wasip2/release/files_importer.wasm dist/chemistry-files-importer/plugin.wasm
	@cd molecular-visualizer && wasm-pack build --target web --release
	@cd molecular-visualizer/web && npm run build
	@cp molecular-visualizer/pkg/molecular_visualizer.js dist/chemistry-molecular-visualizer/
	@cp molecular-visualizer/pkg/molecular_visualizer_bg.wasm dist/chemistry-molecular-visualizer/

.PHONY: copy
copy:
	@mkdir -p ~/.config/mircmd/plugins/mircmd/
	@cp -r dist/* ~/.config/mircmd/plugins/mircmd/
	@cp cartesian-editor/manifest.yaml ~/.config/mircmd/plugins/mircmd/chemistry-cartesian-editor/
	@cp files-importer/manifest.yaml ~/.config/mircmd/plugins/mircmd/chemistry-files-importer
	@cp molecular-visualizer/manifest.yaml ~/.config/mircmd/plugins/mircmd/chemistry-molecular-visualizer
	@mkdir -p ~/.config/mircmd/plugins/mircmd/chemistry-object-icons && \
		cp -r object-icons/* ~/.config/mircmd/plugins/mircmd/chemistry-object-icons

.PHONY: clean
clean:  ## Clean up the project
	@cargo clean
	@rm -rf dist
	@rm -rf cartesian-editor/node_modules
	@rm files-importer/src/bindings.rs
	@rm molecular-visualizer/src/bindings.rs
