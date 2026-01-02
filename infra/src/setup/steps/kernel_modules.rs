use crate::error::InstallError;
use crate::setup::SetupStep;
use hex_literal::hex;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use tracing::info;

pub struct KernelModules;

impl KernelModules {
	pub const CONFIG_PATH: &str = "/etc/modules-load.d/k8s.conf";

	pub fn is_loaded(module_name: &str) -> bool {
		Path::new("/sys/module/").join(module_name).exists()
	}

	pub fn load(module_name: &str) -> Result<(), InstallError> {
		info!("Loading kernel module: {module_name}.");
		Command::new("modprobe").arg(module_name).status()?;
		Ok(())
	}
}

impl SetupStep for KernelModules {
	fn name(&self) -> &'static str {
		"KernelModules"
	}

	fn check(&self) -> Result<bool, InstallError> {
		const EXPECTED: [u8; 32] =
			hex!("fcaf07413a456d658640930cef56ed4d13330123e3b522c481021613c64755e3");
		let Ok(config_txt) = fs::read(KernelModules::CONFIG_PATH) else {
			info!("Kernel module config missing or unreadable.");
			return Ok(false);
		};
		let is_valid = Sha256::digest(&config_txt)[..] == EXPECTED;
		if !is_valid {
			info!("Kernel modules are misconfigured.");
			return Ok(false);
		}
		if !KernelModules::is_loaded("overlay") {
			info!("Overlay fs kernel module not loaded.");
			return Ok(false);
		}
		if !KernelModules::is_loaded("br_netfilter") {
			info!("Bridge netfilter kernel module not loaded.");
			return Ok(false);
		}
		info!("Kernel modules are already configured and loaded.");
		Ok(true)
	}

	fn set(&self) -> Result<(), InstallError> {
		info!("Configuring kernel modules.");
		let config_txt = "overlay\nbr_netfilter\n";
		fs::write(KernelModules::CONFIG_PATH, config_txt)?;
		KernelModules::load("overlay")?;
		KernelModules::load("br_netfilter")?;
		info!("Kernel modules have been successfully configured and loaded.");
		Ok(())
	}
}
