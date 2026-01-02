use crate::error::InstallError;
use crate::setup::SetupStep;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Firewall;

#[derive(Debug, Clone)]
pub struct FirewallRule<'a> {
	port: &'a str,
	protocol: &'a str,
	from: &'a str,
	comment: &'a str,
}

impl Firewall {
	pub const RULES: &[FirewallRule<'static>] = &[
		FirewallRule {
			port: "2379",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "etcd client",
		},
		FirewallRule {
			port: "2380",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "etcd peer",
		},
		FirewallRule {
			port: "6443",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "kube-apiserver",
		},
		FirewallRule {
			port: "8472",
			protocol: "udp",
			from: "192.168.0.0/16",
			comment: "cilium vxlan",
		},
		FirewallRule {
			port: "10250",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "kubelet",
		},
		FirewallRule {
			port: "10257",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "controller-manager",
		},
		FirewallRule {
			port: "10259",
			protocol: "tcp",
			from: "192.168.0.0/16",
			comment: "scheduler",
		},
	];

	fn rule_commands() -> String {
		Firewall::RULES
			.iter()
			.map(|rule| {
				format!(
					"sudo ufw allow from {} to any port {} proto {} comment '8inary: {}'",
					rule.from, rule.port, rule.protocol, rule.comment
				)
			})
			.collect::<Vec<_>>()
			.join("\n")
	}
}

impl SetupStep for Firewall {
	fn name(&self) -> &'static str {
		"Firewall"
	}

	fn check(&self) -> Result<bool, InstallError> {
		let firewall_settings_output = Command::new("sudo")
			.args(["ufw", "show", "added"])
			.output()
			.map_err(|source| InstallError::CommandLaunch {
				cmd: "sudo ufw show added".to_owned(),
				source,
			})?;
		let firewall_settings_status = firewall_settings_output.status;
		if !firewall_settings_status.success() {
			let stderr = if firewall_settings_output.stderr.is_empty() {
				None
			} else {
				Some(
					String::from_utf8_lossy(&firewall_settings_output.stderr)
						.trim()
						.to_owned(),
				)
			};
			return Err(InstallError::CommandFailed {
				cmd: "sudo ufw show added".to_owned(),
				status: firewall_settings_status,
				stderr,
			});
		}
		let mut firewall_settings_sanssudo = str::from_utf8(&firewall_settings_output.stdout)
			.expect("Fatal failure with non-utf8 firewall rules.")
			.lines()
			.filter(|rule| rule.contains("8inary"))
			.collect::<Vec<_>>()
			.iter()
			.map(|rule| rule.split_whitespace().collect::<Vec<_>>())
			.collect::<Vec<_>>();
		firewall_settings_sanssudo.sort_by_key(|rule| {
			(
				rule[7]
					.parse::<u16>()
					.expect("Fatal network port parse error."),
				rule[3],
			)
		});
		let firewall_settings = firewall_settings_sanssudo
			.iter()
			.map(|rule| "sudo ".to_owned() + &rule.join(" "))
			.collect::<Vec<_>>();
		let is_setup = firewall_settings.join("\n") == Firewall::rule_commands();
		if is_setup {
			info!("Firewall ports are open.");
			Ok(true)
		} else {
			info!("Firewall ports are not open.");
			Ok(false)
		}
	}

	fn set(&self) -> Result<(), InstallError> {
		Command::new("sh")
			.arg("-c")
			.arg(format!(
				r#"
				{}
				sudo ufw reload
			"#,
				Firewall::rule_commands()
			))
			.status()?;
		Ok(())
	}
}
