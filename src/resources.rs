use crate::assistant_runtime::ManagedResourceSpec;
use crate::store::{now_unix, read_json, runtime_root, write_json};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceCatalog {
    pub schema: String,
    pub resources: Vec<ManagedResourceSpec>,
    pub refreshed_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceMountStore {
    pub schema: String,
    pub mounts: Vec<ResourceMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceMount {
    pub resource_id: String,
    pub resource_class: String,
    pub mount_scope: String,
    pub mount_reason: String,
    pub mounted_by: String,
    pub mounted_at_unix: u64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceStore {
    pub schema: String,
    pub events: Vec<ProvenanceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceEvent {
    pub resource_id: String,
    pub resource_class: String,
    pub lane: String,
    pub event_type: String,
    pub detail: String,
    pub occurred_at_unix: u64,
}

fn resources_root(base: &Path) -> PathBuf {
    runtime_root(base).join("resources")
}

fn catalog_path(base: &Path) -> PathBuf {
    resources_root(base).join("catalog.json")
}

fn mounts_path(base: &Path) -> PathBuf {
    resources_root(base).join("mounts.json")
}

fn provenance_path(base: &Path) -> PathBuf {
    resources_root(base).join("provenance.json")
}

fn default_catalog() -> ResourceCatalog {
    ResourceCatalog {
        schema: "assistant.runtime.resources.catalog.v1".into(),
        resources: crate::assistant_runtime::managed_resources(),
        refreshed_at_unix: now_unix(),
    }
}

fn default_mounts() -> ResourceMountStore {
    ResourceMountStore {
        schema: "assistant.runtime.resources.mounts.v1".into(),
        mounts: Vec::new(),
    }
}

fn default_provenance() -> ProvenanceStore {
    ProvenanceStore {
        schema: "assistant.runtime.resources.provenance.v1".into(),
        events: Vec::new(),
    }
}

pub fn ensure_initialized(base: &Path) -> Result<(), String> {
    if read_json::<ResourceCatalog>(&catalog_path(base))?.is_none() {
        write_json(&catalog_path(base), &default_catalog())?;
    }
    if read_json::<ResourceMountStore>(&mounts_path(base))?.is_none() {
        write_json(&mounts_path(base), &default_mounts())?;
    }
    if read_json::<ProvenanceStore>(&provenance_path(base))?.is_none() {
        write_json(&provenance_path(base), &default_provenance())?;
    }
    Ok(())
}

fn class_matches(resource: &ManagedResourceSpec, class_filter: &str) -> bool {
    match class_filter {
        "skill" | "skills" => resource.kind == "skill_bundle",
        "tool" | "tools" => resource.kind == "tool_surface",
        "mcp" | "mcps" => resource.kind == "mcp_bundle",
        "prompt" | "prompts" => resource.kind == "prompt_bundle",
        "capability" | "capabilities" => resource.kind == "capability_catalog",
        "utility" | "utilities" => resource.kind == "utility_bundle",
        "agent" | "agent_tooling" => resource.kind == "agent_runtime_surface",
        other => resource.kind == other,
    }
}

pub fn list_resources(base: &Path, class_filter: Option<&str>) -> Result<String, String> {
    ensure_initialized(base)?;
    let mut catalog = read_json::<ResourceCatalog>(&catalog_path(base))?
        .ok_or("resource catalog was not initialized")?;

    if catalog.resources != crate::assistant_runtime::managed_resources() {
        catalog = default_catalog();
        write_json(&catalog_path(base), &catalog)?;
    }

    let resources = if let Some(filter) = class_filter {
        catalog
            .resources
            .into_iter()
            .filter(|resource| class_matches(resource, filter))
            .collect::<Vec<_>>()
    } else {
        catalog.resources
    };

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.resources.list.v1",
        "resources": resources,
    }))
    .map_err(|e| format!("failed to serialize resource list: {e}"))
}

pub fn show_resource(base: &Path, resource_id: &str) -> Result<String, String> {
    ensure_initialized(base)?;
    let catalog = read_json::<ResourceCatalog>(&catalog_path(base))?
        .ok_or("resource catalog was not initialized")?;
    let resource = catalog
        .resources
        .into_iter()
        .find(|resource| resource.id == resource_id)
        .ok_or_else(|| format!("unknown resource id: {resource_id}"))?;

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.resources.show.v1",
        "resource": resource,
    }))
    .map_err(|e| format!("failed to serialize resource: {e}"))
}

pub fn provenance(base: &Path) -> Result<String, String> {
    ensure_initialized(base)?;
    let provenance = read_json::<ProvenanceStore>(&provenance_path(base))?
        .ok_or("resource provenance was not initialized")?;
    serde_json::to_string_pretty(&provenance)
        .map_err(|e| format!("failed to serialize provenance: {e}"))
}
