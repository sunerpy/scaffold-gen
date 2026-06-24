.PHONY: all build build-dev build-prod release release-target clean install fmt fmt-rust fmt-oxfmt lint check test ci help upx-binaries install-upx fmt-check fmt-rust-check fmt-oxfmt-check hooks

# Project configuration
PROJECT_NAME := scaffold-gen
BINARY_NAME := scafgen
CARGO := cargo

# Tooling detection
PROJECT_ROOT := $(abspath .)
OXFMT_IGNORE := --ignore-path "$(PROJECT_ROOT)/.oxfmtignore"
HAS_OXFMT := $(shell command -v oxfmt 2>/dev/null)
HAS_PRECOMMIT := $(shell command -v pre-commit 2>/dev/null)

# Directories
TARGET_DIR := target
DIST_DIR := dist

# Cross-compilation target (set via environment variable or command line)
# Example: make release-target TARGET=x86_64-unknown-linux-musl
TARGET ?=

# UPX configuration
UPX_BIN := upx

# Rust build flags for release optimization
# Similar to Go's -ldflags="-s -w"
# Note: LTO, opt-level, codegen-units, and strip are configured in Cargo.toml [profile.release]
RUSTFLAGS_RELEASE := -C link-arg=-s

# Default target
all: build

# Build in debug mode
build: build-dev

# Build in debug mode and copy to project root
build-dev:
	@echo "🔨 Building $(PROJECT_NAME) in debug mode..."
	$(CARGO) build
	@echo "📦 Copying binary to project root..."
	@cp $(TARGET_DIR)/debug/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME) 2>/dev/null || \
		cp $(TARGET_DIR)/debug/$(PROJECT_NAME) $(DIST_DIR)/$(BINARY_NAME) 2>/dev/null || \
		echo "⚠️  Binary not found, check Cargo.toml [[bin]] configuration"
	@echo "✅ Debug build complete: $(DIST_DIR)/$(BINARY_NAME)"

# Build in release mode with optimizations and copy to project root
build-prod: release

# Build optimized release binary
release:
	@echo "🚀 Building $(PROJECT_NAME) in release mode with optimizations..."
	$(CARGO) build --release
	@mkdir -p $(DIST_DIR)
	@echo "📦 Copying binary to dist directory..."
	@cp $(TARGET_DIR)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME) 2>/dev/null || \
		cp $(TARGET_DIR)/release/$(PROJECT_NAME) $(DIST_DIR)/$(BINARY_NAME) 2>/dev/null || \
		echo "⚠️  Binary not found, check Cargo.toml [[bin]] configuration"
	@echo "✅ Release build complete: $(DIST_DIR)/$(BINARY_NAME)"
	@ls -lh $(DIST_DIR)/$(BINARY_NAME) 2>/dev/null || true

# Build release binary for specific target (cross-compilation)
# Usage: make release-target TARGET=x86_64-unknown-linux-musl
release-target:
ifndef TARGET
	$(error TARGET is not set. Usage: make release-target TARGET=x86_64-unknown-linux-musl)
endif
	@echo "🚀 Building $(PROJECT_NAME) for target $(TARGET)..."
	$(CARGO) build --release --target $(TARGET)
	@mkdir -p $(DIST_DIR)
	@echo "📦 Copying binary to dist directory..."
	@if [ -f "$(TARGET_DIR)/$(TARGET)/release/$(BINARY_NAME)" ]; then \
		cp $(TARGET_DIR)/$(TARGET)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-$(TARGET); \
	elif [ -f "$(TARGET_DIR)/$(TARGET)/release/$(BINARY_NAME).exe" ]; then \
		cp $(TARGET_DIR)/$(TARGET)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-$(TARGET).exe; \
	else \
		echo "⚠️  Binary not found for target $(TARGET)"; \
		exit 1; \
	fi
	@echo "✅ Release build complete for $(TARGET)"
	@ls -lh $(DIST_DIR)/$(BINARY_NAME)-$(TARGET)* 2>/dev/null || true

# Build release and compress with UPX
release-upx: release upx-binaries
	@echo "✅ Release build with UPX compression complete"

# Install UPX if not available
install-upx:
	@echo "🔍 Checking for UPX..."
	@which $(UPX_BIN) > /dev/null 2>&1 || { \
		echo "📦 Installing UPX..."; \
		if command -v apt-get > /dev/null 2>&1; then \
			sudo apt-get update && sudo apt-get install -y upx-ucl; \
		elif command -v brew > /dev/null 2>&1; then \
			brew install upx; \
		elif command -v pacman > /dev/null 2>&1; then \
			sudo pacman -S upx; \
		else \
			echo "❌ Please install UPX manually: https://github.com/upx/upx/releases"; \
			exit 1; \
		fi; \
	}
	@echo "✅ UPX is available"

# Compress binaries with UPX
upx-binaries: install-upx
	@echo "🗜️  Compressing binaries with UPX..."
	@if [ -f "$(DIST_DIR)/$(BINARY_NAME)" ]; then \
		if $(UPX_BIN) -t "$(DIST_DIR)/$(BINARY_NAME)" >/dev/null 2>&1; then \
			echo "⏭️  Skipping $(BINARY_NAME) (already packed by UPX)"; \
		else \
			echo "🗜️  Compressing $(BINARY_NAME)..."; \
			$(UPX_BIN) -9 "$(DIST_DIR)/$(BINARY_NAME)" || echo "⚠️  Failed to compress $(BINARY_NAME)"; \
		fi; \
		ls -lh "$(DIST_DIR)/$(BINARY_NAME)"; \
	else \
		echo "⚠️  Binary not found: $(DIST_DIR)/$(BINARY_NAME)"; \
	fi

# Install the binary to system
install: release
	@echo "📦 Installing $(BINARY_NAME) to ~/.cargo/bin..."
	$(CARGO) install --path .
	@echo "✅ Installation complete"

# Format code
fmt: fmt-rust fmt-oxfmt
	@echo "✨ Formatting complete"

# Format Rust code
fmt-rust:
	@echo "✨ Formatting Rust code..."
	$(CARGO) fmt

# Format non-Rust files (YAML / JSON / Markdown) with oxfmt
fmt-oxfmt:
ifdef HAS_OXFMT
	@echo "✨ Formatting YAML/JSON/Markdown with oxfmt..."
	@oxfmt --write --no-error-on-unmatched-pattern $(OXFMT_IGNORE) "$(PROJECT_ROOT)"
else
	@echo "⏭️  oxfmt not found, skipping non-Rust formatting (https://github.com/oxc-project/oxfmt)"
endif

# Check code formatting (for CI)
fmt-check: fmt-rust-check fmt-oxfmt-check
	@echo "✨ Format check complete"

# Check Rust formatting
fmt-rust-check:
	@echo "✨ Checking Rust code formatting..."
	$(CARGO) fmt --all -- --check

# Check non-Rust formatting with oxfmt
fmt-oxfmt-check:
ifdef HAS_OXFMT
	@echo "✨ Checking YAML/JSON/Markdown formatting with oxfmt..."
	@oxfmt --check --no-error-on-unmatched-pattern $(OXFMT_IGNORE) "$(PROJECT_ROOT)"
else
	@echo "⏭️  oxfmt not found, skipping non-Rust format check"
endif

# Run linter
lint:
	@echo "🔍 Running linter..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Check code without building
check:
	@echo "✅ Checking code..."
	$(CARGO) check

# Run tests
test:
	@echo "🧪 Running tests..."
	$(CARGO) test

# Run all CI checks
ci: fmt-check lint test
	@echo "✅ All CI checks passed!"

# Install git pre-commit + pre-push hooks
hooks:
ifdef HAS_PRECOMMIT
	@echo "🪝 Installing git hooks (pre-commit + pre-push)..."
	@pre-commit install --hook-type pre-commit --hook-type pre-push
	@echo "✅ Hooks installed"
else
	@echo "❌ pre-commit not found: https://pre-commit.com/#install"
endif

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf $(DIST_DIR)
	@echo "✅ Clean complete"

# Show binary size comparison
size-compare: build-dev
	@echo ""
	@echo "📊 Binary size comparison:"
	@echo "Debug build:"
	@ls -lh $(TARGET_DIR)/debug/$(BINARY_NAME) 2>/dev/null || ls -lh $(TARGET_DIR)/debug/$(PROJECT_NAME) 2>/dev/null || echo "  Not found"
	@echo ""
	@if [ -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ] || [ -f "$(TARGET_DIR)/release/$(PROJECT_NAME)" ]; then \
		echo "Release build:"; \
		ls -lh $(TARGET_DIR)/release/$(BINARY_NAME) 2>/dev/null || ls -lh $(TARGET_DIR)/release/$(PROJECT_NAME) 2>/dev/null; \
	fi

# Show help
help:
	@echo "Available targets:"
	@echo ""
	@echo "  Build targets:"
	@echo "    all          - Build in debug mode (default)"
	@echo "    build        - Build in debug mode"
	@echo "    build-dev    - Build in debug mode and copy to project root"
	@echo "    build-prod   - Build in release mode with optimizations"
	@echo "    release      - Build optimized release binary"
	@echo "    release-upx  - Build release and compress with UPX"
	@echo ""
	@echo "  Installation:"
	@echo "    install      - Install binary to ~/.cargo/bin"
	@echo "    install-upx  - Install UPX compression tool"
	@echo ""
	@echo "  Development:"
	@echo "    fmt          - Format code (Rust + YAML/JSON/Markdown via oxfmt)"
	@echo "    fmt-check    - Check formatting without writing (CI gate)"
	@echo "    lint         - Run linter (clippy)"
	@echo "    check        - Check code without building"
	@echo "    test         - Run tests"
	@echo "    ci           - Run all CI checks"
	@echo "    hooks        - Install git pre-commit + pre-push hooks"
	@echo ""
	@echo "  Utilities:"
	@echo "    clean        - Clean build artifacts"
	@echo "    size-compare - Show binary size comparison"
	@echo "    upx-binaries - Compress binaries with UPX"
	@echo "    help         - Show this help message"
	@echo ""
	@echo "  Cross-compilation:"
	@echo "    release-target TARGET=<target> - Build for specific target"
	@echo "    Example: make release-target TARGET=x86_64-unknown-linux-musl"
	@echo ""
	@echo "  Optimization flags used in release build (configured in Cargo.toml):"
	@echo "    opt-level = 3     - Maximum optimization level"
	@echo "    lto = \"fat\"       - Link-time optimization"
	@echo "    codegen-units = 1 - Single codegen unit for better optimization"
	@echo "    strip = true      - Strip symbols (like Go's -ldflags=\"-s -w\")"
