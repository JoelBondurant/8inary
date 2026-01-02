mod context;
mod error;
mod logging;
mod setup;

use tracing::{error, info};

fn main() {
	context::init();
	logging::init();
	info!("Infrastructure setup started.");
	if let Err(err) = setup::setup() {
		error!("Installer failed: {}", err);
		std::process::exit(1);
	};
	info!("Infrastructure setup finished successfully.");
}
