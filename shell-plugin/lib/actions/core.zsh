#!/usr/bin/env zsh

# Core action handlers for basic Goblin operations

# Action handler: Start a new conversation
function _Goblin_action_new() {
    local input_text="$1"
    
    # Clear conversation and save as previous (like cd -)
    _Goblin_clear_conversation
    _GOBLIN_ACTIVE_AGENT="Goblin"
    
    echo
    
    # If input_text is provided, send it to the new conversation
    if [[ -n "$input_text" ]]; then
        # Generate new conversation ID and switch to it
        local new_id=$($_GOBLIN_BIN conversation new)
        _Goblin_switch_conversation "$new_id"
        
        # Execute the Goblin command with the input text
        _Goblin_exec_interactive -p "$input_text" --cid "$_GOBLIN_CONVERSATION_ID"
        
        # Start background sync job if enabled and not already running
        _Goblin_start_background_sync
        # Start background update check
        _Goblin_start_background_update
    else
        # Only show banner if no input text (starting fresh conversation)
        _Goblin_exec banner
    fi
}

# Action handler: Show session info
function _Goblin_action_info() {
    echo
    if [[ -n "$_GOBLIN_CONVERSATION_ID" ]]; then
        _Goblin_exec info --cid "$_GOBLIN_CONVERSATION_ID"
    else
        _Goblin_exec info
    fi
}

# Action handler: Show environment info
function _Goblin_action_env() {
    echo
    _Goblin_exec env
}

# Action handler: Dump conversation
function _Goblin_action_dump() {
    local input_text="$1"
    if [[ "$input_text" == "html" ]]; then
        _Goblin_handle_conversation_command "dump" "--html"
    else
        _Goblin_handle_conversation_command "dump"
    fi
}

# Action handler: Compact conversation
function _Goblin_action_compact() {
    _Goblin_handle_conversation_command "compact"
}

# Action handler: Retry last message
function _Goblin_action_retry() {
    _Goblin_handle_conversation_command "retry"
}

# Helper function to handle conversation commands that require an active conversation
function _Goblin_handle_conversation_command() {
    local subcommand="$1"
    shift  # Remove first argument, remaining args become extra parameters
    
    echo
    
    # Check if GOBLIN_CONVERSATION_ID is set
    if [[ -z "$_GOBLIN_CONVERSATION_ID" ]]; then
        _Goblin_log error "No active conversation. Start a conversation first or use :conversation to see existing ones"
        return 0
    fi
    
    # Execute the conversation command with conversation ID and any extra arguments
    _Goblin_exec conversation "$subcommand" "$_GOBLIN_CONVERSATION_ID" "$@"
}
