use std::{io, process::ExitStatus, str::Utf8Error, string::FromUtf8Error};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
	#[error("I/O error: {0}.")]
	Io(#[from] io::Error),

	#[error("Failed to execute command '{cmd}': {source}")]
	CommandLaunch {
		cmd: String,
		#[source]
		source: io::Error,
	},

	#[error("Command failed: {cmd}")]
	CommandFailed {
		cmd: String,
		status: ExitStatus,
		stderr: Option<String>,
	},

	#[error("Step '{step}' failed after attempt to set it.")]
	StepFailed { step: &'static str },

	#[error("Kubernetes error: {0}")]
	Kube(String),

	#[error("Helm error: {0}")]
	Helm(String),

	#[error("Invalid configuration: {0}.")]
	Config(String),

	#[error("Str error: {0}.")]
	StrError(#[from] Utf8Error),

	#[error("String error: {0}.")]
	StringError(#[from] FromUtf8Error),

	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
