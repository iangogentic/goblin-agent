# !! Contents within this block are managed by 'goblin zsh setup' !!
# !! Do not edit manually - changes will be overwritten !!

# Add required zsh plugins if not already present
if [[ ! " ${plugins[@]} " =~ " zsh-autosuggestions " ]]; then
    plugins+=(zsh-autosuggestions)
fi
if [[ ! " ${plugins[@]} " =~ " zsh-syntax-highlighting " ]]; then
    plugins+=(zsh-syntax-highlighting)
fi

# Load goblin shell plugin (commands, completions, keybindings) if not already loaded
if [[ -z "$_GOBLIN_PLUGIN_LOADED" ]]; then
    eval "$(goblin zsh plugin)"
fi

# Load goblin shell theme (prompt with AI context) if not already loaded
if [[ -z "$_GOBLIN_THEME_LOADED" ]]; then
    eval "$(goblin zsh theme)"
fi
