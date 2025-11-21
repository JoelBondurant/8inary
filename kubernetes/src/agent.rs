use anyhow::Result;
use crate::machines::Machine;
use russh::{
	client::{connect, Config, Handle, Handler},
	keys::{
		key::PrivateKeyWithHashAlg, load_secret_key, ssh_key::PublicKey,
	},
	ChannelMsg, Disconnect, Preferred,
};
use tokio::process::Command;
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
	session: Option<Handle<SshClient>>,
}

impl Agent {

	pub async fn new(
		machine: &Machine,
	) -> Result<Self> {
		if machine.is_local().await? {
			return Ok(Self { session: None });
		}
		let key_path = std::env::var("HOME").expect("HOME not defined.") + "/.ssh/id_ed25519";
		let key_pair = load_secret_key(key_path, None).expect("SSH secret key loading failure.");
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
		let mut session = connect(config, (machine.ip, machine.port), sh).await?;
		let auth_res = session
			.authenticate_publickey(
				machine.user,
				PrivateKeyWithHashAlg::new(
					Arc::new(key_pair),
					session.best_supported_rsa_hash().await?.flatten(),
				),
			)
			.await?;
		if !auth_res.success() {
			anyhow::bail!("Authentication (with publickey) failed");
		}
		Ok(Self { session: Some(session) })
	}

	pub async fn execute(&mut self, command: &str) -> Result<(u32, String)> {
		let mut exit_code = None;
		let mut output = Vec::new();
		match &mut self.session {
			None => {
				let mut cmd_builder = Command::new("/bin/bash");
				cmd_builder.arg("-c");
				cmd_builder.arg(command);
				let response = cmd_builder
					.output()
					.await
					.expect("Bash call failed.");
				exit_code = Some(response.status.code().unwrap_or(1) as u32);
				output = response.stdout;
			}
			Some(session) => {
				let cmd = format!("/bin/bash -c \"{}\"", command);
				let mut channel = session
					.channel_open_session()
					.await
					.expect("SSH session failure.");
				channel.exec(true, cmd).await?;
				loop {
					let Some(msg) = channel.wait().await else {
						break;
					};
					match msg {
						ChannelMsg::Data { ref data } => {
							output.extend_from_slice(data);
						}
						ChannelMsg::ExitStatus { exit_status } => {
							exit_code = Some(exit_status);
						}
						_ => {}
					}
				}
			}
		}
		let stdout = String::from_utf8(output).expect("Non-utf8 output encountered.");
		Ok((
			exit_code.expect("Command did not exit cleanly"),
			stdout,
		))
	}

	pub async fn close(&mut self) -> Result<()> {
		if let Some(session) = &self.session {
			session
				.disconnect(Disconnect::ByApplication, "Disconnected", "English")
				.await?;
		}
		Ok(())
	}
}
