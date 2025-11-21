mod disable_swap;
mod kernel_modules;
mod sysctl;

use crate::agent::Agent;
use crate::machines::{self, Group};
use crate::setup::disable_swap::DisableSwap;
use crate::setup::kernel_modules::KernelModules;
use crate::setup::sysctl::Sysctl;
use tracing::info;

pub trait SetupStep {
	fn check(&self, agent: &Agent) -> bool;
	fn set(&self, agent: &Agent);
}

pub fn setup() {
	info!("Kubernetes setup started.");
	let dev_machines = machines::get_machines(Group::DevGroup);
	let dm = dev_machines[3].clone();
	let ip = dm.ip;
	info!("ip: {ip}");
	let steps: Vec<Box<dyn SetupStep>> = vec![
		Box::new(DisableSwap),
		Box::new(KernelModules),
		Box::new(Sysctl),
	];
	let agent = Agent::new();
	for step in steps.iter() {
		if !step.check(&agent) {
			step.set(&agent);
		}
	}
	info!("Kubernetes setup finished.");
}
