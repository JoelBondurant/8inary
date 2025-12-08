mod containerd;
mod control_plane;
mod disable_swap;
mod helm;
mod istio;
mod kernel_modules;
mod kubes;
mod machines;
mod pkg;
mod sysctl;

use crate::setup::containerd::Containerd;
use crate::setup::control_plane::ControlPlane;
use crate::setup::disable_swap::DisableSwap;
use crate::setup::helm::Helm;
use crate::setup::istio::Istio;
use crate::setup::kernel_modules::KernelModules;
use crate::setup::kubes::Kubes;
use crate::setup::sysctl::Sysctl;
use tracing::info;

pub trait SetupStep {
	fn name(&self) -> &'static str;
	fn check(&self) -> bool;
	fn set(&self);
}

pub fn setup() {
	info!("Kubernetes setup started.");
	const SETUP_STEPS: &[&dyn SetupStep] = &[
		&DisableSwap,
		&KernelModules,
		&Sysctl,
		&Containerd,
		&Kubes,
		&Helm,
		&ControlPlane,
		&Istio,
	];
	for step in SETUP_STEPS {
		if !step.check() {
			step.set();
			if !step.check() {
				panic!("Fatal install step failure: {}.", step.name());
			}
		}
	}
	info!("Kubernetes setup finished.");
}
