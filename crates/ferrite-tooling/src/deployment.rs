use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const COMPOSE_NODES: [&str; 3] = ["node-1", "node-2", "node-3"];

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    verify_container(workspace)?;
    verify_compose(workspace)?;
    verify_kubernetes(workspace)?;
    println!("deployment contract verified: immutable image, 3 Compose nodes, Kubernetes rollout");
    Ok(())
}

fn verify_container(workspace: &Path) -> Result<()> {
    let dockerfile = read(workspace, "Dockerfile")?;
    for required in [
        "cargo build --locked --release -p ferrite-server",
        "COPY --from=builder /source/target/release/ferrite-server",
        "USER ferrite",
        "ENTRYPOINT [\"/usr/local/bin/ferrite-server\"]",
    ] {
        ensure!(
            dockerfile.contains(required),
            "Dockerfile is missing immutable server-image contract: {required}"
        );
    }
    Ok(())
}

fn verify_compose(workspace: &Path) -> Result<()> {
    let compose: Value =
        serde_yaml::from_str(&read(workspace, "compose.yaml")?).context("parse compose.yaml")?;
    let services = mapping_at(&compose, "services")?;
    let names = services
        .keys()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    ensure!(
        names == COMPOSE_NODES.into_iter().collect(),
        "Compose services must be exactly the three Ferrite development nodes"
    );

    let mut node_ids = BTreeSet::new();
    let mut advertised = BTreeSet::new();
    for node in COMPOSE_NODES {
        let path = format!("deploy/compose/{node}.toml");
        let config: toml::Value =
            toml::from_str(&read(workspace, &path)?).with_context(|| format!("parse {path}"))?;
        ensure!(
            config
                .get("schema_version")
                .and_then(toml::Value::as_integer)
                == Some(1),
            "{path} has the wrong schema"
        );
        let node_id = string_at(&config, &["node", "id"])?;
        node_ids.insert(node_id.to_owned());
        let advertise = string_at(&config, &["remoting", "advertise", "host"])?;
        advertised.insert(advertise.to_owned());
        ensure!(
            string_at(&config, &["discovery", "provider"])? == "development-static",
            "{path} must use explicit development-static discovery"
        );
        let peers = value_at(&config, &["discovery", "peers"])?
            .as_array()
            .context("Compose discovery peers must be an array")?;
        ensure!(peers.len() == 3, "{path} must discover all three nodes");
        ensure!(
            value_at(&config, &["limits", "max_region_mailbox"])?
                .as_integer()
                .is_some_and(|value| value > 0),
            "{path} must enforce a bounded Region mailbox"
        );
    }
    let expected = COMPOSE_NODES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    ensure!(
        node_ids == expected,
        "Compose node identities are not exact"
    );
    ensure!(
        advertised == expected,
        "Compose advertised remoting hosts are not exact"
    );
    Ok(())
}

fn verify_kubernetes(workspace: &Path) -> Result<()> {
    let source = read(workspace, "deploy/kubernetes/ferrite.yaml")?;
    let documents = serde_yaml::Deserializer::from_str(&source)
        .map(Value::deserialize)
        .collect::<Result<Vec<_>, _>>()
        .context("parse Kubernetes deployment documents")?;
    let kinds = documents
        .iter()
        .filter_map(|document| document.get("kind").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    for kind in ["ConfigMap", "Service", "PodDisruptionBudget", "StatefulSet"] {
        ensure!(
            kinds.contains(kind),
            "Kubernetes contract is missing {kind}"
        );
    }
    let stateful_set = documents
        .iter()
        .find(|document| document.get("kind").and_then(Value::as_str) == Some("StatefulSet"))
        .context("Kubernetes StatefulSet is missing")?;
    ensure!(
        yaml_path(stateful_set, &["spec", "replicas"]).and_then(Value::as_i64) == Some(3),
        "Kubernetes StatefulSet must launch three nodes"
    );
    ensure!(
        yaml_path(stateful_set, &["spec", "updateStrategy", "type"]).and_then(Value::as_str)
            == Some("RollingUpdate"),
        "Kubernetes StatefulSet must declare rolling updates"
    );
    let container = yaml_path(stateful_set, &["spec", "template", "spec", "containers"])
        .and_then(Value::as_sequence)
        .and_then(|containers| containers.first())
        .context("Kubernetes Ferrite container is missing")?;
    for path in [
        &["readinessProbe", "httpGet", "path"][..],
        &["livenessProbe", "httpGet", "path"][..],
        &["lifecycle", "preStop", "exec", "command"][..],
    ] {
        ensure!(
            yaml_path(container, path).is_some(),
            "Kubernetes container is missing {}",
            path.join(".")
        );
    }
    ensure!(
        yaml_path(container, &["readinessProbe", "httpGet", "path"]).and_then(Value::as_str)
            == Some("/readyz"),
        "Kubernetes readiness must use /readyz"
    );
    ensure!(
        yaml_path(container, &["livenessProbe", "httpGet", "path"]).and_then(Value::as_str)
            == Some("/healthz"),
        "Kubernetes liveness must use /healthz"
    );
    ensure!(
        source.contains("provider = \"kubernetes\"")
            && source.contains("publishNotReadyAddresses: true")
            && source.contains("http://127.0.0.1:7100/drain"),
        "Kubernetes discovery or graceful-drain contract is incomplete"
    );
    Ok(())
}

fn read(workspace: &Path, relative: &str) -> Result<String> {
    let path = workspace.join(relative);
    fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
}

fn mapping_at<'a>(value: &'a Value, key: &str) -> Result<&'a Mapping> {
    value
        .get(key)
        .and_then(Value::as_mapping)
        .with_context(|| format!("YAML mapping {key} is missing"))
}

fn value_at<'a>(value: &'a toml::Value, path: &[&str]) -> Result<&'a toml::Value> {
    path.iter().try_fold(value, |current, segment| {
        current
            .get(*segment)
            .with_context(|| format!("TOML path {} is missing", path.join(".")))
    })
}

fn string_at<'a>(value: &'a toml::Value, path: &[&str]) -> Result<&'a str> {
    value_at(value, path)?
        .as_str()
        .with_context(|| format!("TOML path {} is not a string", path.join(".")))
}

fn yaml_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, segment| current.get(*segment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_value_helpers_fail_closed() {
        let value: toml::Value = toml::from_str("[node]\nid = \"node-1\"\n").unwrap();
        assert_eq!(string_at(&value, &["node", "id"]).unwrap(), "node-1");
        assert!(value_at(&value, &["node", "missing"]).is_err());

        let yaml: Value = serde_yaml::from_str("spec:\n  replicas: 3\n").unwrap();
        assert_eq!(
            yaml_path(&yaml, &["spec", "replicas"]).and_then(Value::as_i64),
            Some(3)
        );
    }
}
