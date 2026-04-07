use crate::store::{now_unix, read_json, runtime_root, write_json};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProgramStatus {
    Draft,
    Running,
    Blocked,
    Complete,
}

impl Default for ProgramStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramRecord {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub operator_intent: String,
    #[serde(default)]
    pub execution_object: String,
    #[serde(default = "default_completion_rule")]
    pub completion_rule: String,
    #[serde(default = "default_root_intake_id")]
    pub root_intake_id: String,
    #[serde(default)]
    pub status: ProgramStatus,
    #[serde(default)]
    pub created_at_unix: u64,
    #[serde(default)]
    pub completed_steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramStore {
    pub schema: String,
    pub programs: Vec<ProgramRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TruthLadderStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub complete: bool,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TruthLadder {
    pub goal: String,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub current_rung: String,
    pub next_rung: String,
    pub steps: Vec<TruthLadderStep>,
}

const STEP_SPECS: [(&str, &str); 4] = [
    ("inventory", "Inventory the current substrate and current launch posture."),
    ("plan", "Write the bounded implementation plan for the next slice."),
    ("verify", "Verify the current behavior and surface proof."),
    ("complete", "Reach complete launch posture for this bounded program."),
];

fn program_root(base: &Path) -> PathBuf {
    runtime_root(base).join("programs")
}

fn program_path(base: &Path) -> PathBuf {
    program_root(base).join("programs.json")
}

fn default_store() -> ProgramStore {
    ProgramStore { schema: "tool_os.programs.v1".into(), programs: Vec::new() }
}

fn read_store(base: &Path) -> Result<ProgramStore, String> {
    match read_json::<ProgramStore>(&program_path(base))? {
        Some(mut store) => {
            for program in &mut store.programs {
                normalize_program(program);
            }
            Ok(store)
        }
        None => {
            let store = default_store();
            write_json(&program_path(base), &store)?;
            Ok(store)
        }
    }
}

fn write_store(base: &Path, store: &ProgramStore) -> Result<(), String> {
    write_json(&program_path(base), store)
}

fn default_completion_rule() -> String {
    "continue until launch posture materially advances or a real blocker is surfaced".into()
}

fn default_root_intake_id() -> String {
    "manual".into()
}

fn normalize_program(program: &mut ProgramRecord) {
    if program.operator_intent.trim().is_empty() {
        program.operator_intent = program.title.clone();
    }
    if program.execution_object.trim().is_empty() {
        program.execution_object = format!("LaunchProgram: {}", program.operator_intent);
    }
    if program.completion_rule.trim().is_empty() {
        program.completion_rule = default_completion_rule();
    }
    if program.root_intake_id.trim().is_empty() {
        program.root_intake_id = default_root_intake_id();
    }
}

pub fn create_program(base: &Path, goal: &str) -> Result<String, String> {
    let goal = goal.trim();
    if goal.is_empty() {
        return Err("operator intent is empty".into());
    }
    let id = format!("program-{}", now_unix());
    let title = goal.split_whitespace().take(8).collect::<Vec<_>>().join(" ");
    let record = ProgramRecord {
        id: id.clone(),
        title,
        operator_intent: goal.to_string(),
        execution_object: format!("LaunchProgram: {goal}"),
        completion_rule: "continue until launch posture materially advances or a real blocker is surfaced".into(),
        root_intake_id: "manual".into(),
        status: ProgramStatus::Draft,
        created_at_unix: now_unix(),
        completed_steps: 0,
    };
    let mut store = read_store(base)?;
    store.programs.push(record.clone());
    write_store(base, &store)?;
    serde_json::to_string_pretty(&record).map_err(|e| format!("failed to serialize program: {e}"))
}

pub fn list_programs(base: &Path) -> Result<String, String> {
    let store = read_store(base)?;
    serde_json::to_string_pretty(&store.programs).map_err(|e| format!("failed to serialize programs: {e}"))
}

pub fn show_program(base: &Path, program_id: &str) -> Result<String, String> {
    let record = find_program(base, program_id)?;
    let ladder = derive_truth_ladder(&record);
    serde_json::to_string_pretty(&serde_json::json!({
        "program": record,
        "truth_ladder": ladder,
        "durable_truth": durable_truth_for(&ladder),
        "next_durable_truth": ladder.next_rung,
    }))
    .map_err(|e| format!("failed to serialize program: {e}"))
}

pub fn show_truth_ladder(base: &Path, program_id: &str) -> Result<String, String> {
    let record = find_program(base, program_id)?;
    serde_json::to_string_pretty(&derive_truth_ladder(&record)).map_err(|e| format!("failed to serialize ladder: {e}"))
}

pub fn broad_plan(base: &Path, program_id: Option<&str>) -> Result<String, String> {
    let record = resolve_program(base, program_id)?;
    let ladder = derive_truth_ladder(&record);
    let priority_order = ladder.steps.iter().filter(|step| !step.complete).map(|step| step.title.clone()).collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({
        "shortcut": "assistant-runtime program broad-plan [program-id]",
        "program_id": record.id,
        "goal": record.operator_intent,
        "summary": "Fresh broad loop plan derived from the live truth ladder.",
        "outer_loop": {
            "name": "LadderLoop",
            "current_rung": ladder.current_rung,
            "next_rung": ladder.next_rung,
            "priority_order": priority_order,
        },
        "inner_loop": {
            "name": "WhileLoop + TightLoop",
            "active_domain": "runtime",
            "current_focus": priority_order,
            "stop_conditions": [
                "program_complete",
                "program_blocked",
                "max_epochs_reached:<n>"
            ],
        },
        "operator_shortcuts": {
            "show_plan": "assistant-runtime program broad-plan",
            "show_program": format!("assistant-runtime program show {}", record.id),
            "advance_once": format!("assistant-runtime program loop {}", record.id),
            "run_epochs": format!("assistant-runtime program while-loop 3 {}", record.id),
        }
    }))
    .map_err(|e| format!("failed to serialize broad loop plan: {e}"))
}

pub fn broad_plan_execute(base: &Path, program_id: Option<&str>) -> Result<String, String> {
    let record = resolve_program(base, program_id)?;
    Ok(format!("assistant-runtime program while-loop 3 {}", record.id))
}

pub fn broad_plan_approve_and_execute(base: &Path, program_id: Option<&str>) -> Result<String, String> {
    let record = resolve_program(base, program_id)?;
    let execution = serde_json::from_str::<serde_json::Value>(&while_loop(base, Some(&record.id), Some(3))?).map_err(|e| format!("failed to parse approved broad plan execution result: {e}"))?;
    serde_json::to_string_pretty(&serde_json::json!({
        "mode": "approved_broad_plan_execution",
        "program_id": record.id,
        "approval": {
            "approved": true,
            "reason": "operator approved execution of the generated broad-plan run command"
        },
        "generated_command": format!("assistant-runtime program while-loop 3 {}", record.id),
        "execution": execution,
    }))
    .map_err(|e| format!("failed to serialize approved plan execution: {e}"))
}

pub fn loop_once(base: &Path, program_id: &str) -> Result<String, String> {
    let advanced = advance(base, program_id, 1)?;
    serde_json::to_string_pretty(&advanced).map_err(|e| format!("failed to serialize loop result: {e}"))
}

pub fn tight_loop(base: &Path, program_id: &str) -> Result<String, String> {
    let advanced = advance(base, program_id, 1)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "program_id": program_id,
        "status": advanced.status,
        "epoch": advanced,
    }))
    .map_err(|e| format!("failed to serialize tight loop result: {e}"))
}

pub fn while_loop(base: &Path, program_id: Option<&str>, max_epochs: Option<usize>) -> Result<String, String> {
    let record = resolve_program(base, program_id)?;
    let limit = max_epochs.unwrap_or(8);
    let mut epochs = Vec::new();
    for _ in 0..limit {
        let advanced = advance(base, &record.id, 1)?;
        let stop = if matches!(advanced.status, ProgramStatus::Complete) {
            "program_complete".to_string()
        } else {
            "still_running".to_string()
        };
        epochs.push(serde_json::json!({
            "status": advanced.status,
            "completed_steps": advanced.completed_steps,
            "stop_reason": stop,
        }));
        if stop != "still_running" {
            break;
        }
    }
    let final_record = find_program(base, &record.id)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "program_id": record.id,
        "requested_mode": format!("bounded_epochs:{limit}"),
        "stop_reason": if matches!(final_record.status, ProgramStatus::Complete) { "program_complete" } else { "max_epochs_reached" },
        "status": final_record.status,
        "epochs": epochs,
        "final_plan": serde_json::from_str::<serde_json::Value>(&broad_plan(base, Some(&record.id))?).unwrap_or(serde_json::json!({})),
    }))
    .map_err(|e| format!("failed to serialize while loop result: {e}"))
}

fn resolve_program(base: &Path, program_id: Option<&str>) -> Result<ProgramRecord, String> {
    if let Some(program_id) = program_id {
        return find_program(base, program_id);
    }
    let store = read_store(base)?;
    store.programs.last().cloned().ok_or_else(|| "no programs found".to_string())
}

fn find_program(base: &Path, program_id: &str) -> Result<ProgramRecord, String> {
    let store = read_store(base)?;
    store.programs.into_iter().find(|p| p.id == program_id).ok_or_else(|| format!("program not found: {program_id}"))
}

fn advance(base: &Path, program_id: &str, steps: usize) -> Result<ProgramRecord, String> {
    let mut store = read_store(base)?;
    let record = store.programs.iter_mut().find(|p| p.id == program_id).ok_or_else(|| format!("program not found: {program_id}"))?;
    record.completed_steps = usize::min(record.completed_steps + steps, STEP_SPECS.len());
    record.status = if record.completed_steps == 0 {
        ProgramStatus::Draft
    } else if record.completed_steps >= STEP_SPECS.len() {
        ProgramStatus::Complete
    } else {
        ProgramStatus::Running
    };
    let updated = record.clone();
    write_store(base, &store)?;
    Ok(updated)
}

fn derive_truth_ladder(record: &ProgramRecord) -> TruthLadder {
    let steps = STEP_SPECS.iter().enumerate().map(|(index, (title, description))| TruthLadderStep {
        id: format!("step-{}", index + 1),
        title: (*title).to_string(),
        description: (*description).to_string(),
        complete: index < record.completed_steps,
        evidence: if index < record.completed_steps { vec![format!("{} complete", title)] } else { Vec::new() },
    }).collect::<Vec<_>>();
    let current_rung = steps.iter().find(|step| !step.complete).map(|step| step.title.clone()).unwrap_or_else(|| "complete".into());
    let next_rung = steps.iter().skip(record.completed_steps + 1).find(|step| !step.complete).map(|step| step.title.clone()).unwrap_or_else(|| "program reaches complete launch posture".into());
    TruthLadder {
        goal: record.operator_intent.clone(),
        completed_steps: record.completed_steps,
        total_steps: STEP_SPECS.len(),
        current_rung,
        next_rung,
        steps,
    }
}

fn durable_truth_for(ladder: &TruthLadder) -> String {
    ladder.steps.iter().filter(|step| step.complete).last().map(|step| step.description.clone()).unwrap_or_else(|| "program has not advanced yet".into())
}
