use crate::assistant_runtime;
use crate::core::{build_boot_plan, PostCheck, PostReport, RuntimeEnvironment, RuntimeImage, RuntimeProfile};
use crate::proof_metrics;
use crate::resources;
use crate::store::{now_unix, read_json, runtime_root, write_json};
use crate::transcript_proof;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub schema: String,
    pub profile: RuntimeProfile,
    pub image: RuntimeImage,
    pub reason: String,
    pub port: u16,
    pub booted_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopState {
    pub schema: String,
    pub image: RuntimeImage,
    pub status: String,
    pub updated_at_unix: u64,
}

fn runtime_state_path(base: &Path) -> PathBuf {
    runtime_root(base).join("runtime/state.json")
}

fn desktop_state_path(base: &Path) -> PathBuf {
    runtime_root(base).join("desktop/state.json")
}

pub fn boot(base: &Path, profile: RuntimeProfile, port: u16, dry_run: bool) -> Result<(), String> {
    let env = RuntimeEnvironment::detect();
    let plan = build_boot_plan(&env, profile, port);

    if dry_run {
        println!("profile: {:?}", plan.profile);
        println!("image:   {}", plan.image.as_str());
        println!("reason:  {}", plan.reason);
        println!("port:    {}", plan.port);
        return Ok(());
    }

    let state = RuntimeState {
        schema: "tool_os.runtime.state.v1".into(),
        profile: plan.profile,
        image: plan.image,
        reason: plan.reason.clone(),
        port: plan.port,
        booted_at_unix: now_unix(),
    };
    write_json(&runtime_state_path(base), &state)?;

    if matches!(plan.image, RuntimeImage::Desktop) {
        let desktop = DesktopState {
            schema: "tool_os.desktop.state.v1".into(),
            image: plan.image,
            status: "ready".into(),
            updated_at_unix: now_unix(),
        };
        write_json(&desktop_state_path(base), &desktop)?;
    }

    println!("booted assistant.runtime");
    println!("{}", status(base)?);
    Ok(())
}

pub fn status(base: &Path) -> Result<String, String> {
    let current = read_json::<RuntimeState>(&runtime_state_path(base))?;
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "tool_os.runtime.status.v1",
        "state": current,
    }))
    .map_err(|e| format!("failed to serialize runtime status: {e}"))
}

pub fn manifest(_base: &Path) -> Result<String, String> {
    serde_json::to_string_pretty(&assistant_runtime::manifest()).map_err(|e| format!("failed to serialize runtime manifest: {e}"))
}

pub fn governed_runtime_manifest(_base: &Path) -> Result<String, String> {
    serde_json::to_string_pretty(&assistant_runtime::governed_runtime_manifest())
        .map_err(|e| format!("failed to serialize governed-runtime manifest: {e}"))
}

pub fn migration_plan(_base: &Path) -> Result<String, String> {
    let manifest = assistant_runtime::manifest();
    serde_json::to_string_pretty(&serde_json::json!({
        "package_id": manifest.package_id,
        "umbrella_runtime": manifest.umbrella_runtime,
        "compatibility_mode": manifest.compatibility_mode,
        "migration_plan": manifest.migration_plan,
    }))
    .map_err(|e| format!("failed to serialize migration plan: {e}"))
}

pub fn list_types() -> Result<String, String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "umbrella_runtime": "assistant_runtime",
        "runtime_types": assistant_runtime::runtime_type_names(),
    }))
    .map_err(|e| format!("failed to serialize runtime types: {e}"))
}

pub fn managed_resources() -> Result<String, String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.managed_resources.v1",
        "resources": assistant_runtime::managed_resources(),
    }))
    .map_err(|e| format!("failed to serialize managed resources: {e}"))
}

pub fn proof_metrics() -> Result<String, String> {
    serde_json::to_string_pretty(&proof_metrics::proof_metrics_report())
        .map_err(|e| format!("failed to serialize proof metrics: {e}"))
}

pub fn benchmark() -> Result<String, String> {
    serde_json::to_string_pretty(&proof_metrics::benchmark_report())
        .map_err(|e| format!("failed to serialize benchmark report: {e}"))
}

pub fn transcript_proof(path: &Path) -> Result<String, String> {
    serde_json::to_string_pretty(&transcript_proof::transcript_proof_report(path)?)
        .map_err(|e| format!("failed to serialize transcript proof report: {e}"))
}

pub fn security_policy() -> Result<String, String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.security_policy.v1",
        "security_policy": assistant_runtime::security_policy(),
    }))
    .map_err(|e| format!("failed to serialize security policy: {e}"))
}

pub fn list_prompts() -> Result<String, String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.prompts.list.v1",
        "prompts": assistant_runtime::prompt_objects(),
    }))
    .map_err(|e| format!("failed to serialize prompt objects: {e}"))
}

pub fn show_prompt(prompt_id: &str) -> Result<String, String> {
    let prompt = assistant_runtime::prompt_objects()
        .into_iter()
        .find(|prompt| prompt.id == prompt_id)
        .ok_or_else(|| format!("unknown prompt id: {prompt_id}"))?;
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.prompts.show.v1",
        "prompt": prompt,
    }))
    .map_err(|e| format!("failed to serialize prompt object: {e}"))
}

pub fn list_resources(base: &Path, class_filter: Option<&str>) -> Result<String, String> {
    resources::list_resources(base, class_filter)
}

pub fn show_resource(base: &Path, resource_id: &str) -> Result<String, String> {
    resources::show_resource(base, resource_id)
}

pub fn provenance(base: &Path) -> Result<String, String> {
    resources::provenance(base)
}

pub fn show_type(runtime_type: &str) -> Result<String, String> {
    let spec = assistant_runtime::runtime_type(runtime_type).ok_or_else(|| format!("unknown runtime type: {runtime_type}"))?;
    serde_json::to_string_pretty(&spec).map_err(|e| format!("failed to serialize runtime type: {e}"))
}

pub fn package(base: &Path, output: Option<&Path>) -> Result<String, String> {
    let output_root = output.map(PathBuf::from).unwrap_or_else(|| base.join("dist/assistant-runtime"));
    let bin_dir = output_root.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| format!("failed to create package output dir: {e}"))?;

    let current_exe = std::env::current_exe().map_err(|e| format!("failed to resolve current executable: {e}"))?;
    let packaged_binary = bin_dir.join("assistant-runtime");
    fs::copy(&current_exe, &packaged_binary).map_err(|e| format!("failed to copy runtime binary: {e}"))?;

    for (name, script) in [
        ("assistant-loop-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$SCRIPT_DIR/assistant-runtime\" run loop_runtime \"$@\"\n"),
        ("assistant-host-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$SCRIPT_DIR/assistant-runtime\" runtime \"$@\"\n"),
        ("assistant-os-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$SCRIPT_DIR/assistant-runtime\" run os_runtime \"$@\"\n"),
        ("assistant-conversation-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec \"$SCRIPT_DIR/assistant-runtime\" run conversation_runtime \"$@\"\n"),
        ("assistant-governance-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\necho \"governance_runtime is intentionally not exposed in this build\" >&2\nexit 1\n"),
        ("assistant-registry-runtime", "#!/usr/bin/env bash\nset -euo pipefail\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\necho \"registry_runtime is intentionally not exposed in this build\" >&2\nexit 1\n"),
    ] {
        let path = bin_dir.join(name);
        fs::write(&path, script).map_err(|e| format!("failed to write wrapper {name}: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).map_err(|e| format!("failed to read wrapper metadata {name}: {e}"))?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).map_err(|e| format!("failed to set wrapper permissions {name}: {e}"))?;
        }
    }

    let manifest_path = output_root.join("assistant-runtime-manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&assistant_runtime::manifest()).map_err(|e| e.to_string())?)
        .map_err(|e| format!("failed to write package manifest: {e}"))?;
    let governed_runtime_path = output_root.join("governed-runtime.json");
    fs::write(&governed_runtime_path, serde_json::to_string_pretty(&assistant_runtime::governed_runtime_manifest()).map_err(|e| e.to_string())?)
        .map_err(|e| format!("failed to write governed-runtime manifest: {e}"))?;
    let install_path = output_root.join("install.sh");
    fs::write(&install_path, install_script()).map_err(|e| format!("failed to write installer: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&packaged_binary).map_err(|e| format!("failed to read binary metadata: {e}"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&packaged_binary, perms).map_err(|e| format!("failed to set binary permissions: {e}"))?;
        let mut perms = fs::metadata(&install_path).map_err(|e| format!("failed to read installer metadata: {e}"))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&install_path, perms).map_err(|e| format!("failed to set installer permissions: {e}"))?;
    }

    serde_json::to_string_pretty(&serde_json::json!({
        "package_id": "assistant.runtime",
        "output_dir": output_root.display().to_string(),
        "binary_path": packaged_binary.display().to_string(),
        "manifest_path": manifest_path.display().to_string(),
        "governed_runtime_path": governed_runtime_path.display().to_string(),
        "installer_path": install_path.display().to_string(),
    }))
    .map_err(|e| format!("failed to serialize package output: {e}"))
}

pub fn post(base: &Path, profile: RuntimeProfile, port: u16) -> Result<String, String> {
    let env = RuntimeEnvironment::detect();
    let plan = build_boot_plan(&env, profile, port);
    let checks = vec![
        PostCheck { name: "runtime_state", ok: runtime_state_path(base).exists(), detail: runtime_state_path(base).display().to_string() },
        PostCheck { name: "desktop_state", ok: !matches!(plan.image, RuntimeImage::Desktop) || desktop_state_path(base).exists(), detail: desktop_state_path(base).display().to_string() },
    ];
    let report = PostReport { schema: "tool_os.runtime.post.v1", ok: checks.iter().all(|c| c.ok), profile: plan.profile, image: plan.image, checks };
    serde_json::to_string_pretty(&report).map_err(|e| format!("failed to serialize post report: {e}"))
}

fn install_script() -> &'static str {
    r#"#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: ./install.sh /absolute/path/to/target-repo" >&2
  exit 1
fi

TARGET_REPO="$1"
PACKAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
TARGET_ROOT="$TARGET_REPO/.assistant-runtime"
TARGET_BIN_DIR="$TARGET_ROOT/bin"

mkdir -p "$TARGET_BIN_DIR"
cp -R "$PACKAGE_DIR/bin/." "$TARGET_BIN_DIR/"
cp "$PACKAGE_DIR/assistant-runtime-manifest.json" "$TARGET_ROOT/assistant-runtime-manifest.json"
if [ -f "$PACKAGE_DIR/governed-runtime.json" ]; then
  cp "$PACKAGE_DIR/governed-runtime.json" "$TARGET_ROOT/governed-runtime.json"
fi

cat > "$TARGET_ROOT/README.txt" <<'EOF'
assistant.runtime installed here.

Run from the target repository root:
  ./.assistant-runtime/bin/assistant-runtime runtime manifest
  ./.assistant-runtime/bin/assistant-conversation-runtime status
  ./.assistant-runtime/bin/assistant-loop-runtime broad-plan
  ./.assistant-runtime/bin/assistant-os-runtime ls
  ./.assistant-runtime/bin/assistant-runtime program broad-plan

This runtime uses the current working directory as its state root, so invoke it from the target repository root.
EOF

echo "installed assistant.runtime into $TARGET_ROOT"
"#
}
