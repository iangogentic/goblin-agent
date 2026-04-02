#!/usr/bin/env zsh

# Keyboard action handler for ZSH keyboard shortcuts

# Action handler: Display ZSH keyboard shortcuts
# Executes the Goblin binary's zsh keyboard command
function _Goblin_action_keyboard() {
    echo
    
    # Execute Goblin zsh keyboard command
    $_GOBLIN_BIN zsh keyboard
}
