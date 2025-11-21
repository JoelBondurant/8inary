use std::process::Command;

pub struct Agent;

impl Agent {
	pub fn new() -> Self {
		Self
	}

	pub fn execute(&self, command: &str) -> (u32, String) {
		let mut cmd_builder = Command::new("/bin/bash");
		cmd_builder.arg("-c");
		cmd_builder.arg(command);
		let response = cmd_builder.output().expect("Bash call failed.");
		let exit_code = response.status.code().unwrap_or(1) as u32;
		let output = response.stdout;
		let stdout = String::from_utf8(output).expect("Non-utf8 output encountered.");
		(exit_code, stdout)
	}
}
