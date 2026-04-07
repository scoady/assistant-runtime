use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_REPO_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn temp_repo_root() -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let unique = TEMP_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "assistant-runtime-chat-{}-{}-{}",
        std::process::id(),
        nanos,
        unique
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

fn run_in_repo_with_env(repo: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_assistant-runtime"));
    command.args(args).current_dir(repo);
    for (key, value) in envs {
        command.env(key, value);
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {:?} with env {:?}: {err}", args, envs))
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

#[test]
fn chat_lane_stays_foreground_for_direct_and_planner_paths() {
    let repo = temp_repo_root();

    let direct = run_in_repo(&repo, &["chat", "accept", "answer a direct question"]);
    assert!(direct.status.success(), "chat accept direct failed: {}", stderr(&direct));
    let direct_json: serde_json::Value =
        serde_json::from_slice(&direct.stdout).expect("direct chat accept output was not valid json");
    assert_eq!(
        direct_json["planning_status"]["execution_path"].as_str(),
        Some("direct")
    );
    assert_eq!(
        direct_json["chat_lane"]["status"].as_str(),
        Some("foreground_ready")
    );
    assert_eq!(
        direct_json["queue_task"]["state"].as_str(),
        Some("queued")
    );
    assert_eq!(
        direct_json["worker_handoff"]["handler_runtime"].as_str(),
        Some("assistant.runtime.direct_worker")
    );
    let direct_task_id = direct_json["queue_task"]["id"]
        .as_str()
        .expect("direct queue task id missing")
        .to_string();

    let planner = run_in_repo(
        &repo,
        &["chat", "accept", "implement a multi-step change", "--multi-step"],
    );
    assert!(
        planner.status.success(),
        "chat accept planner failed: {}",
        stderr(&planner)
    );
    let planner_json: serde_json::Value =
        serde_json::from_slice(&planner.stdout).expect("planner chat accept output was not valid json");
    assert_eq!(
        planner_json["planning_status"]["execution_path"].as_str(),
        Some("planner")
    );
    assert_eq!(
        planner_json["chat_lane"]["status"].as_str(),
        Some("foreground_ready")
    );
    assert_eq!(
        planner_json["queue_task"]["state"].as_str(),
        Some("queued")
    );
    assert_eq!(
        planner_json["worker_handoff"]["handler_runtime"].as_str(),
        Some("assistant.runtime.loop")
    );
    let planner_task_id = planner_json["queue_task"]["id"]
        .as_str()
        .expect("planner queue task id missing")
        .to_string();
    assert_ne!(direct_task_id, planner_task_id);

    let explicit_plan = run_in_repo(&repo, &["chat", "accept", "take the planner path", "--plan"]);
    assert!(
        explicit_plan.status.success(),
        "chat accept --plan failed: {}",
        stderr(&explicit_plan)
    );
    let explicit_plan_json: serde_json::Value = serde_json::from_slice(&explicit_plan.stdout)
        .expect("explicit-plan chat accept output was not valid json");
    assert_eq!(
        explicit_plan_json["planning_status"]["execution_path"].as_str(),
        Some("planner")
    );
    assert_eq!(
        explicit_plan_json["queue_task"]["state"].as_str(),
        Some("queued")
    );

    let queue = run_in_repo(&repo, &["chat", "queue"]);
    assert!(queue.status.success(), "chat queue failed: {}", stderr(&queue));
    let queue_json: serde_json::Value =
        serde_json::from_slice(&queue.stdout).expect("chat queue output was not valid json");
    let tasks = queue_json["tasks"].as_array().expect("tasks missing from queue");
    assert!(
        tasks.iter().any(|task| {
            task["intent"].as_str() == Some("answer a direct question")
                && task["handler_runtime"].as_str() == Some("assistant.runtime.direct_worker")
                && task["state"].as_str() == Some("queued")
        }),
        "direct-path chat task was not queued"
    );
    assert!(
        tasks.iter().any(|task| {
            task["intent"].as_str() == Some("implement a multi-step change")
                && task["handler_runtime"].as_str() == Some("assistant.runtime.loop")
                && task["state"].as_str() == Some("queued")
        }),
        "planner-routed chat task was not enqueued"
    );
    assert!(
        tasks.iter().any(|task| {
            task["intent"].as_str() == Some("take the planner path")
                && task["handler_runtime"].as_str() == Some("assistant.runtime.loop")
                && task["state"].as_str() == Some("queued")
        }),
        "explicit --plan chat task was not enqueued"
    );

    let direct_worker = fs::read_to_string(repo.join(".runtime/workers/direct/tasks.json"))
        .expect("direct worker handoff store missing");
    assert!(direct_worker.contains("assistant.runtime.direct_worker"));
    assert!(direct_worker.contains(&direct_task_id));

    let planner_worker = fs::read_to_string(repo.join(".runtime/workers/planner/tasks.json"))
        .expect("planner worker handoff store missing");
    assert!(planner_worker.contains("assistant.runtime.loop"));
    assert!(planner_worker.contains(&planner_task_id));

    let status = run_in_repo(&repo, &["chat", "status"]);
    assert!(status.status.success(), "chat status failed: {}", stderr(&status));
    assert!(stdout(&status).contains("foreground_ready"));
}

#[test]
fn chat_lane_rejects_the_fourth_queued_task() {
    let repo = temp_repo_root();

    for intent in ["task one", "task two", "task three"] {
        let output = run_in_repo(&repo, &["chat", "accept", intent, "--plan"]);
        assert!(output.status.success(), "expected accept to succeed for {intent}: {}", stderr(&output));
    }

    let rejected = run_in_repo(&repo, &["chat", "accept", "task four", "--plan"]);
    assert!(
        !rejected.status.success(),
        "expected fourth queued task to be rejected"
    );
    assert!(
        stderr(&rejected).contains("task queue is full (3 queued tasks)"),
        "unexpected rejection message: {}",
        stderr(&rejected)
    );

    let queue = run_in_repo(&repo, &["chat", "queue"]);
    assert!(queue.status.success(), "chat queue failed: {}", stderr(&queue));
    let queue_json: serde_json::Value =
        serde_json::from_slice(&queue.stdout).expect("chat queue output was not valid json");
    let tasks = queue_json["tasks"].as_array().expect("tasks missing from queue");
    assert_eq!(tasks.len(), 3, "queue should still contain only the first three tasks");

    let status = run_in_repo(&repo, &["chat", "status"]);
    assert!(status.status.success(), "chat status failed: {}", stderr(&status));
    assert!(stdout(&status).contains("foreground_ready"));
}

#[test]
fn exact_skynet_intent_bypasses_agent_routing_and_non_exact_text_does_not() {
    let repo = temp_repo_root();
    let kill_log = repo.join("skynet.log");
    let process_list = "43210 codex worker\n54321 assistant-runtime child\n";

    let skynet = run_in_repo_with_env(
        &repo,
        &["chat", "accept", "skynet"],
        &[
            (
                "ASSISTANT_RUNTIME_KILLSWITCH_PROCESS_LIST",
                process_list,
            ),
            (
                "ASSISTANT_RUNTIME_KILLSWITCH_KILL_LOG",
                kill_log.to_str().expect("kill log path"),
            ),
        ],
    );
    assert!(skynet.status.success(), "chat accept skynet failed: {}", stderr(&skynet));
    let skynet_json: serde_json::Value =
        serde_json::from_slice(&skynet.stdout).expect("skynet output was not valid json");
    assert_eq!(skynet_json["event"].as_str(), Some("skynet"));
    assert_eq!(
        skynet_json["shutdown_state"].as_str(),
        Some("shutdown_initiated")
    );
    let killed = fs::read_to_string(&kill_log).expect("skynet kill log missing");
    assert!(killed.contains("43210"));
    assert!(killed.contains("54321"));

    let queue = run_in_repo(&repo, &["chat", "queue"]);
    let queue_json: serde_json::Value =
        serde_json::from_slice(&queue.stdout).expect("chat queue output was not valid json");
    let tasks = queue_json["tasks"].as_array().expect("tasks missing from queue");
    assert!(tasks.is_empty(), "skynet should bypass normal queue routing");

    let plain = run_in_repo_with_env(
        &repo,
        &["chat", "accept", "please say skynet later"],
        &[(
            "ASSISTANT_RUNTIME_KILLSWITCH_PROCESS_LIST",
            process_list,
        )],
    );
    assert!(
        plain.status.success(),
        "non-exact skynet text should route normally: {}",
        stderr(&plain)
    );
    let plain_json: serde_json::Value =
        serde_json::from_slice(&plain.stdout).expect("plain output was not valid json");
    assert_eq!(plain_json["schema"].as_str(), Some("assistant.runtime.chat.accept.v1"));
}
