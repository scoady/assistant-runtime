use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DurableTruthItem {
    pub id: &'static str,
    pub title: &'static str,
    pub statement: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImplementationPlanStep {
    pub id: &'static str,
    pub title: &'static str,
    pub why: &'static str,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DurableTruthReport {
    pub schema: &'static str,
    pub durable_truth: Vec<DurableTruthItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImplementationPlanReport {
    pub schema: &'static str,
    pub objective: &'static str,
    pub steps: Vec<ImplementationPlanStep>,
}

pub fn durable_truth_items() -> Vec<DurableTruthItem> {
    vec![
        DurableTruthItem {
            id: "ideation_not_execution_state",
            title: "Ideation is not execution state",
            statement: "Casual or speculative conversation does not become durable work context unless it is explicitly promoted.",
        },
        DurableTruthItem {
            id: "execution_has_separate_persistence",
            title: "Execution has a separate persistent substrate",
            statement: "Real work is represented in durable runtime state under .runtime/ rather than being inferred from chat history.",
        },
        DurableTruthItem {
            id: "task_has_explicit_execution_path",
            title: "Each task has an explicit execution path",
            statement: "Planning outcome and routing are represented as [planning_decision: x] [execution_path: y].",
        },
        DurableTruthItem {
            id: "chat_lane_stays_foreground",
            title: "The orchestrator stays foreground-ready",
            statement: "Planner-routed and direct work may be accepted and queued below the foreground orchestrator, but the chat lane remains available and foreground-ready.",
        },
        DurableTruthItem {
            id: "orchestrator_routes_but_does_not_execute",
            title: "The orchestrator routes but does not execute",
            statement: "The foreground assistant accepts user intent, assigns a task id, and routes work into the task queue without directly performing execution itself.",
        },
        DurableTruthItem {
            id: "all_work_flows_through_task_queue",
            title: "All accepted work flows through the task queue",
            statement: "Both direct and planner paths first become queued tasks with durable ids before a worker consumes them.",
        },
        DurableTruthItem {
            id: "workers_have_isolated_awareness",
            title: "Workers have isolated and complementary awareness",
            statement: "Direct and planner workers execute from queue handoffs and runtime state, not from the full foreground conversation transcript.",
        },
        DurableTruthItem {
            id: "planning_entry_is_rule_based",
            title: "Planner entry is rule-based",
            statement: "Planning is triggered by explicit conditions like multi-step work, ordering, stop conditions, dependencies, mutation risk, or durable record needs.",
        },
        DurableTruthItem {
            id: "runtime_lanes_are_explicit",
            title: "Runtime lanes are explicit and bounded",
            statement: "assistant.runtime, assistant.runtime.conversation, assistant.runtime.loop, assistant.runtime.host, and assistant.runtime.os each own a separate execution concern.",
        },
        DurableTruthItem {
            id: "resources_are_runtime_managed",
            title: "Skills, tools, capabilities, and utilities are runtime-managed resources",
            statement: "These resources should be discovered and mounted by runtime ownership instead of being ambiently loaded into every project or chat session.",
        },
        DurableTruthItem {
            id: "external_mounts_are_denied",
            title: "External mounts are denied by default",
            statement: "Skills, tools, prompts, MCPs, and session context from outside the governed runtime are denied unless they are declared by the governed runtime contract itself.",
        },
        DurableTruthItem {
            id: "dynamic_tool_creation_is_denied",
            title: "Dynamic tool creation is denied",
            statement: "The runtime does not synthesize new tool surfaces over raw operating system primitives; direct command execution only occurs through assistant.runtime.os.",
        },
        DurableTruthItem {
            id: "capabilities_are_explicit_mounts",
            title: "Capabilities are explicit mounts",
            statement: "A capability should be cataloged, inspectable, and mounted on demand for the active lane rather than treated as always-present context.",
        },
        DurableTruthItem {
            id: "os_execution_is_provable",
            title: "OS execution is provable",
            statement: "assistant.runtime.os returns host process output directly rather than synthesizing an answer.",
        },
        DurableTruthItem {
            id: "work_is_resumable_from_runtime_state",
            title: "Long-running work is resumable from runtime state",
            statement: "Program and runtime state can be resumed from .runtime/ without depending on ambiguous prior conversation turns.",
        },
        DurableTruthItem {
            id: "runtime_contract_is_stable",
            title: "Declared runtimes have a stable contract",
            statement: "A runtime is either implemented and working or predictably unavailable with an explicit contract failure.",
        },
        DurableTruthItem {
            id: "runtime_actions_have_provenance",
            title: "Runtime actions have provenance",
            statement: "The runtime should be able to explain which lane, mount, and durable state caused an action to run.",
        },
        DurableTruthItem {
            id: "contract_is_testable",
            title: "The system is testable",
            statement: "Runtime behavior and planning-state formatting are covered by executable tests instead of being left as prose only.",
        },
    ]
}

pub fn implementation_plan() -> ImplementationPlanReport {
    ImplementationPlanReport {
        schema: "assistant.runtime.implementation_plan.v1",
        objective: "Keep ideation cheap and execution durable by moving active work onto explicit runtime surfaces.",
        steps: vec![
            ImplementationPlanStep {
                id: "state_root",
                title: "Persist execution state under .runtime",
                why: "Separates durable work from transient conversation.",
                done: true,
            },
            ImplementationPlanStep {
                id: "runtime_lanes",
                title: "Expose explicit runtime lanes",
                why: "Lets operators route work to loop, host, or OS behavior intentionally.",
                done: true,
            },
            ImplementationPlanStep {
                id: "orchestrator_vocabulary",
                title: "Standardize the orchestrator, task queue, and worker vocabulary",
                why: "Agentic runtime features should be built on stable names that match standard orchestrator and worker semantics.",
                done: false,
            },
            ImplementationPlanStep {
                id: "planning_status",
                title: "Encode planning status in source",
                why: "Makes planning outcome explicit as [planning_decision: x] [execution_path: y].",
                done: true,
            },
            ImplementationPlanStep {
                id: "durable_truth_contract",
                title: "Publish the durable truth contract through the CLI",
                why: "The runtime should be able to explain its own boundary model without relying on external prose.",
                done: true,
            },
            ImplementationPlanStep {
                id: "prove_runtime_contract",
                title: "Prove the runtime contract with tests",
                why: "Implemented and unavailable runtimes should both be covered by executable checks.",
                done: true,
            },
            ImplementationPlanStep {
                id: "resume_from_state",
                title: "Resume work from durable runtime state",
                why: "Execution should restart from stateful truth, not stale conversational residue.",
                done: true,
            },
            ImplementationPlanStep {
                id: "defer_ambient_skill_loading",
                title: "Keep skills and capability documents out of ambient context by default",
                why: "Only the active runtime slice should be mounted into execution context.",
                done: false,
            },
            ImplementationPlanStep {
                id: "catalog_runtime_resources",
                title: "Catalog skills, tools, capabilities, and utilities as runtime-managed resources",
                why: "The runtime needs a durable, inspectable resource layer instead of project-local prompt sprawl.",
                done: true,
            },
            ImplementationPlanStep {
                id: "queue_first_handoff_model",
                title: "Route all accepted work through a task queue before worker execution",
                why: "The foreground orchestrator should only accept and route, while direct and planner workers consume durable queue handoffs below it.",
                done: true,
            },
            ImplementationPlanStep {
                id: "worker_execution_and_drain",
                title: "Implement worker drain and execution from queued handoffs",
                why: "The direct and planner workers need to move from durable queued state into actual background execution rather than remaining enqueued only.",
                done: false,
            },
            ImplementationPlanStep {
                id: "mount_runtime_resources",
                title: "Mount resources on demand for the active lane",
                why: "Conversation, loop, host, and OS lanes should load only the resources they actually need.",
                done: false,
            },
            ImplementationPlanStep {
                id: "deny_external_runtime_mounts",
                title: "Deny external mounts and ambient session imports",
                why: "The governed runtime should only allow declared runtime-owned skills, tools, prompts, MCPs, and utilities.",
                done: false,
            },
            ImplementationPlanStep {
                id: "mount_skills_and_tools_with_provenance",
                title: "Mount skills and tools with provenance per lane",
                why: "The first mounted resource classes should prove explicit activation, per-lane visibility, and durable provenance before broader capability mounting.",
                done: false,
            },
            ImplementationPlanStep {
                id: "runtime_native_agent_tooling",
                title: "Promote agent tooling into the runtime layer",
                why: "The end state is a runtime that provides its own agent-facing tooling and utilities instead of depending on ambient repo conventions.",
                done: false,
            },
        ],
    }
}

pub fn durable_truth_report() -> DurableTruthReport {
    DurableTruthReport {
        schema: "assistant.runtime.durable_truth.v1",
        durable_truth: durable_truth_items(),
    }
}

#[cfg(test)]
mod tests {
    use super::{durable_truth_report, implementation_plan};

    #[test]
    fn durable_truth_report_has_expected_contract_shape() {
        let report = durable_truth_report();
        assert_eq!(report.schema, "assistant.runtime.durable_truth.v1");
        assert!(report.durable_truth.len() >= 8);
        assert!(
            report
                .durable_truth
                .iter()
                .any(|item| item.id == "resources_are_runtime_managed")
        );
    }

    #[test]
    fn implementation_plan_tracks_done_and_not_done_steps() {
        let plan = implementation_plan();
        assert_eq!(plan.schema, "assistant.runtime.implementation_plan.v1");
        assert!(plan.steps.iter().any(|step| step.done));
        assert!(plan.steps.iter().any(|step| !step.done));
    }
}
