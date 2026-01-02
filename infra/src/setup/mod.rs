mod steps;
mod utils;

use crate::error::InstallError;
use crate::setup::steps::{
	Containerd, ControlPlane, DisableSwap, Firewall, Helm, IdentityDatabase, Istio, KernelModules,
	Kubes, Sysctl,
};
use tracing::info;

pub trait SetupStep {
	fn name(&self) -> &'static str;
	fn check(&self) -> Result<bool, InstallError>;
	fn set(&self) -> Result<(), InstallError>;
}

const SETUP_STEPS: &[&dyn SetupStep] = &[
	&DisableSwap,
	&KernelModules,
	&Sysctl,
	&Containerd,
	&Kubes,
	&Helm,
	&Firewall,
	&ControlPlane,
	&Istio,
	&IdentityDatabase,
];

pub fn setup() -> Result<(), InstallError> {
	for step in SETUP_STEPS {
		let step_name = step.name();
		info!("Checking step: {}.", step_name);
		if !step.check()? {
			info!("Applying step: {}.", step_name);
			step.set()?;
			if !step.check()? {
				return Err(InstallError::StepFailed { step: step_name });
			}
		} else {
			info!("Step already satisfied: {}.", step_name);
		}
	}
	Ok(())
}
