use crate::context;

#[derive(Debug, Clone, Copy)]
pub enum Environment {
	Dev,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MachineRole {
	ControlPlane,
	ControlPlaneRoot,
	#[allow(dead_code)]
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
	let this_imachine = INVENTORY
		.iter()
		.find(|ma| ma.id == context::get().machine_id)
		.expect("This machine is not in the inventory.");
	Machine {
		id: this_imachine.id.to_owned(),
		_environment: this_imachine.environment,
		role: this_imachine.role,
	}
}
