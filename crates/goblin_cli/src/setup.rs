//! Interactive Setup Wizard
//!
//! Guides users through initial Goblin configuration.

use crate::{CliConfig, SetupState, SetupStep};
use anyhow::Result;
use std::io::{self, Write};

/// Interactive setup wizard
pub struct SetupWizard {
    config: CliConfig,
    state: SetupState,
}

impl SetupWizard {
    /// Create a new setup wizard
    pub fn new(config: CliConfig) -> Self {
        Self {
            config,
            state: SetupState::new(),
        }
    }

    /// Run the interactive setup
    pub fn run(&mut self) -> Result<SetupState> {
        self.print_welcome();
        
        while !self.state.is_complete() {
            match self.state.step {
                SetupStep::Welcome => self.step_welcome()?,
                SetupStep::Provider => self.step_provider()?,
                SetupStep::Model => self.step_model()?,
                SetupStep::ApiKey => self.step_api_key()?,
                SetupStep::Features => self.step_features()?,
                SetupStep::Complete => break,
            }
            self.state.next_step();
        }
        
        self.print_complete()?;
        
        Ok(self.state.clone())
    }

    fn print_welcome(&self) {
        println!();
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    Welcome to Goblin!                      ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  The self-improving AI coding agent that learns         ║");
        println!("║  from every interaction. Let's get you set up.          ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
    }

    fn step_welcome(&self) -> Result<()> {
        println!("This wizard will help you configure Goblin.");
        println!();
        println!("Press Enter to continue...");
        io::stdin().read_line(&mut String::new())?;
        Ok(())
    }

    fn step_provider(&mut self) -> Result<()> {
        println!();
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Step 1: Choose Your AI Provider                       │");
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
        println!("  1. OpenAI (GPT-4, GPT-3.5)");
        println!("  2. Anthropic (Claude 3.5, Claude 3)");
        println!("  3. Google Vertex AI (Gemini)");
        println!("  4. OpenRouter (200+ models)");
        println!("  5. Nous Portal");
        println!("  6. Local/Ollama");
        println!("  7. Custom Endpoint");
        println!();
        
        loop {
            print!("  Enter choice (1-7): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let choice = input.trim();
            let provider = match choice {
                "1" => Some("openai".to_string()),
                "2" => Some("anthropic".to_string()),
                "3" => Some("vertex".to_string()),
                "4" => Some("openrouter".to_string()),
                "5" => Some("nous".to_string()),
                "6" => Some("ollama".to_string()),
                "7" => Some("custom".to_string()),
                _ => {
                    println!("  Invalid choice. Please enter 1-7.");
                    continue;
                }
            };
            
            self.state.provider = provider;
            break;
        }
        
        Ok(())
    }

    fn step_model(&mut self) -> Result<()> {
        let provider = self.state.provider.as_deref().unwrap_or("openai");
        
        println!();
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Step 2: Choose Your Model                             │");
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
        
        let models = match provider {
            "openai" => vec![
                ("gpt-4o", "GPT-4o - Latest, most capable"),
                ("gpt-4o-mini", "GPT-4o Mini - Fast, affordable"),
                ("gpt-4-turbo", "GPT-4 Turbo - Powerful, good value"),
                ("gpt-3.5-turbo", "GPT-3.5 Turbo - Fastest, cheapest"),
            ],
            "anthropic" => vec![
                ("claude-sonnet-4-20250514", "Claude Sonnet 4 - Latest, balanced"),
                ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet - Great for coding"),
                ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku - Fast, affordable"),
                ("claude-3-opus-20240229", "Claude 3 Opus - Most capable"),
            ],
            "openrouter" => vec![
                ("anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet via OpenRouter"),
                ("openai/gpt-4o", "GPT-4o via OpenRouter"),
                ("google/gemini-pro-1.5", "Gemini Pro via OpenRouter"),
            ],
            _ => vec![
                ("default", "Default model for this provider"),
            ],
        };
        
        for (i, (model, desc)) in models.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, model, desc);
        }
        println!();
        
        print!("  Enter choice (1-{}): ", models.len());
        io::stdout().flush()?;
        
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice >= 1 && choice <= models.len() {
                    self.state.model = Some(models[choice - 1].0.to_string());
                    break;
                }
            }
            print!("  Invalid choice. Enter 1-{}: ", models.len());
            io::stdout().flush()?;
        }
        
        Ok(())
    }

    fn step_api_key(&mut self) -> Result<()> {
        let provider = self.state.provider.as_deref().unwrap_or("openai");
        
        println!();
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Step 3: Enter API Key                                 │");
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
        
        let key_name = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "vertex" => "GOOGLE_CLOUD_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            "nous" => "NOUS_API_KEY",
            _ => "API_KEY",
        };
        
        println!("  You can find your API key at:");
        let url = match provider {
            "openai" => "https://platform.openai.com/api-keys",
            "anthropic" => "https://console.anthropic.com/settings/keys",
            "openrouter" => "https://openrouter.ai/keys",
            "vertex" => "https://console.cloud.google.com/apis/credentials",
            "nous" => "https://portal.nousresearch.com/keys",
            _ => "your provider's website",
        };
        println!("  {}", url);
        println!();
        
        // For security, we just store the env var name
        // User can set it externally or we'll prompt for it
        print!("  Will use {} environment variable", key_name);
        println!();
        println!("  (Press Enter to continue, or enter key now)");
        println!();
        print!("  > ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if !input.is_empty() {
            self.state.api_key = Some(input.to_string());
        }
        
        Ok(())
    }

    fn step_features(&mut self) -> Result<()> {
        println!();
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│  Step 4: Enable Features                               │");
        println!("└─────────────────────────────────────────────────────────┘");
        println!();
        println!("  These features enhance Goblin's capabilities:");
        println!();
        println!("  1. [✓] Memory System - Remember across sessions");
        println!("  2. [✓] Skills System - Learn from experience");
        println!("  3. [✓] DOJO - Self-improvement review");
        println!("  4. [ ] Telegram Gateway - Chat via Telegram");
        println!("  5. [ ] Discord Gateway - Chat via Discord");
        println!("  6. [ ] Cron Scheduler - Scheduled tasks");
        println!();
        println!("  (Features 1-3 are required and enabled by default)");
        println!();
        print!("  Enter feature numbers to enable (e.g., 4,5): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        // Parse feature numbers
        for part in input.trim().split(',') {
            if let Ok(num) = part.trim().parse::<usize>() {
                match num {
                    4 => self.state.features.push("telegram".to_string()),
                    5 => self.state.features.push("discord".to_string()),
                    6 => self.state.features.push("cron".to_string()),
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    fn print_complete(&self) -> Result<()> {
        println!();
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    Setup Complete!                        ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
        println!("  Provider: {}", self.state.provider.as_deref().unwrap_or("unknown"));
        println!("  Model: {}", self.state.model.as_deref().unwrap_or("unknown"));
        if let Some(features) = self.state.features.is_empty().then(|| &self.state.features) {
            println!("  Features: {}", features.join(", "));
        }
        println!();
        println!("  Run 'goblin' to start!");
        println!();
        
        Ok(())
    }

    /// Generate config file from setup state
    pub fn generate_config(&self) -> Result<String> {
        let config = serde_json::json!({
            "model": self.state.model,
            "provider": self.state.provider,
            "features": self.state.features,
            "honcho": {
                "enabled": true,
                "workspace_id": "goblin"
            },
            "memory": {
                "enabled": true
            },
            "skills": {
                "enabled": true,
                "auto_create": true
            },
            "dojo": {
                "enabled": true,
                "morning_review": true
            }
        });
        
        serde_json::to_string_pretty(&config).map_err(Into::into)
    }
}
