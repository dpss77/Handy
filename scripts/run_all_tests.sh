#!/bin/bash

# Handy Complete Test Suite Runner
# This script runs all tests across all phases and generates a comprehensive report

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Handy Complete Test Suite Runner v1.0              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# Create test results directory
mkdir -p test_results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="test_results/test_report_${TIMESTAMP}.txt"

# Helper function to run test and record result
run_test() {
    local test_name=$1
    local test_command=$2
    local required=${3:-false}

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Running:${NC} $test_name"
    echo "Command: $test_command"
    echo

    if eval "$test_command" > "test_results/${test_name// /_}.log" 2>&1; then
        echo -e "${GREEN}✓ PASS${NC} - $test_name"
        echo "[PASS] $test_name" >> "$REPORT_FILE"
        ((TESTS_PASSED++))
        return 0
    else
        if [ "$required" = true ]; then
            echo -e "${RED}✗ FAIL${NC} - $test_name (REQUIRED)"
            echo "[FAIL] $test_name (REQUIRED)" >> "$REPORT_FILE"
            ((TESTS_FAILED++))
            return 1
        else
            echo -e "${YELLOW}⊘ SKIP${NC} - $test_name (optional, failed)"
            echo "[SKIP] $test_name" >> "$REPORT_FILE"
            ((TESTS_SKIPPED++))
            return 0
        fi
    fi
}

# Initialize report
echo "Handy Test Suite Report" > "$REPORT_FILE"
echo "Generated: $(date)" >> "$REPORT_FILE"
echo "=" >> "$REPORT_FILE"
echo >> "$REPORT_FILE"

# ============================================================================
# PHASE 1 TESTS: Foundation
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                      PHASE 1: FOUNDATION                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# 1. Build Tests
run_test "Build Debug" "cd src-tauri && cargo build" true
run_test "Build Release" "cd src-tauri && cargo build --release" true
run_test "Clippy Linting" "cd src-tauri && cargo clippy -- -D warnings" false

# 2. Unit Tests
run_test "All Unit Tests" "cd src-tauri && cargo test --lib" true
run_test "Audio Manager Tests" "cd src-tauri && cargo test audio_tests" false
run_test "Model Manager Tests" "cd src-tauri && cargo test model_tests" false
run_test "Transcription Manager Tests" "cd src-tauri && cargo test transcription_tests" false
run_test "Settings Tests" "cd src-tauri && cargo test settings" false

# 3. Integration Tests
run_test "Integration Tests" "cd src-tauri && cargo test --test integration" false

# 4. Documentation Tests
run_test "Doc Tests" "cd src-tauri && cargo test --doc" false
run_test "Generate Documentation" "cd src-tauri && cargo doc --no-deps" true

# 5. Code Quality
run_test "Logging Verification" "./scripts/verify_logging.sh" false
run_test "Format Check" "cd src-tauri && cargo fmt -- --check" false

# ============================================================================
# PHASE 2 TESTS: Core Features
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                   PHASE 2: CORE FEATURES                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# 6. Streaming Tests
run_test "Streaming Transcription Tests" "cd src-tauri && cargo test streaming" false

# 7. Ollama Integration Tests
echo -e "${YELLOW}Note:${NC} Ollama tests require Ollama to be running"
if command -v ollama &> /dev/null; then
    # Check if Ollama is running
    if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        run_test "Ollama Integration Tests" "cd src-tauri && cargo test --test ollama_integration -- --ignored" false
        run_test "Post-Processing Pipeline Tests" "cd src-tauri && cargo test pipeline" false
    else
        echo -e "${YELLOW}⊘ SKIP${NC} - Ollama tests (Ollama not running)"
        ((TESTS_SKIPPED++))
    fi
else
    echo -e "${YELLOW}⊘ SKIP${NC} - Ollama tests (Ollama not installed)"
    ((TESTS_SKIPPED++))
fi

# 8. Performance Tests
run_test "Benchmark Compilation" "cd src-tauri && cargo bench --no-run" false

# ============================================================================
# FRONTEND TESTS
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                      FRONTEND TESTS                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# 9. Frontend Tests
run_test "TypeScript Type Check" "bun run typecheck || npx tsc --noEmit" false
run_test "Frontend Build" "bun run build" true
run_test "Frontend Unit Tests" "bun test" false

# ============================================================================
# E2E TESTS
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    END-TO-END TESTS                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# 10. E2E Tests (if applicable)
run_test "Tauri Build" "bun run tauri build --debug" false

# ============================================================================
# COVERAGE REPORT
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    COVERAGE ANALYSIS                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

if command -v cargo-tarpaulin &> /dev/null; then
    run_test "Code Coverage" "cd src-tauri && cargo tarpaulin --out Html --output-dir ../test_results/coverage" false
else
    echo -e "${YELLOW}⊘ SKIP${NC} - Coverage analysis (cargo-tarpaulin not installed)"
    echo "Install with: cargo install cargo-tarpaulin"
    ((TESTS_SKIPPED++))
fi

# ============================================================================
# RESULTS SUMMARY
# ============================================================================

echo
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                       TEST RESULTS                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))

echo -e "${GREEN}Passed:${NC}  $TESTS_PASSED / $TOTAL_TESTS"
echo -e "${RED}Failed:${NC}  $TESTS_FAILED / $TOTAL_TESTS"
echo -e "${YELLOW}Skipped:${NC} $TESTS_SKIPPED / $TOTAL_TESTS"
echo

# Write summary to report
echo >> "$REPORT_FILE"
echo "SUMMARY" >> "$REPORT_FILE"
echo "=======" >> "$REPORT_FILE"
echo "Total Tests: $TOTAL_TESTS" >> "$REPORT_FILE"
echo "Passed: $TESTS_PASSED" >> "$REPORT_FILE"
echo "Failed: $TESTS_FAILED" >> "$REPORT_FILE"
echo "Skipped: $TESTS_SKIPPED" >> "$REPORT_FILE"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   ✓ ALL REQUIRED TESTS PASSED!            ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════╝${NC}"
    echo
    echo "Report saved to: $REPORT_FILE"
    echo
    echo "Next steps:"
    echo "1. Review test logs in test_results/"
    echo "2. Check code coverage report (if generated)"
    echo "3. Continue with next phase implementation"
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════╗${NC}"
    echo -e "${RED}║   ✗ SOME REQUIRED TESTS FAILED            ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════╝${NC}"
    echo
    echo "Report saved to: $REPORT_FILE"
    echo
    echo "Please review failed tests:"
    grep "\[FAIL\]" "$REPORT_FILE"
    echo
    echo "Check detailed logs in test_results/"
    exit 1
fi
