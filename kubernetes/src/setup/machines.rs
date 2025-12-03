use std::fs;

#[derive(Debug, Clone, Copy)]
pub enum Environment {
	Dev,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MachineRole {
	ControlPlaneRoot,
	ControlPlane,
	Worker,
}

#[derive(Debug, Clone)]
struct IMachine<'a> {
	id: &'a str,
	environment: Environment,
	role: MachineRole,
}

#[derive(Debug, Clone)]
pub struct Machine {
	pub id: String,
	pub _environment: Environment,
	pub role: MachineRole,
}

const INVENTORY: &[IMachine<'static>] = &[
	IMachine {
		id: "a218e8c2c31942e3acdbae7f4f532c2d",
		environment: Environment::Dev,
		role: MachineRole::ControlPlaneRoot,
	},
	IMachine {
		id: "e65407e7fcd24bc58a7a20ce0b4992dd",
		environment: Environment::Dev,
		role: MachineRole::ControlPlane,
	},
	IMachine {
		id: "75719c8d8ad84e2a8959733440b18233",
		environment: Environment::Dev,
		role: MachineRole::ControlPlane,
	},
	IMachine {
		id: "ca9e447c051b4c18b154810ea3a4dc8a",
		environment: Environment::Dev,
		role: MachineRole::ControlPlane,
	},
	IMachine {
		id: "4142f1ba2e8844d09cba6ea16e97dfa2",
		environment: Environment::Dev,
		role: MachineRole::ControlPlane,
	},
];

pub fn this() -> Machine {
	let machine_id = String::from_utf8(fs::read("/etc/machine-id").expect("No machine-id"))
		.expect("Machine-id is not utf-8.")
		.trim()
		.to_string();
	let this_imachine = INVENTORY
		.iter()
		.find(|ma| ma.id == machine_id)
		.expect("This machine is not in the inventory.");
	Machine {
		id: machine_id,
		_environment: this_imachine.environment,
		role: this_imachine.role,
	}
}
