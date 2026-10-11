pub mod cli;

pub use cli::{Cli, Commands};

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .or_else(|_| tracing_subscriber::EnvFilter::try_new("info")),
        )
        .init();
}
