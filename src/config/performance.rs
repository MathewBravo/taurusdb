use crate::errors::config_errors::{PerformanceConfigError, PerformanceConfigErrors};

pub const DEFAULT_COMPACTION_THREADS: usize = 4;
pub const DEFAULT_READAHEAD_SIZE: usize = 4 * 1024 * 1024;
pub const DEFAULT_BATCH_SIZE: usize = 1000;
pub const DEFAULT_BATCH_BYTES: usize = 4 * 1024 * 1024;
pub const DEFAULT_PERIODIC_INTERVALS_MS: u64 = 1000;
pub const DEFAULT_MAX_READ_THREADS: usize = 8;
pub const DEFAULT_MAX_WRITE_THREADS: usize = 4;
pub const DEFAULT_SCAN_PARALLELISM: usize = 2;

pub enum WalSyncMode {
    EveryWrite,
    Batch,
    Periodic,
}

pub struct WalSyncConfig {
    pub mode: WalSyncMode,
    pub batch_size: usize,
    pub batch_bytes: usize,
    pub periodic_interval_ms: u64,
}

impl Default for WalSyncConfig {
    fn default() -> Self {
        Self {
            mode: WalSyncMode::Batch,
            batch_size: DEFAULT_BATCH_SIZE,
            batch_bytes: DEFAULT_BATCH_BYTES,
            periodic_interval_ms: DEFAULT_PERIODIC_INTERVALS_MS,
        }
    }
}

pub struct ParallelismConfig {
    pub max_read_threads: usize,
    pub max_write_threads: usize,
    pub scan_parallelism: usize,
}

impl Default for ParallelismConfig {
    fn default() -> Self {
        Self {
            max_read_threads: DEFAULT_MAX_READ_THREADS,
            max_write_threads: DEFAULT_MAX_WRITE_THREADS,
            scan_parallelism: DEFAULT_SCAN_PARALLELISM,
        }
    }
}

pub struct PerformanceConfig {
    pub compaction_threads: usize,
    pub wal_sync: WalSyncConfig,
    pub readahead_size: usize,
    pub parallelism: ParallelismConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            compaction_threads: num_cpus::get().clamp(2, 8) / 2,
            wal_sync: WalSyncConfig::default(),
            readahead_size: DEFAULT_READAHEAD_SIZE,
            parallelism: ParallelismConfig::default(),
        }
    }
}

impl PerformanceConfig {
    pub fn validate(&self) -> Result<(), PerformanceConfigErrors> {
        let mut err = PerformanceConfigErrors::new();

        if self.compaction_threads > num_cpus::get() * 2 {
            err.errors
                .push(PerformanceConfigError::CompactionThreadsTooHigh(
                    self.compaction_threads,
                ));
        }

        if self.readahead_size > 64 * 1024 * 1024 {
            err.errors
                .push(PerformanceConfigError::ReadaheadSizeTooHigh(
                    self.readahead_size,
                ));
        }

        match self.wal_sync.mode {
            WalSyncMode::Batch => {
                if self.wal_sync.batch_size == 0 {
                    err.errors.push(PerformanceConfigError::WalBatchSizeZero);
                }
                if self.wal_sync.batch_bytes == 0 {
                    err.errors.push(PerformanceConfigError::WalBatchBytesZero);
                }
            }
            WalSyncMode::Periodic => {
                if self.wal_sync.periodic_interval_ms == 0 {
                    err.errors
                        .push(PerformanceConfigError::WalPeriodicIntervalZero);
                }
            }
            WalSyncMode::EveryWrite => {}
        }

        if self.parallelism.scan_parallelism > self.parallelism.max_read_threads {
            err.errors
                .push(PerformanceConfigError::ScanParallelismExceedsReadThreads(
                    self.parallelism.scan_parallelism,
                    self.parallelism.max_read_threads,
                ));
        }

        if err.errors.is_empty() {
            return Ok(());
        }
        Err(err)
    }
}
