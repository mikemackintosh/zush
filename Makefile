# Zush Prompt - Makefile
# A Rust-powered Zsh prompt with theme support

# Variables
CARGO := cargo
INSTALL_DIR := $(HOME)/.local/bin
CONFIG_DIR := $(HOME)/.config/zush
THEMES_DIR := $(CONFIG_DIR)/themes
BINARY_NAME := zush-prompt
SOURCE_DIR := $(shell pwd)

# Color output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[1;37m
NC := \033[0m # No Color

# Default target
.PHONY: all
all: build

# Help target
.PHONY: help
help:
	@echo "$(CYAN)Zush Prompt - Make Targets$(NC)"
	@echo ""
	@echo "$(WHITE)Building:$(NC)"
	@echo "  $(GREEN)make build$(NC)       - Build the project in release mode"
	@echo "  $(GREEN)make dev$(NC)         - Build in debug mode"
	@echo "  $(GREEN)make clean$(NC)       - Clean build artifacts"
	@echo ""
	@echo "$(WHITE)Installation:$(NC)"
	@echo "  $(GREEN)make install$(NC)     - Install zush-prompt and themes"
	@echo "  $(GREEN)make uninstall$(NC)   - Remove zush-prompt and themes"
	@echo "  $(GREEN)make themes$(NC)      - Install themes only"
	@echo "  $(GREEN)make config$(NC)      - Install default config"
	@echo ""
	@echo "$(WHITE)Shell Integration:$(NC)"
	@echo "  $(GREEN)make shell$(NC)       - Install Zsh integration"
	@echo "  $(GREEN)make shell-theme$(NC) - Install theme switcher"
	@echo "  $(GREEN)make shell-all$(NC)   - Install complete shell integration"
	@echo ""
	@echo "$(WHITE)Testing:$(NC)"
	@echo "  $(GREEN)make test$(NC)        - Run tests"
	@echo "  $(GREEN)make test-themes$(NC) - Test all themes"
	@echo "  $(GREEN)make demo$(NC)        - Run interactive demo"
	@echo ""
	@echo "$(WHITE)Development:$(NC)"
	@echo "  $(GREEN)make check$(NC)       - Run cargo check"
	@echo "  $(GREEN)make fmt$(NC)         - Format code"
	@echo "  $(GREEN)make clippy$(NC)      - Run clippy lints"

# Build targets
.PHONY: build
build:
	@echo "$(CYAN)Building zush-prompt...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN) Build complete$(NC)"

.PHONY: dev
dev:
	@echo "$(CYAN)Building zush-prompt (debug)...$(NC)"
	@$(CARGO) build
	@echo "$(GREEN) Debug build complete$(NC)"

.PHONY: clean
clean:
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	@$(CARGO) clean
	@echo "$(GREEN) Clean complete$(NC)"

# Installation targets
.PHONY: install
install: build install-binary install-themes install-config install-shell-theme
	@echo ""
	@echo "$(GREEN) Installation complete!$(NC)"
	@echo ""
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "1. Add to your ~/.zshrc:"
	@echo "   $(CYAN)source <(zush-prompt init zsh)$(NC)"
	@echo "   $(CYAN)source $(SOURCE_DIR)/zush-theme.zsh$(NC)"
	@echo ""
	@echo "2. Reload your shell:"
	@echo "   $(CYAN)source ~/.zshrc$(NC)"
	@echo ""
	@echo "3. Try switching themes:"
	@echo "   $(CYAN)zush-theme minimal$(NC)"
	@echo "   $(CYAN)zush-theme powerline$(NC)"
	@echo "   $(CYAN)zush-theme dcs$(NC)"

.PHONY: install-binary
install-binary: build
	@echo "$(CYAN)Installing binary...$(NC)"
	@$(CARGO) install --path . --force
	@echo "$(GREEN) Binary installed to $(INSTALL_DIR)/$(BINARY_NAME)$(NC)"

.PHONY: install-themes
install-themes: themes

.PHONY: themes
themes:
	@echo "$(CYAN)Installing themes...$(NC)"
	@mkdir -p $(THEMES_DIR)
	@# Only copy theme files, not Cargo.toml or config.example.toml
	@for theme in dcs.toml minimal.toml powerline.toml; do \
		if [ -f "$(SOURCE_DIR)/$$theme" ]; then \
			cp -v "$(SOURCE_DIR)/$$theme" "$(THEMES_DIR)/" 2>/dev/null || true; \
		fi; \
	done
	@if [ -d "$(SOURCE_DIR)/themes" ]; then \
		cp -v $(SOURCE_DIR)/themes/*.toml $(THEMES_DIR)/ 2>/dev/null || true; \
	fi
	@if [ -f "$(THEMES_DIR)/README.md" ]; then \
		rm $(THEMES_DIR)/README.md 2>/dev/null || true; \
	fi
	@if [ -f "$(SOURCE_DIR)/themes/README.md" ]; then \
		cp $(SOURCE_DIR)/themes/README.md $(THEMES_DIR)/; \
	fi
	@echo "$(GREEN) Themes installed to $(THEMES_DIR)$(NC)"

.PHONY: install-config
install-config: config

.PHONY: config
config:
	@echo "$(CYAN)Installing default config...$(NC)"
	@mkdir -p $(CONFIG_DIR)
	@if [ ! -f "$(CONFIG_DIR)/config.toml" ]; then \
		echo '# Zush Prompt Configuration' > $(CONFIG_DIR)/config.toml; \
		echo 'theme = "dcs"  # Options: dcs, minimal, powerline' >> $(CONFIG_DIR)/config.toml; \
		echo '' >> $(CONFIG_DIR)/config.toml; \
		echo '[settings]' >> $(CONFIG_DIR)/config.toml; \
		echo 'true_color = true' >> $(CONFIG_DIR)/config.toml; \
		echo 'powerline_fonts = true' >> $(CONFIG_DIR)/config.toml; \
		echo "$(GREEN) Created default config at $(CONFIG_DIR)/config.toml$(NC)"; \
	else \
		echo "$(YELLOW)� Config already exists at $(CONFIG_DIR)/config.toml$(NC)"; \
	fi

.PHONY: install-shell-theme
install-shell-theme: shell-theme

.PHONY: shell-theme
shell-theme:
	@echo "$(CYAN)Installing theme switcher...$(NC)"
	@echo "$(YELLOW)Add this to your ~/.zshrc:$(NC)"
	@echo "  source $(SOURCE_DIR)/zush-theme.zsh"
	@echo "$(GREEN) Theme switcher ready$(NC)"

.PHONY: shell
shell:
	@echo "$(CYAN)Setting up Zsh integration...$(NC)"
	@echo "$(YELLOW)Add this to your ~/.zshrc:$(NC)"
	@echo "  source <($(BINARY_NAME) init zsh)"
	@echo "$(GREEN) Shell integration ready$(NC)"

.PHONY: shell-all
shell-all: shell shell-theme
	@echo ""
	@echo "$(GREEN) Complete shell integration ready$(NC)"
	@echo "$(YELLOW)Add both lines to your ~/.zshrc:$(NC)"
	@echo "  source <($(BINARY_NAME) init zsh)"
	@echo "  source $(SOURCE_DIR)/zush-theme.zsh"

# Uninstall
.PHONY: uninstall
uninstall:
	@echo "$(CYAN)Uninstalling zush-prompt...$(NC)"
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(YELLOW)Note: Config and themes preserved in $(CONFIG_DIR)$(NC)"
	@echo "$(YELLOW)To completely remove, run: rm -rf $(CONFIG_DIR)$(NC)"
	@echo "$(GREEN) Uninstall complete$(NC)"

# Testing targets
.PHONY: test
test:
	@echo "$(CYAN)Running tests...$(NC)"
	@$(CARGO) test
	@echo "$(GREEN) Tests complete$(NC)"

.PHONY: test-themes
test-themes:
	@echo "$(CYAN)Testing themes...$(NC)"
	@if [ -f "./test_themes.sh" ]; then \
		bash ./test_themes.sh; \
	else \
		echo "$(RED) test_themes.sh not found$(NC)"; \
	fi

.PHONY: demo
demo:
	@echo "$(CYAN)Running interactive demo...$(NC)"
	@if [ -f "./demo_theme_switch.sh" ]; then \
		bash ./demo_theme_switch.sh; \
	else \
		echo "$(YELLOW)Demo: Switching themes$(NC)"; \
		echo ""; \
		echo "DCS theme:"; \
		$(BINARY_NAME) --theme dcs --format raw prompt --context '{"pwd":"~/demo","user":"demo","git_branch":"main"}' --exit-code 0; \
		echo ""; \
		echo "Minimal theme:"; \
		$(BINARY_NAME) --theme minimal --format raw prompt --context '{"pwd":"~/demo","user":"demo","git_branch":"main"}' --exit-code 0; \
		echo ""; \
		echo "Powerline theme:"; \
		$(BINARY_NAME) --theme powerline --format raw prompt --context '{"pwd":"~/demo","user":"demo","git_branch":"main"}' --exit-code 0; \
	fi

# Development targets
.PHONY: check
check:
	@echo "$(CYAN)Running cargo check...$(NC)"
	@$(CARGO) check
	@echo "$(GREEN) Check complete$(NC)"

.PHONY: fmt
fmt:
	@echo "$(CYAN)Formatting code...$(NC)"
	@$(CARGO) fmt
	@echo "$(GREEN) Format complete$(NC)"

.PHONY: clippy
clippy:
	@echo "$(CYAN)Running clippy...$(NC)"
	@$(CARGO) clippy -- -W clippy::all
	@echo "$(GREEN) Clippy complete$(NC)"

# Quick setup for development
.PHONY: setup
setup: build install-themes install-config
	@echo "$(GREEN) Development setup complete$(NC)"

# Version info
.PHONY: version
version:
	@echo "$(CYAN)Zush Prompt Version Info$(NC)"
	@$(CARGO) version
	@echo "Binary: $(BINARY_NAME)"
	@if [ -f "$(INSTALL_DIR)/$(BINARY_NAME)" ]; then \
		echo "Installed: $(GREEN)Yes$(NC) - $(INSTALL_DIR)/$(BINARY_NAME)"; \
	else \
		echo "Installed: $(RED)No$(NC)"; \
	fi
	@echo "Config: $(CONFIG_DIR)/config.toml"
	@echo "Themes: $(THEMES_DIR)/"

# Watch for changes during development
.PHONY: watch
watch:
	@echo "$(CYAN)Watching for changes...$(NC)"
	@$(CARGO) watch -x build

.DEFAULT_GOAL := help