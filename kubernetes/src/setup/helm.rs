use crate::setup::pkg;
use crate::setup::SetupStep;
use std::{fs, process::Command};
use tracing::info;

pub struct Helm;

impl Helm {
	pub const PACKAGE_NAME: &str = "helm";
	pub const DEPENDENCIES: &[&str] = &["apt-transport-https"];
	pub const BASE_KEY_URL: &str = "https://packages.buildkite.com/helm-linux/helm-debian";
	pub const APT_KEY_PATH: &str = "/usr/share/keyrings/helm.gpg";
	pub const APT_CONFIG_PATH: &str = "/etc/apt/sources.list.d/helm-stable-debian.list";
}

impl SetupStep for Helm {
	fn name(&self) -> &'static str {
		"Helm"
	}

	fn check(&self) -> bool {
		if pkg::is_installed(Helm::PACKAGE_NAME) {
			info!("Helm is already installed.");
			true
		} else {
			info!("Helm is not installed.");
			false
		}
	}

	fn set(&self) {
		info!("Installing Helm.");
		pkg::apt_install(Helm::DEPENDENCIES);
		let key_command = format!(
			"curl -fsSL {}/gpgkey | gpg --dearmor --yes -o {}",
			Helm::BASE_KEY_URL,
			Helm::APT_KEY_PATH,
		);
		Command::new("sh")
			.arg("-c")
			.arg(key_command)
			.status()
			.unwrap();
		let apt_config_txt = format!(
			"deb [signed-by={}] {}/any/ any main",
			Helm::APT_KEY_PATH,
			Helm::BASE_KEY_URL,
		);
		fs::write(Helm::APT_CONFIG_PATH, apt_config_txt).unwrap();
		pkg::apt_update();
		pkg::apt_install(&[Helm::PACKAGE_NAME]);
		pkg::apt_mark(&[Helm::PACKAGE_NAME]);
		info!("Helm has been installed.");
	}
}
