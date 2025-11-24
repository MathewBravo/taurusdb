use std::fmt::Display;

pub enum CompactionConfigError {
    LevelSizeMultiplierTooLow(u8),
    MaxLevelTooSmall(u8),
    MaxLevelTooBig(u8),
    L0NotEnoughFiles(u8),
    TargetFileSizeTooLow(u64),
    MaxBytesTargetSizeMismatch,
}

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
