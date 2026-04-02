//! Dialectic Tracker
//!
//! Tracks and analyzes conversation patterns to understand how users communicate.
//!
//! The dialectic tracker observes:
//! - Language formality level
//! - Question asking patterns
//! - Response to explanations (patient vs impatient)
//! - Technical vs non-technical communication
//! - Emotional tone

use crate::{CommunicationStyle, DialecticAnalysis, Message, MessageRole, ToneIndicator};
use std::collections::HashMap;

/// Tracks dialectic patterns in conversations
pub struct DialecticTracker {
    /// Message history for analysis
    messages: Vec<Message>,
    /// Word frequency counts
    word_counts: HashMap<String, usize>,
    /// Question tracking
    questions_asked: usize,
    /// Longest response length
    max_response_length: usize,
    /// Code block indicators
    code_blocks: usize,
    /// Technical terms used
    technical_terms: usize,
}

impl DialecticTracker {
    /// Create a new dialectic tracker
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            word_counts: HashMap::new(),
            questions_asked: 0,
            max_response_length: 0,
            code_blocks: 0,
            technical_terms: 0,
        }
    }

    /// Add a message to track
    pub fn track(&mut self, message: &Message) {
        self.messages.push(message.clone());
        
        if message.role == MessageRole::User {
            let content = &message.content;
            
            // Track word frequencies
            for word in content.split_whitespace() {
                *self.word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
            }
            
            // Track response length
            if content.len() > self.max_response_length {
                self.max_response_length = content.len();
            }
            
            // Track questions
            if content.contains('?') {
                self.questions_asked += 1;
            }
            
            // Track code blocks
            if content.contains("```") {
                self.code_blocks += 1;
            }
            
            // Track technical terms
            let technical_markers = [
                "api", "function", "method", "class", "struct", "enum",
                "async", "trait", "impl", "generic", "closure", "borrow",
                "pointer", "thread", "mutex", "channel", "runtime",
            ];
            
            let content_lower = content.to_lowercase();
            for term in technical_markers {
                if content_lower.contains(term) {
                    self.technical_terms += 1;
                }
            }
        }
    }

    /// Analyze the tracked conversations
    pub fn analyze(&self) -> DialecticAnalysis {
        let style = self.determine_communication_style();
        let tone = self.determine_tone();
        let learning_patterns = self.detect_learning_patterns();
        let inferred_preferences = self.infer_preferences();

        DialecticAnalysis {
            style,
            tone,
            learning_patterns,
            inferred_preferences,
        }
    }

    /// Determine communication style from tracked messages
    fn determine_communication_style(&self) -> CommunicationStyle {
        let total_messages = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .count();

        if total_messages == 0 {
            return CommunicationStyle::Mixed;
        }

        // High technical term usage = Technical
        let technical_ratio = self.technical_terms as f32 / total_messages as f32;
        if technical_ratio > 0.3 {
            return CommunicationStyle::Technical;
        }

        // High code block usage = Technical
        let code_ratio = self.code_blocks as f32 / total_messages as f32;
        if code_ratio > 0.2 {
            return CommunicationStyle::Technical;
        }

        // Short responses = Casual/Concise
        let avg_length = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .map(|m| m.content.len())
            .sum::<usize>() as f32
            / total_messages as f32;

        if avg_length < 100.0 {
            return CommunicationStyle::Casual;
        }

        // Long responses with formal words = Formal
        let formal_words = ["therefore", "however", "furthermore", "nevertheless", "hence"];
        let formal_count = self.word_counts.iter()
            .filter(|(word, _)| formal_words.contains(&word.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>();

        let formal_ratio = formal_count as f32 / total_messages as f32;
        if formal_ratio > 0.1 && avg_length > 200.0 {
            return CommunicationStyle::Formal;
        }

        CommunicationStyle::Mixed
    }

    /// Determine tone indicators from tracked messages
    fn determine_tone(&self) -> Vec<ToneIndicator> {
        let mut tones = Vec::new();

        let total_messages = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .count();

        if total_messages == 0 {
            return tones;
        }

        // Check for questions (curious)
        let question_ratio = self.questions_asked as f32 / total_messages as f32;
        if question_ratio > 0.3 {
            tones.push(ToneIndicator::Curious);
        }

        // Check for patience/impatience based on response length
        let avg_length = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .map(|m| m.content.len())
            .sum::<usize>() as f32
            / total_messages as f32;

        if avg_length < 50.0 {
            tones.push(ToneIndicator::Impatient);
        } else if avg_length > 200.0 {
            tones.push(ToneIndicator::Patient);
        }

        // Check for verbosity
        if self.max_response_length > 500 {
            tones.push(ToneIndicator::Verbose);
        } else if avg_length < 100.0 {
            tones.push(ToneIndicator::Concise);
        }

        // Check for directness (short sentences, imperative)
        let direct_words = ["do", "make", "fix", "create", "delete", "run"];
        let direct_count = self.word_counts.iter()
            .filter(|(word, _)| direct_words.contains(&word.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>();

        let direct_ratio = direct_count as f32 / total_messages as f32;
        if direct_ratio > 0.2 {
            tones.push(ToneIndicator::Direct);
        }

        // Check for enthusiasm
        let enthusiastic_words = ["great", "awesome", "perfect", "thanks", "love", "nice"];
        let enthusiastic_count = self.word_counts.iter()
            .filter(|(word, _)| enthusiastic_words.contains(&word.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>();

        let enthusiastic_ratio = enthusiastic_count as f32 / total_messages as f32;
        if enthusiastic_ratio > 0.1 {
            tones.push(ToneIndicator::Enthusiastic);
        }

        // Check for skepticism
        let skeptical_words = ["but", "however", "actually", "wait", "really"];
        let skeptical_count = self.word_counts.iter()
            .filter(|(word, _)| skeptical_words.contains(&word.as_str()))
            .map(|(_, count)| count)
            .sum::<usize>();

        let skeptical_ratio = skeptical_count as f32 / total_messages as f32;
        if skeptical_ratio > 0.15 {
            tones.push(ToneIndicator::Skeptical);
        }

        tones
    }

    /// Detect learning patterns
    fn detect_learning_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // High question ratio = asks for explanations
        let total_messages = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .count();

        if total_messages > 0 {
            let question_ratio = self.questions_asked as f32 / total_messages as f32;
            if question_ratio > 0.4 {
                patterns.push("Prefers explanations over direct answers".to_string());
            }
        }

        // Uses code blocks = learns by example
        if self.code_blocks > 0 {
            patterns.push("Learns best through code examples".to_string());
        }

        // High technical term usage = self-directed learner
        if self.technical_terms > 5 {
            patterns.push("Self-directed learner who researches independently".to_string());
        }

        // Checks understanding
        let understanding_checks = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .filter(|m| m.content.to_lowercase().contains("understand") || 
                      m.content.to_lowercase().contains("correct") ||
                      m.content.to_lowercase().contains("right"))
            .count();

        if understanding_checks > 0 {
            patterns.push("Verifies understanding frequently".to_string());
        }

        patterns
    }

    /// Infer user preferences from behavior
    fn infer_preferences(&self) -> Vec<String> {
        let mut preferences = Vec::new();

        // Prefers code over prose
        if self.code_blocks > 2 {
            preferences.push("Prefers code examples to textual explanations".to_string());
        }

        // Prefers brevity
        let total_messages = self.messages.iter()
            .filter(|m| m.role == MessageRole::User)
            .count();

        if total_messages > 0 {
            let avg_length = self.messages.iter()
                .filter(|m| m.role == MessageRole::User)
                .map(|m| m.content.len())
                .sum::<usize>() as f32
                / total_messages as f32;

            if avg_length < 100.0 {
                preferences.push("Prefers concise, to-the-point responses".to_string());
            } else if avg_length > 300.0 {
                preferences.push("Appreciates detailed explanations".to_string());
            }
        }

        // Prefers technical language
        if self.technical_terms > 5 {
            preferences.push("Comfortable with technical terminology".to_string());
        }

        // Asks follow-up questions
        if self.questions_asked > 3 {
            preferences.push("Digs deeper with follow-up questions".to_string());
        }

        preferences
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.messages.clear();
        self.word_counts.clear();
        self.questions_asked = 0;
        self.max_response_length = 0;
        self.code_blocks = 0;
        self.technical_terms = 0;
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for DialecticTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casual_style_detection() {
        let mut tracker = DialecticTracker::new();
        
        tracker.track(&Message {
            content: "hey can you help me".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        tracker.track(&Message {
            content: "cool thanks".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        let analysis = tracker.analyze();
        assert_eq!(analysis.style, CommunicationStyle::Casual);
    }

    #[test]
    fn test_technical_style_detection() {
        let mut tracker = DialecticTracker::new();
        
        tracker.track(&Message {
            content: "I'm trying to implement an async trait with generic bounds".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        tracker.track(&Message {
            content: "```rust\nimpl<T> MyTrait for T where T: Send {}\n```".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
        });
        
        let analysis = tracker.analyze();
        assert_eq!(analysis.style, CommunicationStyle::Technical);
    }
}
