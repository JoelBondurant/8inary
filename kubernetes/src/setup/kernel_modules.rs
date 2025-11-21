use anyhow::Result;
use crate::setup::CheckSetCommand;
use crate::agent::Agent;
use tracing::info;

pub struct KernelModules;

impl CheckSetCommand for KernelModules {
	async fn check(&self, agent: &mut Agent) -> Result<bool> {
		info!("Check for kernel modules.");
		let output = agent.execute("sha256sum /etc/modules-load.d/k8s.conf 2> /dev/null | cut -d ' ' -f 1").await?;
		if output.1.trim() != "fcaf07413a456d658640930cef56ed4d13330123e3b522c481021613c64755e3" {
			info!("Kernel modules are not configured in k8s.conf.");
			return Ok(false);
		}
		let output = agent.execute("lsmod | grep overlay").await?;
		if output.1.trim().is_empty() {
			info!("Overlay fs kernel module not loaded.");
			return Ok(false);
		}
		let output = agent.execute("lsmod | grep br_netfilter").await?;
		if output.1.trim().is_empty() {
			info!("Bridge netfilter kernel module not loaded.");
			return Ok(false);
		}
		info!("Kernel modules are already configured and loaded.");
		Ok(true)
	}

	async fn set(&self, agent: &mut Agent) -> Result<()> {
		info!("Configuring kernel modules.");
		let content = "overlay\\nbr_netfilter\\n";
		let command = format!(
			r#"sudo sh -c 'printf "{}" > /etc/modules-load.d/k8s.conf'"#,
			content
		);
		agent.execute(&command).await?;
		agent.execute("sudo modprobe overlay").await?;
		agent.execute("sudo modprobe br_netfilter").await?;
		info!("Kernel modules have been successfully configured and loaded.");
		Ok(())
	}
}
