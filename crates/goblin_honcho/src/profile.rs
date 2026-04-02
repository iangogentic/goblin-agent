//! User Profile Builder
//!
//! Builds comprehensive user profiles from dialectic analysis and behavior tracking.

use crate::{
    CommunicationStyle, DialecticAnalysis, ExplanationDepth, FormatPreferences,
    InteractionPattern, LearnedProfile, Preference,
};
use anyhow::Result;

/// Builds user profiles from various data sources
pub struct UserProfile {
    /// Communication style
    pub communication_style: CommunicationStyle,
    /// Technical level (1-10)
    technical_level: u8,
    /// Preferred explanation depth
    pub explanation_depth: ExplanationDepth,
    /// Response format preferences
    pub format_preferences: FormatPreferences,
    /// Explicit preferences
    preferences: Vec<Preference>,
    /// Interaction patterns
    patterns: Vec<InteractionPattern>,
}

impl UserProfile {
    /// Create a new profile builder
    pub fn new() -> Self {
        Self {
            communication_style: CommunicationStyle::Mixed,
            technical_level: 5,
            explanation_depth: ExplanationDepth::Standard,
            format_preferences: FormatPreferences {
                use_code_blocks: true,
                use_bullets: true,
                use_headers: true,
                include_examples: true,
                technical_jargon: false,
            },
            preferences: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Build from dialectic analysis
    pub fn from_dialectic(&mut self, analysis: &DialecticAnalysis) -> &mut Self {
        self.communication_style = analysis.style.clone();
        
        // Infer technical level from communication style
        self.technical_level = match analysis.style {
            CommunicationStyle::Technical => 8,
            CommunicationStyle::Formal => 6,
            CommunicationStyle::Casual => 4,
            CommunicationStyle::Mixed => 5,
        };
        
        // Determine explanation depth from tone
        let is_patient = analysis.tone.iter().any(|t| {
            matches!(t, crate::ToneIndicator::Patient | crate::ToneIndicator::Verbose)
        });
        let is_impatient = analysis.tone.iter().any(|t| {
            matches!(t, crate::ToneIndicator::Impatient | crate::ToneIndicator::Concise)
        });
        
        self.explanation_depth = if is_impatient {
            ExplanationDepth::Brief
        } else if is_patient {
            ExplanationDepth::Detailed
        } else {
            ExplanationDepth::Standard
        };
        
        // Update format preferences based on learning patterns
        for pattern in &analysis.learning_patterns {
            if pattern.contains("code examples") {
                self.format_preferences.include_examples = true;
                self.format_preferences.use_code_blocks = true;
            }
            if pattern.contains("explanations") {
                self.explanation_depth = ExplanationDepth::Detailed;
            }
        }
        
        self
    }

    /// Add explicit preference
    pub fn add_preference(&mut self, key: &str, value: &str, confidence: f32) -> &mut Self {
        self.preferences.push(Preference {
            key: key.to_string(),
            value: value.to_string(),
            confidence,
        });
        self
    }

    /// Add interaction pattern
    pub fn add_pattern(&mut self, trigger: &str, response_style: &str) -> &mut Self {
        // Check if pattern already exists
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.trigger == trigger) {
            existing.frequency += 1;
            existing.response_style = response_style.to_string();
        } else {
            self.patterns.push(InteractionPattern {
                trigger: trigger.to_string(),
                response_style: response_style.to_string(),
                frequency: 1,
            });
        }
        self
    }

    /// Set technical level explicitly
    pub fn set_technical_level(&mut self, level: u8) -> &mut Self {
        self.technical_level = level.clamp(1, 10);
        self
    }

    /// Set format preferences
    pub fn set_format_preferences(&mut self, prefs: FormatPreferences) -> &mut Self {
        self.format_preferences = prefs;
        self
    }

    /// Build the final profile
    pub fn build(&self) -> LearnedProfile {
        LearnedProfile {
            communication_style: self.communication_style.clone(),
            technical_level: self.technical_level,
            explanation_depth: self.explanation_depth.clone(),
            format_preferences: self.format_preferences.clone(),
            preferences: self.preferences.clone(),
            patterns: self.patterns.clone(),
        }
    }

    /// Load from existing profile
    pub fn from_profile(&mut self, profile: &LearnedProfile) -> &mut Self {
        self.communication_style = profile.communication_style.clone();
        self.technical_level = profile.technical_level;
        self.explanation_depth = profile.explanation_depth.clone();
        self.format_preferences = profile.format_preferences.clone();
        self.preferences = profile.preferences.clone();
        self.patterns = profile.patterns.clone();
        self
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl LearnedProfile {
    /// Create a default profile
    pub fn default_profile() -> Self {
        Self {
            communication_style: CommunicationStyle::Mixed,
            technical_level: 5,
            explanation_depth: ExplanationDepth::Standard,
            format_preferences: FormatPreferences {
                use_code_blocks: true,
                use_bullets: true,
                use_headers: true,
                include_examples: true,
                technical_jargon: false,
            },
            preferences: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Get a specific preference
    pub fn get_preference(&self, key: &str) -> Option<&str> {
        self.preferences
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.as_str())
    }

    /// Check if user prefers code examples
    pub fn prefers_code_examples(&self) -> bool {
        self.format_preferences.use_code_blocks
            || self.format_preferences.include_examples
    }

    /// Check if user is technical
    pub fn is_technical(&self) -> bool {
        self.technical_level >= 7
    }

    /// Get response format hint for LLM
    pub fn get_response_hint(&self) -> String {
        let mut hints = Vec::new();

        match self.explanation_depth {
            ExplanationDepth::Brief => hints.push("Keep responses brief and direct"),
            ExplanationDepth::Detailed => hints.push("Provide detailed explanations"),
            ExplanationDepth::Comprehensive => hints.push("Be thorough and comprehensive"),
            ExplanationDepth::Standard => {}
        }

        if self.format_preferences.use_code_blocks {
            hints.push("Use code blocks for examples");
        }

        if self.format_preferences.use_bullets {
            hints.push("Use bullet points for lists");
        }

        if self.is_technical() {
            hints.push("Use technical terminology");
        } else {
            hints.push("Avoid jargon, explain technical terms");
        }

        match self.communication_style {
            CommunicationStyle::Casual => hints.push("Keep tone casual and friendly"),
            CommunicationStyle::Formal => hints.push("Use formal language"),
            CommunicationStyle::Technical => hints.push("Be precise and technical"),
            CommunicationStyle::Mixed => {}
        }

        hints.join(". ") + "."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_build() {
        let profile = UserProfile::new()
            .set_technical_level(8)
            .add_preference("theme", "dark", 0.9)
            .add_pattern("error", "provide fix")
            .build();

        assert_eq!(profile.technical_level, 8);
        assert_eq!(profile.preferences.len(), 1);
        assert_eq!(profile.patterns.len(), 1);
    }

    #[test]
    fn test_profile_serialization() {
        let profile = LearnedProfile::default_profile();
        let json = profile.to_json().unwrap();
        let parsed = LearnedProfile::from_json(&json).unwrap();
        
        assert_eq!(parsed.technical_level, profile.technical_level);
    }
}
