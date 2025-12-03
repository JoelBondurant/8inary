use std::{env, process::Command, sync::OnceLock};

#[derive(Debug)]
pub struct Context {
	pub home: String,
	pub user: String,
}

static CONTEXT: OnceLock<Context> = OnceLock::new();

pub fn init() {
	let user = env::var("SUDO_USER").expect("Fatal failure to resolve sudo user.");
	let home_output = Command::new("bash")
		.arg("-c")
		.arg(format!("getent passwd {} | awk -F: '{{print $6}}'", user))
		.output()
		.expect("Fatal: Failed to lookup home directory via getent.")
		.stdout;
	let home = str::from_utf8(&home_output)
		.expect("Fatal non-utf8 getent output.")
		.trim()
		.to_string();
	let context = Context { home, user };
	CONTEXT.set(context).expect("Fatal context initialization.");
}

pub fn get() -> &'static Context {
	CONTEXT.get().expect("Fatal failure to get context.")
}
