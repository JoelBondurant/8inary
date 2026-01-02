use crate::error::InstallError;
use std::process::Command;

pub enum PkgManager {
	Apt,
}

fn get_pkg_manager() -> PkgManager {
	PkgManager::Apt
}

pub fn is_installed(package_name: &str) -> Result<bool, InstallError> {
	let installed;
	match get_pkg_manager() {
		PkgManager::Apt => {
			let output = Command::new("dpkg-query")
				.args(["-W", "-f=${Status}", package_name])
				.output()
				.map_err(|source| InstallError::CommandLaunch {
					cmd: format!("dpkg-query -W -f=${{Status}}) {package_name}"),
					source,
				})?;
			if !output.status.success() {
				return Ok(false);
			}
			let stdout = String::from_utf8_lossy(&output.stdout);
			let status = stdout.trim();
			installed = status == "install ok installed" || status == "hold ok installed";
		}
	}
	Ok(installed)
}

pub fn update() -> Result<(), InstallError> {
	match get_pkg_manager() {
		PkgManager::Apt => {
			Command::new("apt-get")
				.arg("update")
				.status()
				.map_err(|err| InstallError::CommandLaunch {
					cmd: "apt-get update".to_owned(),
					source: err,
				})?;
		}
	}
	Ok(())
}

pub fn install(package_names: &[&str]) -> Result<(), InstallError> {
	match get_pkg_manager() {
		PkgManager::Apt => {
			let args = ["install", "-y", "--no-install-recommends"];
			Command::new("apt-get")
				.args(args.iter().chain(package_names.iter()))
				.status()
				.map_err(|err| InstallError::CommandLaunch {
					cmd: format!(
						"apt-get install -y --no-install-recommends {}",
						package_names.join(", ")
					),
					source: err,
				})?;
		}
	}
	Ok(())
}

pub fn mark(package_names: &[&str]) -> Result<(), InstallError> {
	match get_pkg_manager() {
		PkgManager::Apt => {
			let args = ["hold"];
			Command::new("apt-mark")
				.args(args.iter().chain(package_names.iter()))
				.status()
				.map_err(|err| InstallError::CommandLaunch {
					cmd: format!("apt-mark hold {}", package_names.join(", ")),
					source: err,
				})?;
		}
	}
	Ok(())
}
