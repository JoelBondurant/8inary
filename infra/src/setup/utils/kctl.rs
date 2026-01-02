use crate::error::InstallError;
use std::{
	io::Write,
	process::{Command, Output, Stdio},
};

pub const KUBECONFIG: &str = "/etc/kubernetes/admin.conf";

fn kubectl_status(args: &[&str]) -> Result<(), InstallError> {
	let full_cmd = format!("kubectl {}", args.join(" "));
	let status = Command::new("kubectl")
		.args(["--kubeconfig", KUBECONFIG])
		.args(args)
		.status()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: full_cmd.clone(),
			source: err,
		})?;
	if !status.success() {
		return Err(InstallError::CommandFailed {
			cmd: full_cmd,
			status,
			stderr: None,
		});
	}
	Ok(())
}

fn kubectl_output(args: &[&str]) -> Result<Output, InstallError> {
	let full_cmd = format!("kubectl {}", args.join(" "));
	let output = Command::new("kubectl")
		.args(["--kubeconfig", KUBECONFIG])
		.args(args)
		.output()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: full_cmd.clone(),
			source: err,
		})?;
	if !output.status.success() {
		let stderr = if output.stderr.is_empty() {
			None
		} else {
			Some(String::from_utf8_lossy(&output.stderr).trim().to_owned())
		};
		return Err(InstallError::CommandFailed {
			cmd: full_cmd,
			status: output.status,
			stderr,
		});
	}
	Ok(output)
}

pub fn generate_yaml(create_args: &[&str]) -> Result<String, InstallError> {
	let mut args = create_args.to_vec();
	args.extend_from_slice(&["--dry-run=client", "-o", "yaml"]);
	let output = kubectl_output(&args)?;
	Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn apply_yaml(yaml: &str) -> Result<(), InstallError> {
	let mut child = Command::new("kubectl")
		.args(["--kubeconfig", KUBECONFIG])
		.args(["apply", "-f", "-"])
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: "kubectl apply -f -".to_owned(),
			source: err,
		})?;
	let stdin = child
		.stdin
		.as_mut()
		.ok_or_else(|| InstallError::Other("Failed to open stdin for kubectl apply".into()))?;
	stdin.write_all(yaml.as_bytes())?;
	let output = child
		.wait_with_output()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: "kubectl apply -f -".to_owned(),
			source: err,
		})?;
	if !output.status.success() {
		let stderr = Some(String::from_utf8_lossy(&output.stderr).trim().to_owned());
		return Err(InstallError::CommandFailed {
			cmd: "kubectl apply -f -".to_owned(),
			status: output.status,
			stderr,
		});
	}
	Ok(())
}

pub fn apply(create_args: &[&str]) -> Result<(), InstallError> {
	let yaml = generate_yaml(create_args)?;
	apply_yaml(&yaml)?;
	Ok(())
}

pub fn is_deployment_installed(name: &str, namespace: &str) -> Result<bool, InstallError> {
	let status = Command::new("kubectl")
		.args(["--kubeconfig", KUBECONFIG])
		.args(["get", "deployment", name])
		.args(["-n", namespace])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.status()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: format!("kubectl get deployment {name} -n {namespace}"),
			source: err,
		})?;
	Ok(status.success())
}

pub fn get_pods(namespace: &str, label: &str) -> Result<String, InstallError> {
	let output = Command::new("kubectl")
		.args(["--kubeconfig", KUBECONFIG])
		.args(["get", "pods"])
		.args(["--namespace", namespace])
		.args(["-l", label])
		.args(["-o", "name"])
		.output()
		.map_err(|err| InstallError::CommandLaunch {
			cmd: format!("kubectl get pods -n {namespace} -l {label}"),
			source: err,
		})?;
	if !output.status.success() {
		let stderr = Some(String::from_utf8_lossy(&output.stderr).trim().to_owned());
		return Err(InstallError::CommandFailed {
			cmd: format!("kubectl get pods -n {namespace} -l {label}"),
			status: output.status,
			stderr,
		});
	}
	Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
