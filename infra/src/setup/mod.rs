mod steps;
mod utils;

use crate::setup::steps::{
	Containerd, ControlPlane, DisableSwap, Helm, Istio, KernelModules, Kubes, Sysctl,
};
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
