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
        "assistant-runtime-smoke-{}-{}",
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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

#[test]
fn every_declared_runtime_has_a_stable_smoke_path() {
    let repo = temp_repo_root();

    let list_types = run_in_repo(&repo, &["runtime", "list-types"]);
    assert!(
        list_types.status.success(),
        "runtime list-types failed: {}",
        stderr(&list_types)
    );
    let listed: serde_json::Value =
        serde_json::from_slice(&list_types.stdout).expect("runtime list-types returned invalid json");
    let runtime_types = listed["runtime_types"]
        .as_array()
        .expect("runtime_types missing");
    let expected = [
        "assistant_runtime",
        "loop_runtime",
        "conversation_runtime",
        "governance_runtime",
        "host_runtime",
        "os_runtime",
        "registry_runtime",
    ];
    for name in expected {
        assert!(
            runtime_types.iter().any(|value| value.as_str() == Some(name)),
            "missing runtime type: {name}"
        );
    }

    let umbrella = run_in_repo(&repo, &["run", "assistant_runtime", "runtime", "manifest"]);
    assert!(
        umbrella.status.success(),
        "assistant_runtime failed: {}",
        stderr(&umbrella)
    );

    let created = run_in_repo(&repo, &["program", "create", "runtime smoke"]);
    assert!(created.status.success(), "program create failed: {}", stderr(&created));
    let created_json: serde_json::Value =
        serde_json::from_slice(&created.stdout).expect("program create returned invalid json");
    let program_id = created_json["id"]
        .as_str()
        .expect("program id missing")
        .to_string();

    let loop_runtime = run_in_repo(&repo, &["run", "loop_runtime", "broad-plan", &program_id]);
    assert!(
        loop_runtime.status.success(),
        "loop_runtime failed: {}",
        stderr(&loop_runtime)
    );
    assert!(
        stdout(&loop_runtime).contains("LadderLoop"),
        "loop_runtime output did not contain LadderLoop"
    );

    let host_runtime = run_in_repo(&repo, &["run", "host_runtime", "boot", "--dry-run"]);
    assert!(
        host_runtime.status.success(),
        "host_runtime failed: {}",
        stderr(&host_runtime)
    );
    assert!(
        stdout(&host_runtime).contains("profile:"),
        "host_runtime dry-run output did not contain a profile"
    );

    let os_runtime = run_in_repo(&repo, &["run", "os_runtime", "pwd"]);
    assert!(
        os_runtime.status.success(),
        "os_runtime failed: {}",
        stderr(&os_runtime)
    );
    let reported = fs::canonicalize(stdout(&os_runtime)).expect("failed to canonicalize os_runtime pwd output");
    let expected = fs::canonicalize(&repo).expect("failed to canonicalize temp repo root");
    assert_eq!(reported, expected);

    let conversation_runtime = run_in_repo(&repo, &["run", "conversation_runtime", "status"]);
    assert!(
        conversation_runtime.status.success(),
        "conversation_runtime failed: {}",
        stderr(&conversation_runtime)
    );
    assert!(
        stdout(&conversation_runtime).contains("foreground_ready"),
        "conversation_runtime status did not contain foreground_ready"
    );

    for runtime in ["governance_runtime", "registry_runtime"] {
        let output = run_in_repo(&repo, &["run", runtime]);
        assert!(
            !output.status.success(),
            "{runtime} unexpectedly succeeded"
        );
        assert!(
            stderr(&output).contains("intentionally not exposed in this build"),
            "{runtime} did not return the expected contract failure: {}",
            stderr(&output)
        );
    }
}
