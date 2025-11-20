mod agent;
mod machines;
mod setup;

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
	fmt()
		.with_ansi(true)
		.with_env_filter(EnvFilter::new("info"))
		.with_file(true)
		.with_line_number(true)
		.with_target(true)
		.with_thread_ids(true)
		.with_thread_names(true)
		.with_timer(fmt::time::SystemTime)
		.compact()
		.init();
	setup::setup().await?;
	Ok(())
}
