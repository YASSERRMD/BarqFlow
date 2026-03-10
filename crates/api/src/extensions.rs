use crate::contracts::{
    ExtensionBundleResponse, ExtensionPermissionScopeResponse, ExtensionProvidedAssetsResponse,
};
use crate::workflow_templates::find_workflow_template;
use barqflow_registry::registry::NodeRegistry;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_EXTENSION_DIR: &str = "extensions";
const MANIFEST_FILE_NAME: &str = "barqflow-plugin.json";
const SUPPORTED_RUNTIMES: &[&str] = &["builtin-pack", "wasm-preview1", "native-static"];
const SUPPORTED_CAPABILITIES: &[&str] = &[
    "workflow:compose",
    "workflow:read",
    "workflow:create",
    "runtime:read",
    "network:outbound",
    "credentials:bind",
    "ai:draft",
    "nodes:llm",
    "execution:annotate",
    "ui:panel",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionManifest {
    id: String,
    name: String,
    vendor: String,
    version: String,
    runtime: String,
    description: String,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    entrypoint: Option<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    permissions: ExtensionPermissionManifest,
    #[serde(default)]
    provides: ExtensionProvidesManifest,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionPermissionManifest {
    #[serde(default)]
    network: Vec<String>,
    #[serde(default)]
    credentials: Vec<String>,
    #[serde(default)]
    workflow: Vec<String>,
    #[serde(default)]
    filesystem: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionProvidesManifest {
    #[serde(default)]
    nodes: Vec<String>,
    #[serde(default)]
    templates: Vec<String>,
    #[serde(default)]
    panels: Vec<String>,
}

pub fn discover_extensions(
    node_registry: &NodeRegistry,
) -> Result<Vec<ExtensionBundleResponse>, String> {
    let search_roots = extension_search_roots();
    discover_extensions_in(&search_roots, node_registry)
}

pub fn discover_extensions_in(
    search_roots: &[PathBuf],
    node_registry: &NodeRegistry,
) -> Result<Vec<ExtensionBundleResponse>, String> {
    let mut bundles = Vec::new();

    for root in search_roots {
        for manifest_path in manifest_paths(root)? {
            let bundle = load_manifest(&manifest_path, node_registry)?;
            bundles.push(bundle);
        }
    }

    bundles.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(bundles)
}

fn extension_search_roots() -> Vec<PathBuf> {
    if let Ok(raw) = env::var("BARQFLOW_EXTENSION_DIRS") {
        let parsed = raw
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![PathBuf::from(DEFAULT_EXTENSION_DIR)]
}

fn manifest_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    let direct_manifest = root.join(MANIFEST_FILE_NAME);
    if direct_manifest.exists() {
        manifests.push(direct_manifest);
    }

    let entries = fs::read_dir(root).map_err(|error| {
        format!(
            "Failed to read extension directory {}: {error}",
            root.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to enumerate extension directory {}: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest = path.join(MANIFEST_FILE_NAME);
        if manifest.exists() {
            manifests.push(manifest);
        }
    }

    manifests.sort();
    Ok(manifests)
}

fn load_manifest(
    manifest_path: &Path,
    node_registry: &NodeRegistry,
) -> Result<ExtensionBundleResponse, String> {
    let raw = fs::read(manifest_path).map_err(|error| {
        format!(
            "Failed to read extension manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let manifest: ExtensionManifest = serde_json::from_slice(&raw).map_err(|error| {
        format!(
            "Failed to parse extension manifest {}: {error}",
            manifest_path.display()
        )
    })?;

    let digest = hex::encode(Sha256::digest(&raw));
    let mut warnings = Vec::new();

    validate_manifest(manifest_path, &manifest, node_registry, &mut warnings);

    let status = if warnings.iter().any(|warning| {
        warning.contains("Unknown runtime")
            || warning.contains("requires an entrypoint")
            || warning.contains("not registered")
            || warning.contains("not available")
    }) {
        "needsAttention"
    } else if warnings.is_empty() {
        "validated"
    } else {
        "validatedWithWarnings"
    };

    Ok(ExtensionBundleResponse {
        id: manifest.id,
        name: manifest.name,
        vendor: manifest.vendor,
        version: manifest.version,
        runtime: manifest.runtime,
        description: manifest.description,
        homepage: manifest.homepage,
        entrypoint: manifest.entrypoint,
        capabilities: dedup_sorted(manifest.capabilities),
        permissions: ExtensionPermissionScopeResponse {
            network: dedup_sorted(manifest.permissions.network),
            credentials: dedup_sorted(manifest.permissions.credentials),
            workflow: dedup_sorted(manifest.permissions.workflow),
            filesystem: dedup_sorted(manifest.permissions.filesystem),
        },
        provided_assets: ExtensionProvidedAssetsResponse {
            nodes: dedup_sorted(manifest.provides.nodes),
            templates: dedup_sorted(manifest.provides.templates),
            panels: dedup_sorted(manifest.provides.panels),
        },
        source_path: manifest_path.display().to_string(),
        digest,
        status: status.to_string(),
        warnings,
    })
}

fn validate_manifest(
    manifest_path: &Path,
    manifest: &ExtensionManifest,
    node_registry: &NodeRegistry,
    warnings: &mut Vec<String>,
) {
    if manifest.id.trim().is_empty() {
        warnings.push("Manifest is missing a stable id.".to_string());
    }
    if manifest.name.trim().is_empty() {
        warnings.push("Manifest is missing a display name.".to_string());
    }
    if manifest.version.trim().is_empty() {
        warnings.push("Manifest is missing a version.".to_string());
    }

    if !SUPPORTED_RUNTIMES.contains(&manifest.runtime.as_str()) {
        warnings.push(format!("Unknown runtime '{}'.", manifest.runtime));
    }

    if matches!(manifest.runtime.as_str(), "wasm-preview1" | "native-static")
        && manifest.entrypoint.is_none()
    {
        warnings.push(format!(
            "Runtime '{}' requires an entrypoint but none was declared.",
            manifest.runtime
        ));
    }

    if let Some(entrypoint) = manifest.entrypoint.as_deref() {
        let entrypoint_path = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(entrypoint);
        if !entrypoint_path.exists() {
            warnings.push(format!(
                "Declared entrypoint '{}' does not exist next to the manifest.",
                entrypoint
            ));
        }
    }

    for capability in &manifest.capabilities {
        if !SUPPORTED_CAPABILITIES.contains(&capability.as_str()) {
            warnings.push(format!(
                "Capability '{}' is not part of the current allow-list.",
                capability
            ));
        }
    }

    for node_type in &manifest.provides.nodes {
        if node_registry.get_latest_node(node_type).is_none() {
            warnings.push(format!(
                "Provided node '{}' is not registered in this BarqFlow build.",
                node_type
            ));
        }
    }

    for template_id in &manifest.provides.templates {
        if find_workflow_template(template_id).is_none() {
            warnings.push(format!(
                "Provided workflow template '{}' is not available in this BarqFlow build.",
                template_id
            ));
        }
    }
}

fn dedup_sorted(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_registry::registry::NodeRegistry;
    use tempfile::tempdir;

    #[test]
    fn discover_extensions_reads_builtin_pack_manifests() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().join("extensions").join("ops-pack");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join(MANIFEST_FILE_NAME),
            r#"{
                "id": "barqflow.ops.pack",
                "name": "Ops Pack",
                "vendor": "BarqFlow",
                "version": "0.1.0",
                "runtime": "builtin-pack",
                "description": "Operational automations.",
                "capabilities": ["workflow:compose", "runtime:read"],
                "permissions": {
                    "network": ["status.example.com"],
                    "credentials": ["slackApi"],
                    "workflow": ["create"]
                },
                "provides": {
                    "nodes": ["barqflow-nodes.slack"],
                    "templates": ["incident-slack-escalation"],
                    "panels": ["runtime-health"]
                }
            }"#,
        )
        .unwrap();

        let registry = NodeRegistry::new();
        barqflow_nodes::register_all_nodes(&registry);

        let bundles =
            discover_extensions_in(&[temp_dir.path().join("extensions")], &registry).unwrap();
        assert_eq!(bundles.len(), 1);
        assert_eq!(bundles[0].id, "barqflow.ops.pack");
        assert_eq!(bundles[0].status, "validated");
        assert!(bundles[0].warnings.is_empty());
        assert_eq!(
            bundles[0].provided_assets.nodes,
            vec!["barqflow-nodes.slack"]
        );
    }

    #[test]
    fn discover_extensions_surfaces_missing_assets_as_warnings() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().join("extensions").join("broken-pack");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join(MANIFEST_FILE_NAME),
            r#"{
                "id": "barqflow.broken.pack",
                "name": "Broken Pack",
                "vendor": "BarqFlow",
                "version": "0.1.0",
                "runtime": "builtin-pack",
                "description": "Broken extension descriptor.",
                "provides": {
                    "nodes": ["barqflow-nodes.unknown"],
                    "templates": ["missing-template"]
                }
            }"#,
        )
        .unwrap();

        let registry = NodeRegistry::new();
        barqflow_nodes::register_all_nodes(&registry);

        let bundles =
            discover_extensions_in(&[temp_dir.path().join("extensions")], &registry).unwrap();
        assert_eq!(bundles[0].status, "needsAttention");
        assert_eq!(bundles[0].warnings.len(), 2);
    }
}
