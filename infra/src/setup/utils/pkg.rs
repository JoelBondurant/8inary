use std::process::Command;

pub enum PkgManager {
	Apt,
}

fn get_pkg_manager() -> PkgManager {
	PkgManager::Apt
}

pub fn is_installed(package_name: &str) -> bool {
	match get_pkg_manager() {
		PkgManager::Apt => {
			let output = Command::new("dpkg-query")
				.args(["-W", "-f=${Status}", package_name])
				.output()
				.expect("Failed to run dpkg-query.");
			let stdout = String::from_utf8(output.stdout)
				.expect("dpkg-query returned non-utf-8 output.")
				.trim()
				.to_owned();
			output.status.success()
				&& (stdout == "install ok installed" || stdout == "hold ok installed")
		}
	}
}

pub fn update() {
	match get_pkg_manager() {
		PkgManager::Apt => {
			Command::new("apt-get")
				.arg("update")
				.status()
				.expect("Fatal apt-get update failure.");
		}
	}
}

pub fn install(package_names: &[&str]) {
	match get_pkg_manager() {
		PkgManager::Apt => {
			let args = ["install", "-y", "--no-install-recommends"];
			Command::new("apt-get")
				.args(args.iter().chain(package_names.iter()))
				.status()
				.expect("Fatal apt-get install failure.");
		}
	}
}

pub fn mark(package_names: &[&str]) {
	match get_pkg_manager() {
		PkgManager::Apt => {
			let args = ["hold"];
			Command::new("apt-mark")
				.args(args.iter().chain(package_names.iter()))
				.status()
				.expect("Fatal apt-mark failure.");
		}
	}
}
