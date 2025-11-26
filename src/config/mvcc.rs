use crate::errors::config_errors::{MvccConfigError, MvccConfigErrors};

const DEFAULT_MIN_SNAPSHOTS: usize = 5;
const DEFAULT_MAX_SNAPSHOTS: usize = 1000;
const DEFAULT_SNAPSHOT_AGE_SECS: u64 = 3600;
const DEFAULT_GC_INTERVAL_SECS: u64 = 300;
const DEFAULT_GC_BATCH_SIZE: usize = 1000;
const DEFAULT_MIN_OBSOLETE_VERSION: usize = 10000;
const DEFAULT_SNAPSHOT_AGE_WARNING_THRESHOLD_SECS: u64 = 1800;

pub struct SnapshotRetentionPolicy {
    pub min_snapshots: usize,
    pub max_snapshots: usize,
    pub max_snapshot_age_secs: u64,
}

impl Default for SnapshotRetentionPolicy {
    fn default() -> Self {
        SnapshotRetentionPolicy {
            min_snapshots: DEFAULT_MIN_SNAPSHOTS,
            max_snapshots: DEFAULT_MAX_SNAPSHOTS,
            max_snapshot_age_secs: DEFAULT_SNAPSHOT_AGE_SECS,
        }
    }
}

pub struct GarbageCollectionConfig {
    pub gc_interval_secs: u64,
    pub gc_batch_size: usize,
    pub min_obsolete_versions: usize,
}

impl Default for GarbageCollectionConfig {
    fn default() -> Self {
        GarbageCollectionConfig {
            gc_interval_secs: DEFAULT_GC_INTERVAL_SECS,
            gc_batch_size: DEFAULT_GC_BATCH_SIZE,
            min_obsolete_versions: DEFAULT_MIN_OBSOLETE_VERSION,
        }
    }
}

pub struct MvccConfig {
    pub snapshot_retention: SnapshotRetentionPolicy,
    pub gc_config: GarbageCollectionConfig,
    pub snapshot_age_warning_threshold_secs: u64,
}

impl Default for MvccConfig {
    fn default() -> Self {
        MvccConfig {
            snapshot_retention: SnapshotRetentionPolicy::default(),
            gc_config: GarbageCollectionConfig::default(),
            snapshot_age_warning_threshold_secs: DEFAULT_SNAPSHOT_AGE_WARNING_THRESHOLD_SECS,
        }
    }
}

impl MvccConfig {
    pub fn validate(&self) -> Result<(), MvccConfigErrors> {
        let mut err = MvccConfigErrors::new();

        if self.snapshot_retention.min_snapshots >= self.snapshot_retention.max_snapshots {
            err.errors.push(MvccConfigError::InvertedRange(
                self.snapshot_retention.min_snapshots,
                self.snapshot_retention.max_snapshots,
            ));
        }

        if self.snapshot_retention.max_snapshots > 10_000 {
            err.errors.push(MvccConfigError::MaxSnapShotTooHigh(
                self.snapshot_retention.max_snapshots,
            ));
        }

        if self.snapshot_age_warning_threshold_secs >= self.snapshot_retention.max_snapshot_age_secs
        {
            err.errors.push(MvccConfigError::WarningThresholdBelowMax(
                self.snapshot_age_warning_threshold_secs,
                self.snapshot_retention.max_snapshot_age_secs,
            ));
        }

        if self.gc_config.gc_batch_size < 100 {
            err.errors.push(MvccConfigError::GcBatchSizeTooSmall(
                self.gc_config.gc_batch_size,
            ));
        }

        if err.errors.is_empty() {
            return Ok(());
        }
        Err(err)
    }
}
