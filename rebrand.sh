#!/bin/bash
# Goblin Rebrand Script - Replaces Goblin with Goblin throughout the codebase

set -e

echo "🎭 Rebranding Goblin → Goblin..."

cd "$(dirname "$0")"

# Process all text files
echo "Replacing text in files..."
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.json" -o -name "*.yaml" -o -name "*.yml" -o -name "*.sh" -o -name "*.py" -o -name "*.js" -o -name "*.ts" -o -name "*.html" -o -name "*.css" \) ! -path "./.git/*" ! -path "./target/*" -exec sed -i \
    -e 's/Goblin/Goblin/g' \
    -e 's/goblin/goblin/g' \
    -e 's/GOBLIN/GOBLIN/g' \
    -e 's/Goblin-code-evals/goblin-agent-evals/g' \
    -e 's/goblin_code/goblin_agent/g' \
    -e 's/goblin-code/goblin-agent/g' \
    -e 's/goblincode/goblinagent/g' \
    -e 's/GOBLIN_/GOBLIN_/g' \
    {} \;

# Rename config files
echo "Renaming config files..."
[ -f "goblin.default.yaml" ] && mv goblin.default.yaml goblin.default.yaml && echo "  goblin.default.yaml → goblin.default.yaml"
[ -f "goblin.schema.json" ] && mv goblin.schema.json goblin.schema.json && echo "  goblin.schema.json → goblin.schema.json"

# Rename .goblin directory
[ -d ".goblin" ] && mv .goblin .goblin && echo "  .goblin → .goblin"

# Rename crate directories (snake_case)
if [ -d "crates" ]; then
    echo "Renaming crate directories..."
    cd crates
    for dir in goblin_*; do
        if [ -d "$dir" ]; then
            newname=$(echo "$dir" | sed 's/goblin_/goblin_/')
            mv "$dir" "$newname"
            echo "  crates/$dir → crates/$newname"
            
            # Update Cargo.toml inside the crate
            if [ -f "$newname/Cargo.toml" ]; then
                sed -i 's/^name = "goblin_/name = "goblin_/g' "$newname/Cargo.toml"
                sed -i 's/goblin_/goblin_/g' "$newname/Cargo.toml"
                echo "  Updated $newname/Cargo.toml"
            fi
        fi
    done
    cd ..
fi

echo ""
echo "✅ Rebranding complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff --stat"
echo "  2. Test build: cargo build --release"
echo "  3. Commit: git add -A && git commit -m 'rebrand: Goblin → Goblin'"
echo "  4. Push: git push origin main"
