use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn runtime_root(base: &Path) -> PathBuf {
    base.join(".runtime")
}

pub fn ensure_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("failed to create parent dirs for {}: {e}", path.display()))
}

pub fn read_json<T>(path: &Path) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let value = serde_json::from_str::<T>(&raw)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    Ok(Some(value))
}

pub fn write_json<T>(path: &Path, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    ensure_parent(path)?;
    let raw = serde_json::to_string_pretty(value)
        .map_err(|e| format!("failed to serialize {}: {e}", path.display()))?;
    fs::write(path, raw).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn now_unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

pub fn new_runtime_id(prefix: &str) -> String {
    format!("{prefix}-{}-{}", now_unix_nanos(), std::process::id())
}
