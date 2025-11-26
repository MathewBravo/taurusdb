use crate::errors::config_errors::{TaurusConfigError, TaurusConfigErrors};

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

impl TaurusConfig {
    pub fn validate(&self) -> Result<(), TaurusConfigErrors> {
        let mut err = TaurusConfigErrors::new();

        // Check if block_size is a power of 2
        if self.block_size == 0 || (self.block_size & (self.block_size - 1)) != 0 {
            err.errors
                .push(TaurusConfigError::BlockSizeNotPowerOfTwo(self.block_size));
        }

        if self.block_size < 4 * 1024 {
            err.errors
                .push(TaurusConfigError::BlockSizeTooSmall(self.block_size));
        }

        if self.block_size > 128 * 1024 {
            err.errors
                .push(TaurusConfigError::BlockSizeTooLarge(self.block_size));
        }

        if self.mem_table_size < 1024 * 1024 {
            err.errors
                .push(TaurusConfigError::MemtableSizeTooSmall(self.mem_table_size));
        }

        if self.mem_table_size > 1024 * 1024 * 1024 {
            err.errors
                .push(TaurusConfigError::MemtableSizeTooLarge(self.mem_table_size));
        }

        if self.mem_table_size <= self.block_size {
            err.errors.push(TaurusConfigError::MemtableSmallerThanBlock(
                self.mem_table_size,
                self.block_size,
            ));
        }

        if self.bloom_bits_per_key < 4 {
            err.errors.push(TaurusConfigError::BloomBitsPerKeyTooLow(
                self.bloom_bits_per_key,
            ));
        }

        if self.bloom_bits_per_key > 20 {
            err.errors.push(TaurusConfigError::BloomBitsPerKeyTooHigh(
                self.bloom_bits_per_key,
            ));
        }

        if err.errors.is_empty() {
            return Ok(());
        }
        Err(err)
    }
}
