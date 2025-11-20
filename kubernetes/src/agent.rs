use anyhow::Result;
use crate::machines::Machine;
use russh::{
	client::{connect, Config, Handle, Handler},
	keys::{
		key::PrivateKeyWithHashAlg, load_secret_key, ssh_key::PublicKey,
	},
	ChannelMsg, Disconnect, Preferred,
};
use std::{borrow::Cow, sync::Arc, time::Duration};

struct SshClient {}

impl Handler for SshClient {
	type Error = russh::Error;

	async fn check_server_key(
		&mut self,
		_server_public_key: &PublicKey,
	) -> Result<bool, Self::Error> {
		Ok(true)
	}
}

pub struct Agent {
	session: Handle<SshClient>,
}

impl Agent {

	pub async fn new(
		machine: &Machine,
	) -> Result<Self> {
		let key_path = std::env::var("HOME").expect("HOME not defined.") + "/.ssh/id_ed25519";
		let user = machine.user;
		let host = machine.ip;
		let port = machine.port;
		let addrs = (host, port);
		let key_pair = load_secret_key(key_path, None)?;
		let config = Config {
			inactivity_timeout: Some(Duration::from_secs(5)),
			preferred: Preferred {
				kex: Cow::Owned(vec![
					russh::kex::CURVE25519_PRE_RFC_8731,
					russh::kex::EXTENSION_SUPPORT_AS_CLIENT,
				]),
				..Default::default()
			},
			..<_>::default()
		};
		let config = Arc::new(config);
		let sh = SshClient {};
		let mut session = connect(config, addrs, sh).await?;
		let auth_res = session
			.authenticate_publickey(
				user,
				PrivateKeyWithHashAlg::new(
					Arc::new(key_pair),
					session.best_supported_rsa_hash().await?.flatten(),
				),
			)
			.await?;
		if !auth_res.success() {
			anyhow::bail!("Authentication (with publickey) failed");
		}
		Ok(Self { session })
	}

	pub async fn execute(&mut self, command: &str) -> Result<(u32, String)> {
		let cmd = format!("/bin/bash -c \"{}\"", command);
		let mut channel = self.session.channel_open_session().await?;
		channel.exec(true, cmd).await?;
		let mut code = None;
		let mut output = Vec::new();
		loop {
			let Some(msg) = channel.wait().await else {
				break;
			};
			match msg {
				ChannelMsg::Data { ref data } => {
					output.extend_from_slice(data);
				}
				ChannelMsg::ExitStatus { exit_status } => {
					code = Some(exit_status);
				}
				_ => {}
			}
		}
		let stdout = String::from_utf8(output).expect("Non-utf8 output encountered.");
		Ok((
			code.expect("Command did not exit cleanly"),
			stdout,
		))
	}

	pub async fn close(&mut self) -> Result<()> {
		self.session
			.disconnect(Disconnect::ByApplication, "Disconnected", "English")
			.await?;
		Ok(())
	}
}
