use crate::setup::SetupStep;
use std::{fs, process::Command};
use tracing::info;

pub struct DisableSwap;

impl SetupStep for DisableSwap {
	fn name(&self) -> &'static str {
		"DisableSwap"
	}

	fn check(&self) -> bool {
		info!("Check if swap is disabled.");
		let is_swap_on = fs::read_to_string("/proc/swaps").unwrap().lines().count() > 1;
		if is_swap_on {
			info!("Swap is enabled.");
			return false;
		}
		let Ok(config_txt) = fs::read_to_string("/etc/fstab") else {
			info!("fstab is missing or unreadable.");
			return false;
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
			return false;
		}
		info!("Swap is already disabled and absent in fstab.");
		true
	}

	fn set(&self) {
		info!("Disabling swap.");
		Command::new("swapoff")
			.arg("-a")
			.status()
			.expect("Fatal swapoff failure.");
		let config_path = "/etc/fstab";
		let original = fs::read_to_string(config_path).expect("Fatal fstab read failure.");
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
			sudo::escalate_if_needed().expect("Failed to escalate privileges.");
			fs::write(config_path, final_content).expect("Fatal fstab write failure.");
		}
	}
}
