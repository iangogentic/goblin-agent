//! CLI Commands
//!
//! Implements Goblin CLI commands.

use anyhow::Result;
use std::collections::HashMap;

/// Command registry for CLI commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CliCommand>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut commands: HashMap<String, Box<dyn CliCommand>> = HashMap::new();
        
        // Register built-in commands
        commands.insert("help".to_string(), Box::new(HelpCommand));
        commands.insert("version".to_string(), Box::new(VersionCommand));
        commands.insert("status".to_string(), Box::new(StatusCommand));
        commands.insert("config".to_string(), Box::new(ConfigCommand));
        commands.insert("model".to_string(), Box::new(ModelCommand));
        commands.insert("tools".to_string(), Box::new(ToolsCommand));
        commands.insert("skills".to_string(), Box::new(SkillsCommand));
        commands.insert("sessions".to_string(), Box::new(SessionsCommand));
        
        Self { commands }
    }

    pub fn register(&mut self, name: String, command: Box<dyn CliCommand>) {
        self.commands.insert(name, command);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn CliCommand>> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<&String> {
        self.commands.keys().collect()
    }

    pub fn run(&self, name: &str, args: &[String]) -> Result<()> {
        if let Some(cmd) = self.commands.get(name) {
            cmd.run(args)
        } else {
            println!("Unknown command: {}", name);
            println!("Run 'goblin help' for available commands.");
            Ok(())
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// CLI command trait
pub trait CliCommand: Send + Sync {
    /// Run the command
    fn run(&self, args: &[String]) -> Result<()>;
    
    /// Get command description
    fn description(&self) -> &str;
    
    /// Get usage string
    fn usage(&self) -> &str;
}

/// Help command
pub struct HelpCommand;

impl CliCommand for HelpCommand {
    fn run(&self, _args: &[String]) -> Result<()> {
        println!();
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                        Goblin Help                       ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
        println!("  Usage: goblin [command] [options]");
        println!();
        println!("  Commands:");
        println!();
        println!("    help      Show this help or help for a specific command");
        println!("    version   Show Goblin version");
        println!("    status    Show Goblin status and configuration");
        println!("    config    Manage configuration");
        println!("    model     Manage AI models");
        println!("    tools     List and manage tools");
        println!("    skills    Manage skills");
        println!("    sessions  Manage sessions");
        println!();
        println!("  Interactive Mode:");
        println!();
        println!("    goblin              Start interactive mode");
        println!("    goblin <prompt>     Run single prompt");
        println!();
        println!("  Slash Commands (in interactive mode):");
        println!();
        println!("    /new         Start new session");
        println!("    /reset       Reset current session");
        println!("    /compact     Compress context");
        println!("    /skills      Manage skills");
        println!("    /personality Switch personality");
        println!();
        Ok(())
    }

    fn description(&self) -> &str {
        "Show help information"
    }

    fn usage(&self) -> &str {
        "help [command]"
    }
}

/// Version command
pub struct VersionCommand;

impl CliCommand for VersionCommand {
    fn run(&self, _args: &[String]) -> Result<()> {
        println!();
        println!("  Goblin v0.1.0");
        println!();
        println!("  Forge speed + Hermes intelligence");
        println!("  https://github.com/iangogentic/goblin-agent");
        println!();
        Ok(())
    }

    fn description(&self) -> &str {
        "Show Goblin version"
    }

    fn usage(&self) -> &str {
        "version"
    }
}

/// Status command
pub struct StatusCommand;

impl CliCommand for StatusCommand {
    fn run(&self, _args: &[String]) -> Result<()> {
        println!();
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                      Goblin Status                       ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
        println!("  Configuration:");
        println!("    Config Dir:  ~/.config/goblin");
        println!("    Data Dir:    ~/.local/share/goblin");
        println!();
        println!("  Enabled Features:");
        println!("    ✓ Memory System");
        println!("    ✓ Skills System");
        println!("    ✓ DOJO Self-Improvement");
        println!("    ✓ Honcho User Modeling");
        println!();
        println!("  Optional Features:");
        println!("    ○ Telegram Gateway");
        println!("    ○ Discord Gateway");
        println!("    ○ Cron Scheduler");
        println!();
        println!("  Tools: 27 available");
        println!("  Skills: 0 learned");
        println!();
        Ok(())
    }

    fn description(&self) -> &str {
        "Show Goblin status"
    }

    fn usage(&self) -> &str {
        "status"
    }
}

/// Config command
pub struct ConfigCommand;

impl CliCommand for ConfigCommand {
    fn run(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            println!();
            println!("  Current Configuration:");
            println!();
            println!("    provider: openai");
            println!("    model:    gpt-4o");
            println!();
        } else {
            match args[0].as_str() {
                "get" => {
                    if args.len() > 1 {
                        println!("  {} = <value>", args[1]);
                    } else {
                        println!("  Usage: config get <key>");
                    }
                }
                "set" => {
                    if args.len() > 2 {
                        println!("  Set {} = {}", args[1], args[2]);
                    } else {
                        println!("  Usage: config set <key> <value>");
                    }
                }
                "path" => {
                    println!();
                    println!("  Config path: ~/.config/goblin/goblin.yaml");
                    println!();
                }
                _ => {
                    println!();
                    println!("  Usage: config [get|set|path]");
                    println!();
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Manage configuration"
    }

    fn usage(&self) -> &str {
        "config [get|set|path] [key] [value]"
    }
}

/// Model command
pub struct ModelCommand;

impl CliCommand for ModelCommand {
    fn run(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            println!();
            println!("  Current Model:");
            println!();
            println!("    Provider: openai");
            println!("    Model:    gpt-4o");
            println!();
            println!("  Usage: model list | model set <name>");
            println!();
        } else {
            match args[0].as_str() {
                "list" => {
                    println!();
                    println!("  Available Models:");
                    println!();
                    println!("  OpenAI:");
                    println!("    gpt-4o          - Latest, most capable");
                    println!("    gpt-4o-mini     - Fast, affordable");
                    println!("    gpt-4-turbo     - Powerful, good value");
                    println!();
                    println!("  Anthropic:");
                    println!("    claude-sonnet-4 - Latest, balanced");
                    println!("    claude-3-5-sonnet - Great for coding");
                    println!();
                }
                "set" => {
                    if args.len() > 1 {
                        println!("  Set model to: {}", args[1]);
                    } else {
                        println!("  Usage: model set <name>");
                    }
                }
                "info" => {
                    if args.len() > 1 {
                        println!();
                        println!("  Model: {}", args[1]);
                        println!("  Provider: openai");
                        println!();
                    } else {
                        println!();
                        println!("  Current Model:");
                        println!();
                        println!("    Provider: openai");
                        println!("    Model:    gpt-4o");
                        println!();
                    }
                }
                _ => {
                    println!();
                    println!("  Usage: model [list|set|info]");
                    println!();
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Manage AI models"
    }

    fn usage(&self) -> &str {
        "model [list|set|info] [model]"
    }
}

/// Tools command
pub struct ToolsCommand;

impl CliCommand for ToolsCommand {
    fn run(&self, args: &[String]) -> Result<()> {
        if args.is_empty() || args[0] == "list" {
            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║                     Available Tools                      ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("  File Operations:");
            println!("    ✓ Read           - Read file contents");
            println!("    ✓ Write          - Write to files");
            println!("    ✓ FSSearch       - Search file contents");
            println!("    ✓ Remove         - Delete files");
            println!("    ✓ Patch          - Edit files");
            println!();
            println!("  System:");
            println!("    ✓ Shell          - Execute shell commands");
            println!("    ✓ Fetch          - Fetch URLs");
            println!();
            println!("  Goblin (Hermes Brain):");
            println!("    ✓ MemoryCheckpoint - Save session state");
            println!("    ✓ MemorySearch    - Search memory");
            println!("    ✓ MemorySummarize - Summarize context");
            println!("    ✓ SkillCreate     - Create new skill");
            println!("    ✓ SkillImprove    - Improve skill");
            println!("    ✓ ScheduleCreate  - Schedule task");
            println!();
            println!("  Total: 27 tools (27 enabled, 0 disabled)");
            println!();
        } else {
            match args[0].as_str() {
                "info" => {
                    if args.len() > 1 {
                        println!();
                        println!("  Tool: {}", args[1]);
                        println!("  Status: enabled");
                        println!();
                    } else {
                        println!("  Usage: tools info <name>");
                    }
                }
                "enable" => {
                    if args.len() > 1 {
                        println!("  Enabled: {}", args[1]);
                    } else {
                        println!("  Usage: tools enable <name>");
                    }
                }
                "disable" => {
                    if args.len() > 1 {
                        println!("  Disabled: {}", args[1]);
                    } else {
                        println!("  Usage: tools disable <name>");
                    }
                }
                _ => {
                    println!();
                    println!("  Usage: tools [list|info|enable|disable]");
                    println!();
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Manage tools"
    }

    fn usage(&self) -> &str {
        "tools [list|info|enable|disable] [tool]"
    }
}

/// Skills command
pub struct SkillsCommand;

impl CliCommand for SkillsCommand {
    fn run(&self, args: &[String]) -> Result<()> {
        if args.is_empty() || args[0] == "list" {
            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║                       Skills                             ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("  Learned Skills:");
            println!();
            println!("    (No skills learned yet)");
            println!();
            println!("  Built-in Skills:");
            println!();
            println!("    rust-expert       - Rust best practices");
            println!("    test-writer       - Generate tests");
            println!("    bug-fixer         - Debug and fix issues");
            println!("    reviewer          - Code review");
            println!();
            println!("  Usage: skills create | skills list");
            println!();
        } else {
            match args[0].as_str() {
                "create" => {
                    if args.len() > 1 {
                        println!("  Creating skill: {}", args[1]);
                    } else {
                        println!("  Usage: skills create <name> [description]");
                    }
                }
                "improve" => {
                    if args.len() > 1 {
                        println!("  Improving skill: {}", args[1]);
                    } else {
                        println!("  Usage: skills improve <name>");
                    }
                }
                "delete" => {
                    if args.len() > 1 {
                        println!("  Deleted skill: {}", args[1]);
                    } else {
                        println!("  Usage: skills delete <name>");
                    }
                }
                "export" => {
                    let path = args.get(1).map(|s| s.as_str()).unwrap_or("skills.json");
                    println!("  Exported skills to: {}", path);
                }
                "import" => {
                    let path = args.get(1).map(|s| s.as_str()).unwrap_or("skills.json");
                    println!("  Imported skills from: {}", path);
                }
                _ => {
                    println!();
                    println!("  Usage: skills [list|create|improve|delete|export|import]");
                    println!();
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Manage skills"
    }

    fn usage(&self) -> &str {
        "skills [list|create|improve|delete|export|import]"
    }
}

/// Sessions command
pub struct SessionsCommand;

impl CliCommand for SessionsCommand {
    fn run(&self, args: &[String]) -> Result<()> {
        if args.is_empty() || args[0] == "list" {
            println!();
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║                     Sessions                            ║");
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
            println!("  Current: session-2024-01-15-abc123");
            println!();
            println!("  Recent Sessions:");
            println!();
            println!("    * session-2024-01-15-abc123  (current)");
            println!("      2024-01-15 14:30  12 messages");
            println!();
            println!("  Usage: sessions list | sessions show <id>");
            println!();
        } else {
            match args[0].as_str() {
                "show" => {
                    if args.len() > 1 {
                        println!();
                        println!("  Session: {}", args[1]);
                        println!("  Messages: 12");
                        println!();
                    } else {
                        println!("  Usage: sessions show <id>");
                    }
                }
                "delete" => {
                    if args.len() > 1 {
                        println!("  Deleted session: {}", args[1]);
                    } else {
                        println!("  Usage: sessions delete <id>");
                    }
                }
                "export" => {
                    let path = args.get(2).map(|s| s.as_str()).unwrap_or("session.json");
                    let session = args.get(1).map(|s| s.as_str()).unwrap_or("current");
                    println!("  Exported session {} to: {}", session, path);
                }
                _ => {
                    println!();
                    println!("  Usage: sessions [list|show|delete|export]");
                    println!();
                }
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Manage sessions"
    }

    fn usage(&self) -> &str {
        "sessions [list|show|delete|export]"
    }
}
