BINARY_NAME=xui-offline-builder
TEST_DIR=tests
TEST_BIN=$(TEST_DIR)/$(BINARY_NAME)_test

.PHONY: all build test testAll clean

all: build

build:
	@echo "Building for Linux..."
	cargo build

test: build
	@echo "Preparing test environment..."
	mkdir -p $(TEST_DIR)
	cp target/debug/$(BINARY_NAME) $(TEST_BIN)
	@echo "Running test binary..."
	./$(TEST_BIN)

testAll:
	@echo "========================================"
	@echo "🚀 Starting Comprehensive Test Suite..."
	@echo "========================================"
	@GIT_STATUS=$$(git status --porcelain); \
	if [ -z "$$GIT_STATUS" ]; then \
		echo "✅ [1/3] Git Status: Clean"; \
		GIT_RES="✅"; \
	else \
		echo "⚠️  [1/3] Git Status: Dirty (uncommitted changes exist)"; \
		GIT_RES="⚠️ "; \
	fi; \
	echo "⏳ [2/3] Checking codebase (cargo check & build)..."; \
	if cargo check && cargo build; then \
		echo "✅ [2/3] Build Status: Success"; \
		BUILD_RES="✅"; \
	else \
		echo "❌ [2/3] Build Status: FAILED"; \
		BUILD_RES="❌"; \
		echo "========================================"; \
		echo "📊 COMPREHENSIVE REPORT"; \
		echo "========================================"; \
		echo "$$GIT_RES Git Status"; \
		echo "$$BUILD_RES Build & Compilation"; \
		echo "➖ Package Mirror Availability (Skipped)"; \
		exit 1; \
	fi; \
	echo "⏳ [3/3] Running package mirror tests..."; \
	if ./target/debug/$(BINARY_NAME) --test-packages; then \
		echo "✅ [3/3] Packages Status: Success"; \
		PKG_RES="✅"; \
	else \
		echo "❌ [3/3] Packages Status: FAILED"; \
		PKG_RES="❌"; \
		echo "========================================"; \
		echo "📊 COMPREHENSIVE REPORT"; \
		echo "========================================"; \
		echo "$$GIT_RES Git Status"; \
		echo "$$BUILD_RES Build & Compilation"; \
		echo "$$PKG_RES Package Mirror Availability"; \
		exit 1; \
	fi; \
	echo "========================================"; \
	echo "📊 COMPREHENSIVE REPORT"; \
	echo "========================================"; \
	echo "$$GIT_RES Git Status"; \
	echo "$$BUILD_RES Build & Compilation"; \
	echo "$$PKG_RES Package Mirror Availability"; \
	echo "========================================"

clean:
	cargo clean
	rm -rf $(TEST_DIR)
