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

// ===========================================
// |        Mvcc Config Errors               |
// ===========================================

#[derive(Debug)]
pub enum MvccConfigError {
    InvertedRange(usize, usize),
    MaxSnapShotTooHigh(usize),
    WarningThresholdBelowMax(u64, u64),
    GcBatchSizeTooSmall(usize),
}

impl Error for MvccConfigError {}

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

#[derive(Debug)]
pub struct MvccConfigErrors {
    pub errors: Vec<MvccConfigError>,
}

impl MvccConfigErrors {
    pub fn new() -> Self {
        MvccConfigErrors { errors: Vec::new() }
    }
}

impl Display for MvccConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error for MvccConfigErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.errors.first().map(|e| e as &(dyn Error + 'static))
    }
}

// ===========================================
// |        Performance Config Errors        |
// ===========================================

#[derive(Debug)]
pub enum PerformanceConfigError {
    CompactionThreadsTooHigh(usize),
    ReadaheadSizeTooHigh(usize),
    WalBatchSizeZero,
    WalBatchBytesZero,
    WalPeriodicIntervalZero,
    ScanParallelismExceedsReadThreads(usize, usize),
}

impl Error for PerformanceConfigError {}

impl Display for PerformanceConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceConfigError::CompactionThreadsTooHigh(threads) => {
                write!(
                    f,
                    "Performance Config Err: compaction threads too high, must be <= num_cpus * 2 (found {})",
                    threads
                )
            }
            PerformanceConfigError::ReadaheadSizeTooHigh(size) => {
                write!(
                    f,
                    "Performance Config Err: readahead size too high, must be <= 64 MB (found {})",
                    size
                )
            }
            PerformanceConfigError::WalBatchSizeZero => {
                write!(
                    f,
                    "Performance Config Err: WAL batch size must be > 0 when using batch mode"
                )
            }
            PerformanceConfigError::WalBatchBytesZero => {
                write!(
                    f,
                    "Performance Config Err: WAL batch bytes must be > 0 when using batch mode"
                )
            }
            PerformanceConfigError::WalPeriodicIntervalZero => {
                write!(
                    f,
                    "Performance Config Err: WAL periodic interval must be > 0 when using periodic mode"
                )
            }
            PerformanceConfigError::ScanParallelismExceedsReadThreads(scan, read) => {
                write!(
                    f,
                    "Performance Config Err: scan parallelism ({}) cannot exceed max_read_threads ({})",
                    scan, read
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct PerformanceConfigErrors {
    pub errors: Vec<PerformanceConfigError>,
}

impl PerformanceConfigErrors {
    pub fn new() -> Self {
        PerformanceConfigErrors { errors: Vec::new() }
    }
}

impl Display for PerformanceConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error for PerformanceConfigErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.errors.first().map(|e| e as &(dyn Error + 'static))
    }
}

// ===========================================
// |        Taurus Config Errors             |
// ===========================================

#[derive(Debug)]
pub enum TaurusConfigError {
    BlockSizeNotPowerOfTwo(u64),
    BlockSizeTooSmall(u64),
    BlockSizeTooLarge(u64),
    MemtableSizeTooSmall(u64),
    MemtableSizeTooLarge(u64),
    MemtableSmallerThanBlock(u64, u64),
    BloomBitsPerKeyTooLow(u8),
    BloomBitsPerKeyTooHigh(u8),
}

impl Error for TaurusConfigError {}

impl Display for TaurusConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaurusConfigError::BlockSizeNotPowerOfTwo(size) => {
                write!(
                    f,
                    "Taurus Config Err: block size must be a power of 2 (found {})",
                    size
                )
            }
            TaurusConfigError::BlockSizeTooSmall(size) => {
                write!(
                    f,
                    "Taurus Config Err: block size must be >= 4 KB (found {})",
                    size
                )
            }
            TaurusConfigError::BlockSizeTooLarge(size) => {
                write!(
                    f,
                    "Taurus Config Err: block size must be <= 128 KB (found {})",
                    size
                )
            }
            TaurusConfigError::MemtableSizeTooSmall(size) => {
                write!(
                    f,
                    "Taurus Config Err: memtable size must be >= 1 MB (found {})",
                    size
                )
            }
            TaurusConfigError::MemtableSizeTooLarge(size) => {
                write!(
                    f,
                    "Taurus Config Err: memtable size must be <= 1 GB (found {})",
                    size
                )
            }
            TaurusConfigError::MemtableSmallerThanBlock(memtable, block) => {
                write!(
                    f,
                    "Taurus Config Err: memtable size ({}) must be larger than block size ({})",
                    memtable, block
                )
            }
            TaurusConfigError::BloomBitsPerKeyTooLow(bits) => {
                write!(
                    f,
                    "Taurus Config Err: bloom bits per key must be >= 4 (found {})",
                    bits
                )
            }
            TaurusConfigError::BloomBitsPerKeyTooHigh(bits) => {
                write!(
                    f,
                    "Taurus Config Err: bloom bits per key must be <= 20 (found {})",
                    bits
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct TaurusConfigErrors {
    pub errors: Vec<TaurusConfigError>,
}

impl TaurusConfigErrors {
    pub fn new() -> Self {
        TaurusConfigErrors { errors: Vec::new() }
    }
}

impl Display for TaurusConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error for TaurusConfigErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.errors.first().map(|e| e as &(dyn Error + 'static))
    }
}

// ===========================================
// |        Cache Config Errors              |
// ===========================================

#[derive(Debug)]
pub enum CacheConfigError {
    BlockCacheSizeTooSmall(u64),
}

impl Error for CacheConfigError {}

impl Display for CacheConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheConfigError::BlockCacheSizeTooSmall(size) => {
                write!(
                    f,
                    "Cache Config Err: block cache size must be >= 1 MB (found {})",
                    size
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct CacheConfigErrors {
    pub errors: Vec<CacheConfigError>,
}

impl CacheConfigErrors {
    pub fn new() -> Self {
        CacheConfigErrors { errors: Vec::new() }
    }
}

impl Display for CacheConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl Error for CacheConfigErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.errors.first().map(|e| e as &(dyn Error + 'static))
    }
}
