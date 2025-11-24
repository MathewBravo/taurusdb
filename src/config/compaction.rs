#[derive(Debug)]
pub enum CompactionStrategy {
    Leveled,
    Tiered,
    Hybrid,
}

#[derive(Debug)]
pub struct CompactionConfig {
    compaction_strategy: CompactionStrategy,
    level_size_muliplier: u8,
    max_levels: u8,
    l0_file_count_compaction_trigger: u8,
    max_bytes_for_level_base: u64,
    target_file_size_base: u64,
}

const DEFAULT_LEVEL_SIZE_MULITPLIER: u8 = 10;
const DEFAULT_MAX_LEVELS: u8 = 7;
const DEFAULT_LEVEL_0_FILE_COUNT_COMPACTION_TRIGGER: u8 = 10;
const DEFAULT_MAX_BYTES_FOR_LEVEL_BASE: u64 = 512 * 1024 * 1024;
const DEFAULT_TARGET_FILE_SIZE_BASE: u64 = 64 * 1024 * 1024;

impl Default for CompactionConfig {
    fn default() -> Self {
        CompactionConfig {
            compaction_strategy: CompactionStrategy::Leveled,
            level_size_muliplier: DEFAULT_LEVEL_SIZE_MULITPLIER,
            max_levels: DEFAULT_MAX_LEVELS,
            l0_file_count_compaction_trigger: DEFAULT_LEVEL_0_FILE_COUNT_COMPACTION_TRIGGER,
            max_bytes_for_level_base: DEFAULT_MAX_BYTES_FOR_LEVEL_BASE,
            target_file_size_base: DEFAULT_TARGET_FILE_SIZE_BASE,
        }
    }
}

impl CompactionConfig {}
