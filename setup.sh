#!/bin/bash
# Goblin Setup Script
# The self-improving AI coding agent

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "                    GOBLIN SETUP WIZARD"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Step 1: Check Rust
echo -e "${BLUE}Step 1:${NC} Checking Rust installation..."
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓${NC} $RUST_VERSION"
else
    echo -e "${RED}✗${NC} Rust not found!"
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo -e "${GREEN}✓${NC} Rust installed"
fi
echo ""

# Step 2: Clone/Fetch Goblin
echo -e "${BLUE}Step 2:${NC} Setting up Goblin repository..."
GOBLIN_DIR="${GOBLIN_DIR:-$HOME/goblin}"
if [ -d "$GOBLIN_DIR" ]; then
    echo -e "${GREEN}✓${NC} Goblin found at $GOBLIN_DIR"
    cd "$GOBLIN_DIR"
    echo "Updating repository..."
    git pull origin main 2>/dev/null || true
else
    echo "Cloning Goblin repository..."
    git clone https://github.com/iangogentic/goblin-agent.git "$GOBLIN_DIR"
    cd "$GOBLIN_DIR"
fi
echo ""

# Step 3: Configure API Keys
echo -e "${BLUE}Step 3:${NC} API Key Configuration..."
echo ""
echo "Enter your MiniMax API key (or press Enter to skip):"
read -r MINIMAX_KEY

echo ""
echo "Enter your OpenRouter API key (optional, for Claude/GPT access):"
read -r OPENROUTER_KEY

# Create credentials file
mkdir -p ~/.goblin
cat > ~/.goblin/.credentials.json << 'CRED'
{
  "minimax": {
    "api_key": "MINIMAX_KEY_PLACEHOLDER"
  },
  "open_router": {
    "api_key": "OPENROUTER_KEY_PLACEHOLDER"
  }
}
CRED

# Replace placeholders
if [ -n "$MINIMAX_KEY" ]; then
    sed -i "s/MINIMAX_KEY_PLACEHOLDER/$MINIMAX_KEY/g" ~/.goblin/.credentials.json
fi
if [ -n "$OPENROUTER_KEY" ]; then
    sed -i "s/OPENROUTER_KEY_PLACEHOLDER/$OPENROUTER_KEY/g" ~/.goblin/.credentials.json
fi

echo -e "${GREEN}✓${NC} Credentials saved to ~/.goblin/.credentials.json"
echo ""

# Step 4: Configure Default Model
echo -e "${BLUE}Step 4:${NC} Default Model Configuration..."
echo ""
echo "Available model configurations:"
echo "  1) MiniMax 2.7B (fast, local-style)"
echo "  2) MiniMax 2.5B (balanced)"
echo "  3) Claude via OpenRouter (powerful)"
echo "  4) Custom configuration"
echo ""
read -p "Select model (1-4, default 1): " MODEL_CHOICE

case $MODEL_CHOICE in
    2)
        MODEL_ID="minimax-minimax-2.5b"
        ;;
    3)
        MODEL_ID="anthropic/claude-3-5-sonnet"
        PROVIDER="open_router"
        ;;
    4)
        echo "Enter provider ID (e.g., minimax, open_router, anthropic):"
        read -r PROVIDER
        echo "Enter model ID (e.g., minimax-minimax-2.7b):"
        read -r MODEL_ID
        ;;
    *)
        MODEL_ID="minimax-minimax-2.7b"
        PROVIDER="minimax"
        ;;
esac

cat > ~/.goblin/.config.json << CONFIG
{
  "model": {
    "provider_id": "${PROVIDER:-minimax}",
    "model_id": "$MODEL_ID"
  },
  "max_tokens": 8192,
  "top_p": 0.95,
  "temperature": 0.7,
  "max_tool_failure_per_turn": 3,
  "tool_supported": true,
  "compact": {
    "max_tokens": 2000,
    "token_threshold": 100000,
    "retention_window": 6,
    "message_threshold": 200,
    "eviction_window": 0.2,
    "on_turn_end": false
  }
}
CONFIG

echo -e "${GREEN}✓${NC} Configuration saved to ~/.goblin/.config.json"
echo ""

# Step 5: Build
echo -e "${BLUE}Step 5:${NC} Building Goblin..."
echo "This may take 10-15 minutes on first build..."
echo ""

cd "$GOBLIN_DIR"
cargo build --release 2>&1 | tail -20

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ BUILD SUCCESSFUL!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "To install Goblin globally, run:"
    echo "  sudo cp target/release/goblin /usr/local/bin/"
    echo ""
    echo "Or add to PATH:"
    echo "  export PATH=\"\$PATH:$GOBLIN_DIR/target/release\""
    echo ""
    echo "Then run Goblin with:"
    echo "  goblin"
    echo ""
else
    echo ""
    echo -e "${RED}✗ BUILD FAILED${NC}"
    echo "Check the errors above and try again."
    exit 1
fi
