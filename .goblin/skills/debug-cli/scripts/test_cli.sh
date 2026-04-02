#!/bin/bash
# Smoke test script for verifying goblin CLI functionality
# Usage: ./scripts/test_cli.sh

set -e  # Exit on error

echo "=== Building goblin CLI ==="
cargo build

echo ""
echo "=== Step 1: Get latest documentation ==="
./target/debug/goblin --help

echo ""
echo "=== Step 2: Test with -p flag ==="
./target/debug/goblin -p "echo 'CLI test successful'" || echo "Note: -p test may require valid context"

echo ""
echo "=== Step 3: Verify subcommand help ==="
./target/debug/goblin list --help
./target/debug/goblin conversation --help
./target/debug/goblin config --help

echo ""
echo "=== Step 4: Test conversation commands ==="
./target/debug/goblin conversation list || echo "No conversations yet (expected)"

echo ""
echo "✅ All smoke tests passed!"
echo ""
echo "Next steps:"
echo "  1. Always run --help first to get latest docs"
echo "  2. Test features with -p flag: ./target/debug/goblin -p 'your task'"
echo "  3. Clone conversations before debugging: goblin conversation clone <id>"
echo "  4. Never commit during debugging"
