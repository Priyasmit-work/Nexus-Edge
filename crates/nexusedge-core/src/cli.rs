use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "nexusedge",
    author = "NexusEdge Contributors",
    version = "0.1.0",
    about = "Autonomous Offline-First Edge Compute & Content Swarming Engine",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Fetch or swarm a content-addressed asset by name or hash
    Get {
        /// Asset name or CAS content hash
        asset: String,

        /// Output path for reassembled asset
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run the NexusEdge background daemon node
    Daemon {
        /// Listening port for P2P swarming
        #[arg(short, long, default_value_t = 9000)]
        port: u16,

        /// Path to custom node configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Display current node status, eBPF telemetry, and connected peers
    Status,

    /// Submit a compute task (e.g., WASM module) to the edge mesh
    Submit {
        /// Path to WASM file to execute
        #[arg(short, long)]
        wasm: PathBuf,

        /// Task memory limit in MB
        #[arg(long, default_value_t = 256)]
        max_memory_mb: usize,
    },

    /// Print local node ID and cryptographic identity details
    NodeInfo,
}
