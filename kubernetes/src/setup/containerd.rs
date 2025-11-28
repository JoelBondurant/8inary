use crate::setup::pkg;
use crate::setup::SetupStep;
use std::{fs, path::Path, process::Command};
use tracing::info;

pub struct Containerd;

impl Containerd {
	pub const CONFIG_PATH: &str = "/etc/containerd/config.toml";
}

impl SetupStep for Containerd {
	fn name(&self) -> &'static str {
		"Containerd"
	}

	fn check(&self) -> bool {
		info!("Check if containerd is installed.");
		let is_installed = pkg::is_installed("containerd");
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
			.args(["is-active", "--quiet", "containerd"])
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
		sudo::escalate_if_needed().expect("Failed to escalate privileges.");
		let status = Command::new("apt-get")
			.args(["install", "-y", "--no-install-recommends", "containerd"])
			.status()
			.expect("Fatal apt-get failure.");
		if !status.success() {
			panic!("Fatal failure to install containerd: {status}.");
		}
		fs::create_dir_all("/etc/containerd").expect("Failed to create /etc/containerd");
		let config_path = Path::new(Containerd::CONFIG_PATH);
		if !config_path.exists() || fs::read(config_path).unwrap().is_empty() {
			info!("Generating default containerd config.");
			let default_config = Command::new("containerd")
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
			.args(["restart", "containerd"])
			.status()
			.expect("Fatal failure to restart containerd.");
		info!("Containerd successfully installed.");
	}
}
