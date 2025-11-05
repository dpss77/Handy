#!/bin/bash

# Verify that all println! statements have been converted to log macros

set -e

echo "╔════════════════════════════════════════════╗"
echo "║     Logging Consolidation Verification     ║"
echo "╚════════════════════════════════════════════╝"
echo

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if we're in the right directory
if [ ! -d "src-tauri" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Count println! statements in source files (excluding tests and examples)
echo "Checking for println! statements..."
PRINTLN_COUNT=$(grep -r "println!" src-tauri/src/ --include="*.rs" | grep -v "//.*println!" | wc -l)

# Count eprintln! statements
echo "Checking for eprintln! statements..."
EPRINTLN_COUNT=$(grep -r "eprintln!" src-tauri/src/ --include="*.rs" | grep -v "//.*eprintln!" | wc -l)

# Count print! statements
echo "Checking for print! statements..."
PRINT_COUNT=$(grep -r "print!" src-tauri/src/ --include="*.rs" | grep -v "println!" | grep -v "eprint!" | grep -v "//.*print!" | wc -l)

TOTAL_VIOLATIONS=$((PRINTLN_COUNT + EPRINTLN_COUNT + PRINT_COUNT))

echo
echo "Results:"
echo "--------"
echo "println!  statements: $PRINTLN_COUNT"
echo "eprintln! statements: $EPRINTLN_COUNT"
echo "print!    statements: $PRINT_COUNT"
echo "Total violations:     $TOTAL_VIOLATIONS"
echo

if [ $TOTAL_VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}✓ PASS${NC} - No raw print statements found"
    echo "All logging uses the log crate (log::info!, log::error!, etc.)"
    exit 0
else
    echo -e "${RED}✗ FAIL${NC} - Found $TOTAL_VIOLATIONS raw print statement(s)"
    echo
    echo "Locations:"
    echo "----------"

    if [ $PRINTLN_COUNT -gt 0 ]; then
        echo
        echo "println! statements:"
        grep -rn "println!" src-tauri/src/ --include="*.rs" | grep -v "//.*println!" | head -20
    fi

    if [ $EPRINTLN_COUNT -gt 0 ]; then
        echo
        echo "eprintln! statements:"
        grep -rn "eprintln!" src-tauri/src/ --include="*.rs" | grep -v "//.*eprintln!" | head -20
    fi

    if [ $PRINT_COUNT -gt 0 ]; then
        echo
        echo "print! statements:"
        grep -rn "print!" src-tauri/src/ --include="*.rs" | grep -v "println!" | grep -v "eprint!" | grep -v "//.*print!" | head -20
    fi

    echo
    echo "Recommendation:"
    echo "Replace with appropriate log macros:"
    echo "  println!(\"Info: {}\", x)   → log::info!(\"Info: {}\", x)"
    echo "  eprintln!(\"Error: {}\", e) → log::error!(\"Error: {}\", e)"
    echo "  println!(\"Debug: {}\", d)  → log::debug!(\"Debug: {}\", d)"

    exit 1
fi
