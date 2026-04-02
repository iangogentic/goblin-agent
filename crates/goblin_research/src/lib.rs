//! Goblin Research Features
//!
//! Research tools for AI agent training and experimentation:
//! - Batch trajectory generation for dataset collection
//! - Atropos RL environments for reinforcement learning
//! - Trajectory compression for efficient storage

pub mod batch;
pub mod atropos;
pub mod compression;

pub use batch::{BatchRunner, Trajectory, TrajectoryStep, BatchConfig};
pub use atropos::{RLEnvironment, RLAction, RLState, RLReward, EnvironmentConfig};
pub use compression::{TrajectoryCompressor, CompressionConfig, CompressedChunk};
