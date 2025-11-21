use crate::agent::Agent;
use crate::setup::SetupStep;
use tracing::info;

pub struct Sysctl;

impl SetupStep for Sysctl {
	fn check(&self, agent: &Agent) -> bool {
		info!("Check for sysctl configuration.");
		let output =
			agent.execute("sha256sum /etc/sysctl.d/k8s.conf 2> /dev/null | cut -d ' ' -f 1");
		if output.1.trim() != "6e3f751b8409493b80fb7154ee21989dece3322d8b9018157ffef64dfbc10799" {
			info!("Sysctl is not configured in.");
			return false;
		}
		info!("Sysctl already configured.");
		true
	}

	fn set(&self, agent: &Agent) {
		info!("Configuring sysctl.");
		agent.execute("sudo sysctl --system");
		info!("Sysctl has been successfully configured.");
	}
}
