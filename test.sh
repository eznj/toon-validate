#!/bin/bash

# Test script for tval (TOON Validator)

set -e

TVAL="./target/release/tval"

# Check if binary exists
if [ ! -f "$TVAL" ]; then
    echo "Error: tval binary not found. Please run cargo build --release first"
    exit 1
fi

echo "============================="
echo "Testing tval (TOON Validator)"
echo "============================="
echo ""

echo "1. Testing analyze command with TOON file:"
echo "-------------------------------------------"
$TVAL analyze tests/example.toon
echo ""

echo "2. Testing analyze command with JSON file:"
echo "-------------------------------------------"
$TVAL analyze tests/example.json
echo ""

echo "3. Testing analyze with JSON output:"
echo "-------------------------------------"
$TVAL analyze tests/example.toon --json
echo ""

echo "4. Testing check command with valid file:"
echo "------------------------------------------"
$TVAL check tests/example.toon
echo ""

echo "5. Testing check command with invalid file (should exit with code 2):"
echo "----------------------------------------------------------------------"
$TVAL check tests/invalid.toon || echo "Exit code: $?"
echo ""

echo "6. Testing profile command:"
echo "----------------------------"
$TVAL profile tests/
echo ""

echo "7. Testing profile with specific extension:"
echo "--------------------------------------------"
$TVAL profile tests/ --ext toon
echo ""

echo "8. Testing help:"
echo "----------------"
$TVAL --help
echo ""

echo "All tests completed!"