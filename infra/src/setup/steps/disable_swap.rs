use crate::error::InstallError;
use crate::setup::SetupStep;
use std::{fs, process::Command};
use tracing::info;

pub struct DisableSwap;

impl SetupStep for DisableSwap {
	fn name(&self) -> &'static str {
		"DisableSwap"
	}

	fn check(&self) -> Result<bool, InstallError> {
		let is_swap_on = fs::read_to_string("/proc/swaps")?.lines().count() > 1;
		if is_swap_on {
			info!("Swap is enabled.");
			return Ok(false);
		}
		let Ok(config_txt) = fs::read_to_string("/etc/fstab") else {
			info!("fstab is missing or unreadable.");
			return Ok(false);
		};
		let is_configured = config_txt
			.lines()
			.filter(|line| !line.trim_start().starts_with('#'))
			.any(|line| {
				let fields = line.split_whitespace().collect::<Vec<&str>>();
				fields.len() >= 3 && fields[2] == "swap"
			});
		if is_configured {
			info!("Swap is enabled in fstab.");
			return Ok(false);
		}
		Ok(true)
	}

	fn set(&self) -> Result<(), InstallError> {
		let output = Command::new("swapoff")
			.arg("-a")
			.output()
			.map_err(|source| InstallError::CommandLaunch {
				cmd: "swapoff -a".to_owned(),
				source,
			})?;
		let status = output.status;
		if !status.success() {
			let stderr = if output.stderr.is_empty() {
				None
			} else {
				Some(String::from_utf8_lossy(&output.stderr).trim().to_owned())
			};

			return Err(InstallError::CommandFailed {
				cmd: "swapoff -a".to_owned(),
				status,
				stderr,
			});
		}
		let config_path = "/etc/fstab";
		let original = fs::read_to_string(config_path)?;
		let cleaned = original
			.lines()
			.filter(|line| {
				line.split_whitespace()
					.nth(2)
					.is_none_or(|fs_type| fs_type != "swap")
			})
			.collect::<Vec<_>>()
			.join("\n");
		let final_content = if original.ends_with('\n') {
			cleaned + "\n"
		} else {
			cleaned
		};
		if final_content.as_bytes() != original.as_bytes() {
			info!("Removing swap entries from /etc/fstab.");
			fs::write(config_path, final_content)?;
		}
		Ok(())
	}
}
