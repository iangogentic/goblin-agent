#!/usr/bin/env zsh

# Key bindings and widget registration for Goblin plugin

# Register ZLE widgets
zle -N Goblin-accept-line
zle -N Goblin-completion

# Custom bracketed-paste handler to fix syntax highlighting after paste
# Addresses timing issues by ensuring buffer state stabilizes before prompt reset
function Goblin-bracketed-paste() {
    # Call the built-in bracketed-paste widget first
    zle .$WIDGET "$@"
    
    # Explicitly redisplay the buffer to ensure paste content is visible
    # This is critical for large or multiline pastes
    zle redisplay
    
    # Reset the prompt to trigger syntax highlighting refresh
    # The redisplay before reset-prompt ensures the buffer is fully rendered
    zle reset-prompt
}

# Register the bracketed paste widget to fix highlighting on paste
zle -N bracketed-paste Goblin-bracketed-paste

# Bind Enter to our custom accept-line that transforms :commands
bindkey '^M' Goblin-accept-line
bindkey '^J' Goblin-accept-line
# Update the Tab binding to use the new completion widget
bindkey '^I' Goblin-completion  # Tab for both @ and :command completion
