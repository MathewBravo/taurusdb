use crate::errors::config_errors::{CacheConfigError, CacheConfigErrors};

const DEFAULT_BLOCK_CACHE_SIZE: u64 = 32 * 1024 * 1024;
const DEFAULT_CACHE_BLOOM_FILTER: bool = true;
const DEFAULT_CACHE_INDEX_BLOCKS: bool = true;

#[derive(Debug)]
enum CacheEvictionPolicy {
    WTinyLFU,
}

#[derive(Debug)]
pub struct CacheConfig {
    block_cache_size: u64,
    cache_index_blocks: bool,
    cache_bloom_filters: bool,
    cache_eviction_policy: CacheEvictionPolicy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            block_cache_size: DEFAULT_BLOCK_CACHE_SIZE,
            cache_index_blocks: DEFAULT_CACHE_INDEX_BLOCKS,

            cache_bloom_filters: DEFAULT_CACHE_BLOOM_FILTER,
            cache_eviction_policy: CacheEvictionPolicy::WTinyLFU,
        }
    }
}

impl CacheConfig {
    pub fn validate(&self) -> Result<(), CacheConfigErrors> {
        let mut err = CacheConfigErrors::new();

        if self.block_cache_size < 1024 * 1024 {
            err.errors.push(CacheConfigError::BlockCacheSizeTooSmall(
                self.block_cache_size,
            ));
        }

        if err.errors.is_empty() {
            return Ok(());
        }
        Err(err)
    }
}
