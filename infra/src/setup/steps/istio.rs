use crate::error::InstallError;
use crate::setup::utils::kctl;
use crate::setup::SetupStep;
use std::process::Command;
use tracing::info;

pub struct Istio;

impl Istio {
	pub const VERSION: &str = "1.28.0";
	pub const URL: &str = "https://istio.io/downloadIstio";
}

impl SetupStep for Istio {
	fn name(&self) -> &'static str {
		"Istio"
	}

	fn check(&self) -> Result<bool, InstallError> {
		let is_installed = kctl::is_deployment_installed("istio", "istio-system")?;
		if is_installed {
			info!("Istio is already installed.");
			Ok(true)
		} else {
			info!("Istio is not installed.");
			Ok(false)
		}
	}

	fn set(&self) -> Result<(), InstallError> {
		info!("Installing Istio.");
		Command::new("sh")
			.arg("-c")
			.arg(format!(
				r#"
				(cd /tmp && curl -L {} | ISTIO_VERSION={} sh -) && \
				sudo mv /tmp/istio-{} /tmp/istio && \
				sudo cp /tmp/istio/bin/istioctl /usr/local/bin/ && \
				sudo cp /tmp/istio/tools/istioctl.bash /etc/bash_completion.d/ && \
				sudo rm -rf /tmp/istio
				"#,
				Istio::URL,
				Istio::VERSION,
				Istio::VERSION,
			))
			.status()
			.map_err(|err| InstallError::CommandLaunch {
				cmd: format!("istio path bash commands"),
				source: err,
			})?;
		Command::new("istioctl")
			.arg("install")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["--set", "profile=default"])
			.arg("-y")
			.status()
			.map_err(|err| InstallError::CommandLaunch {
				cmd: format!("istioctl --set profile=default -y"),
				source: err,
			})?;
		Ok(())
	}
}
