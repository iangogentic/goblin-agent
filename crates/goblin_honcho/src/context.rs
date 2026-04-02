//! Context Injector
//!
//! Injects user profile context into prompts for personalized responses.

use crate::{LearnedProfile, Message, MessageRole};

/// Injects user context into prompts
pub struct ContextInjector {
    /// Cached profile
    profile: Option<LearnedProfile>,
}

impl ContextInjector {
    /// Create a new injector
    pub fn new() -> Self {
        Self { profile: None }
    }

    /// Set the user profile
    pub fn set_profile(&mut self, profile: LearnedProfile) -> &mut Self {
        self.profile = Some(profile);
        self
    }

    /// Update the profile
    pub fn update_profile(&mut self, profile: Option<LearnedProfile>) -> &mut Self {
        self.profile = profile;
        self
    }

    /// Inject context into a system prompt
    pub fn inject_into_system_prompt(&self, base_prompt: &str) -> String {
        let mut result = base_prompt.to_string();

        if let Some(profile) = &self.profile {
            let user_context = self.build_user_context(profile);
            result.push_str("\n\n# User Context\n");
            result.push_str(&user_context);
        }

        result
    }

    /// Build user context string
    fn build_user_context(&self, profile: &LearnedProfile) -> String {
        let mut context = String::new();

        // Communication style
        context.push_str(&format!(
            "- Communication style: {}\n",
            match profile.communication_style {
                crate::CommunicationStyle::Formal => "formal",
                crate::CommunicationStyle::Casual => "casual",
                crate::CommunicationStyle::Technical => "technical",
                crate::CommunicationStyle::Mixed => "mixed",
            }
        ));

        // Technical level
        context.push_str(&format!(
            "- Technical level: {}/10\n",
            profile.technical_level
        ));

        // Explanation depth
        context.push_str(&format!(
            "- Prefers {} explanations\n",
            match profile.explanation_depth {
                crate::ExplanationDepth::Brief => "brief",
                crate::ExplanationDepth::Standard => "standard",
                crate::ExplanationDepth::Detailed => "detailed",
                crate::ExplanationDepth::Comprehensive => "comprehensive",
            }
        ));

        // Format preferences
        context.push_str("- Format preferences: ");
        let mut formats = Vec::new();
        if profile.format_preferences.use_code_blocks {
            formats.push("code blocks");
        }
        if profile.format_preferences.use_bullets {
            formats.push("bullet points");
        }
        if profile.format_preferences.include_examples {
            formats.push("examples");
        }
        if !formats.is_empty() {
            context.push_str(&formats.join(", "));
            context.push('\n');
        } else {
            context.push_str("standard formatting\n");
        }

        // Explicit preferences
        if !profile.preferences.is_empty() {
            context.push_str("- Known preferences:\n");
            for pref in &profile.preferences {
                context.push_str(&format!(
                    "  * {}: {}\n",
                    pref.key, pref.value
                ));
            }
        }

        // Response hint
        context.push_str(&format!(
            "- Response guidance: {}\n",
            profile.get_response_hint()
        ));

        context
    }

    /// Inject context into a single message
    pub fn inject_into_message(&self, message: &str, _role: &str) -> String {
        let mut result = message.to_string();

        if let Some(profile) = &self.profile {
            // Add format hints for code-related messages
            if message.contains("code") || message.contains("function") || message.contains("example") {
                if profile.format_preferences.use_code_blocks {
                    result.push_str("\n\n[Consider using code blocks for any code examples]");
                }
                if profile.format_preferences.include_examples {
                    result.push_str("\n[Including working examples would be helpful]");
                }
            }
        }

        result
    }

    /// Generate a personalized greeting
    pub fn generate_greeting(&self) -> String {
        if let Some(profile) = &self.profile {
            match profile.communication_style {
                crate::CommunicationStyle::Casual => {
                    "Hey! What can I help you with today?".to_string()
                }
                crate::CommunicationStyle::Formal => {
                    "Good day. How may I assist you?".to_string()
                }
                crate::CommunicationStyle::Technical => {
                    "Ready. What are we debugging?".to_string()
                }
                crate::CommunicationStyle::Mixed => {
                    "Hi there! What would you like to work on?".to_string()
                }
            }
        } else {
            "Hello! I'm Goblin, your AI coding assistant. What would you like to do?".to_string()
        }
    }

    /// Generate a context-aware error message
    pub fn generate_error_message(&self, error: &str) -> String {
        if let Some(profile) = &self.profile {
            if matches!(
                profile.explanation_depth,
                crate::ExplanationDepth::Detailed | crate::ExplanationDepth::Comprehensive
            ) {
                return format!(
                    "I encountered an error: {}\n\nI'll need to investigate this further. Let me provide more details about what went wrong.",
                    error
                );
            }
        }
        
        format!("I hit an error: {}", error)
    }

    /// Get the current profile
    pub fn get_profile(&self) -> Option<&LearnedProfile> {
        self.profile.as_ref()
    }

    /// Check if profile is loaded
    pub fn has_profile(&self) -> bool {
        self.profile.is_some()
    }
}

impl Default for ContextInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform messages for OpenAI format
pub struct MessageTransformer;

impl MessageTransformer {
    /// Convert messages to OpenAI format with context injection
    pub fn to_openai_format(
        messages: &[Message],
        profile: Option<&LearnedProfile>,
    ) -> Vec<serde_json::Value> {
        let mut result = Vec::new();

        for msg in messages {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Agent => "assistant",
            };

            result.push(serde_json::json!({
                "role": role,
                "content": msg.content
            }));
        }

        // Add system message with profile context if available
        if let Some(p) = profile {
            let context = Self::build_context_message(p);
            result.insert(0, serde_json::json!({
                "role": "system",
                "content": context
            }));
        }

        result
    }

    /// Build context message from profile
    fn build_context_message(profile: &LearnedProfile) -> String {
        let mut msg = String::from(
            "You are Goblin, an AI coding assistant. Adapt your responses to the user's preferences:\n\n"
        );

        msg.push_str(&format!(
            "- Communication style: {} (technical level: {}/10)\n",
            match profile.communication_style {
                crate::CommunicationStyle::Formal => "formal",
                crate::CommunicationStyle::Casual => "casual",
                crate::CommunicationStyle::Technical => "technical",
                crate::CommunicationStyle::Mixed => "mixed",
            },
            profile.technical_level
        ));

        msg.push_str(&format!(
            "- Explanation depth: {}\n",
            match profile.explanation_depth {
                crate::ExplanationDepth::Brief => "brief",
                crate::ExplanationDepth::Standard => "standard",
                crate::ExplanationDepth::Detailed => "detailed",
                crate::ExplanationDepth::Comprehensive => "comprehensive",
            }
        ));

        if profile.format_preferences.use_code_blocks {
            msg.push_str("- Use code blocks for code examples\n");
        }

        msg.push_str(&format!("- Response guidance: {}\n", profile.get_response_hint()));

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LearnedProfile;

    #[test]
    fn test_context_injection() {
        let profile = LearnedProfile::default_profile();
        let mut injector = ContextInjector::new();
        injector.set_profile(profile);

        let prompt = "You are a helpful assistant.";
        let result = injector.inject_into_system_prompt(prompt);

        assert!(result.contains("# User Context"));
        assert!(result.contains("Communication style"));
    }

    #[test]
    fn test_greeting() {
        let mut injector = ContextInjector::new();
        
        // No profile
        let greeting = injector.generate_greeting();
        assert!(greeting.contains("Goblin"));
        
        // With casual profile
        let mut profile = LearnedProfile::default_profile();
        profile.communication_style = crate::CommunicationStyle::Casual;
        injector.set_profile(profile);
        
        let greeting = injector.generate_greeting();
        assert!(greeting.to_lowercase().contains("hey"));
    }
}
