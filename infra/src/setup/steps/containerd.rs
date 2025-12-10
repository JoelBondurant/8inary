use crate::setup::utils::pkg;
use crate::setup::SetupStep;
use std::{fs, path::Path, process::Command};
use tracing::info;

pub struct Containerd;

impl Containerd {
	pub const PACKAGE_NAME: &str = "containerd";
	pub const CONFIG_PATH: &str = "/etc/containerd/config.toml";
}

impl SetupStep for Containerd {
	fn name(&self) -> &'static str {
		"Containerd"
	}

	fn check(&self) -> bool {
		let is_installed = pkg::is_installed(Containerd::PACKAGE_NAME);
		if !is_installed {
			info!("Containerd is not installed.");
			return false;
		}
		let is_configured = Path::new(Containerd::CONFIG_PATH).exists();
		if !is_configured {
			info!("Containerd is not configured.");
			return false;
		}
		let is_active = Command::new("systemctl")
			.args(["is-active", "--quiet", Containerd::PACKAGE_NAME])
			.status()
			.is_ok_and(|s| s.success());
		if !is_active {
			info!("Containerd is not active.");
			false
		} else {
			info!("Containerd is already configured and active.");
			true
		}
	}

	fn set(&self) {
		info!("Installing containerd via apt-get.");
		pkg::install(&[Containerd::PACKAGE_NAME]);
		fs::create_dir_all("/etc/containerd").expect("Failed to create /etc/containerd");
		let config_path = Path::new(Containerd::CONFIG_PATH);
		if !config_path.exists() || fs::read(config_path).unwrap().is_empty() {
			info!("Generating default containerd config.");
			let default_config = Command::new(Containerd::PACKAGE_NAME)
				.arg("config")
				.arg("default")
				.output()
				.expect("Fatal containerd config failure.");
			fs::write(config_path, default_config.stdout)
				.expect("Failed to write /etc/containerd/config.toml");
		} else {
			info!("Containerd config already exists, skipping generation.");
		}
		info!("Restarting containerd service.");
		Command::new("systemctl")
			.args(["restart", Containerd::PACKAGE_NAME])
			.status()
			.expect("Fatal failure to restart containerd.");
		info!("Containerd successfully installed.");
	}
}
