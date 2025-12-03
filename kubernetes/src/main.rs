mod context;
mod logging;
mod setup;

fn main() {
	context::init();
	logging::init();
	setup::setup();
}
