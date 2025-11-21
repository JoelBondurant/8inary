mod disable_swap;

use crate::agent::Agent;
use crate::machines::{self, Group};
use crate::setup::disable_swap::DisableSwap;
use anyhow::Result;
use tracing::info;

pub trait CheckSetCommand {
	async fn check(&self, agent: &mut Agent) -> Result<bool>;
	async fn set(&self, agent: &mut Agent) -> Result<()>;
}

pub async fn setup() -> Result<()> {
	info!("Kubernetes setup started.");
	let dev_machines = machines::get_machines(Group::DevGroup);
	let dm = dev_machines[0].clone();
	let is_local = dm.is_local().await?;
	let ip = dm.ip;
	info!("ip: {ip}");
	info!("is_local: {is_local:?}");
	let mut agent = Agent::new(&dm).await?;
	let cmd = DisableSwap;
	if !cmd.check(&mut agent).await? {
		cmd.set(&mut agent).await?;
	}
	agent.close().await?;
	info!("Kubernetes setup finished.");
	Ok(())
}
