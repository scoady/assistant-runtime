use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo_root() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    path.push(format!(
        "assistant-runtime-resources-{}-{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&path).expect("failed to create temp repo root");
    path
}

fn run_in_repo(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_assistant-runtime"))
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {:?}: {err}", args))
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

#[test]
fn resource_catalog_and_provenance_scaffolding_are_persisted() {
    let repo = temp_repo_root();

    let list = run_in_repo(&repo, &["runtime", "list-resources"]);
    assert!(
        list.status.success(),
        "runtime list-resources failed: {}",
        stderr(&list)
    );
    let list_json: serde_json::Value =
        serde_json::from_slice(&list.stdout).expect("list-resources returned invalid json");
    assert_eq!(
        list_json["schema"].as_str(),
        Some("assistant.runtime.resources.list.v1")
    );
    let resources = list_json["resources"]
        .as_array()
        .expect("resources missing from list-resources");
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("skill_bundle")));
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("tool_surface")));
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("mcp_bundle")));

    let skills = run_in_repo(&repo, &["runtime", "list-resources", "--class", "skill"]);
    assert!(
        skills.status.success(),
        "runtime list-resources --class skill failed: {}",
        stderr(&skills)
    );
    let skills_json: serde_json::Value =
        serde_json::from_slice(&skills.stdout).expect("skill-filtered resources returned invalid json");
    let skill_resources = skills_json["resources"]
        .as_array()
        .expect("resources missing from skill-filtered list");
    assert!(skill_resources.iter().all(|item| item["kind"].as_str() == Some("skill_bundle")));

    let show = run_in_repo(&repo, &["runtime", "show-resource", "assistant.runtime.skills"]);
    assert!(
        show.status.success(),
        "runtime show-resource failed: {}",
        stderr(&show)
    );
    let show_json: serde_json::Value =
        serde_json::from_slice(&show.stdout).expect("show-resource returned invalid json");
    assert_eq!(
        show_json["resource"]["kind"].as_str(),
        Some("skill_bundle")
    );

    let mcps = run_in_repo(&repo, &["runtime", "list-resources", "--class", "mcp"]);
    assert!(
        mcps.status.success(),
        "runtime list-resources --class mcp failed: {}",
        stderr(&mcps)
    );
    let mcp_json: serde_json::Value =
        serde_json::from_slice(&mcps.stdout).expect("mcp-filtered resources returned invalid json");
    let mcp_resources = mcp_json["resources"]
        .as_array()
        .expect("resources missing from mcp-filtered list");
    assert!(mcp_resources.iter().all(|item| item["kind"].as_str() == Some("mcp_bundle")));

    let governed_runtime = run_in_repo(&repo, &["runtime", "governed-runtime"]);
    assert!(
        governed_runtime.status.success(),
        "runtime governed-runtime failed: {}",
        stderr(&governed_runtime)
    );
    let governed_runtime_json: serde_json::Value = serde_json::from_slice(&governed_runtime.stdout)
        .expect("governed-runtime returned invalid json");
    assert_eq!(
        governed_runtime_json["schema"].as_str(),
        Some("assistant.runtime.governed_runtime.v1")
    );
    assert_eq!(
        governed_runtime_json["runtime"]["queue_runtime"].as_str(),
        Some("assistant.runtime.task_queue")
    );

    let provenance = run_in_repo(&repo, &["runtime", "provenance"]);
    assert!(
        provenance.status.success(),
        "runtime provenance failed: {}",
        stderr(&provenance)
    );
    let provenance_json: serde_json::Value =
        serde_json::from_slice(&provenance.stdout).expect("provenance returned invalid json");
    assert_eq!(
        provenance_json["schema"].as_str(),
        Some("assistant.runtime.resources.provenance.v1")
    );
    assert_eq!(
        provenance_json["events"]
            .as_array()
            .expect("events missing from provenance")
            .len(),
        0
    );

    assert!(repo.join(".runtime/resources/catalog.json").exists());
    assert!(repo.join(".runtime/resources/mounts.json").exists());
    assert!(repo.join(".runtime/resources/provenance.json").exists());
}
