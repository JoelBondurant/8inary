use crate::error::InstallError;
use crate::setup::SetupStep;
use hex_literal::hex;
use sha2::{Digest, Sha256};
use std::{fs, process::Command};
use tracing::info;

pub struct Sysctl;

impl Sysctl {
	pub const CONFIG_PATH: &str = "/etc/sysctl.d/k8s.conf";
}

impl SetupStep for Sysctl {
	fn name(&self) -> &'static str {
		"Sysctl"
	}

	fn check(&self) -> Result<bool, InstallError> {
		const EXPECTED: [u8; 32] =
			hex!("6e3f751b8409493b80fb7154ee21989dece3322d8b9018157ffef64dfbc10799");
		let Ok(config_txt) = fs::read(Sysctl::CONFIG_PATH) else {
			info!("Sysctl config missing or unreadable.");
			return Ok(false);
		};
		let is_valid = Sha256::digest(&config_txt)[..] == EXPECTED;
		if !is_valid {
			info!("Kernel modules are misconfigured.");
			return Ok(false);
		}
		info!("Sysctl already configured.");
		Ok(true)
	}

	fn set(&self) -> Result<(), InstallError> {
		info!("Configuring sysctl.");
		let config_txt = [
			"net.bridge.bridge-nf-call-iptables = 1",
			"net.bridge.bridge-nf-call-ip6tables = 1",
			"net.ipv4.ip_forward = 1",
		]
		.join("\n")
			+ "\n";
		fs::write(Sysctl::CONFIG_PATH, config_txt)?;
		Command::new("sysctl").arg("--system").status()?;
		info!("Sysctl has been successfully configured.");
		Ok(())
	}
}
