use anyhow::Result;
use crate::setup::CheckSetCommand;
use crate::agent::Agent;
use tracing::info;

pub struct DisableSwap;

impl CheckSetCommand for DisableSwap {
	async fn check(&self, agent: &mut Agent) -> Result<bool> {
		info!("Check if swap is disabled.");
		let output = agent.execute(r"swapon -s").await?;
		if !output.1.trim().is_empty() {
			info!("Swap is enabled.");
			return Ok(false);
		}
		let fstab_output = agent.execute(r"grep -vE '^\s*#' /etc/fstab | grep 'swap'").await?;
		if !fstab_output.1.trim().is_empty() {
			info!("Swap is enabled in fstab.");
			return Ok(false);
		}
		info!("Swap is already disabled and absent in fstab.");
		Ok(true)
	}

	async fn set(&self, agent: &mut Agent) -> Result<()> {
		info!("Disabling swap.");
		agent.execute(r"sudo swapoff -a").await?;
		agent.execute(r"sudo sed -i '/\s*swap\s*/d' /etc/fstab").await?;
		info!("Swap has been successfully disabled and removed from fstab.");
		Ok(())
	}
}



