use std::process::Command;
use tracing::info;

pub fn is_installed(package_name: &str) -> bool {
	let output = Command::new("dpkg-query")
		.args(["-W", "-f=${Status}", package_name])
		.output()
		.expect("Failed to run dpkg-query.");
	let stdout = String::from_utf8(output.stdout)
		.expect("dpkg-query returned non-utf-8 output.")
		.trim()
		.to_owned();
	info!("{package_name}: {stdout}");
	output.status.success() && (stdout == "install ok installed" || stdout == "hold ok installed")
}

pub fn apt_update() {
	Command::new("apt-get")
		.arg("update")
		.status()
		.expect("Fatal apt-get update failure.");
}

pub fn apt_install(package_names: &[&str]) {
	let args = ["install", "-y", "--no-install-recommends"];
	Command::new("apt-get")
		.args(args.iter().chain(package_names.iter()))
		.status()
		.expect("Fatal apt-get install failure.");
}

pub fn apt_mark(package_names: &[&str]) {
	let args = ["hold"];
	Command::new("apt-mark")
		.args(args.iter().chain(package_names.iter()))
		.status()
		.expect("Fatal apt-mark failure.");
}
