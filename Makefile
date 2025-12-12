.PHONY: help storylets storylets-verbose clean-storylets test-storylets validate-storylets

help:
	@echo "SYN Build Targets"
	@echo ""
	@echo "  make storylets           - Compile storylets from JSON to binary"
	@echo "  make storylets-verbose   - Compile storylets with verbose output"
	@echo "  make validate-storylets  - Validate storylet JSON files"
	@echo "  make clean-storylets     - Remove compiled storylet binary"
	@echo "  make test-storylets      - Run storylet integration tests"

# Compile all storylets from JSON to binary format
storylets:
	@echo "Compiling storylets..."
	@./build_storylets.sh

# Compile storylets with verbose output
storylets-verbose:
	@echo "Compiling storylets (verbose)..."
	@./build_storylets.sh --verbose

# Validate storylet JSON files without compiling
validate-storylets:
	@echo "Validating storylets..."
	@cd rust && cargo run --bin storyletc -- \
		--input ../storylets \
		--output /tmp/storylets_validation.bin \
		--verbose
	@rm -f /tmp/storylets_validation.bin
	@echo "✓ All storylets are valid"

# Clean compiled storylet binary
clean-storylets:
	@echo "Cleaning compiled storylets..."
	@rm -f rust/syn_director/data/storylets.bin
	@echo "✓ Cleaned"

# Run storylet integration tests
test-storylets:
	@echo "Running storylet integration tests..."
	@cd rust && cargo test --test storylet_source_integration
	@cd rust/syn_director && cargo test storylet
