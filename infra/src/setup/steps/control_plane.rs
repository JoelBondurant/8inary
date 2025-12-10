use crate::context;
use crate::setup::utils::inventory;
use crate::setup::SetupStep;
use std::{
	fs,
	io::Write,
	process::{Command, Stdio},
	thread::sleep,
	time::Duration,
};
use tracing::info;

pub struct ControlPlane;

impl ControlPlane {
	pub const CILIUM_CLI_VERSION: &str = "v0.18.9";
	pub const CILIUM_VERSION: &str = "v1.18.4";
	pub const K8S_VERSION: &str = "v1.34.2";
	pub const KUBE_VIP: &str = "192.168.0.2";
	pub const KUBE_VIP_CONTAINER: &str = "ghcr.io/kube-vip/kube-vip";
	pub const KUBE_VIP_CONTAINER_HASH: &str =
		"f86c774c4c0dcab81e56e3bdb42a5a6105c324767cfbc3a44df044f8a2666f8e";
	pub const KUBE_VIP_PORT: &str = "6443";
	pub const KUBE_VIP_VERSION: &str = "v1.0.2";
	pub const NETWORK_INTERFACE: &str = "wlo1";
	pub const POD_CIDR: &str = "10.0.0.0/16";
}

impl SetupStep for ControlPlane {
	fn name(&self) -> &'static str {
		"ControlPlane"
	}

	fn check(&self) -> bool {
		match inventory::this().role {
			inventory::MachineRole::Worker => {
				info!("This machine is a worker, no control plane setup required.");
				return true;
			}
			inventory::MachineRole::ControlPlaneRoot | inventory::MachineRole::ControlPlane => {}
		}
		let is_setup = str::from_utf8(
			&Command::new("kubectl")
				.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
				.args(["get", "node", &context::get().hostname, "--show-labels"])
				.output()
				.expect("Fatal failure resolving control plane status.")
				.stdout,
		)
		.expect("Fatal failure to check control plane membership.")
		.trim()
		.contains("node-role.kubernetes.io/control-plane=");
		if is_setup {
			info!("ControlPlane is already set up.");
			true
		} else {
			info!("ControlPlane is not set up.");
			false
		}
	}

	fn set(&self) {
		info!("ControlPlane setup started.");
		info!("Machine Id: {}", inventory::this().id);
		match inventory::this().role {
			inventory::MachineRole::Worker => {
				info!("This machine is a worker, skipping control plane setup.");
			}
			inventory::MachineRole::ControlPlaneRoot => {
				setup_control_plane_pre();
				setup_control_plane_root();
				setup_control_plane_post();
			}
			inventory::MachineRole::ControlPlane => {
				setup_control_plane_pre();
				setup_control_plane();
				setup_control_plane_post();
			}
		}
		info!("Control plane setup finished.");
	}
}

fn setup_control_plane_pre() {
	info!("Opening Kubernetes ports.");
	Command::new("sh")
		.arg("-c")
		.arg(
			r#"
			sudo ufw allow from 192.168.0.0/16 to any port 2379 proto tcp comment 'etcd client'
			sudo ufw allow from 192.168.0.0/16 to any port 2380 proto tcp comment 'etcd peer'
			sudo ufw allow from 192.168.0.0/16 to any port 6443 proto tcp comment 'kube-apiserver'
			sudo ufw allow from 192.168.0.0/16 to any port 8472 proto udp comment 'cilium vxlan'
			sudo ufw allow from 192.168.0.0/16 to any port 10250 proto tcp comment 'kubelet'
			sudo ufw allow from 192.168.0.0/16 to any port 10257 proto tcp comment 'controller-manager'
			sudo ufw allow from 192.168.0.0/16 to any port 10259 proto tcp comment 'scheduler'
			sudo ufw reload
		"#,
		)
		.status()
		.expect("Fatal failure in port opening.");
	info!("Kubernetes ports are open.");
}

fn setup_control_plane_post() {
	info!("Removing NoSchedule taint for control plane worker mode.");
	sleep(Duration::from_secs(4));
	Command::new("kubectl")
		.args([
			"taint",
			"nodes",
			&context::get().hostname,
			"node-role.kubernetes.io/control-plane:NoSchedule-",
		])
		.status()
		.expect("Fatal failure in NoSchedule taint removal.");
	info!("NoSchedule taint removed.");
}

fn setup_control_plane_root() {
	info!("Bootstrapping control plane root node.");
	info!("Pulling kube-vip container.");
	Command::new("ctr")
		.arg("image")
		.arg("pull")
		.arg(format!(
			"{}:{}@sha256:{}",
			ControlPlane::KUBE_VIP_CONTAINER,
			ControlPlane::KUBE_VIP_VERSION,
			ControlPlane::KUBE_VIP_CONTAINER_HASH,
		))
		.status()
		.expect("Fatal failure to pull kube-vip container.");
	info!("Installing cilium cluster mesh.");
	Command::new("sh")
		.arg("-c")
		.arg(format!(
			r#"
			set -euo pipefail
			cd /tmp
			BASE="https://github.com/cilium/cilium-cli/releases/download/{}"
			curl -fsSL --location "$BASE/cilium-linux-amd64.tar.gz" -o cilium-linux-amd64.tar.gz
			curl -fsSL --location "$BASE/cilium-linux-amd64.tar.gz.sha256sum" -o cilium-linux-amd64.tar.gz.sha256sum
			sha256sum --check cilium-linux-amd64.tar.gz.sha256sum
			tar xzf cilium-linux-amd64.tar.gz cilium
			sudo install -m 0755 cilium /usr/local/bin/cilium
			rm -f cilium-linux-amd64.tar.gz*
		"#
		, ControlPlane::CILIUM_CLI_VERSION))
		.status()
		.expect("Fatal Cilium install failure.");
	info!("Cilium is installed.");
	info!("Hard reset Kubernetes control plane root node.");
	Command::new("sh")
		.arg("-c")
		.arg(
			r#"
			set -euo pipefail
			sudo systemctl stop kubelet || true
			sudo kubeadm reset --force || true
			sudo rm -rf /etc/kubernetes/
			sudo rm -rf /var/lib/kubelet/
			sudo rm -rf /var/lib/etcd/
			sudo rm -rf /opt/cni/
			sudo mkdir -p /etc/kubernetes/manifests/
			sudo mkdir /var/lib/kubelet/
			sudo mkdir /var/lib/etcd/
			sudo mkdir /opt/cni/
			sudo iptables -X || true
			sudo systemctl restart containerd || true
			sudo systemctl start kubelet || true
		"#,
		)
		.status()
		.expect("Fatal Kubernetes reset/cleanup failure.");
	info!("Node has been hard reset.");
	info!("Bootstrapping kube-vip config.");
	let kube_vip_config_out = Command::new("ctr")
		.arg("run")
		.arg("--rm")
		.arg("--net-host")
		.args([
			"--mount",
			"type=bind,src=/etc/kubernetes/manifests,dst=/etc/kubernetes/manifests,options=rbind:rw"
		])
		.arg(format!(
			"{}:{}",
			ControlPlane::KUBE_VIP_CONTAINER,
			ControlPlane::KUBE_VIP_VERSION,
		))
		.arg("kube-vip")
		.arg("manifest")
		.arg("pod")
		.args(["--vip", ControlPlane::KUBE_VIP])
		.args(["--interface", ControlPlane::NETWORK_INTERFACE])
		.arg("--arp")
		.arg("--controlplane")
		.arg("--leaderElection")
		.output()
		.expect("Fatal failure to run kube-vip.");
	let kube_vip_config = String::from_utf8(kube_vip_config_out.stdout)
		.expect("kube-vip manifest returned non-utf-8 output.");
	let kube_vip_config_path = "/etc/kubernetes/manifests/kube-vip.yaml";
	fs::write(kube_vip_config_path, kube_vip_config)
		.expect("Fatal failure to write kube-vip config.");
	info!("Sleeping for kube-vip to bootstrap.");
	sleep(Duration::from_secs(4));
	info!("Kube-vip config written.");
	info!("Kubeadm init.");
	Command::new("kubeadm")
		.arg("init")
		.args([
			"--control-plane-endpoint",
			&format!("{}:{}", ControlPlane::KUBE_VIP, ControlPlane::KUBE_VIP_PORT,),
		])
		.arg("--upload-certs")
		.args(["--pod-network-cidr", ControlPlane::POD_CIDR])
		.args(["--apiserver-advertise-address", ControlPlane::KUBE_VIP])
		.args([
			"--apiserver-cert-extra-sans",
			&format!("{},127.0.0.1,localhost", ControlPlane::KUBE_VIP),
		])
		.args(["--kubernetes-version", ControlPlane::K8S_VERSION])
		.arg("--feature-gates=UserNamespacesSupport=true")
		.arg("--ignore-preflight-errors=NumCPU,Mem")
		.arg("--skip-phases=addon/kube-proxy")
		.status()
		.expect("Fatal kubeadm init failure.");
	info!("Kubeadm initalized.");
	sleep(Duration::from_secs(2));
	info!("Setting cluster trust using embedded CA data.");
	Command::new("bash")
		.arg("-c")
		.arg(format!(
			r#"
			kubectl config set-cluster kubernetes \
				--certificate-authority=<(sudo cat /etc/kubernetes/pki/ca.crt) \
				--embed-certs=true \
				--server=https://{}:{}
		"#,
			ControlPlane::KUBE_VIP,
			ControlPlane::KUBE_VIP_PORT,
		))
		.status()
		.expect("Fatal failure setting cluster trust configuration.");
	sleep(Duration::from_secs(2));
	let home = &context::get().home;
	let user = &context::get().user;
	Command::new("sh")
		.arg("-c")
		.arg(format!(
			r#"
			mkdir -p {}/.kube
			sudo cp -f /etc/kubernetes/admin.conf {}/.kube/config
			sudo chown {}:{} {}/.kube/config
		"#,
			home, home, user, user, home
		))
		.status()
		.expect("Fatal failure to setup Kubeconfig.");
	info!("Kubeconfig set for current user.");
	sleep(Duration::from_secs(2));
	info!("Cilium installing.");
	Command::new("cilium")
		.env("KUBECONFIG", format!("{}/.kube/config", home))
		.arg("install")
		.args(["--version", ControlPlane::CILIUM_VERSION])
		.args(["--set", "kubeProxyReplacement=true"])
		.args([
			"--set",
			&format!(r#"cluster-pool.cidr="{}""#, ControlPlane::POD_CIDR),
		])
		.args(["--set", "hubble.enabled=true"])
		.args(["--set", "hubble.relay.enabled=true"])
		.args(["--set", "hubble.ui.enabled=true"])
		.args(["--set", "tls.ca.enabled=true"])
		.args(["--set", "tls.ca.manage=true"])
		.arg("--wait")
		.status()
		.expect("Fatal failure to install Cilium.");
	info!("Cilium installed.");
}

fn get_control_plane_join_command() -> String {
	let home = &context::get().home;
	let mut child = Command::new("ssh")
		.args(["-o", "LogLevel=ERROR"])
		.args(["-i", &format!("{}/.ssh/id_ed25519", home)])
		.arg(format!("mgmt@{}", ControlPlane::KUBE_VIP))
		.arg("bash")
		.arg("-s")
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("Failed to spawn ssh to first control-plane node.");
	if let Some(mut stdin) = child.stdin.take() {
		stdin
			.write_all(
				br#"
				set -e
				sudo -n bash -c '
					export KUBECONFIG=/etc/kubernetes/admin.conf
					K8S_CERT_KEY=$(kubeadm init phase upload-certs --upload-certs | tail -1 | tr -d "\n")
					kubeadm token create --print-join-command --certificate-key $K8S_CERT_KEY
				'
			"#,
			)
			.expect("Failed to write script to ssh stdin.");
	}
	let output = child
		.wait_with_output()
		.expect("Fatal join command build failure.");
	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		panic!(
			"Failed to get join command from first node. Stderr: {}",
			stderr
		);
	}
	let join_cmd = String::from_utf8(output.stdout)
		.expect("Join command contains invalid UTF-8.")
		.trim()
		.to_owned();
	if join_cmd.is_empty() || !join_cmd.contains("--control-plane") {
		panic!("Received empty or invalid join command: {join_cmd:?}");
	}
	info!("Successfully obtained fresh control-plane join command.");
	join_cmd + " --v=5"
}

fn setup_control_plane() {
	info!("Joining additional control plane node.");
	info!("Hard reset Kubernetes control plane node.");
	Command::new("sh")
		.arg("-c")
		.arg(
			r#"
			set -euo pipefail
			sudo systemctl stop kubelet || true
			sudo kubeadm reset --force || true
			sudo rm /etc/kubernetes/manifests/kube-apiserver.yaml || true
			sudo rm /etc/kubernetes/manifests/kube-controller-manager.yaml || true
			sudo rm /etc/kubernetes/manifests/kube-scheduler.yaml || true
			sudo rm /etc/kubernetes/manifests/etcd.yaml || true
			sudo rm -rf /etc/kubernetes/pki || true
			sudo rm -rf /etc/kubernetes/tmp || true
			sudo systemctl restart containerd || true
			sudo systemctl start kubelet || true
		"#,
		)
		.status()
		.expect("Fatal Kubernetes reset/cleanup failure.");
	info!("Node has been hard reset.");
	let join_command = get_control_plane_join_command();
	info!("Executing join command:\n{join_command}\n");
	Command::new("bash")
		.arg("-c")
		.arg(join_command)
		.status()
		.expect("Fatal failure in control plane join command.");
	info!("This node has joined the control plane.");
	sleep(Duration::from_secs(2));
	let home = &context::get().home;
	let user = &context::get().user;
	Command::new("sh")
		.arg("-c")
		.arg(format!(
			r#"
			mkdir -p {}/.kube
			sudo cp -f /etc/kubernetes/admin.conf {}/.kube/config
			sudo chown {}:{} {}/.kube/config
		"#,
			home, home, user, user, home
		))
		.status()
		.expect("Fatal failure to setup Kubeconfig.");
	info!("Kubeconfig set for current user.");
}
