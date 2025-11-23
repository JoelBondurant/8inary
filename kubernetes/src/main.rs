mod logging;
mod setup;

fn main() {
	logging::init();
	setup::setup();
}
