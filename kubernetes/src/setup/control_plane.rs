use crate::context;
use crate::setup::kubes;
use crate::setup::machines;
use crate::setup::SetupStep;
use std::{fs, process::Command, thread::sleep, time::Duration};
use tracing::info;

pub struct ControlPlane;

impl ControlPlane {
	pub const CILIUM_CLI_VERSION: &str = "v0.18.9";
	pub const CILIUM_VERSION: &str = "v1.18.4";
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
		info!("Checking ControlPlane setup.");
		let this_machine = machines::this();
		if this_machine.role == machines::MachineRole::Worker {
			info!(
				"This machine #{} is a worker, no control plane setup required.",
				this_machine.id
			);
			return true;
		}
		info!("ControlPlane is not set up.");
		false
	}

	fn set(&self) {
		info!("ControlPlane setup started.");
		let this_machine = machines::this();
		if this_machine.role == machines::MachineRole::Worker {
			info!("This machine is a worker, skipping control plane setup.");
			return;
		}
		info!("Pulling kube-vip container.");
		let home = &context::get().home;
		let user = &context::get().user;
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
		if this_machine.role == machines::MachineRole::ControlPlaneRoot {
			info!("Bootstrapping control plane root node.");
			info!("Hard reset Kubernetes node.");
			Command::new("sh")
				.arg("-c")
				.arg(
					r#"
					set -euo pipefail
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
				.arg("--mount")
				.arg("type=bind,src=/etc/kubernetes/manifests,dst=/etc/kubernetes/manifests")
				.arg(format!(
					"{}:{}",
					ControlPlane::KUBE_VIP_CONTAINER,
					ControlPlane::KUBE_VIP_VERSION,
				))
				.arg("kube-vip")
				.arg("manifest")
				.arg("pod")
				.arg("--vip")
				.arg(ControlPlane::KUBE_VIP)
				.arg("--interface")
				.arg(ControlPlane::NETWORK_INTERFACE)
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
				.arg("--control-plane-endpoint")
				.arg(format!(
					"{}:{}",
					ControlPlane::KUBE_VIP,
					ControlPlane::KUBE_VIP_PORT,
				))
				.arg("--upload-certs")
				.arg("--pod-network-cidr")
				.arg(ControlPlane::POD_CIDR)
				.arg("--apiserver-advertise-address")
				.arg(ControlPlane::KUBE_VIP)
				.arg("--apiserver-cert-extra-sans")
				.arg(format!("{},127.0.0.1,localhost", ControlPlane::KUBE_VIP))
				.arg("--kubernetes-version")
				.arg(kubes::Kubes::K8S_VERSION)
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
				.arg("--version")
				.arg(ControlPlane::CILIUM_VERSION)
				.arg("--set")
				.arg("kubeProxyReplacement=true")
				.arg("--set")
				.arg(format!(r#"cluster-pool.cidr="{}""#, ControlPlane::POD_CIDR))
				.arg("--set")
				.arg("hubble.enabled=true")
				.arg("--set")
				.arg("hubble.relay.enabled=true")
				.arg("--set")
				.arg("hubble.ui.enabled=true")
				.arg("--set")
				.arg("tls.ca.enabled=true")
				.arg("--set")
				.arg("tls.ca.manage=true")
				.arg("--wait")
				.status()
				.expect("Fatal failure to install Cilium.");
			info!("Cilium installed.");
		}
		if this_machine.role == machines::MachineRole::ControlPlane {
			info!("Joining additional control plane node.");
			info!("Bootstrapping kube-vip config.");
			let kube_vip_config_out = Command::new("ctr")
				.arg("run")
				.arg("--rm")
				.arg("--net-host")
				.arg("--mount")
				.arg("type=bind,src=/etc/kubernetes/manifests,dst=/etc/kubernetes/manifests")
				.arg(format!(
					"{}:{}",
					ControlPlane::KUBE_VIP_CONTAINER,
					ControlPlane::KUBE_VIP_VERSION,
				))
				.arg("kube-vip")
				.arg("manifest")
				.arg("pod")
				.arg("--vip")
				.arg(ControlPlane::KUBE_VIP)
				.arg("--interface")
				.arg(ControlPlane::NETWORK_INTERFACE)
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
			sleep(Duration::from_secs(8));
			info!("Kube-vip config written.");
			info!("Kubeadm init.");
			Command::new("kubeadm")
				.arg("join")
				.arg(format!(
					"{}:{}",
					ControlPlane::KUBE_VIP,
					ControlPlane::KUBE_VIP_PORT,
				))
				.arg("--token")
				.arg("--discovery-token-ca-cert-hash")
				.arg("sha256:<HASH_FROM_INIT>") //????
				.arg("--control-plane")
				.arg("--certificate-key")
				.arg("<CERT_KEY_FROM_INIT>") // WTF????
				.arg("--apiserver-advertise-address")
				.arg("$NODE_IP")
				.arg("--apiserver-cert-extra-sans")
				.arg(format!("{},127.0.0.1,localhost", ControlPlane::KUBE_VIP))
				.status()
				.expect("Fatal kubeadm init failure.");
		}
		info!("Control plane setup finished.");
	}
}
