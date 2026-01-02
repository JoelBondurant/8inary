use crate::error::InstallError;
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

	fn check(&self) -> Result<bool, InstallError> {
		let is_installed = pkg::is_installed(Containerd::PACKAGE_NAME)?;
		if !is_installed {
			info!("Containerd is not installed.");
			return Ok(false);
		}
		let is_configured = Path::new(Containerd::CONFIG_PATH).exists();
		if !is_configured {
			info!("Containerd is not configured.");
			return Ok(false);
		}
		let is_active = Command::new("systemctl")
			.args(["is-active", "--quiet", Containerd::PACKAGE_NAME])
			.status()
			.is_ok_and(|s| s.success());
		if !is_active {
			info!("Containerd is not active.");
			Ok(false)
		} else {
			Ok(true)
		}
	}

	fn set(&self) -> Result<(), InstallError> {
		pkg::install(&[Containerd::PACKAGE_NAME])?;
		fs::create_dir_all(format!("/etc/{}", Containerd::PACKAGE_NAME))?;
		let config_path = Path::new(Containerd::CONFIG_PATH);
		if !config_path.exists() || fs::read(config_path)?.is_empty() {
			info!("Generating default containerd config.");
			let default_config = Command::new(Containerd::PACKAGE_NAME)
				.arg("config")
				.arg("default")
				.output()
				.map_err(|err| InstallError::CommandLaunch {
					cmd: format!("{} config default", Containerd::PACKAGE_NAME),
					source: err,
				})?;
			fs::write(config_path, default_config.stdout)?;
		} else {
			info!("Containerd config already exists, skipping generation.");
		}
		info!("Restarting containerd service.");
		let status = Command::new("systemctl")
			.args(["restart", Containerd::PACKAGE_NAME])
			.status()
			.map_err(|source| InstallError::CommandLaunch {
				cmd: format!("systemctl restart {}", Containerd::PACKAGE_NAME),
				source,
			})?;
		if !status.success() {
			return Err(InstallError::CommandFailed {
				cmd: format!("systemctl restart {}", Containerd::PACKAGE_NAME),
				status,
				stderr: None,
			});
		}
		Ok(())
	}
}
