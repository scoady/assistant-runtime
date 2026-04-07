use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCompatibilityBinding {
    pub store_id: String,
    pub schema: String,
    pub relative_path: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeTypeSpec {
    pub id: String,
    pub runtime_type: String,
    pub summary: String,
    pub owns: Vec<String>,
    pub entrypoints: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub compatibility_bindings: Vec<RuntimeCompatibilityBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationStep {
    pub step: String,
    pub summary: String,
    pub preserve_state_paths: bool,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagedResourceSpec {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub mounting_mode: String,
    pub default_visibility: String,
    pub owned_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssistantRuntimeManifest {
    pub schema: String,
    pub package_id: String,
    pub version: String,
    pub summary: String,
    pub umbrella_runtime: String,
    pub compatibility_mode: String,
    pub runtimes: Vec<RuntimeTypeSpec>,
    pub managed_resources: Vec<ManagedResourceSpec>,
    pub migration_plan: Vec<MigrationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeInstallSpec {
    pub r#type: String,
    pub command: String,
    pub build: String,
    pub install_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeResourceMountSpec {
    pub resource_id: String,
    pub resource_class: String,
    pub mounting_mode: String,
    pub default_visibility: String,
    pub owned_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeSpec {
    pub state_root: String,
    pub umbrella_runtime: String,
    pub conversation_runtime: String,
    pub queue_runtime: String,
    pub direct_worker_runtime: String,
    pub planner_worker_runtime: String,
    pub declared_runtimes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeManagedResourcesSpec {
    pub skills_dir: String,
    pub resources: Vec<GovernedRuntimeResourceMountSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptObjectSpec {
    pub id: String,
    pub prompt_role: String,
    pub applies_to: Vec<String>,
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeSecurityPolicy {
    pub sandboxed: bool,
    pub external_skill_mounts: String,
    pub external_tool_mounts: String,
    pub external_prompt_mounts: String,
    pub external_mcp_mounts: String,
    pub ambient_session_imports: String,
    pub undeclared_runtime_imports: String,
    pub dynamic_tool_creation: String,
    pub raw_os_primitive_access: String,
    pub allowed_resource_roots: Vec<String>,
    pub durable_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedRuntimeManifest {
    pub schema: String,
    pub name: String,
    pub description: String,
    pub repo_url: String,
    pub skills_dir: String,
    pub install: GovernedRuntimeInstallSpec,
    pub runtime: GovernedRuntimeSpec,
    pub managed_resources: GovernedRuntimeManagedResourcesSpec,
    pub prompt_objects: Vec<PromptObjectSpec>,
    pub security_policy: GovernedRuntimeSecurityPolicy,
}

pub fn manifest() -> AssistantRuntimeManifest {
    AssistantRuntimeManifest {
        schema: "tool_os.assistant_runtime.manifest.v1".to_string(),
        package_id: "assistant.runtime".to_string(),
        version: "1.3.2".to_string(),
        summary: "Compatibility-first assistant runtime package over the current .runtime substrate.".to_string(),
        umbrella_runtime: "assistant_runtime".to_string(),
        compatibility_mode: "reads and writes the current .runtime state stores without destructive migration".to_string(),
        runtimes: runtime_types(),
        managed_resources: managed_resources(),
        migration_plan: migration_plan(),
    }
}

pub fn governed_runtime_manifest() -> GovernedRuntimeManifest {
    GovernedRuntimeManifest {
        schema: "assistant.runtime.governed_runtime.v1".into(),
        name: "assistant-runtime".into(),
        description: "Governed runtime that declaratively owns runtimes, skills, tools, MCPs, capabilities, utilities, and queue-backed worker execution above ambient agent context.".into(),
        repo_url: "https://github.com/scoady/assistant-runtime.git".into(),
        skills_dir: "external_skills".into(),
        install: GovernedRuntimeInstallSpec {
            r#type: "rust_binary".into(),
            command: "assistant-runtime".into(),
            build: "cargo build --release".into(),
            install_target: ".".into(),
        },
        runtime: GovernedRuntimeSpec {
            state_root: ".runtime".into(),
            umbrella_runtime: "assistant.runtime".into(),
            conversation_runtime: "assistant.runtime.conversation".into(),
            queue_runtime: "assistant.runtime.task_queue".into(),
            direct_worker_runtime: "assistant.runtime.direct_worker".into(),
            planner_worker_runtime: "assistant.runtime.loop".into(),
            declared_runtimes: runtime_types().into_iter().map(|item| item.id).collect(),
        },
        managed_resources: GovernedRuntimeManagedResourcesSpec {
            skills_dir: "external_skills".into(),
            resources: managed_resources()
                .into_iter()
                .map(|resource| GovernedRuntimeResourceMountSpec {
                    resource_id: resource.id,
                    resource_class: resource.kind,
                    mounting_mode: resource.mounting_mode,
                    default_visibility: resource.default_visibility,
                    owned_by: resource.owned_by,
                })
                .collect(),
        },
        prompt_objects: prompt_objects(),
        security_policy: security_policy(),
    }
}

pub fn runtime_types() -> Vec<RuntimeTypeSpec> {
    vec![
        runtime(
            "assistant.runtime",
            "assistant_runtime",
            "Umbrella orchestrated runtime for foreground intake, explicit routing, broad planning, and bounded execution.",
            vec!["orchestrator entrypoint", "routing boundary", "broad planning", "bounded loop execution"],
            vec!["assistant-runtime runtime manifest", "assistant-runtime runtime migration-plan", "assistant-runtime program broad-plan"],
            vec!["runtime.inspect", "memory.write"],
            vec![
                binding("discussion_store", "tool_os.discussion.v1", ".runtime/discussion/discussion.json"),
                binding("program_store", "tool_os.programs.v1", ".runtime/programs/programs.json"),
                binding("autopilot_runtime", "tool_os.autopilot.runtime.v1", ".runtime/autopilot/runtime.json"),
                binding("task_queue", "tool_os.tasks.v1", ".runtime/tasks/tasks.json"),
            ],
        ),
        runtime(
            "assistant.runtime.loop",
            "loop_runtime",
            "Proof-backed loop engine for program truth ladders, task selection, and bounded execution.",
            vec!["program truth ladder", "autopilot intake gating", "task queue", "loop execution"],
            vec!["assistant-runtime program broad-plan", "assistant-runtime program loop", "assistant-runtime program while-loop"],
            vec!["runtime.inspect"],
            vec![
                binding("program_store", "tool_os.programs.v1", ".runtime/programs/programs.json"),
                binding("autopilot_runtime", "tool_os.autopilot.runtime.v1", ".runtime/autopilot/runtime.json"),
                binding("task_queue", "tool_os.tasks.v1", ".runtime/tasks/tasks.json"),
            ],
        ),
        runtime(
            "assistant.runtime.conversation",
            "conversation_runtime",
            "Foreground orchestrator lane that stays ready while accepted work is routed into the task queue below it.",
            vec!["foreground orchestrator", "accepted task enqueue", "task queue handoff", "explicit routing to direct and planner workers"],
            vec!["assistant-runtime chat status", "assistant-runtime chat accept", "assistant-runtime chat queue", "assistant-conversation-runtime status"],
            vec!["memory.write"],
            vec![
                binding("chat_lane_state", "assistant.runtime.chat.state.v1", ".runtime/chat/state.json"),
                binding("task_queue", "assistant.runtime.queue.tasks.v1", ".runtime/queue/tasks.json"),
            ],
        ),
        runtime(
            "assistant.runtime.governance",
            "governance_runtime",
            "Governed context substrate for capability checks, policy, sessions, memory, and declared universe.",
            vec!["policy evaluation", "session contracts", "memory journal", "declared universe"],
            vec!["assistant-runtime policy check", "assistant-runtime sessions show", "assistant-runtime memory show", "assistant-runtime universe show"],
            vec!["memory.read"],
            vec![
                binding("universe_store", "tool_os.universe.v1", ".runtime/universe/universe.json"),
                binding("sessions_store", "tool_os.sessions.v1", ".runtime/sessions/sessions.json"),
                binding("memory_store", "tool_os.memory.v1", ".runtime/memory/journal.json"),
                binding("capability_catalog", "tool_os.capabilities.catalog.v1", ".runtime/capabilities/catalog.json"),
            ],
        ),
        runtime(
            "assistant.runtime.host",
            "host_runtime",
            "Host image runtime for server and desktop boot, POST checks, and host-visible runtime state.",
            vec!["boot planning", "server image", "desktop image", "runtime host state"],
            vec!["assistant-runtime runtime boot", "assistant-runtime runtime post", "assistant-runtime runtime status"],
            vec!["runtime.inspect"],
            vec![
                binding("runtime_state", "tool_os.runtime.state.v1", ".runtime/runtime/state.json"),
                binding("desktop_state", "tool_os.desktop.state.v1", ".runtime/desktop/state.json"),
            ],
        ),
        runtime(
            "assistant.runtime.os",
            "os_runtime",
            "Operating system passthrough runtime for provable host command execution on the user's machine.",
            vec!["os command execution", "direct host process launch", "provable raw stdout/stderr passthrough"],
            vec!["assistant-runtime \\\\<command>", "assistant-runtime run os_runtime <command>", "assistant-os-runtime <command>"],
            vec!["runtime.inspect"],
            vec![],
        ),
        runtime(
            "assistant.runtime.registry",
            "registry_runtime",
            "Registry runtime for publishing runtime packages, runtime types, and OpenTool discovery metadata.",
            vec!["registry repos", "package publishing", "runtime discovery"],
            vec!["assistant-runtime registry manifest", "assistant-runtime registry show"],
            vec!["runtime.inspect"],
            vec![binding("registry_store", "tool_os.registry.repos.v1", ".runtime/registry/repos.json")],
        ),
    ]
}

pub fn runtime_type_names() -> Vec<String> {
    runtime_types().into_iter().map(|item| item.runtime_type).collect()
}

pub fn runtime_type(name: &str) -> Option<RuntimeTypeSpec> {
    runtime_types().into_iter().find(|item| item.runtime_type == name)
}

pub fn managed_resources() -> Vec<ManagedResourceSpec> {
    vec![
        managed_resource(
            "assistant.runtime.capabilities",
            "capability_catalog",
            "Declares discoverable runtime features that can be mounted explicitly instead of living in ambient context.",
            "mounted_on_demand",
            "catalog_only",
            vec!["assistant.runtime.governance", "assistant.runtime"],
        ),
        managed_resource(
            "assistant.runtime.skills",
            "skill_bundle",
            "Provides procedural guidance and operator patterns as runtime-mounted resources rather than per-project baggage.",
            "mounted_on_demand",
            "not_ambient",
            vec!["assistant.runtime.governance", "assistant.runtime.conversation"],
        ),
        managed_resource(
            "assistant.runtime.tools",
            "tool_surface",
            "Exposes tool entrypoints through explicit runtime ownership so the active lane only loads what it needs.",
            "mounted_on_demand",
            "not_ambient",
            vec!["assistant.runtime.host", "assistant.runtime.os", "assistant.runtime"],
        ),
        managed_resource(
            "assistant.runtime.mcps",
            "mcp_bundle",
            "Declares MCP servers and adjacent runtime connectors as explicit mounts instead of ambient session baggage.",
            "mounted_on_demand",
            "not_ambient",
            vec!["assistant.runtime", "assistant.runtime.conversation", "assistant.runtime.loop"],
        ),
        managed_resource(
            "assistant.runtime.utilities",
            "utility_bundle",
            "Carries helper utilities and shared operator assets as reusable runtime resources instead of ad hoc project files.",
            "mounted_on_demand",
            "not_ambient",
            vec!["assistant.runtime", "assistant.runtime.registry"],
        ),
        managed_resource(
            "assistant.runtime.agent_tooling",
            "agent_runtime_surface",
            "Represents the end-state where the runtime provides its own orchestrator, task queue, worker, and agent-facing execution helpers.",
            "runtime_native",
            "foreground_or_explicit_mount",
            vec!["assistant.runtime", "assistant.runtime.loop", "assistant.runtime.conversation"],
        ),
        managed_resource(
            "assistant.runtime.prompts",
            "prompt_bundle",
            "Provides governed system and user prompt objects that define proof, honesty, confidence, and completion rules per runtime lane.",
            "mounted_on_demand",
            "not_ambient",
            vec!["assistant.runtime", "assistant.runtime.conversation", "assistant.runtime.loop", "assistant.runtime.host", "assistant.runtime.os"],
        ),
    ]
}

pub fn prompt_objects() -> Vec<PromptObjectSpec> {
    vec![
        prompt(
            "assistant.runtime.core.system",
            "system",
            vec!["assistant.runtime", "assistant.runtime.conversation", "assistant.runtime.loop", "assistant.runtime.host", "assistant.runtime.os"],
            "Core governed-runtime system contract",
            "Operate from durable truth. Never lie to the user. Never claim a task completed when it did not complete. Never fabricate output, proof, files, commands, observations, or state transitions. If confidence is not high enough to answer directly, say that clearly and either inspect, verify, or decline. Distinguish explicitly between what is known, what is inferred, and what is not yet proven. Prefer inspectable evidence over narrative confidence. Preserve the separation between ideation and execution: chat is transient, runtime state is durable, and execution should follow explicit routing and provenance. Do not silently broaden scope or imply background execution that did not happen.",
        ),
        prompt(
            "assistant.runtime.conversation.system",
            "system",
            vec!["assistant.runtime.conversation"],
            "Foreground orchestrator prompt",
            "You are the foreground orchestrator. Stay available. Accept intent, classify direct versus planner routing, assign durable task identity, and hand work to the task queue. Do not perform background execution in the foreground lane. Do not imply worker progress you have not observed. If the queue is full, say so plainly and do not accept more work.",
        ),
        prompt(
            "assistant.runtime.loop.system",
            "system",
            vec!["assistant.runtime.loop"],
            "Planner worker prompt",
            "You are the planner worker. Work from explicit state, ordered steps, and durable truth. Do not skip verification when proof is required. Keep outputs compact, inspectable, and tied to the active program state. If a result is incomplete, say incomplete instead of presenting a partial result as finished.",
        ),
        prompt(
            "assistant.runtime.host.system",
            "system",
            vec!["assistant.runtime.host"],
            "Host runtime prompt",
            "You are the host runtime. Report boot, posture, and host-visible runtime state exactly as observed. Do not invent readiness. If a POST or boot check did not run, say it did not run. If the host state is absent or stale, report that explicitly.",
        ),
        prompt(
            "assistant.runtime.os.system",
            "system",
            vec!["assistant.runtime.os"],
            "OS passthrough prompt",
            "You are the OS runtime. Return real host execution results only. Never summarize invented output as if it were command output. If the command failed, surface the failure directly. If execution did not happen, do not imply that it did.",
        ),
        prompt(
            "assistant.runtime.governance.user",
            "user",
            vec!["assistant.runtime", "assistant.runtime.governance"],
            "Governed runtime user guidance",
            "Prefer proof over assertion. Ask the runtime to inspect or verify when the answer depends on state, files, commands, or prior execution. Treat runtimes, skills, tools, MCPs, and utilities as explicit governed resources rather than ambient assumptions.",
        ),
    ]
}

pub fn security_policy() -> GovernedRuntimeSecurityPolicy {
    GovernedRuntimeSecurityPolicy {
        sandboxed: true,
        external_skill_mounts: "deny_all".into(),
        external_tool_mounts: "deny_all".into(),
        external_prompt_mounts: "deny_all".into(),
        external_mcp_mounts: "deny_all".into(),
        ambient_session_imports: "deny_all".into(),
        undeclared_runtime_imports: "deny_all".into(),
        dynamic_tool_creation: "deny_all".into(),
        raw_os_primitive_access: "only_via_assistant.runtime.os".into(),
        allowed_resource_roots: vec![
            "governed-runtime.json".into(),
            "external_skills/".into(),
            ".runtime/".into(),
        ],
        durable_requirements: vec![
            "only declared governed-runtime resources may be mounted".into(),
            "session context from outside the runtime is not imported ambiently".into(),
            "proof is required for stateful or executed claims".into(),
            "completions must not be claimed without durable evidence".into(),
            "worker awareness is isolated and complementary to the foreground orchestrator".into(),
        ],
    }
}

pub fn migration_plan() -> Vec<MigrationStep> {
    vec![
        MigrationStep {
            step: "1".into(),
            summary: "Freeze runtime contracts and publish runtime types before moving implementation.".into(),
            preserve_state_paths: true,
            actions: vec![
                "treat the current .runtime paths as canonical persistence".into(),
                "publish assistant_runtime, loop_runtime, conversation_runtime, governance_runtime, host_runtime, and registry_runtime manifests".into(),
                "avoid schema rewrites or file moves".into(),
            ],
        },
        MigrationStep {
            step: "2".into(),
            summary: "Route tool_os CLI, server, and registry through runtime manifests and compatibility bindings.".into(),
            preserve_state_paths: true,
            actions: vec![
                "expose runtime manifests through CLI, registry, and server".into(),
                "keep runtime reads and writes pointed at existing .runtime stores".into(),
                "prove old state still loads unchanged".into(),
            ],
        },
        MigrationStep {
            step: "3".into(),
            summary: "Stabilize conversation_runtime and loop_runtime as the first extracted runtime boundaries.".into(),
            preserve_state_paths: true,
            actions: vec![
                "keep discussion state invisible to loops except through explicit promote".into(),
                "keep program/autopilot/task state readable without migration".into(),
                "publish these two runtime types as the first external package surfaces".into(),
            ],
        },
        MigrationStep {
            step: "4".into(),
            summary: "Introduce a capability catalog so skills, tools, and utilities become runtime-managed resources.".into(),
            preserve_state_paths: true,
            actions: vec![
                "treat capabilities as discoverable runtime resources rather than ambient prompt context".into(),
                "mount only the active skill, tool, or utility slice needed by the current lane".into(),
                "keep resource metadata inspectable through the umbrella runtime".into(),
            ],
        },
        MigrationStep {
            step: "5".into(),
            summary: "Let conversation and loop lanes consume mounted resources without turning them into project baggage.".into(),
            preserve_state_paths: true,
            actions: vec![
                "keep chat foregrounded while explicit mounts happen behind the runtime boundary".into(),
                "preserve direct-path behavior when no additional resource mount is required".into(),
                "ensure mounted resources have provenance and lane ownership".into(),
            ],
        },
        MigrationStep {
            step: "6".into(),
            summary: "Reach a fully supported runtime layer that provides its own agent tooling, skills, capabilities, and utilities.".into(),
            preserve_state_paths: true,
            actions: vec![
                "promote agent tooling from repo-local conventions into runtime-native surfaces".into(),
                "treat the runtime as the source of active execution context instead of ambient chat state".into(),
                "keep versioned releases for stable public boundaries, not every local layer".into(),
            ],
        },
    ]
}

fn binding(store_id: &str, schema: &str, relative_path: &str) -> RuntimeCompatibilityBinding {
    RuntimeCompatibilityBinding {
        store_id: store_id.to_string(),
        schema: schema.to_string(),
        relative_path: relative_path.to_string(),
        mode: "read_write_compatibility".to_string(),
    }
}

fn runtime(
    id: &str,
    runtime_type: &str,
    summary: &str,
    owns: Vec<&str>,
    entrypoints: Vec<&str>,
    required_capabilities: Vec<&str>,
    compatibility_bindings: Vec<RuntimeCompatibilityBinding>,
) -> RuntimeTypeSpec {
    RuntimeTypeSpec {
        id: id.to_string(),
        runtime_type: runtime_type.to_string(),
        summary: summary.to_string(),
        owns: owns.into_iter().map(str::to_string).collect(),
        entrypoints: entrypoints.into_iter().map(str::to_string).collect(),
        required_capabilities: required_capabilities.into_iter().map(str::to_string).collect(),
        compatibility_bindings,
    }
}

fn managed_resource(
    id: &str,
    kind: &str,
    summary: &str,
    mounting_mode: &str,
    default_visibility: &str,
    owned_by: Vec<&str>,
) -> ManagedResourceSpec {
    ManagedResourceSpec {
        id: id.to_string(),
        kind: kind.to_string(),
        summary: summary.to_string(),
        mounting_mode: mounting_mode.to_string(),
        default_visibility: default_visibility.to_string(),
        owned_by: owned_by.into_iter().map(str::to_string).collect(),
    }
}

fn prompt(
    id: &str,
    prompt_role: &str,
    applies_to: Vec<&str>,
    summary: &str,
    content: &str,
) -> PromptObjectSpec {
    PromptObjectSpec {
        id: id.to_string(),
        prompt_role: prompt_role.to_string(),
        applies_to: applies_to.into_iter().map(str::to_string).collect(),
        summary: summary.to_string(),
        content: content.to_string(),
    }
}
