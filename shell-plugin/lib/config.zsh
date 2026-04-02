#!/usr/bin/env zsh

# Configuration variables for Goblin plugin
# Using typeset to keep variables local to plugin scope and prevent public exposure

typeset -h _GOBLIN_BIN="${GOBLIN_BIN:-Goblin}"
typeset -h _GOBLIN_CONVERSATION_PATTERN=":"
typeset -h _GOBLIN_MAX_COMMIT_DIFF="${GOBLIN_MAX_COMMIT_DIFF:-100000}"
typeset -h _GOBLIN_DELIMITER='\s\s+'
typeset -h _GOBLIN_PREVIEW_WINDOW="--preview-window=bottom:75%:wrap:border-sharp"

# Detect fd command - Ubuntu/Debian use 'fdfind', others use 'fd'
typeset -h _GOBLIN_FD_CMD="$(command -v fdfind 2>/dev/null || command -v fd 2>/dev/null || echo 'fd')"

# Detect bat command - use bat if available, otherwise fall back to cat
if command -v bat &>/dev/null; then
    typeset -h _GOBLIN_CAT_CMD="bat --color=always --style=numbers,changes --line-range=:500"
else
    typeset -h _GOBLIN_CAT_CMD="cat"
fi

# Commands cache - loaded lazily on first use
typeset -h _GOBLIN_COMMANDS=""

# Hidden variables to be used only via the GoblinCLI
typeset -h _GOBLIN_CONVERSATION_ID
typeset -h _GOBLIN_ACTIVE_AGENT

# Previous conversation ID for :conversation - (like cd -)
typeset -h _GOBLIN_PREVIOUS_CONVERSATION_ID

# Session-scoped model and provider overrides (set via :model / :m).
# When non-empty, these are passed as --model / --provider to every Goblin
# invocation for the lifetime of the current shell session.
typeset -h _GOBLIN_SESSION_MODEL
typeset -h _GOBLIN_SESSION_PROVIDER
