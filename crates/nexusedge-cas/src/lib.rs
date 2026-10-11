//! nexusedge-cas: Content-Addressed Storage engine with FastCDC chunking and Merkle indexing.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkInfo {
    pub hash: String,
    pub size: usize,
    pub offset: u64,
}

pub struct CasEngine {
    storage_dir: PathBuf,
}

impl CasEngine {
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
        }
    }

    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }
}
