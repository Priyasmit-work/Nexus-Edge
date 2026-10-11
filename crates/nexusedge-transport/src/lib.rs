//! nexusedge-transport: MPQUIC transport and network scheduling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddress {
    pub peer_id: String,
    pub address: String,
}

pub struct TransportManager;

impl TransportManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}
