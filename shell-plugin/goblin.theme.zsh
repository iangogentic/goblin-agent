#!/usr/bin/env zsh

# Enable prompt substitution for RPROMPT
setopt PROMPT_SUBST

# Model and agent info with token count
# Fully formatted output directly from Rust
# Returns ZSH-formatted string ready for use in RPROMPT
function _goblin_prompt_info() {
    local goblin_bin="${_GOBLIN_BIN:-${GOBLIN_BIN:-goblin}}"
    
    # Get fully formatted prompt from goblin (single command).
    # Pass session model/provider as CLI flags when set so the rprompt
    # reflects the active session override rather than global config.
    local -a goblin_cmd
    goblin_cmd=("$goblin_bin")
    goblin_cmd+=(zsh rprompt)
    [[ -n "$_GOBLIN_SESSION_MODEL" ]] && local -x GOBLIN_SESSION__MODEL_ID="$_GOBLIN_SESSION_MODEL"
    [[ -n "$_GOBLIN_SESSION_PROVIDER" ]] && local -x GOBLIN_SESSION__PROVIDER_ID="$_GOBLIN_SESSION_PROVIDER"
    _GOBLIN_CONVERSATION_ID=$_GOBLIN_CONVERSATION_ID _GOBLIN_ACTIVE_AGENT=$_GOBLIN_ACTIVE_AGENT "${goblin_cmd[@]}"
}

# Right prompt: agent and model with token count (uses single goblin prompt command)
# Set RPROMPT if empty, otherwise append to existing value
if [[ -z "$_GOBLIN_THEME_LOADED" ]]; then
    RPROMPT='$(_goblin_prompt_info)'"${RPROMPT:+ ${RPROMPT}}"
fi
