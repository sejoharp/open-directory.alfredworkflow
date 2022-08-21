.DEFAULT_GOAL:= help

.PHONY:dependencies 
dependencies: ## download and install dependencies
	cargo update
	cargo build

.PHONY:test 
test: ## executes tests
	cargo test

.PHONY: package
package: ## creates alfred workflow binary
	cargo install powerpack-cli
	~/.cargo/bin/powerpack package

.PHONY: install 
install: package ## builds and installs workflow in alfred
	open target/workflow/open-directory.alfredworkflow

.PHONY: help	
help: ## shows help message
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m\033[0m\n"} /^[$$()% a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)