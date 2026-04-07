use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

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
fn cli_exposes_durable_truth_and_planning_status() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));

    let durable_truth = run_in_repo(repo, &["runtime", "durable-truth"]);
    assert!(
        durable_truth.status.success(),
        "runtime durable-truth failed: {}",
        stderr(&durable_truth)
    );
    let durable_truth_json: serde_json::Value =
        serde_json::from_slice(&durable_truth.stdout).expect("durable-truth output was not valid json");
    assert_eq!(
        durable_truth_json["schema"].as_str(),
        Some("assistant.runtime.durable_truth.v1")
    );

    let implementation_plan = run_in_repo(repo, &["runtime", "implementation-plan"]);
    assert!(
        implementation_plan.status.success(),
        "runtime implementation-plan failed: {}",
        stderr(&implementation_plan)
    );
    let implementation_plan_json: serde_json::Value = serde_json::from_slice(&implementation_plan.stdout)
        .expect("implementation-plan output was not valid json");
    assert_eq!(
        implementation_plan_json["schema"].as_str(),
        Some("assistant.runtime.implementation_plan.v1")
    );

    let proof_metrics = run_in_repo(repo, &["runtime", "proof-metrics"]);
    assert!(
        proof_metrics.status.success(),
        "runtime proof-metrics failed: {}",
        stderr(&proof_metrics)
    );
    let proof_metrics_json: serde_json::Value =
        serde_json::from_slice(&proof_metrics.stdout).expect("proof-metrics output was not valid json");
    assert_eq!(
        proof_metrics_json["schema"].as_str(),
        Some("assistant.runtime.proof_metrics.v1")
    );
    assert_eq!(proof_metrics_json["governed"]["irrelevant_tokens"].as_u64(), Some(0));
    assert!(
        proof_metrics_json["stock"]["irrelevant_tokens"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert!(
        proof_metrics_json["governed"]["truth_delivery_ratio"]
            .as_f64()
            .unwrap_or_default()
            > proof_metrics_json["stock"]["truth_delivery_ratio"]
                .as_f64()
                .unwrap_or_default()
    );

    let benchmark = run_in_repo(repo, &["runtime", "benchmark"]);
    assert!(
        benchmark.status.success(),
        "runtime benchmark failed: {}",
        stderr(&benchmark)
    );
    let benchmark_json: serde_json::Value =
        serde_json::from_slice(&benchmark.stdout).expect("benchmark output was not valid json");
    assert_eq!(
        benchmark_json["schema"].as_str(),
        Some("assistant.runtime.benchmark.v1")
    );
    assert_eq!(benchmark_json["modeled_profiles"].as_bool(), Some(true));
    assert_eq!(benchmark_json["governed"]["turns"].as_u64(), Some(20));
    assert_eq!(benchmark_json["claude"]["turns"].as_u64(), Some(20));
    assert_eq!(benchmark_json["codex"]["turns"].as_u64(), Some(20));
    assert_eq!(benchmark_json["stock"]["turns"].as_u64(), Some(20));
    assert_eq!(
        benchmark_json["governed"]["metrics"]["irrelevant_tokens"].as_u64(),
        Some(0)
    );
    assert!(
        benchmark_json["claude"]["metrics"]["irrelevant_tokens"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert!(
        benchmark_json["codex"]["metrics"]["irrelevant_tokens"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert!(
        benchmark_json["stock"]["metrics"]["irrelevant_tokens"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert_eq!(
        benchmark_json["summary"]["best_truth_delivery_route"].as_str(),
        Some("governed")
    );

    let managed_resources = run_in_repo(repo, &["runtime", "managed-resources"]);
    assert!(
        managed_resources.status.success(),
        "runtime managed-resources failed: {}",
        stderr(&managed_resources)
    );
    let managed_resources_json: serde_json::Value =
        serde_json::from_slice(&managed_resources.stdout)
            .expect("managed-resources output was not valid json");
    assert_eq!(
        managed_resources_json["schema"].as_str(),
        Some("assistant.runtime.managed_resources.v1")
    );
    let resources = managed_resources_json["resources"]
        .as_array()
        .expect("resources missing from managed-resources output");
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("skill_bundle")));
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("tool_surface")));
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("prompt_bundle")));
    assert!(resources.iter().any(|item| item["kind"].as_str() == Some("capability_catalog")));

    let prompts = run_in_repo(repo, &["runtime", "list-prompts"]);
    assert!(
        prompts.status.success(),
        "runtime list-prompts failed: {}",
        stderr(&prompts)
    );
    let prompts_json: serde_json::Value =
        serde_json::from_slice(&prompts.stdout).expect("list-prompts output was not valid json");
    assert_eq!(
        prompts_json["schema"].as_str(),
        Some("assistant.runtime.prompts.list.v1")
    );
    let prompt_list = prompts_json["prompts"]
        .as_array()
        .expect("prompts missing from list-prompts output");
    assert!(prompt_list.iter().any(|item| item["id"].as_str() == Some("assistant.runtime.core.system")));

    let show_prompt = run_in_repo(repo, &["runtime", "show-prompt", "assistant.runtime.core.system"]);
    assert!(
        show_prompt.status.success(),
        "runtime show-prompt failed: {}",
        stderr(&show_prompt)
    );
    let show_prompt_json: serde_json::Value =
        serde_json::from_slice(&show_prompt.stdout).expect("show-prompt output was not valid json");
    assert_eq!(
        show_prompt_json["prompt"]["prompt_role"].as_str(),
        Some("system")
    );

    let default_status = run_in_repo(repo, &["planning", "status"]);
    assert!(
        default_status.status.success(),
        "planning status failed: {}",
        stderr(&default_status)
    );
    assert_eq!(
        stdout(&default_status),
        "[planning_decision: no] [execution_path: direct]"
    );

    let planner_status = run_in_repo(repo, &["planning", "status", "--multi-step"]);
    assert!(
        planner_status.status.success(),
        "planning status --multi-step failed: {}",
        stderr(&planner_status)
    );
    assert_eq!(
        stdout(&planner_status),
        "[planning_decision: yes] [execution_path: planner]"
    );

    let explicit_plan = run_in_repo(repo, &["planning", "status", "--plan"]);
    assert!(
        explicit_plan.status.success(),
        "planning status --plan failed: {}",
        stderr(&explicit_plan)
    );
    assert_eq!(
        stdout(&explicit_plan),
        "[planning_decision: yes] [execution_path: planner]"
    );

    let transcript_path = std::env::temp_dir().join(format!(
        "assistant-runtime-transcript-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    fs::write(
        &transcript_path,
        r#"
[
  {
    "user": "keep chat open while you analyze the queue drift issue",
    "assistant": "Yes. I will route the work below the foreground lane.",
    "accepted": "analyze the queue drift issue"
  },
  {
    "user": "what queue id did that receive",
    "assistant": "I can reference the durable queue id without reopening the work."
  }
]
"#,
    )
    .expect("write transcript");

    let transcript_proof = run_in_repo(
        repo,
        &[
            "runtime",
            "transcript-proof",
            "--file",
            transcript_path.to_str().expect("path"),
        ],
    );
    assert!(
        transcript_proof.status.success(),
        "runtime transcript-proof failed: {}",
        stderr(&transcript_proof)
    );
    let transcript_json: serde_json::Value = serde_json::from_slice(&transcript_proof.stdout)
        .expect("transcript-proof output was not valid json");
    let _ = fs::remove_file(&transcript_path);
    assert_eq!(
        transcript_json["schema"].as_str(),
        Some("assistant.runtime.transcript_proof.v1")
    );
    assert!(
        transcript_json["governed"]["truth_delivery_ratio"]
            .as_f64()
            .unwrap_or_default()
            > transcript_json["stock"]["truth_delivery_ratio"]
                .as_f64()
                .unwrap_or_default()
    );
}
