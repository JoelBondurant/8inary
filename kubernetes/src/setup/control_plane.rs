use crate::setup::kubes;
use crate::setup::SetupStep;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, process::Command};
use tracing::info;

pub struct ControlPlane;

impl ControlPlane {
	pub const POD_CIDR: &str = "10.0.0.1/16";
	pub const NETWORK_INTERFACE: &str = "wlo1";
	pub const KUBE_VIP_CONTAINER: &str = "ghcr.io/kube-vip/kube-vip";
	pub const KUBE_VIP_VERSION: &str = "v1.0.2";
	pub const KUBE_VIP_CONTAINER_HASH: &str =
		"f86c774c4c0dcab81e56e3bdb42a5a6105c324767cfbc3a44df044f8a2666f8e";
	pub const KUBE_VIP: &str = "192.168.6.78";
	pub const KUBE_VIP_PORT: &str = "6443";
}

impl SetupStep for ControlPlane {
	fn name(&self) -> &'static str {
		"ControlPlane"
	}

	fn check(&self) -> bool {
		info!("Checking ControlPlane setup.");
		info!("ControlPlane is not set up.");
		false
	}

	fn set(&self) {
		info!("ControlPlane setup started.");
		info!("Pulling kube-vip container.");
		sudo::escalate_if_needed().expect("Failed to escalate privileges.");
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
		info!("Kube-vip config written.");
		info!("Sleeping for kube-vip to bootstrap.");
		sleep(Duration::from_secs(8));
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
			.arg(ControlPlane::KUBE_VIP)
			.arg("--kubernetes-version")
			.arg(kubes::Kubes::K8S_VERSION)
			.arg("--ignore-preflight-errors=NumCPU,Mem")
			.arg("--skip-phases=addon/kube-proxy")
			.status()
			.expect("Fatal kubeadm init failure.");
		info!("Control plane setup finished.");
	}
}
/*
	sudo snap install cilium --classic

	cilium install \
	  --version 1.18.4 \
	  --set kubeProxyReplacement=strict \
	  --set k8s.cluster.cidr="10.244.0.0/16" \
	  --wait
*/
