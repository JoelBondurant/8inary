use anyhow::Result;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use tokio::process::Command;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Machine {
	pub ip: Ipv4Addr,
	pub port: u16,
	pub user: &'static str,
}

#[allow(dead_code)]
impl Machine {
	pub async fn is_local(&self) -> Result<bool> {
		let ip_response = Command::new("hostname")
			.arg("-I")
			.output()
			.await
			.expect("hostname call failed to resolve ip address.");
		let ip_self = std::str::from_utf8(&ip_response.stdout)
			.expect("hostname ip address was not utf8.")
			.trim()
			.parse::<Ipv4Addr>()
			.expect("hostname ip address was not ipv4.");
		Ok(self.ip == ip_self)
	}
}

const fn ip(ip_bytes: [u8; 4]) -> Ipv4Addr {
	Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3])
}

const DEFAULT_PORT: u16 = 22;
const DEFAULT_USER: &str = "mgmt";

const DEFAULT_MACHINE: Machine = Machine {
	ip: ip([0, 0, 0, 0]),
	port: DEFAULT_PORT,
	user: DEFAULT_USER,
};

const DEV1: Machine = Machine {
	ip: ip([192, 168, 0, 2]),
	..DEFAULT_MACHINE
};

const DEV2: Machine = Machine {
	ip: ip([192, 168, 0, 3]),
	..DEFAULT_MACHINE
};

const DEV3: Machine = Machine {
	ip: ip([192, 168, 0, 4]),
	..DEFAULT_MACHINE
};

const DEV4: Machine = Machine {
	ip: ip([192, 168, 0, 5]),
	..DEFAULT_MACHINE
};

const DEV5: Machine = Machine {
	ip: ip([192, 168, 0, 6]),
	..DEFAULT_MACHINE
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Group {
	DevGroup,
}

#[derive(Debug)]
pub struct MachineGroup {
	pub groups: HashMap<Group, Vec<Machine>>,
}

fn inventory() -> MachineGroup {
	let mut groups = HashMap::new();
	groups.insert(Group::DevGroup, vec![DEV1, DEV2, DEV3, DEV4, DEV5]);
	MachineGroup { groups }
}

pub fn get_machines(group: Group) -> Vec<Machine> {
	inventory()
		.groups
		.get(&group)
		.cloned()
		.expect("invalid machine group.")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_machines_dev_group_returns_expected_machines() {
		let machines = get_machines(Group::DevGroup);
		assert_eq!(machines.len(), 5);
		assert_eq!(machines[0].ip, ip([192, 168, 0, 2]));
		assert_eq!(machines[0].port, DEFAULT_PORT);
		assert_eq!(machines[0].user, DEFAULT_USER);
	}
}
