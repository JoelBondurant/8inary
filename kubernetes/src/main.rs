mod setup;

use tracing_subscriber::{fmt, EnvFilter};

fn main() {
	fmt()
		.with_env_filter(EnvFilter::new("info"))
		.with_thread_ids(true)
		.with_thread_names(true)
		.with_target(true)
		.with_file(true)
		.with_line_number(true)
		.with_ansi(true)
		.with_timer(fmt::time::SystemTime)
		.compact()
		.init();
	setup::setup();
}
