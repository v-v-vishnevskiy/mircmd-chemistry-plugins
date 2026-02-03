COLOUR_GREEN=\033[0;32m
END_COLOUR=\033[0m

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: init
init: ## Install dependencies for all plugins
	@echo "$(COLOUR_GREEN)Initializing Cartesian Editor...$(END_COLOUR)"
	@make -C cartesian-editor init

	@echo "$(COLOUR_GREEN)Initializing Files Importer...$(END_COLOUR)"
	@make -C files-importer init

	@echo "$(COLOUR_GREEN)Initializing Molecular Visualizer...$(END_COLOUR)"
	@make -C molecular-visualizer init

.PHONY: build
build:  ## Build all plugins
	@echo "$(COLOUR_GREEN)Building Cartesian Editor...$(END_COLOUR)"
	@make -C cartesian-editor build

	@echo "$(COLOUR_GREEN)Building Files Importer...$(END_COLOUR)"
	@make -C files-importer build

	@echo "$(COLOUR_GREEN)Building Molecular Visualizer...$(END_COLOUR)"
	@make -C molecular-visualizer build

.PHONY: install
install:  ## Copy all plugins to Mir Commander's plugins directory
	@echo "$(COLOUR_GREEN)Installing Cartesian Editor...$(END_COLOUR)"
	@make -C cartesian-editor install

	@echo "$(COLOUR_GREEN)Installing Files Importer...$(END_COLOUR)"
	@make -C files-importer install

	@echo "$(COLOUR_GREEN)Installing Molecular Visualizer...$(END_COLOUR)"
	@make -C molecular-visualizer install

	@echo "$(COLOUR_GREEN)Installing Object Icons...$(END_COLOUR)"
	@make -C object-icons install

.PHONY: clean
clean:  ## Clean up all plugins from build files
	@rm -rf target
	@make -C cartesian-editor clean
	@make -C files-importer clean
	@make -C molecular-visualizer clean
