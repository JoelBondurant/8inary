use std::panic;
use tracing_journald::layer as journald_layer;
use tracing_panic::panic_hook;
use tracing_subscriber::{fmt, layer::SubscriberExt, registry::Registry, EnvFilter};

// sudo journald -t 8inary-k8s
pub fn init() {
	panic::set_hook(Box::new(panic_hook));
	let log_sub = Registry::default()
		.with(
			EnvFilter::builder()
				.with_default_directive(tracing::Level::INFO.into())
				.from_env_lossy(),
		)
		.with(
			fmt::layer()
				.with_ansi(true)
				.with_file(true)
				.with_line_number(true)
				.with_target(true)
				.with_thread_ids(true)
				.with_thread_names(true)
				.with_timer(fmt::time::SystemTime)
				.compact(),
		)
		.with(
			journald_layer()
				.map_err(|err| eprintln!("journald not available: {err}"))
				.ok()
				.map(|layr| layr.with_syslog_identifier("8inary-k8s".into())),
		);
	tracing::subscriber::set_global_default(log_sub).expect("Failed to set log subscriber.");
}
