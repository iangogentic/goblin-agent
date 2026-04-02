#!/usr/bin/env zsh

# Doctor action handler for Goblin environment diagnostics

# Action handler: Run Goblin environment diagnostics
# Executes the Goblin binary's zsh doctor command
function _Goblin_action_doctor() {
    echo
    
    # Execute Goblin zsh doctor command
    $_GOBLIN_BIN zsh doctor
}
