#[derive(Debug)]
pub enum CompressionType {
    None,
    LZ4,
    Snappy,
    Zstd,
}

const BLOCK_SIZE: u64 = 32 * 1024;
const MEMTABLE_SIZE: u64 = 64 * 1024 * 1024;
const BLOOM_BITS_PER_KEY: u8 = 10;

#[derive(Debug)]
pub struct TaurusConfig {
    block_size: u64,
    mem_table_size: u64,
    compression_algo: CompressionType,
    bloom_bits_per_key: u8,
}

impl Default for TaurusConfig {
    fn default() -> Self {
        TaurusConfig {
            block_size: BLOCK_SIZE,
            mem_table_size: MEMTABLE_SIZE,
            compression_algo: CompressionType::LZ4,
            bloom_bits_per_key: BLOOM_BITS_PER_KEY,
        }
    }
}
