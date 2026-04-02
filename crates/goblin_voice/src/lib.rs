//! Goblin Voice - Speech Transcription
//!
//! Voice transcription using Whisper for speech-to-text capabilities.
//! Supports audio capture, transcription, and voice command parsing.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

/// Audio format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioFormat {
    /// WAV format
    Wav,
    /// MP3 format
    Mp3,
    /// OGG format
    Ogg,
    /// Raw PCM
    Pcm,
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::Wav
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate
    pub sample_rate: u32,
    /// Channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Format
    pub format: AudioFormat,
    /// Max duration in seconds
    pub max_duration_secs: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bits_per_sample: 16,
            format: AudioFormat::Wav,
            max_duration_secs: 60,
        }
    }
}

/// Transcription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRequest {
    /// Audio data
    pub audio_data: Vec<u8>,
    /// Audio format
    pub format: AudioFormat,
    /// Language (None for auto-detect)
    pub language: Option<String>,
    /// Model to use
    pub model: TranscriptionModel,
    /// Task type
    pub task: TranscriptionTask,
}

impl TranscriptionRequest {
    /// Create from file
    pub async fn from_file(path: &Path) -> Result<Self> {
        let data = tokio::fs::read(path).await?;
        let format = match path.extension().and_then(|e| e.to_str()) {
            Some("wav") | Some("WAV") => AudioFormat::Wav,
            Some("mp3") | Some("MP3") => AudioFormat::Mp3,
            Some("ogg") | Some("OGG") => AudioFormat::Ogg,
            _ => AudioFormat::Wav,
        };

        Ok(Self {
            audio_data: data,
            format,
            language: None,
            model: TranscriptionModel::Base,
            task: TranscriptionTask::Transcribe,
        })
    }

    /// Create from base64 encoded audio
    pub fn from_base64(data: &str, format: AudioFormat) -> Result<Self> {
        use base64::Engine;
        let audio_data = base64::engine::general_purpose::STANDARD.decode(data)?;
        Ok(Self {
            audio_data,
            format,
            language: None,
            model: TranscriptionModel::Base,
            task: TranscriptionTask::Transcribe,
        })
    }
}

/// Transcription model size
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranscriptionModel {
    /// Tiny model (fastest, lowest quality)
    Tiny,
    /// Base model
    Base,
    /// Small model
    Small,
    /// Medium model
    Medium,
    /// Large model (slowest, highest quality)
    Large,
}

impl Default for TranscriptionModel {
    fn default() -> Self {
        TranscriptionModel::Base
    }
}

impl TranscriptionModel {
    /// Get model identifier for Whisper API
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptionModel::Tiny => "tiny",
            TranscriptionModel::Base => "base",
            TranscriptionModel::Small => "small",
            TranscriptionModel::Medium => "medium",
            TranscriptionModel::Large => "large",
        }
    }
}

/// Transcription task
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranscriptionTask {
    /// Transcribe audio to text
    Transcribe,
    /// Translate to English
    Translate,
}

impl Default for TranscriptionTask {
    fn default() -> Self {
        TranscriptionTask::Transcribe
    }
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Transcription ID
    pub id: String,
    /// Transcribed text
    pub text: String,
    /// Language detected
    pub language: Option<String>,
    /// Duration in seconds
    pub duration_secs: f32,
    /// Confidence score (0-1)
    pub confidence: f32,
    /// Word-level timestamps
    pub words: Vec<WordTimestamp>,
    /// Model used
    pub model: TranscriptionModel,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl TranscriptionResult {
    /// Create a new result
    pub fn new(text: String, model: TranscriptionModel) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text,
            language: None,
            duration_secs: 0.0,
            confidence: 0.0,
            words: Vec::new(),
            model,
            timestamp: Utc::now(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

/// Word-level timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    /// Word text
    pub word: String,
    /// Start time in seconds
    pub start_secs: f32,
    /// End time in seconds
    pub end_secs: f32,
    /// Probability
    pub probability: f32,
}

/// Whisper transcription engine
pub struct WhisperEngine {
    config: WhisperConfig,
    client: reqwest::Client,
}

impl WhisperEngine {
    /// Create a new engine
    pub fn new(config: WhisperConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Transcribe audio
    pub async fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptionResult> {
        match self.config.backend {
            WhisperBackend::OpenAI => {
                self.transcribe_openai(request).await
            }
            WhisperBackend::Local => {
                self.transcribe_local(request).await
            }
            WhisperBackend::Mock => {
                Ok(self.mock_transcription(&request))
            }
        }
    }

    /// Transcribe using OpenAI Whisper API
    async fn transcribe_openai(&self, request: TranscriptionRequest) -> Result<TranscriptionResult> {
        // In production, use reqwest with multipart feature
        // For now, return mock result
        let _ = request;
        Ok(self.mock_transcription(&request))
    }

    /// Transcribe using local Whisper model
    async fn transcribe_local(&self, request: TranscriptionRequest) -> Result<TranscriptionResult> {
        // In production, use whisper.cpp bindings
        // For now, return mock result
        Ok(self.mock_transcription(&request))
    }

    /// Mock transcription for testing
    fn mock_transcription(&self, request: &TranscriptionRequest) -> TranscriptionResult {
        let audio_len = request.audio_data.len();
        let duration = (audio_len as f32 / (request.format.sample_rate() * 2) as f32).max(1.0);
        
        TranscriptionResult {
            id: Uuid::new_v4().to_string(),
            text: format!("Mock transcription of {:.1}s audio clip", duration),
            language: Some("en".to_string()),
            duration_secs: duration,
            confidence: 0.95,
            words: vec![
                WordTimestamp {
                    word: "Mock".to_string(),
                    start_secs: 0.0,
                    end_secs: 0.3,
                    probability: 0.98,
                },
                WordTimestamp {
                    word: "transcription".to_string(),
                    start_secs: 0.3,
                    end_secs: 0.8,
                    probability: 0.95,
                },
            ],
            model: request.model,
            timestamp: Utc::now(),
        }
    }
}

/// Whisper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// Backend to use
    pub backend: WhisperBackend,
    /// API URL (for OpenAI-compatible APIs)
    pub api_url: String,
    /// API key (for OpenAI)
    pub api_key: Option<String>,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            backend: WhisperBackend::Mock,
            api_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
        }
    }
}

/// Whisper backend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WhisperBackend {
    /// OpenAI API
    OpenAI,
    /// Local whisper.cpp
    Local,
    /// Mock for testing
    Mock,
}

impl Default for WhisperBackend {
    fn default() -> Self {
        WhisperBackend::Mock
    }
}

/// Voice command parser
pub struct VoiceCommandParser {
    /// Command patterns
    patterns: Vec<CommandPattern>,
}

impl VoiceCommandParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            patterns: default_commands(),
        }
    }

    /// Parse transcription into command
    pub fn parse(&self, transcription: &TranscriptionResult) -> Option<VoiceCommand> {
        let text = transcription.text.to_lowercase();
        
        for pattern in &self.patterns {
            if let Some(captures) = pattern.regex.captures(&text) {
                return Some(VoiceCommand {
                    command: pattern.command.clone(),
                    args: captures
                        .iter()
                        .skip(1)
                        .filter_map(|m| m.map(|m| m.as_str().to_string()))
                        .collect(),
                    confidence: transcription.confidence,
                    original_text: transcription.text.clone(),
                });
            }
        }
        
        None
    }
}

impl Default for VoiceCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Voice command
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    /// Command name
    pub command: String,
    /// Arguments
    pub args: Vec<String>,
    /// Confidence
    pub confidence: f32,
    /// Original transcription
    pub original_text: String,
}

/// Command pattern
struct CommandPattern {
    pattern: String,
    regex: regex::Regex,
    command: String,
}

/// Default voice commands
fn default_commands() -> Vec<CommandPattern> {
    vec![
        CommandPattern {
            pattern: r"run (.+)".to_string(),
            regex: regex::Regex::new(r"run (.+)").unwrap(),
            command: "run".to_string(),
        },
        CommandPattern {
            pattern: r"search for (.+)".to_string(),
            regex: regex::Regex::new(r"search for (.+)").unwrap(),
            command: "search".to_string(),
        },
        CommandPattern {
            pattern: r"open (.+)".to_string(),
            regex: regex::Regex::new(r"open (.+)").unwrap(),
            command: "open".to_string(),
        },
        CommandPattern {
            pattern: r"create file (.+)".to_string(),
            regex: regex::Regex::new(r"create file (.+)").unwrap(),
            command: "create_file".to_string(),
        },
        CommandPattern {
            pattern: r"commit".to_string(),
            regex: regex::Regex::new(r"commit").unwrap(),
            command: "commit".to_string(),
        },
    ]
}

/// Audio capture interface
pub trait AudioCapture: Send + Sync {
    /// Start capturing
    fn start(&mut self) -> Result<()>;
    
    /// Stop capturing
    fn stop(&mut self) -> Result<Vec<u8>>;
    
    /// Check if capturing
    fn is_capturing(&self) -> bool;
}

/// Microphone capture
pub struct MicrophoneCapture {
    config: AudioConfig,
    capturing: bool,
    samples: Vec<u8>,
}

impl MicrophoneCapture {
    /// Create a new microphone capture
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            capturing: false,
            samples: Vec::new(),
        }
    }
}

impl AudioCapture for MicrophoneCapture {
    fn start(&mut self) -> Result<()> {
        self.capturing = true;
        self.samples.clear();
        Ok(())
    }
    
    fn stop(&mut self) -> Result<Vec<u8>> {
        self.capturing = false;
        Ok(self.samples.clone())
    }
    
    fn is_capturing(&self) -> bool {
        self.capturing
    }
}

/// WAV file utilities
pub mod wav {
    use super::*;

    /// Read WAV file
    pub fn read_wav(data: &[u8]) -> Result<WavData> {
        let cursor = std::io::Cursor::new(data);
        let reader = hound::WavReader::new(cursor)?;
        
        let spec = reader.spec();
        let samples: Vec<i16> = reader.into_samples::<i16>()
            .filter_map(|s| s.ok())
            .collect();
        
        Ok(WavData {
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            samples,
        })
    }

    /// Write WAV file
    pub fn write_wav(data: &WavData) -> Result<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: data.channels,
            sample_rate: data.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
            for sample in &data.samples {
                writer.write_sample(*sample)?;
            }
            writer.finalize()?;
        }
        
        Ok(cursor.into_inner())
    }
}

/// WAV data
#[derive(Debug, Clone)]
pub struct WavData {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>,
}

impl AudioFormat {
    /// Get sample rate for this format
    pub fn sample_rate(&self) -> u32 {
        16000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_result() {
        let result = TranscriptionResult::new("Hello world".to_string(), TranscriptionModel::Base);
        assert_eq!(result.word_count(), 2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_voice_command_parser() {
        let parser = VoiceCommandParser::new();
        
        let transcription = TranscriptionResult {
            id: "test".to_string(),
            text: "run cargo build".to_string(),
            language: Some("en".to_string()),
            duration_secs: 1.0,
            confidence: 0.9,
            words: Vec::new(),
            model: TranscriptionModel::Base,
            timestamp: Utc::now(),
        };
        
        let command = parser.parse(&transcription);
        assert!(command.is_some());
        assert_eq!(command.unwrap().command, "run");
    }
}
