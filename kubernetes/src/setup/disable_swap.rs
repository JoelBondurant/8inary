use crate::agent::Agent;
use crate::setup::SetupStep;
use tracing::info;

pub struct DisableSwap;

impl SetupStep for DisableSwap {
	fn check(&self, agent: &Agent) -> bool {
		info!("Check if swap is disabled.");
		let output = agent.execute(r"swapon -s");
		if !output.1.trim().is_empty() {
			info!("Swap is enabled.");
			return false;
		}
		let fstab_output = agent.execute(r"grep -vE '^\s*#' /etc/fstab | grep 'swap'");
		if !fstab_output.1.trim().is_empty() {
			info!("Swap is enabled in fstab.");
			return false;
		}
		info!("Swap is already disabled and absent in fstab.");
		true
	}

	fn set(&self, agent: &Agent) {
		info!("Disabling swap.");
		agent.execute(r"sudo swapoff -a");
		agent.execute(r"sudo sed -i '/\s*swap\s*/d' /etc/fstab");
		info!("Swap has been successfully disabled and removed from fstab.");
	}
}
