use crate::error::InstallError;
use crate::setup::SetupStep;
use std::{process::Command, thread::sleep, time::Duration};
use tracing::info;

#[derive(Debug, Clone)]
pub struct IdentityDatabase;

impl IdentityDatabase {
	pub const VERSION: &str = "v1.6.3";
	pub const CRD_URL: &str =
		"https://raw.githubusercontent.com/pingcap/tidb-operator/{VERSION}/manifests/crd.yaml";
	pub const HELM_REPO: &str = "https://charts.pingcap.org/";
	pub const NAMESPACE: &str = "identity";
	pub const CONFIG: &str = "https://raw.githubusercontent.com/pingcap/tidb-operator/{VERSION}/examples/basic/tidb-cluster.yaml";
	pub const MONITOR_CONFIG: &str = "https://raw.githubusercontent.com/pingcap/tidb-operator/{VERSION}/examples/basic/tidb-monitor.yaml";
}

impl SetupStep for IdentityDatabase {
	fn name(&self) -> &'static str {
		"IdentityDatabase"
	}

	fn check(&self) -> bool {
		false
	}

	fn set(&self) {
		info!("Installing TiDB for identity service.");
		Command::new("sh")
			.arg("-c")
			.arg(
				r#"
				set -euo pipefail
				sudo mkdir -p /mnt/disks/identity/pd
				sudo mkdir -p /mnt/disks/identity/tikv
				sudo mkdir -p /mnt/disks/identity/monitor
				"#,
			)
			.status()
			.expect("Fatal failure to create TiDB local storage mount paths.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args([
				"create",
				"-f",
				&IdentityDatabase::CRD_URL.replace("{VERSION}", IdentityDatabase::VERSION),
			])
			.status()
			.expect("Fatal failure to install custom resource definitions for the TiDB operator.");
		Command::new("helm")
			.args(["repo", "add", "pingcap", IdentityDatabase::HELM_REPO])
			.status()
			.expect("Fatal failure to add pingcap helm repo.");
		Command::new("helm")
			.args(["repo", "update"])
			.status()
			.expect("Fatal failure to update helm.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["create", "namespace", IdentityDatabase::NAMESPACE])
			.status()
			.expect("Fatal failure to create identity admin namespace.");
		Command::new("helm")
			.arg("install")
			.args(["--namespace", IdentityDatabase::NAMESPACE])
			.args(["tidb-operator", "pingcap/tidb-operator"])
			.args(["--version", IdentityDatabase::VERSION])
			.status()
			.expect("Fatal failure to install TiDB operator.");
		let mut is_ready = false;
		for retry in 2..8 {
			sleep(Duration::from_secs(u64::pow(2, retry)));
			let ready_ouput = Command::new("kubectl")
				.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
				.args(["get", "pods"])
				.args(["--namespace", IdentityDatabase::NAMESPACE])
				.args(["-l", "app.kubernetes.io/instance=tidb-operator"])
				.output()
				.expect("Fatal failure to get tidb-operator pod.");
			let ready_msg = str::from_utf8(&ready_ouput.stdout)
				.expect("Fatal failure in tidb-operator pod status output.");
			if ready_msg.lines().count() == 2 {
				is_ready = true;
				info!("TiDB operator pod is ready.");
				break;
			}
		}
		if !is_ready {
			panic!("Fatal failure to ready tidb-operator pod.");
		}
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"<<EOF
					apiVersion: pingcap.com/v1alpha1
					kind: TidbCluster
					metadata:
					  name: {NAMESPACE}
					  namespace: {NAMESPACE}
					spec:
					  version: v8.5.2
					  timezone: UTC
					  pvReclaimPolicy: Retain
					  pd:
						baseImage: pingcap/pd
						replicas: 5
						requests:
						  storage: "10Gi"
						tolerations:
						- key: node-role.kubernetes.io/control-plane
						  operator: Exists
						  effect: NoSchedule
					  tikv:
						baseImage: pingcap/tikv
						replicas: 5
						requests:
						  storage: "100Gi"
						tolerations:
						- key: node-role.kubernetes.io/control-plane
						  operator: Exists
						  effect: NoSchedule
					  tidb:
						baseImage: pingcap/tidb
						replicas: 5
						service:
						  type: ClusterIP
						tolerations:
						- key: node-role.kubernetes.io/control-plane
						  operator: Exists
						  effect: NoSchedule
					EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply tolerations for TiDB.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["--namespace", IdentityDatabase::NAMESPACE])
			.args([
				"apply",
				"-f",
				&IdentityDatabase::CONFIG.replace("{VERSION}", IdentityDatabase::VERSION),
			])
			.status()
			.expect("Fatal failure to create identity database cluster.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["--namespace", IdentityDatabase::NAMESPACE])
			.args([
				"apply",
				"-f",
				&IdentityDatabase::MONITOR_CONFIG.replace("{VERSION}", IdentityDatabase::VERSION),
			])
			.status()
			.expect("Fatal failure to create identity database cluster monitoring.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["patch", "storageclass", "local-storage"])
			.args([
				"-p",
				r#"'{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'"#,
			])
			.status()
			.expect("Fatal failure to make local-storage the default storage class.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"<<EOF
					apiVersion: v1
					kind: PersistentVolume
					metadata:
					  name: local-pv-pd
					spec:
					  capacity:
						storage: 10Gi
					  accessModes:
					  - ReadWriteOnce
					  persistentVolumeReclaimPolicy: Delete
					  storageClassName: local-storage
					  local:
						path: /mnt/disks/identity/pd
					---
					apiVersion: v1
					kind: PersistentVolume
					metadata:
					  name: local-pv-tikv
					spec:
					  capacity:
						storage: 100Gi
					  accessModes:
					  - ReadWriteOnce
					  persistentVolumeReclaimPolicy: Delete
					  storageClassName: local-storage
					  local:
						path: /mnt/disks/identity/tikv
					---
					apiVersion: v1
					kind: PersistentVolume
					metadata:
					  name: local-pv-monitor
					spec:
					  capacity:
						storage: 20Gi
					  accessModes:
					  - ReadWriteOnce
					  persistentVolumeReclaimPolicy: Delete
					  storageClassName: local-storage
					  local:
						path: /mnt/disks/identity/monitor
					EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply persistent volumes for TiDB.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"<<EOF
					---
					# Policy 1: TiDB Component Communication (Port-Specific)
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: tidb-component-communication
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/component: tidb
					  ingress:
					  # Allow TiDB clients from same namespace
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
						toPorts:
						- ports:
						  - port: "4000"
							protocol: TCP
							rules:
								tcp:
								- method: "TLS"
					  # Allow status port for monitoring
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: prometheus
						toPorts:
						- ports:
						  - port: "10080"
							protocol: TCP
					  egress:
					  # Allow connection to PD servers
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: pd
						toPorts:
						- ports:
						  - port: "2379"
							protocol: TCP
					  # DNS resolution
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: kube-system
							k8s-app: kube-dns
						toPorts:
						- ports:
						  - port: "53"
							protocol: UDP
						  - port: "53"
							protocol: TCP

					---
					# Policy 2: PD (Placement Driver) Communication
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: pd-communication
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/component: pd
					  ingress:
					  # Allow TiDB servers to connect
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: tidb
						toPorts:
						- ports:
						  - port: "2379"
							protocol: TCP
					  # Allow TiKV servers to connect
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: tikv
						toPorts:
						- ports:
						  - port: "2379"
							protocol: TCP
					  # PD peer communication
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: pd
						toPorts:
						- ports:
						  - port: "2380"
							protocol: TCP
					  egress:
					  # PD peer communication
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: pd
						toPorts:
						- ports:
						  - port: "2380"
							protocol: TCP
					  # DNS resolution
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: kube-system
							k8s-app: kube-dns
						toPorts:
						- ports:
						  - port: "53"
							protocol: UDP

					---
					# Policy 3: TiKV Storage Communication
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: tikv-communication
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/component: tikv
					  ingress:
					  # Allow TiDB to connect
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: tidb
						toPorts:
						- ports:
						  - port: "20160"
							protocol: TCP
					  # TiKV peer communication
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: tikv
						toPorts:
						- ports:
						  - port: "20160"
							protocol: TCP
					  # Allow PD to connect
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: pd
						toPorts:
						- ports:
						  - port: "20160"
							protocol: TCP
					  egress:
					  # Connect to PD
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: pd
						toPorts:
						- ports:
						  - port: "2379"
							protocol: TCP
					  # TiKV peer communication
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app.kubernetes.io/component: tikv
						toPorts:
						- ports:
						  - port: "20160"
							protocol: TCP
					  # DNS resolution
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: kube-system
							k8s-app: kube-dns
						toPorts:
						- ports:
						  - port: "53"
							protocol: UDP

					---
					# Policy 4: TiDB Operator Access (Restricted)
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: tidb-operator
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/name: tidb-operator
					  egress:
					  # API server access
					  - toEntities:
						- kube-apiserver
						toPorts:
						- ports:
						  - port: "443"
							protocol: TCP
						  - port: "6443"
							protocol: TCP
					  # Access to TiDB components for management
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
						toPorts:
						- ports:
						  - port: "4000"   # TiDB
							protocol: TCP
						  - port: "2379"   # PD client
							protocol: TCP
						  - port: "20160"  # TiKV
							protocol: TCP
						  - port: "10080"  # TiDB status
							protocol: TCP
					  # DNS resolution
					  - toEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: kube-system
							k8s-app: kube-dns
						toPorts:
						- ports:
						  - port: "53"
							protocol: UDP
					  # - toFQDNs:
					  #   - matchName: "8inary.com"

					---
					# Policy 5: External Client Access
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: tidb-external-clients
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/component: tidb
					  ingress:
					  # Allow specific namespaces/apps to connect
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: {NAMESPACE}
							app: {NAMESPACE}
						toPorts:
						- ports:
						  - port: "4000"
							protocol: TCP
							rules:
								tcp:
								- method: "TLS"

					---
					# Policy 6: Monitoring Access (Prometheus/Grafana)
					apiVersion: cilium.io/v2
					kind: CiliumNetworkPolicy
					metadata:
					  name: tidb-monitoring
					  namespace: {NAMESPACE}
					spec:
					  endpointSelector:
						matchLabels:
						  app.kubernetes.io/instance: {NAMESPACE}-db
					  ingress:
					  # Allow Prometheus scraping
					  - fromEndpoints:
						- matchLabels:
							io.kubernetes.pod.namespace: monitoring
							app: prometheus
						toPorts:
						- ports:
						  - port: "10080"  # TiDB status port
							protocol: TCP
						  - port: "2379"   # PD metrics
							protocol: TCP
						  - port: "20180"  # TiKV status port
							protocol: TCP
					EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply Cilium network policies for TiDB.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"
				<<EOF
					apiVersion: v1
					kind: ConfigMap
					metadata:
					  name: {NAMESPACE}-pd
					  namespace: {NAMESPACE}
					  labels:
						app.kubernetes.io/name: {NAMESPACE}
						app.kubernetes.io/instance: {NAMESPACE}-db
						app.kubernetes.io/component: pd
						app.kubernetes.io/part-of: {NAMESPACE}
					data:
					  config-file: |
						[replication]
						max-replicas = 5
					  startup-script: |
						#!/bin/sh
						set -uo pipefail
						ARGS="--name=\${HOSTNAME} \
						--data-dir=/var/lib/pd \
						--peer-urls=http://0.0.0.0:2380 \
						--advertise-peer-urls=http://\${HOSTNAME}.{NAMESPACE}-pd-peer.{NAMESPACE}.svc:2380 \
						--client-urls=http://0.0.0.0:2379 \
						--advertise-client-urls=http://\${HOSTNAME}.{NAMESPACE}-pd-peer.{NAMESPACE}.svc:2379"
						if [ -f /etc/pd/config-file ]; then
						  ARGS="\${ARGS} --config=/etc/pd/config-file"
						fi
						ARGS="\${ARGS} --initial-cluster=\${HOSTNAME}=http://\${HOSTNAME}.{NAMESPACE}-pd-peer.{NAMESPACE}.svc:2380"
						exec /pd-server \${ARGS}
				EOF"#.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply TiKV placement driver config map.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"
				<<EOF
					apiVersion: v1
					kind: ConfigMap
					metadata:
					  name: {NAMESPACE}-tikv
					  namespace: {NAMESPACE}
					  labels:
						app.kubernetes.io/name: {NAMESPACE}
						app.kubernetes.io/instance: {NAMESPACE}-db
						app.kubernetes.io/component: tikv
						app.kubernetes.io/part-of: {NAMESPACE}
					data:
					  config-file: |
						[storage]
						reserve-space = "10GB"
						[raftstore]
						capacity = "0"
						sync-log = true
						[rocksdb.wal-cf]
						disable-wal = true
					  startup-script: |
						#!/bin/sh
						set -uo pipefail
						ARGS="--addr=0.0.0.0:20160 \
						--advertise-addr=\${HOSTNAME}.{NAMESPACE}-tikv-peer.{NAMESPACE}.svc:20160 \
						--data-dir=/var/lib/tikv \
						--pd={NAMESPACE}-pd.{NAMESPACE}.svc:2379"
						if [ -f /etc/tikv/config-file ]; then
						  ARGS="\${ARGS} --config=/etc/tikv/config-file"
						fi
						exec /tikv-server \${ARGS}
				EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply TiKV config map.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"
				<<EOF
					apiVersion: v1
					kind: ConfigMap
					metadata:
					  name: {NAMESPACE}-tidb
					  namespace: {NAMESPACE}
					  labels:
						app.kubernetes.io/name: {NAMESPACE}
						app.kubernetes.io/instance: {NAMESPACE}-db
						app.kubernetes.io/component: tidb
						app.kubernetes.io/part-of: {NAMESPACE}
					data:
					  config-file: |
						[performance]
						tcp-keep-alive = true
						max-txn-ttl = 10000
					  startup-script: |
						#!/bin/sh
						set -uo pipefail
						ARGS="--store=tikv \
						--advertise-address=\${HOSTNAME}.{NAMESPACE}-tidb-peer.{NAMESPACE}.svc \
						--host=0.0.0.0 \
						-P=4000 \
						--status=10080 \
						--path={NAMESPACE}-pd.{NAMESPACE}.svc:2379"
						if [ -f /etc/tidb/config-file ]; then
						  ARGS="\${ARGS} --config=/etc/tidb/config-file"
						fi
						exec /tidb-server \${ARGS}
				EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply TiDB config map.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["label", "namespace", IdentityDatabase::NAMESPACE])
			.arg("istio-injection=enabled")
			.arg("--overwrite")
			.status()
			.expect("Fatal failure to label identity namespace with istio-injection.");
		Command::new("kubectl")
			.args(["--kubeconfig", "/etc/kubernetes/admin.conf"])
			.args(["apply", "-f", "-"])
			.arg(
				r#"
				<<EOF
					apiVersion: security.istio.io/v1
					kind: PeerAuthentication
					metadata:
					  name: default-identity-mtls
					  namespace: identity
					spec:
					  mtls:
						mode: STRICT
				EOF"#
					.replace("{NAMESPACE}", IdentityDatabase::NAMESPACE),
			)
			.status()
			.expect("Fatal failure to apply TiDB config map.");
	}
}
