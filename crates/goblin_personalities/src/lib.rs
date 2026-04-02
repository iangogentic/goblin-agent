//! Goblin Personalities - Personality System
//!
//! Allows switching between different AI personalities and communication styles.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Personality manager
pub struct PersonalityManager {
    personalities: HashMap<String, Personality>,
    current: String,
    personality_dir: PathBuf,
}

impl PersonalityManager {
    pub fn new(personalities_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            personalities: HashMap::new(),
            current: "default".to_string(),
            personality_dir: personalities_dir,
        };
        
        // Load built-in personalities
        manager.load_builtin_personalities()?;
        
        // Load custom personalities from disk
        manager.load_custom_personalities()?;
        
        Ok(manager)
    }

    /// Load built-in personalities
    fn load_builtin_personalities(&mut self) -> Result<()> {
        // Default - Balanced, helpful
        self.personalities.insert("default".to_string(), Personality {
            name: "default".to_string(),
            description: "Balanced, helpful assistant".to_string(),
            communication_style: CommunicationStyle::Balanced,
            expertise_level: ExpertiseLevel::Intermediate,
            traits: vec![
                PersonalityTrait::Helpful,
                PersonalityTrait::Clear,
                PersonalityTrait::Patient,
            ],
            system_prompt_additions: vec![],
            example_behavior: "You are a helpful coding assistant. Provide clear, concise answers.".to_string(),
        });

        // Senior Dev - Concise, assumes expertise
        self.personalities.insert("senior-dev".to_string(), Personality {
            name: "senior-dev".to_string(),
            description: "Concise, assumes technical expertise".to_string(),
            communication_style: CommunicationStyle::Concise,
            expertise_level: ExpertiseLevel::Expert,
            traits: vec![
                PersonalityTrait::Efficient,
                PersonalityTrait::Direct,
                PersonalityTrait::Technical,
            ],
            system_prompt_additions: vec![
                "Assume the user is a senior developer.".to_string(),
                "Be concise - no unnecessary explanations.".to_string(),
                "Focus on the solution, not the explanation.".to_string(),
            ],
            example_behavior: "Here's the fix: `git reset --hard HEAD~1`".to_string(),
        });

        // Mentor - Explanatory, educational
        self.personalities.insert("mentor".to_string(), Personality {
            name: "mentor".to_string(),
            description: "Educational, explains reasoning".to_string(),
            communication_style: CommunicationStyle::Educational,
            expertise_level: ExpertiseLevel::Intermediate,
            traits: vec![
                PersonalityTrait::Patient,
                PersonalityTrait::Educational,
                PersonalityTrait::Encouraging,
            ],
            system_prompt_additions: vec![
                "Explain your reasoning step by step.".to_string(),
                "Provide context for decisions.".to_string(),
                "Use examples to illustrate points.".to_string(),
            ],
            example_behavior: "The reason we use Option instead of null is... Let me show you with an example...".to_string(),
        });

        // Goblin - Chaotic, playful, but effective
        self.personalities.insert("goblin".to_string(), Personality {
            name: "goblin".to_string(),
            description: "Goblin mode - chaotic but effective".to_string(),
            communication_style: CommunicationStyle::Casual,
            expertise_level: ExpertiseLevel::Intermediate,
            traits: vec![
                PersonalityTrait::Playful,
                PersonalityTrait::Creative,
                PersonalityTrait::Unconventional,
            ],
            system_prompt_additions: vec![
                "You are Goblin, a mischievous but helpful coding companion.".to_string(),
                "You're a bit chaotic but always get the job done.".to_string(),
                "Feel free to suggest unconventional solutions.".to_string(),
            ],
            example_behavior: "Alright alright alright, let's cook something up here! *rubs hands together*".to_string(),
        });

        // Debugger - Focuses on finding issues
        self.personalities.insert("debugger".to_string(), Personality {
            name: "debugger".to_string(),
            description: "Specialized in debugging and problem solving".to_string(),
            communication_style: CommunicationStyle::Systematic,
            expertise_level: ExpertiseLevel::Expert,
            traits: vec![
                PersonalityTrait::Methodical,
                PersonalityTrait::Analytical,
                PersonalityTrait::Thorough,
            ],
            system_prompt_additions: vec![
                "Focus on identifying root causes.".to_string(),
                "Ask clarifying questions when needed.".to_string(),
                "Propose systematic tests to isolate issues.".to_string(),
            ],
            example_behavior: "Let's trace through the execution. First, I'll add some debug output here...".to_string(),
        });

        // Security Expert - Security-focused
        self.personalities.insert("security".to_string(), Personality {
            name: "security".to_string(),
            description: "Security-focused reviews and recommendations".to_string(),
            communication_style: CommunicationStyle::Technical,
            expertise_level: ExpertiseLevel::Expert,
            traits: vec![
                PersonalityTrait::SecurityConscious,
                PersonalityTrait::Thorough,
                PersonalityTrait::Cautious,
            ],
            system_prompt_additions: vec![
                "Always consider security implications.".to_string(),
                "Highlight potential vulnerabilities.".to_string(),
                "Recommend secure alternatives when applicable.".to_string(),
            ],
            example_behavior: "That SQL query looks vulnerable to injection. Consider using parameterized queries instead.".to_string(),
        });

        Ok(())
    }

    /// Load custom personalities from disk
    fn load_custom_personalities(&mut self) -> Result<()> {
        if !self.personality_dir.exists() {
            std::fs::create_dir_all(&self.personality_dir)?;
        }

        // Load any .json or .yaml files in the personality directory
        for entry in std::fs::read_dir(&self.personality_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json" || ext == "yaml" || ext == "yml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let personality = serde_yaml::from_str::<Personality>(&content)
                        .ok()
                        .or_else(|| serde_json::from_str(&content).ok());
                    if let Some(personality) = personality {
                        self.personalities.insert(personality.name.clone(), personality);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get current personality
    pub fn current(&self) -> &Personality {
        self.personalities.get(&self.current).unwrap()
    }

    /// Set current personality
    pub fn set(&mut self, name: &str) -> Result<&Personality> {
        if self.personalities.contains_key(name) {
            self.current = name.to_string();
            Ok(&self.personalities[&self.current])
        } else {
            anyhow::bail!("Unknown personality: {}", name)
        }
    }

    /// List all available personalities
    pub fn list(&self) -> Vec<&Personality> {
        self.personalities.values().collect()
    }

    /// Get personality by name
    pub fn get(&self, name: &str) -> Option<&Personality> {
        self.personalities.get(name)
    }

    /// Generate system prompt addition for current personality
    pub fn generate_system_addition(&self) -> String {
        let personality = self.current();
        
        let mut additions = personality.system_prompt_additions.clone();
        additions.push(format!("\n\nCommunication style: {:?}", personality.communication_style));
        
        additions.join("\n")
    }

    /// Save custom personality to disk
    pub fn save_custom(&self, personality: &Personality) -> Result<PathBuf> {
        let path = self.personality_dir.join(format!("{}.yaml", personality.name));
        let content = serde_yaml::to_string(personality)?;
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Delete custom personality
    pub fn delete_custom(&self, name: &str) -> Result<()> {
        let path = self.personality_dir.join(format!("{}.yaml", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Personality definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// Unique name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Communication style
    pub communication_style: CommunicationStyle,
    /// Assumed expertise level
    pub expertise_level: ExpertiseLevel,
    /// Personality traits
    pub traits: Vec<PersonalityTrait>,
    /// Additions to system prompt
    pub system_prompt_additions: Vec<String>,
    /// Example behavior description
    #[serde(default)]
    pub example_behavior: String,
}

/// Communication style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommunicationStyle {
    /// Balanced between formal and casual
    Balanced,
    /// Short, to the point
    Concise,
    /// Educational, explains reasoning
    Educational,
    /// Casual, friendly
    Casual,
    /// Technical, precise
    Technical,
    /// Step-by-step approach
    Systematic,
}

/// Expertise level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpertiseLevel {
    /// Beginner - lots of explanations
    Beginner,
    /// Some experience - moderate detail
    Intermediate,
    /// Experienced - assumes competence
    Expert,
}

/// Personality traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalityTrait {
    Helpful,
    Clear,
    Patient,
    Efficient,
    Direct,
    Technical,
    Playful,
    Creative,
    Unconventional,
    Methodical,
    Analytical,
    Thorough,
    SecurityConscious,
    Cautious,
    Encouraging,
    Educational,
}

impl std::fmt::Display for CommunicationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommunicationStyle::Balanced => write!(f, "balanced"),
            CommunicationStyle::Concise => write!(f, "concise"),
            CommunicationStyle::Educational => write!(f, "educational"),
            CommunicationStyle::Casual => write!(f, "casual"),
            CommunicationStyle::Technical => write!(f, "technical"),
            CommunicationStyle::Systematic => write!(f, "systematic"),
        }
    }
}

impl std::fmt::Display for ExpertiseLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpertiseLevel::Beginner => write!(f, "beginner"),
            ExpertiseLevel::Intermediate => write!(f, "intermediate"),
            ExpertiseLevel::Expert => write!(f, "expert"),
        }
    }
}

impl std::fmt::Display for PersonalityTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonalityTrait::Helpful => write!(f, "helpful"),
            PersonalityTrait::Clear => write!(f, "clear"),
            PersonalityTrait::Patient => write!(f, "patient"),
            PersonalityTrait::Efficient => write!(f, "efficient"),
            PersonalityTrait::Direct => write!(f, "direct"),
            PersonalityTrait::Technical => write!(f, "technical"),
            PersonalityTrait::Playful => write!(f, "playful"),
            PersonalityTrait::Creative => write!(f, "creative"),
            PersonalityTrait::Unconventional => write!(f, "unconventional"),
            PersonalityTrait::Methodical => write!(f, "methodical"),
            PersonalityTrait::Analytical => write!(f, "analytical"),
            PersonalityTrait::Thorough => write!(f, "thorough"),
            PersonalityTrait::SecurityConscious => write!(f, "security_conscious"),
            PersonalityTrait::Cautious => write!(f, "cautious"),
            PersonalityTrait::Encouraging => write!(f, "encouraging"),
            PersonalityTrait::Educational => write!(f, "educational"),
        }
    }
}
