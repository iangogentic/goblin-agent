//! Trajectory Compression
//!
//! Efficient compression for agent trajectories to reduce storage
//! and enable faster training data loading.

use anyhow::{ensure, Result};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use zstd::stream::{decode_all, encode_all};
use lz4::block;

/// A compressed chunk of trajectory data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedChunk {
    /// Chunk ID
    pub id: String,
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compressed data
    pub data: Vec<u8>,
    /// Original size in bytes
    pub original_size: usize,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Checksum for integrity
    pub checksum: u32,
}

impl CompressedChunk {
    /// Create a new compressed chunk
    pub fn new(id: String, algorithm: CompressionAlgorithm, data: Vec<u8>, original_size: usize) -> Self {
        let compression_ratio = if original_size > 0 {
            data.len() as f32 / original_size as f32
        } else {
            1.0
        };
        
        Self {
            id,
            algorithm,
            data,
            original_size,
            compression_ratio,
            checksum: 0, // Would compute checksum here
        }
    }
    
    /// Decompress this chunk
    pub fn decompress(&self) -> Result<Vec<u8>> {
        match self.algorithm {
            CompressionAlgorithm::Zstd => {
                Ok(decode_all(&self.data[..])?)
            }
            CompressionAlgorithm::Lz4 => {
                Ok(block::decompress(&self.data, Some(self.original_size as i32))?.to_vec())
            }
            CompressionAlgorithm::None => {
                Ok(self.data.clone())
            }
        }
    }
}

/// Generate UUID
fn generate_uuid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Zstandard compression (better ratio)
    Zstd,
    /// LZ4 compression (faster)
    Lz4,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        CompressionAlgorithm::Zstd
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-22 for zstd)
    pub level: i32,
    /// Chunk size in bytes
    pub chunk_size: usize,
    /// Enable dictionary compression
    pub use_dictionary: bool,
    /// Dictionary for this domain
    pub dictionary: Option<Vec<u8>>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            chunk_size: 64 * 1024, // 64KB chunks
            use_dictionary: false,
            dictionary: None,
        }
    }
}

/// Trajectory compressor
pub struct TrajectoryCompressor {
    config: CompressionConfig,
    dictionary: Option<Vec<u8>>,
}

impl TrajectoryCompressor {
    /// Create a new compressor
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            dictionary: config.dictionary.clone(),
            config,
        }
    }
    
    /// Create with a learned dictionary
    pub fn with_dictionary(config: CompressionConfig, dictionary: Vec<u8>) -> Self {
        Self {
            dictionary: Some(dictionary),
            config,
        }
    }
    
    /// Compress trajectory data
    pub fn compress(&self, data: &[u8]) -> Result<CompressedChunk> {
        let id = generate_uuid();
        let original_size = data.len();
        
        let compressed = match self.config.algorithm {
            CompressionAlgorithm::Zstd => {
                if let Some(ref dict) = self.dictionary {
                    // Use dictionary compression
                    zstd_compress_with_dict(data, dict, self.config.level)?
                } else {
                    encode_all(data, self.config.level)?
                }
            }
            CompressionAlgorithm::Lz4 => {
                // LZ4 compression - use default compression
                block::compress(data, None, true)?
            }
            CompressionAlgorithm::None => {
                data.to_vec()
            }
        };
        
        Ok(CompressedChunk::new(
            id,
            self.config.algorithm,
            compressed,
            original_size,
        ))
    }
    
    /// Compress a trajectory
    pub fn compress_trajectory(&self, trajectory: &crate::Trajectory) -> Result<CompressedChunk> {
        let json = serde_json::to_vec(trajectory)?;
        self.compress(&json)
    }
    
    /// Decompress trajectory
    pub fn decompress_trajectory(&self, chunk: &CompressedChunk) -> Result<crate::Trajectory> {
        let json = chunk.decompress()?;
        Ok(serde_json::from_slice(&json)?)
    }
    
    /// Build a dictionary from sample trajectories
    pub fn build_dictionary(&self, samples: Vec<&[u8]>) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::Zstd => {
                // Train dictionary from samples
                // In production, use zstd_train_dictionary
                let mut dict = Vec::new();
                for sample in &samples {
                    dict.extend_from_slice(sample);
                }
                // Truncate to reasonable size
                dict.truncate(100_000);
                Ok(dict)
            }
            _ => {
                Ok(Vec::new())
            }
        }
    }
    
    /// Split data into chunks
    pub fn chunk_data(&self, data: &[u8]) -> Vec<Vec<u8>> {
        data.chunks(self.config.chunk_size)
            .map(|c| c.to_vec())
            .collect()
    }
    
    /// Merge chunks
    pub fn merge_chunks(&self, chunks: Vec<CompressedChunk>) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        for chunk in chunks {
            let decompressed = chunk.decompress()?;
            result.extend_from_slice(&decompressed);
        }
        Ok(result)
    }
}

/// Trajectory sequence compression (compress multiple trajectories together)
pub struct SequenceCompressor {
    config: CompressionConfig,
}

impl SequenceCompressor {
    /// Create a new sequence compressor
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }
    
    /// Compress multiple trajectories as a sequence
    pub fn compress_sequence(&self, trajectories: &[crate::Trajectory]) -> Result<CompressedSequence> {
        // Encode all trajectories as a single JSON array
        let json = serde_json::to_vec(trajectories)?;
        let _original_size = json.len();
        
        // Compress
        let compressor = TrajectoryCompressor::new(self.config.clone());
        let chunk = compressor.compress(&json)?;
        
        Ok(CompressedSequence {
            num_trajectories: trajectories.len(),
            total_steps: trajectories.iter().map(|t| t.steps.len()).sum(),
            chunk,
        })
    }
    
    /// Decompress sequence
    pub fn decompress_sequence(&self, sequence: &CompressedSequence) -> Result<Vec<crate::Trajectory>> {
        let decompressed = sequence.chunk.decompress()?;
        Ok(serde_json::from_slice(&decompressed)?)
    }
}

/// A compressed sequence of trajectories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedSequence {
    pub num_trajectories: usize,
    pub total_steps: usize,
    pub chunk: CompressedChunk,
}

impl CompressedSequence {
    /// Get compression statistics
    pub fn stats(&self) -> SequenceStats {
        SequenceStats {
            num_trajectories: self.num_trajectories,
            total_steps: self.total_steps,
            compression_ratio: self.chunk.compression_ratio,
            compressed_size: self.chunk.data.len(),
            original_size: self.chunk.original_size,
        }
    }
}

/// Statistics for a compressed sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStats {
    pub num_trajectories: usize,
    pub total_steps: usize,
    pub compression_ratio: f32,
    pub compressed_size: usize,
    pub original_size: usize,
}

/// Format conversion utilities
pub mod format {
    use super::*;
    
    /// Export compressed trajectory to file
    pub async fn export_to_file(chunk: &CompressedChunk, path: &str) -> Result<()> {
        use tokio::fs;
        let data = serde_json::to_vec(chunk)?;
        fs::write(path, data).await?;
        Ok(())
    }
    
    /// Import compressed trajectory from file
    pub async fn import_from_file(path: &str) -> Result<CompressedChunk> {
        use tokio::fs;
        let data = fs::read(path).await?;
        Ok(serde_json::from_slice(&data)?)
    }
    
    /// Export to binary format (smaller than JSON)
    pub fn export_binary(chunk: &CompressedChunk) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        
        // Magic bytes
        data.extend_from_slice(b"GOBLINCMP");
        
        // Version
        data.push(1u8);
        
        // Algorithm
        data.push(match chunk.algorithm {
            CompressionAlgorithm::None => 0u8,
            CompressionAlgorithm::Zstd => 1u8,
            CompressionAlgorithm::Lz4 => 2u8,
        });
        
        // Original size (8 bytes)
        data.extend_from_slice(&chunk.original_size.to_le_bytes());
        
        // Data
        data.extend_from_slice(&chunk.data);
        
        Ok(data)
    }
    
    /// Import from binary format
    pub fn import_binary(data: &[u8]) -> Result<CompressedChunk> {
        ensure!(data.len() >= 10, "Invalid binary format");
        ensure!(&data[0..10] == b"GOBLINCMP", "Invalid magic bytes");
        
        let version = data[10];
        ensure!(version == 1, "Unsupported version");
        
        let algorithm = match data[11] {
            0 => CompressionAlgorithm::None,
            1 => CompressionAlgorithm::Zstd,
            2 => CompressionAlgorithm::Lz4,
            _ => anyhow::bail!("Unknown algorithm"),
        };
        
        let original_size = usize::from_le_bytes(data[12..20].try_into()?);
        let chunk_data = data[20..].to_vec();
        
        Ok(CompressedChunk::new(
            generate_uuid(),
            algorithm,
            chunk_data,
            original_size,
        ))
    }
}

/// Helper for zstd with dictionary
fn zstd_compress_with_dict(data: &[u8], dict: &[u8], level: i32) -> Result<Vec<u8>> {
    // In production, use zstd with dictionary training
    // For now, fall back to regular compression
    let _dict = dict; // Silence unused warning
    let _level = level;
    Ok(encode_all(data, 3)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Trajectory;

    #[test]
    fn test_compression_ratio() {
        let trajectory = Trajectory::new(
            "test task".to_string(),
            "test".to_string(),
            "gpt-4".to_string(),
        );
        
        let config = CompressionConfig::default();
        let compressor = TrajectoryCompressor::new(config);
        
        let chunk = compressor.compress_trajectory(&trajectory).unwrap();
        assert!(chunk.compression_ratio <= 1.0);
    }

    #[test]
    fn test_roundtrip() {
        let trajectory = Trajectory::new(
            "roundtrip test".to_string(),
            "coding".to_string(),
            "claude".to_string(),
        );
        
        let config = CompressionConfig::default();
        let compressor = TrajectoryCompressor::new(config);
        
        let chunk = compressor.compress_trajectory(&trajectory).unwrap();
        let decompressed = compressor.decompress_trajectory(&chunk).unwrap();
        
        assert_eq!(decompressed.task, trajectory.task);
    }
}
