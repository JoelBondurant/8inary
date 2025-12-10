use crate::setup::utils::pkg;
use crate::setup::SetupStep;
use std::{fs, process::Command};
use tracing::info;

pub struct Kubes;

impl Kubes {
	pub const PACKAGE_NAMES: &[&str] = &["kubelet", "kubeadm", "kubectl"];
	pub const APT_CONFIG_PATH: &str = "/etc/apt/sources.list.d/kubernetes.list";
	pub const APT_KEY_PATH: &str = "/etc/apt/keyrings/kubernetes-apt-keyring.gpg";
	pub const K8S_BASE_URL: &str = "https://pkgs.k8s.io/core:/stable:/v1.34/deb";
}

impl SetupStep for Kubes {
	fn name(&self) -> &'static str {
		"Kubes"
	}

	fn check(&self) -> bool {
		for package_name in Kubes::PACKAGE_NAMES {
			let is_installed = pkg::is_installed(package_name);
			if !is_installed {
				info!("{package_name} is not installed.");
				return false;
			}
		}
		info!("Kubes are installed.");
		true
	}

	fn set(&self) {
		info!("Installing Kubernetes tooling via apt-get.");
		let key_command = format!(
			"curl -fsSL {}/Release.key | gpg --dearmor --yes -o {}",
			Kubes::K8S_BASE_URL,
			Kubes::APT_KEY_PATH,
		);
		Command::new("sh")
			.arg("-c")
			.arg(key_command)
			.status()
			.unwrap();
		let apt_config_txt = format!(
			"deb [signed-by={}] {} /",
			Kubes::APT_KEY_PATH,
			Kubes::K8S_BASE_URL,
		);
		fs::write(Kubes::APT_CONFIG_PATH, apt_config_txt).unwrap();
		pkg::update();
		pkg::install(Kubes::PACKAGE_NAMES);
		pkg::mark(Kubes::PACKAGE_NAMES);
		info!("Kubernetes tooling installed.");
	}
}
