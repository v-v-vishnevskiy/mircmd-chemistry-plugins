INSTALL_DIR=~/.config/mircmd/plugins/mircmd/chemistry-cartesian-editor

.PHONY: init
init:
	npm install

.PHONY: build
build:
	npm run build

.PHONY: install
install:
	rm -rf $(INSTALL_DIR)
	mkdir -p $(INSTALL_DIR)
	cp manifest.yaml $(INSTALL_DIR)/
	cp dist/plugin.js $(INSTALL_DIR)/

.PHONY: clean
clean:
	rm -rf node_modules
	rm -rf dist
