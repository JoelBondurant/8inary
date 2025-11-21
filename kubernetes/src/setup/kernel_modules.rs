use crate::agent::Agent;
use crate::setup::SetupStep;
use tracing::info;

pub struct KernelModules;

impl SetupStep for KernelModules {
	fn check(&self, agent: &Agent) -> bool {
		info!("Check for kernel modules.");
		let output =
			agent.execute("sha256sum /etc/modules-load.d/k8s.conf 2> /dev/null | cut -d ' ' -f 1");
		if output.1.trim() != "fcaf07413a456d658640930cef56ed4d13330123e3b522c481021613c64755e3" {
			info!("Kernel modules are not configured in k8s.conf.");
			return false;
		}
		let output = agent.execute("lsmod | grep overlay");
		if output.1.trim().is_empty() {
			info!("Overlay fs kernel module not loaded.");
			return false;
		}
		let output = agent.execute("lsmod | grep br_netfilter");
		if output.1.trim().is_empty() {
			info!("Bridge netfilter kernel module not loaded.");
			return false;
		}
		info!("Kernel modules are already configured and loaded.");
		true
	}

	fn set(&self, agent: &Agent) {
		info!("Configuring kernel modules.");
		let content = "overlay\\nbr_netfilter\\n";
		let command = format!(
			r#"sudo sh -c 'printf "{}" > /etc/modules-load.d/k8s.conf'"#,
			content
		);
		agent.execute(&command);
		agent.execute("sudo modprobe overlay");
		agent.execute("sudo modprobe br_netfilter");
		info!("Kernel modules have been successfully configured and loaded.");
	}
}
