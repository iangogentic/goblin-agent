//! Conversation Analyzer
//!
//! Analyzes conversations to extract insights about user behavior and preferences.

use crate::{DialecticTracker, Message, MessageRole, UserProfile};
use std::collections::HashMap;

/// Analyzes conversations to build user understanding
pub struct ConversationAnalyzer {
    /// Dialectic tracker for this conversation
    dialectic: DialecticTracker,
    /// Topic frequency
    topics: HashMap<String, usize>,
    /// Completed tasks
    completed_tasks: Vec<String>,
    /// Frustration indicators
    frustration_score: f32,
    /// Success indicators
    success_score: f32,
}

impl ConversationAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            dialectic: DialecticTracker::new(),
            topics: HashMap::new(),
            completed_tasks: Vec::new(),
            frustration_score: 0.0,
            success_score: 0.0,
        }
    }

    /// Analyze a batch of messages
    pub fn analyze_messages(&mut self, messages: &[Message]) -> &mut Self {
        for message in messages {
            self.analyze_message(message);
        }
        self
    }

    /// Analyze a single message
    pub fn analyze_message(&mut self, message: &Message) -> &mut Self {
        // Track in dialectic
        self.dialectic.track(message);

        // Extract topics
        self.extract_topics(&message.content);

        // Detect frustration
        if message.role == MessageRole::User {
            self.detect_frustration(&message.content);
        }

        // Detect success
        if message.role == MessageRole::User {
            self.detect_success(&message.content);
        }

        self
    }

    /// Extract topics from message content
    fn extract_topics(&mut self, content: &str) {
        let topic_markers = [
            "rust", "python", "javascript", "typescript", "go", "java",
            "api", "database", "web", "frontend", "backend", "devops",
            "testing", "deployment", "docker", "kubernetes", "git",
        ];

        let content_lower = content.to_lowercase();
        for topic in topic_markers {
            if content_lower.contains(topic) {
                *self.topics.entry(topic.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Detect frustration indicators
    fn detect_frustration(&mut self, content: &str) {
        let frustration_markers = [
            "still not working",
            "this is frustrating",
            "i've tried",
            "doesn't work",
            "failed",
            "error",
            "again",
            "already",
            "why isn't",
            "can't figure out",
        ];

        let content_lower = content.to_lowercase();
        for marker in frustration_markers {
            if content_lower.contains(marker) {
                self.frustration_score += 0.1;
            }
        }

        // Cap at 1.0
        self.frustration_score = self.frustration_score.min(1.0);
    }

    /// Detect success indicators
    fn detect_success(&mut self, content: &str) {
        let success_markers = [
            "thanks",
            "great",
            "perfect",
            "works",
            "awesome",
            "nice",
            "got it",
            "that works",
            "excellent",
            "solved",
        ];

        let content_lower = content.to_lowercase();
        for marker in success_markers {
            if content_lower.contains(marker) {
                self.success_score += 0.2;
            }
        }

        // Cap at 1.0
        self.success_score = self.success_score.min(1.0);
    }

    /// Mark a task as completed
    pub fn mark_task_completed(&mut self, task: &str) -> &mut Self {
        self.completed_tasks.push(task.to_string());
        self.success_score = (self.success_score + 0.2).min(1.0);
        self
    }

    /// Get dialectic analysis
    pub fn get_dialectic_analysis(&self) -> crate::DialecticAnalysis {
        self.dialectic.analyze()
    }

    /// Build user profile from analysis
    pub fn build_profile(&self) -> UserProfile {
        let analysis = self.dialectic.analyze();
        let mut profile = UserProfile::new();
        profile.from_dialectic(&analysis);
        
        // Adjust based on frustration/success
        if self.frustration_score > 0.5 {
            // User is frustrated, suggest patience/explanations
            profile.explanation_depth = crate::ExplanationDepth::Detailed;
        }
        
        profile
    }

    /// Get top topics
    pub fn get_top_topics(&self, limit: usize) -> Vec<(String, usize)> {
        let mut topics: Vec<_> = self.topics.iter().collect();
        topics.sort_by(|a, b| b.1.cmp(a.1));
        topics.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }

    /// Get frustration level
    pub fn get_frustration_level(&self) -> FrustrationLevel {
        if self.frustration_score < 0.3 {
            FrustrationLevel::Calm
        } else if self.frustration_score < 0.6 {
            FrustrationLevel::Impatient
        } else {
            FrustrationLevel::Frustrated
        }
    }

    /// Get success level
    pub fn get_success_level(&self) -> SuccessLevel {
        if self.success_score < 0.3 {
            SuccessLevel::Struggling
        } else if self.success_score < 0.6 {
            SuccessLevel::MakingProgress
        } else {
            SuccessLevel::Successful
        }
    }

    /// Get analysis summary
    pub fn get_summary(&self) -> AnalysisSummary {
        AnalysisSummary {
            message_count: self.dialectic.message_count(),
            dialectic: self.get_dialectic_analysis(),
            top_topics: self.get_top_topics(5),
            frustration_level: self.get_frustration_level(),
            success_level: self.get_success_level(),
            completed_tasks: self.completed_tasks.clone(),
        }
    }

    /// Reset the analyzer
    pub fn reset(&mut self) {
        self.dialectic.reset();
        self.topics.clear();
        self.completed_tasks.clear();
        self.frustration_score = 0.0;
        self.success_score = 0.0;
    }
}

impl Default for ConversationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Frustration level
#[derive(Debug, Clone, Copy)]
pub enum FrustrationLevel {
    Calm,
    Impatient,
    Frustrated,
}

/// Success level
#[derive(Debug, Clone, Copy)]
pub enum SuccessLevel {
    Struggling,
    MakingProgress,
    Successful,
}

/// Analysis summary
#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub message_count: usize,
    pub dialectic: crate::DialecticAnalysis,
    pub top_topics: Vec<(String, usize)>,
    pub frustration_level: FrustrationLevel,
    pub success_level: SuccessLevel,
    pub completed_tasks: Vec<String>,
}

impl AnalysisSummary {
    /// Generate adaptive response hints based on summary
    pub fn get_adaptive_hints(&self) -> Vec<String> {
        let mut hints: Vec<String> = Vec::new();

        // Based on frustration
        match self.frustration_level {
            FrustrationLevel::Frustrated => {
                hints.push("User seems frustrated - be extra patient and reassuring".to_string());
                hints.push("Double-check your solution before responding".to_string());
                hints.push("Consider asking if they want to try a different approach".to_string());
            }
            FrustrationLevel::Impatient => {
                hints.push("User is getting impatient - be concise".to_string());
                hints.push("Get to the point quickly".to_string());
            }
            FrustrationLevel::Calm => {}
        }

        // Based on success
        match self.success_level {
            SuccessLevel::Struggling => {
                hints.push("User is struggling - provide more guidance".to_string());
                hints.push("Consider breaking down the problem into smaller steps".to_string());
            }
            SuccessLevel::MakingProgress => {
                hints.push("User is making progress - encourage them".to_string());
            }
            SuccessLevel::Successful => {
                hints.push("User is successful - acknowledge their progress".to_string());
            }
        }

        // Based on communication style
        let style = &self.dialectic.style;
        match style {
            crate::CommunicationStyle::Technical => {
                hints.push("Use precise technical language".to_string());
            }
            crate::CommunicationStyle::Casual => {
                hints.push("Keep tone friendly and casual".to_string());
            }
            _ => {}
        }

        hints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustration_detection() {
        let mut analyzer = ConversationAnalyzer::new();
        
        analyzer.analyze_message(&Message {
            content: "I tried everything but it's still not working".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        assert!(matches!(
            analyzer.get_frustration_level(),
            FrustrationLevel::Impatient | FrustrationLevel::Frustrated
        ));
    }

    #[test]
    fn test_success_detection() {
        let mut analyzer = ConversationAnalyzer::new();
        
        analyzer.analyze_message(&Message {
            content: "Thanks, that works great!".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        assert!(matches!(
            analyzer.get_success_level(),
            SuccessLevel::MakingProgress | SuccessLevel::Successful
        ));
    }
}
