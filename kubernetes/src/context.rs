use std::{env, fs, process::Command, sync::OnceLock};

#[derive(Debug)]
pub struct Context {
	pub home: String,
	pub hostname: String,
	pub machine_id: String,
	pub user: String,
}

static CONTEXT: OnceLock<Context> = OnceLock::new();

pub fn init() {
	let user = env::var("SUDO_USER").expect("Fatal failure to resolve sudo user.");
	let home = str::from_utf8(
		&Command::new("bash")
			.arg("-c")
			.arg(format!("getent passwd {} | awk -F: '{{print $6}}'", user))
			.output()
			.expect("Fatal failure to lookup home directory via getent.")
			.stdout,
	)
	.expect("Fatal failure in home path non-utf8 encoding.")
	.trim()
	.to_owned();
	let hostname = str::from_utf8(
		&Command::new("hostname")
			.arg("-f")
			.output()
			.expect("Fatal failure resolving hostname.")
			.stdout,
	)
	.expect("Fatal failure in hostname non-utf8 encoding.")
	.trim()
	.to_owned();
	let machine_id = fs::read_to_string("/etc/machine-id")
		.expect("Fatal failure to resolve machine-id.")
		.trim()
		.to_owned();
	let context = Context {
		home,
		hostname,
		machine_id,
		user,
	};
	CONTEXT.set(context).expect("Fatal context initialization.");
}

pub fn get() -> &'static Context {
	CONTEXT.get().expect("Fatal failure to get context.")
}
