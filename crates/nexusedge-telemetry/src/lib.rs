//! nexusedge-telemetry: eBPF & hardware metrics collector.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceVector {
    pub cpu_usage_pct: f32,
    pub ram_available_mb: u64,
    pub gpu_vram_available_mb: Option<u64>,
    pub thermal_status: String,
}

impl Default for ResourceVector {
    fn default() -> Self {
        Self {
            cpu_usage_pct: 0.0,
            ram_available_mb: 8192,
            gpu_vram_available_mb: None,
            thermal_status: "Normal".to_string(),
        }
    }
}

pub struct TelemetryCollector;

impl TelemetryCollector {
    pub fn poll_resource_vector() -> ResourceVector {
        ResourceVector::default()
    }
}
