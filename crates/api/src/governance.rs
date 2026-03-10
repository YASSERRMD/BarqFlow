use crate::auth::AuthenticatedUser;
use crate::repositories::governance::GovernanceRepository;
use barqflow_core::types::GenericValue;
use barqflow_db::models::{SecretProviderEntity, WorkspacePolicyEntity};
use barqflow_nodes::{node_support_tier, NodeSupportTier};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSecretRef {
    pub provider_id: Uuid,
    #[serde(default)]
    pub path: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowPolicyEvaluation {
    pub node_count: usize,
    pub requires_approval: bool,
    pub approval_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecretProviderValidationResult {
    pub status: String,
    pub message: Option<String>,
    pub validated_at: DateTime<Utc>,
}

pub async fn resolve_credential_data(
    governance_repo: &GovernanceRepository,
    workspace_id: Uuid,
    data: &Value,
) -> Result<Value, String> {
    let Some(object) = data.as_object() else {
        return Err("Credential payload must be a JSON object".to_string());
    };

    let mut resolved = serde_json::Map::new();
    for (key, value) in object {
        if let Some(secret_ref) = extract_external_secret_ref(value) {
            let secret =
                resolve_secret_reference(governance_repo, workspace_id, &secret_ref).await?;
            resolved.insert(key.clone(), Value::String(secret));
        } else {
            resolved.insert(key.clone(), value.clone());
        }
    }

    Ok(Value::Object(resolved))
}

pub async fn resolve_credential_map(
    governance_repo: &GovernanceRepository,
    workspace_id: Uuid,
    data: &Value,
) -> Result<HashMap<String, GenericValue>, String> {
    let resolved = resolve_credential_data(governance_repo, workspace_id, data).await?;
    let object = resolved
        .as_object()
        .ok_or_else(|| "Resolved credential payload is not an object".to_string())?;

    Ok(object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

pub async fn resolve_secret_reference(
    governance_repo: &GovernanceRepository,
    workspace_id: Uuid,
    secret_ref: &ExternalSecretRef,
) -> Result<String, String> {
    let provider = governance_repo
        .find_secret_provider_in_workspace(workspace_id, secret_ref.provider_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Secret provider '{}' not found", secret_ref.provider_id))?;

    match provider.provider_type.trim().to_ascii_lowercase().as_str() {
        "env" => resolve_env_secret(&provider, secret_ref),
        "vault" => resolve_vault_secret(&provider, secret_ref).await,
        other => Err(format!("Unsupported secret provider type '{}'", other)),
    }
}

pub async fn validate_secret_provider(
    provider_type: &str,
    config: &Value,
) -> SecretProviderValidationResult {
    let validated_at = Utc::now();
    let (status, message) = match provider_type.trim().to_ascii_lowercase().as_str() {
        "env" => validate_env_provider(config),
        "vault" => validate_vault_provider(config).await,
        other => (
            "needsAttention".to_string(),
            Some(format!("Unsupported secret provider type '{}'", other)),
        ),
    };

    SecretProviderValidationResult {
        status,
        message,
        validated_at,
    }
}

pub async fn enforce_workflow_policy(
    governance_repo: &GovernanceRepository,
    workspace_id: Uuid,
    nodes: &Value,
) -> Result<WorkflowPolicyEvaluation, String> {
    let policy = governance_repo
        .get_workspace_policy(workspace_id)
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| default_workspace_policy(workspace_id));

    evaluate_workflow_policy(&policy, nodes)
}

pub async fn record_governance_event(
    governance_repo: &GovernanceRepository,
    auth: &AuthenticatedUser,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    summary: &str,
    metadata: Value,
) -> Result<(), String> {
    governance_repo
        .create_audit_log(
            auth.workspace_id,
            Some(auth.id),
            Some(auth.email.as_str()),
            action,
            resource_type,
            resource_id,
            summary,
            metadata,
        )
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn extract_external_secret_ref(value: &Value) -> Option<ExternalSecretRef> {
    let secret_ref = value.get("__secretRef")?.clone();
    serde_json::from_value(secret_ref).ok()
}

pub fn default_workspace_policy(workspace_id: Uuid) -> WorkspacePolicyEntity {
    let now = Utc::now();
    WorkspacePolicyEntity {
        workspace_id,
        blocked_node_types: json!([]),
        blocked_support_tiers: json!([]),
        approval_required_node_types: json!([]),
        max_workflow_nodes: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn evaluate_workflow_policy(
    policy: &WorkspacePolicyEntity,
    nodes: &Value,
) -> Result<WorkflowPolicyEvaluation, String> {
    let raw_nodes = nodes
        .as_array()
        .ok_or_else(|| "Workflow nodes must be a JSON array".to_string())?;

    let blocked_node_types = json_string_set(&policy.blocked_node_types);
    let blocked_support_tiers = json_string_set(&policy.blocked_support_tiers)
        .into_iter()
        .map(|tier| tier.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let approval_required_node_types = json_string_set(&policy.approval_required_node_types);

    if let Some(max_workflow_nodes) = policy.max_workflow_nodes {
        if raw_nodes.len() > max_workflow_nodes.max(0) as usize {
            return Err(format!(
                "Workflow exceeds the workspace node limit of {} nodes",
                max_workflow_nodes
            ));
        }
    }

    let mut approval_reasons = Vec::new();
    for node in raw_nodes {
        let node_type = node
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        let node_name = node
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(node_type);

        if blocked_node_types
            .iter()
            .any(|blocked| blocked == node_type)
        {
            return Err(format!(
                "Workflow policy blocks node '{}' ({}) in this workspace",
                node_name, node_type
            ));
        }

        let support_tier = node_support_tier(node_type)
            .map(node_support_tier_label)
            .unwrap_or("beta");
        if blocked_support_tiers
            .iter()
            .any(|blocked_tier| blocked_tier == support_tier)
        {
            return Err(format!(
                "Workflow policy blocks '{}' tier nodes. '{}' ({}) is classified as {}",
                support_tier, node_name, node_type, support_tier
            ));
        }

        if approval_required_node_types
            .iter()
            .any(|required| required == node_type)
        {
            approval_reasons.push(format!(
                "Node '{}' ({}) requires promotion approval",
                node_name, node_type
            ));
        }
    }

    Ok(WorkflowPolicyEvaluation {
        node_count: raw_nodes.len(),
        requires_approval: !approval_reasons.is_empty(),
        approval_reasons,
    })
}

fn node_support_tier_label(tier: NodeSupportTier) -> &'static str {
    match tier {
        NodeSupportTier::Supported => "supported",
        NodeSupportTier::Beta => "beta",
        NodeSupportTier::Hidden => "hidden",
    }
}

fn json_string_set(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|entry| entry.trim())
        .filter(|entry| !entry.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn validate_env_provider(config: &Value) -> (String, Option<String>) {
    let prefix = config
        .get("prefix")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty());

    let Some(prefix) = prefix else {
        return (
            "needsAttention".to_string(),
            Some("Environment providers require a non-empty prefix".to_string()),
        );
    };

    let missing_vars = config
        .get("requiredSecrets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|key| env_secret_name(prefix, "", key))
        .filter(|env_key| std::env::var(env_key).is_err())
        .collect::<Vec<_>>();

    if missing_vars.is_empty() {
        ("validated".to_string(), None)
    } else {
        (
            "needsAttention".to_string(),
            Some(format!(
                "Missing required environment secrets: {}",
                missing_vars.join(", ")
            )),
        )
    }
}

async fn validate_vault_provider(config: &Value) -> (String, Option<String>) {
    let Some(address) = config
        .get("address")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|address| !address.is_empty())
    else {
        return (
            "needsAttention".to_string(),
            Some("Vault providers require an address".to_string()),
        );
    };

    let endpoint = format!("{}/v1/sys/health", address.trim_end_matches('/'));
    match reqwest::Client::new()
        .get(endpoint)
        .timeout(Duration::from_secs(4))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() || response.status().as_u16() == 429 => {
            ("validated".to_string(), None)
        }
        Ok(response) => (
            "needsAttention".to_string(),
            Some(format!(
                "Vault health check returned status {}",
                response.status()
            )),
        ),
        Err(error) => (
            "needsAttention".to_string(),
            Some(format!("Vault health check failed: {}", error)),
        ),
    }
}

fn resolve_env_secret(
    provider: &SecretProviderEntity,
    secret_ref: &ExternalSecretRef,
) -> Result<String, String> {
    let prefix = provider
        .config
        .get("prefix")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty())
        .ok_or_else(|| {
            format!(
                "Environment provider '{}' is missing a prefix",
                provider.name
            )
        })?;

    let env_key = env_secret_name(prefix, &secret_ref.path, &secret_ref.key);
    std::env::var(&env_key).map_err(|_| {
        format!(
            "Environment secret '{}' is not set for provider '{}'",
            env_key, provider.name
        )
    })
}

async fn resolve_vault_secret(
    provider: &SecretProviderEntity,
    secret_ref: &ExternalSecretRef,
) -> Result<String, String> {
    let address = provider
        .config
        .get("address")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|address| !address.is_empty())
        .ok_or_else(|| format!("Vault provider '{}' is missing an address", provider.name))?;
    let mount_path = provider
        .config
        .get("mountPath")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|mount_path| !mount_path.is_empty())
        .ok_or_else(|| format!("Vault provider '{}' is missing a mountPath", provider.name))?;
    let token_env_var = provider
        .config
        .get("tokenEnvVar")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|token_env_var| !token_env_var.is_empty())
        .ok_or_else(|| format!("Vault provider '{}' is missing tokenEnvVar", provider.name))?;
    let token = std::env::var(token_env_var).map_err(|_| {
        format!(
            "Vault token environment variable '{}' is not set for provider '{}'",
            token_env_var, provider.name
        )
    })?;

    let path = secret_ref.path.trim().trim_matches('/');
    if path.is_empty() {
        return Err("Vault secret references require a non-empty path".to_string());
    }

    let endpoint = format!(
        "{}/v1/{}/data/{}",
        address.trim_end_matches('/'),
        mount_path.trim_matches('/'),
        path
    );
    let payload = reqwest::Client::new()
        .get(endpoint)
        .header("X-Vault-Token", token)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|error| format!("Vault request failed: {}", error))?
        .error_for_status()
        .map_err(|error| format!("Vault returned an error: {}", error))?
        .json::<Value>()
        .await
        .map_err(|error| format!("Failed to decode Vault response: {}", error))?;

    payload
        .get("data")
        .and_then(|data| data.get("data"))
        .and_then(|data| data.get(&secret_ref.key))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            format!(
                "Vault secret '{}' does not contain key '{}'",
                path, secret_ref.key
            )
        })
}

fn env_secret_name(prefix: &str, path: &str, key: &str) -> String {
    let mut segments = Vec::new();
    segments.push(prefix.to_string());
    if !path.trim().is_empty() {
        segments.extend(
            path.split(['/', '.', '-'])
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToString::to_string),
        );
    }
    segments.push(key.to_string());

    segments
        .into_iter()
        .map(|segment| {
            segment
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() {
                        character.to_ascii_uppercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_policy_blocks_beta_nodes_and_flags_approval_paths() {
        let workspace_id = Uuid::new_v4();
        let policy = WorkspacePolicyEntity {
            workspace_id,
            blocked_node_types: json!(["barqflow-nodes.github"]),
            blocked_support_tiers: json!(["beta"]),
            approval_required_node_types: json!(["barqflow-nodes.openai"]),
            max_workflow_nodes: Some(5),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let blocked = evaluate_workflow_policy(
            &policy,
            &json!([
                {"name": "Writer", "type": "barqflow-nodes.openai"},
                {"name": "CRM", "type": "barqflow-nodes.monday"}
            ]),
        )
        .unwrap_err();
        assert!(blocked.contains("blocks 'beta' tier nodes"));

        let allowed = evaluate_workflow_policy(
            &WorkspacePolicyEntity {
                blocked_support_tiers: json!([]),
                ..policy.clone()
            },
            &json!([
                {"name": "Writer", "type": "barqflow-nodes.openai"}
            ]),
        )
        .unwrap();
        assert_eq!(allowed.node_count, 1);
        assert!(allowed.requires_approval);
        assert_eq!(allowed.approval_reasons.len(), 1);
    }

    #[test]
    fn extracts_and_resolves_env_secret_names() {
        std::env::set_var("BARQFLOW_SHARED_SLACK_TOKEN", "xoxb-secret");

        let provider = SecretProviderEntity {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Shared env".to_string(),
            provider_type: "env".to_string(),
            config: json!({"prefix": "BARQFLOW"}),
            status: "validated".to_string(),
            last_validated_at: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let secret_ref = ExternalSecretRef {
            provider_id: provider.id,
            path: "shared/slack".to_string(),
            key: "token".to_string(),
        };

        let resolved = resolve_env_secret(&provider, &secret_ref).unwrap();
        assert_eq!(resolved, "xoxb-secret");
        assert_eq!(
            env_secret_name("BARQFLOW", "shared/slack", "token"),
            "BARQFLOW_SHARED_SLACK_TOKEN"
        );
    }
}
