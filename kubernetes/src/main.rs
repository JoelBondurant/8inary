mod agent;
mod machines;

use anyhow::Result;
use agent::Agent;
use machines::{Group, get_machines};

#[tokio::main]
async fn main() -> Result<()> {
	let dev_machines = get_machines(Group::DevGroup);
	let dm = dev_machines[0].clone();
	let is_local = dm.is_local().await?;
	let ip = dm.ip;
	println!("ip: {ip}");
	println!("Is Local: {is_local:?}");
	let mut agent = Agent::new(&dm).await?;
	let (exit_code, msg) = agent
		.execute("swapon --show")
		.await?;
	println!("Exitcode: {exit_code:?}");
	println!("Message: {msg:?}");
	agent.close().await?;
	Ok(())
}
