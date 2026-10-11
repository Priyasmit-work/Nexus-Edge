//! nexusedge-execution: Sandbox execution engine (WASM / cgroups).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub max_memory_mb: usize,
    pub max_execution_time_ms: u64,
    pub cpu_cores: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 256,
            max_execution_time_ms: 5000,
            cpu_cores: 1,
        }
    }
}

pub struct ExecutionEngine {
    limits: ExecutionLimits,
}

impl ExecutionEngine {
    pub fn new(limits: ExecutionLimits) -> Self {
        Self { limits }
    }

    pub fn limits(&self) -> &ExecutionLimits {
        &self.limits
    }
}
