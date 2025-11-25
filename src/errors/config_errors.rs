use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CompactionConfigError {
    LevelSizeMultiplierTooLow(u8),
    MaxLevelTooSmall(u8),
    MaxLevelTooBig(u8),
    L0NotEnoughFiles(u8),
    TargetFileSizeTooLow(u64),
    MaxBytesTargetSizeMismatch,
}

impl Error for CompactionConfigError {}

impl Display for CompactionConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompactionConfigError::LevelSizeMultiplierTooLow(num) => {
                write!(
                    f,
                    "Compaction Config Err: level size multiplier too low must be >= 2, is {}",
                    num
                )
            }
            CompactionConfigError::MaxLevelTooSmall(num) => {
                write!(
                    f,
                    "Compaction Config Err: max level size too small, must be >= 3, is {}",
                    num
                )
            }
            CompactionConfigError::MaxLevelTooBig(num) => {
                write!(
                    f,
                    "Compaction Config Err: max level size too big, must be <= 10, is {}",
                    num
                )
            }
            CompactionConfigError::L0NotEnoughFiles(num) => {
                write!(
                    f,
                    "Compaction Config Err: level 0 file count trigger, must be >= 2, is {}",
                    num
                )
            }
            CompactionConfigError::TargetFileSizeTooLow(num) => {
                write!(
                    f,
                    "Compaction Config Err: target file size base must be at least 1 MB, found {}",
                    num
                )
            }
            CompactionConfigError::MaxBytesTargetSizeMismatch => {
                write!(
                    f,
                    "Compaction Config Err: max_bytes_for_level_base should be a multiple of or larger than target_file_size_base"
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct CompactionConfigErrors {
    pub errors: Vec<CompactionConfigError>,
}

impl Display for CompactionConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error for CompactionConfigErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.errors.first().map(|e| e as &(dyn Error + 'static))
    }
}

impl CompactionConfigErrors {
    pub fn new() -> Self {
        CompactionConfigErrors { errors: Vec::new() }
    }
}

pub enum MvccConfigError {
    InvertedRange(usize, usize),
    MaxSnapShotTooHigh(usize),
    WarningThresholdBelowMax(u64, u64),
    GcBatchSizeTooSmall(usize),
}

impl Display for MvccConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MvccConfigError::InvertedRange(min, max) => {
                write!(
                    f,
                    "Mvcc Config Err: max snapshot ({}) must exceed min({})",
                    max, min
                )
            }
            MvccConfigError::MaxSnapShotTooHigh(max) => {
                write!(
                    f,
                    "Mvcc Config Err: max snapshots exceeds limit 10_000: is {}",
                    max
                )
            }
            MvccConfigError::WarningThresholdBelowMax(threshold, max_age) => {
                write!(
                    f,
                    "Mvcc Config Err: warning threshold (found: {}) must exceed max snapshot age (found: {})",
                    threshold, max_age
                )
            }
            MvccConfigError::GcBatchSizeTooSmall(batch_size) => {
                write!(
                    f,
                    "Mvcc Config Err: gc batch size has a minimum of 100 (found {})",
                    batch_size
                )
            }
        }
    }
}

pub struct MvccConfigErrors {
    pub errors: Vec<MvccConfigError>,
}
