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

	fn check(&self) -> bool {
		let is_installed = Command::new("kubectl")
			.args(["get", "deployment", "istiod", "-n", "istio-system"])
			.status()
			.expect("Fatal failure to check Istio installation.")
			.success();
		if is_installed {
			info!("Istio is already installed.");
			true
		} else {
			info!("Istio is not installed.");
			false
		}
	}

	fn set(&self) {
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
			.expect("Fatal failure in Istio installation.");
		Command::new("istioctl")
			.arg("install")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["--set", "profile=default"])
			.arg("-y")
			.status()
			.expect("Fatal failure installing Istio into cluster.");
	}
}
