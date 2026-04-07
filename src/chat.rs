use crate::planning::{evaluate_planning_status, ExecutionPath, PlanningContext};
use crate::store::{new_runtime_id, now_unix, read_json, runtime_root, write_json};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_QUEUED_TASKS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatLaneState {
    pub schema: String,
    pub lane: String,
    pub status: String,
    pub last_event_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueLaneTask {
    pub id: String,
    pub intent: String,
    pub state: String,
    pub accepted_at_unix: u64,
    pub planning_decision: String,
    pub execution_path: String,
    pub handler_runtime: String,
    pub queue_lane: String,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueLaneStore {
    pub schema: String,
    pub tasks: Vec<QueueLaneTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerHandoff {
    pub queue_task_id: String,
    pub handler_runtime: String,
    pub state: String,
    pub received_at_unix: u64,
    pub awareness_scope: String,
    pub complementary_only: bool,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerHandoffStore {
    pub schema: String,
    pub worker_lane: String,
    pub tasks: Vec<WorkerHandoff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KillSwitchReport {
    pub schema: String,
    pub accepted: bool,
    pub lane: String,
    pub event: String,
    pub shutdown_state: String,
    pub killed_pids: Vec<u32>,
    pub matched_processes: Vec<String>,
    pub logs: Vec<String>,
}

fn chat_root(base: &Path) -> PathBuf {
    runtime_root(base).join("chat")
}

fn queue_lane_root(base: &Path) -> PathBuf {
    runtime_root(base).join("queue-lane")
}

fn workers_root(base: &Path) -> PathBuf {
    runtime_root(base).join("workers")
}

fn lane_state_path(base: &Path) -> PathBuf {
    chat_root(base).join("state.json")
}

fn queue_lane_path(base: &Path) -> PathBuf {
    queue_lane_root(base).join("tasks.json")
}

fn worker_tasks_path(base: &Path, worker_lane: &str) -> PathBuf {
    workers_root(base).join(worker_lane).join("tasks.json")
}

fn skynet_path(base: &Path) -> PathBuf {
    chat_root(base).join("skynet.json")
}

fn default_lane_state() -> ChatLaneState {
    ChatLaneState {
        schema: "assistant.runtime.chat.state.v1".into(),
        lane: "foreground".into(),
        status: "foreground_ready".into(),
        last_event_at_unix: now_unix(),
    }
}

fn default_queue_lane_store() -> QueueLaneStore {
    QueueLaneStore {
        schema: "assistant.runtime.queue_lane.tasks.v1".into(),
        tasks: Vec::new(),
    }
}

fn default_worker_store(worker_lane: &str) -> WorkerHandoffStore {
    WorkerHandoffStore {
        schema: "assistant.runtime.worker.handoffs.v1".into(),
        worker_lane: worker_lane.into(),
        tasks: Vec::new(),
    }
}

fn read_lane_state(base: &Path) -> Result<ChatLaneState, String> {
    match read_json::<ChatLaneState>(&lane_state_path(base))? {
        Some(state) => Ok(state),
        None => {
            let state = default_lane_state();
            write_json(&lane_state_path(base), &state)?;
            Ok(state)
        }
    }
}

fn write_lane_state(base: &Path, mut state: ChatLaneState) -> Result<ChatLaneState, String> {
    state.last_event_at_unix = now_unix();
    write_json(&lane_state_path(base), &state)?;
    Ok(state)
}

fn touch_foreground_ready(base: &Path) -> Result<ChatLaneState, String> {
    let state = read_lane_state(base)?;
    write_lane_state(
        base,
        ChatLaneState {
            schema: state.schema,
            lane: "foreground".into(),
            status: "foreground_ready".into(),
            last_event_at_unix: state.last_event_at_unix,
        },
    )
}

fn read_queue_lane(base: &Path) -> Result<QueueLaneStore, String> {
    match read_json::<QueueLaneStore>(&queue_lane_path(base))? {
        Some(queue) => Ok(queue),
        None => {
            let queue = default_queue_lane_store();
            write_json(&queue_lane_path(base), &queue)?;
            Ok(queue)
        }
    }
}

fn write_queue_lane(base: &Path, queue: &QueueLaneStore) -> Result<(), String> {
    write_json(&queue_lane_path(base), queue)
}

fn read_worker_store(base: &Path, worker_lane: &str) -> Result<WorkerHandoffStore, String> {
    let path = worker_tasks_path(base, worker_lane);
    match read_json::<WorkerHandoffStore>(&path)? {
        Some(store) => Ok(store),
        None => {
            let store = default_worker_store(worker_lane);
            write_json(&path, &store)?;
            Ok(store)
        }
    }
}

fn write_worker_store(base: &Path, worker_lane: &str, store: &WorkerHandoffStore) -> Result<(), String> {
    write_json(&worker_tasks_path(base, worker_lane), store)
}

fn handler_runtime_for(path: ExecutionPath) -> (&'static str, &'static str) {
    match path {
        ExecutionPath::Direct => ("direct", "assistant.runtime.direct_worker"),
        ExecutionPath::Planner => ("planner", "assistant.runtime.loop"),
    }
}

pub fn status(base: &Path) -> Result<String, String> {
    let state = touch_foreground_ready(base)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.chat.status.v1",
        "chat_lane": state,
    }))
    .map_err(|e| format!("failed to serialize chat status: {e}"))
}

pub fn queue(base: &Path) -> Result<String, String> {
    let queue = read_queue_lane(base)?;
    serde_json::to_string_pretty(&queue).map_err(|e| format!("failed to serialize task queue: {e}"))
}

pub fn accept(base: &Path, intent: &str, planning_context: PlanningContext) -> Result<String, String> {
    let intent = intent.trim();
    if intent.is_empty() {
        return Err("chat intent is empty".into());
    }

    if is_skynet_intent(intent) {
        return skynet(base);
    }

    let planning_status = evaluate_planning_status(planning_context);
    let chat_lane = touch_foreground_ready(base)?;
    let queue_lane = read_queue_lane(base)?;
    let queued_count = queue_lane
        .tasks
        .iter()
        .filter(|task| task.state == "queued")
        .count();
    if queued_count >= MAX_QUEUED_TASKS {
        return Err(format!(
            "task queue is full ({MAX_QUEUED_TASKS} queued tasks); wait for the workers to drain it before accepting more work"
        ));
    }
    let accepted_at_unix = now_unix();
    let queue_task_id = new_runtime_id("task");
    let (worker_lane, handler_runtime) = handler_runtime_for(planning_status.execution_path);

    let queue_task = QueueLaneTask {
        id: queue_task_id.clone(),
        intent: intent.to_string(),
        state: "queued".into(),
        accepted_at_unix,
        planning_decision: planning_status.planning_decision.as_str().into(),
        execution_path: planning_status.execution_path.as_str().into(),
        handler_runtime: handler_runtime.into(),
        queue_lane: "assistant.runtime.task_queue".into(),
        logs: vec![
            "orchestrator accepted message".into(),
            "task saved to task queue".into(),
            format!("handoff prepared for {handler_runtime}"),
            "orchestrator remains foreground_ready".into(),
        ],
    };

    let mut queue_lane = queue_lane;
    queue_lane.tasks.push(queue_task.clone());
    write_queue_lane(base, &queue_lane)?;

    let worker_handoff = WorkerHandoff {
        queue_task_id: queue_task_id.clone(),
        handler_runtime: handler_runtime.into(),
        state: "enqueued".into(),
        received_at_unix: accepted_at_unix,
        awareness_scope: "isolated_from_foreground_chat".into(),
        complementary_only: true,
        logs: vec![
            "received task-queue handoff".into(),
            format!("waiting for {handler_runtime}"),
            "foreground orchestrator remains available".into(),
        ],
    };

    let mut worker_store = read_worker_store(base, worker_lane)?;
    worker_store.tasks.push(worker_handoff.clone());
    write_worker_store(base, worker_lane, &worker_store)?;

    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "assistant.runtime.chat.accept.v1",
        "chat_lane": chat_lane,
        "planning_status": {
            "planning_decision": planning_status.planning_decision.as_str(),
            "execution_path": planning_status.execution_path.as_str(),
        },
        "queue_task": queue_task,
        "worker_handoff": worker_handoff,
        "logs": [
            "orchestrator accepted message",
            "task saved to task queue",
            format!("handoff enqueued for {handler_runtime}"),
            "orchestrator remains foreground_ready"
        ],
    }))
    .map_err(|e| format!("failed to serialize chat accept report: {e}"))
}

pub fn skynet(base: &Path) -> Result<String, String> {
    let _ = touch_foreground_ready(base)?;
    let targets = discover_agent_targets()?;
    let mut killed_pids = Vec::new();
    let mut matched_processes = Vec::new();

    for target in &targets {
        kill_pid(target.pid)?;
        killed_pids.push(target.pid);
        matched_processes.push(target.command.clone());
    }

    let report = KillSwitchReport {
        schema: "assistant.runtime.chat.skynet.v1".into(),
        accepted: true,
        lane: "foreground".into(),
        event: "skynet".into(),
        shutdown_state: "shutdown_initiated".into(),
        killed_pids,
        matched_processes,
        logs: vec![
            "skynet accepted immediately".into(),
            "agent autonomy bypassed".into(),
            "planning bypassed".into(),
            "task queue bypassed".into(),
            "matched agent processes killed".into(),
            "runtime shutdown initiated".into(),
        ],
    };

    write_json(&skynet_path(base), &report)?;
    write_lane_state(
        base,
        ChatLaneState {
            schema: "assistant.runtime.chat.state.v1".into(),
            lane: "foreground".into(),
            status: "shutdown_initiated".into(),
            last_event_at_unix: now_unix(),
        },
    )?;

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to serialize skynet report: {e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentTarget {
    pid: u32,
    command: String,
}

fn is_skynet_intent(intent: &str) -> bool {
    intent.trim().eq_ignore_ascii_case("skynet")
}

fn discover_agent_targets() -> Result<Vec<AgentTarget>, String> {
    if let Ok(raw) = env::var("ASSISTANT_RUNTIME_KILLSWITCH_PROCESS_LIST") {
        return Ok(parse_process_list(&raw));
    }

    let output = Command::new("ps")
        .args(["-Ao", "pid=,command="])
        .output()
        .map_err(|e| format!("failed to inspect processes for skynet: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect processes for skynet: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(parse_process_list(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_process_list(raw: &str) -> Vec<AgentTarget> {
    let self_pid = std::process::id();
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.splitn(2, char::is_whitespace);
            let pid = parts.next()?.trim().parse::<u32>().ok()?;
            let command = parts.next().unwrap_or("").trim().to_string();
            if pid == self_pid || command.is_empty() || !looks_like_agent_process(&command) {
                return None;
            }
            Some(AgentTarget { pid, command })
        })
        .collect()
}

fn looks_like_agent_process(command: &str) -> bool {
    let lowered = command.to_ascii_lowercase();
    [
        "assistant-runtime",
        "codex",
        "chatgpt",
        "claude",
        "cursor-agent",
        "agent",
    ]
    .iter()
    .any(|pattern| lowered.contains(pattern))
}

fn kill_pid(pid: u32) -> Result<(), String> {
    if let Ok(path) = env::var("ASSISTANT_RUNTIME_KILLSWITCH_KILL_LOG") {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("failed to open skynet log: {e}"))?;
        writeln!(file, "{pid}").map_err(|e| format!("failed to write skynet log: {e}"))?;
        return Ok(());
    }

    let status = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .map_err(|e| format!("failed to execute kill -9 {pid}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill -9 {pid} exited with status {:?}", status.code()))
    }
}
