mod disable_swap;
mod kernel_modules;
mod sysctl;

use crate::setup::disable_swap::DisableSwap;
use crate::setup::kernel_modules::KernelModules;
use crate::setup::sysctl::Sysctl;
use tracing::info;

pub trait SetupStep {
	fn check(&self) -> bool;
	fn set(&self);
}

pub fn setup() {
	info!("Kubernetes setup started.");
	const SETUP_STEPS: &[&dyn SetupStep] = &[&DisableSwap, &KernelModules, &Sysctl];
	for step in SETUP_STEPS {
		if !step.check() {
			step.set();
		}
	}
	info!("Kubernetes setup finished.");
}
