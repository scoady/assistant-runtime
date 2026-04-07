# Loops And Runtime Model

This file carries the deeper modeling and loop-specific detail that supports the runtime story in the main README.

## Runtime Model

In this project, the runtime is a small embedded governed runtime that runs inside the host machine runtime.

That means:

- the operating system is the outer runtime
- `assistant-runtime` is an inner runtime for agent execution
- chat is not the runtime; it is only one input surface
- the runtime owns durable state, routing, policy, and declared resources

Technically, it is:

- a native CLI process
- a local state model under `.runtime/`
- a governed contract in `governed-runtime.json`
- routing logic for orchestrator, queue, and worker behavior
- proof/reporting surfaces that explain what happened

Without a runtime boundary, exploratory conversation, rough ideas, and execution state all blur together. That makes it harder to tell what is durable versus provisional.

This runtime model keeps those lanes separate:

- ideation can stay lightweight without automatically mutating execution state
- loop execution can read and write durable local state under `.runtime/`
- host and OS actions become explicit, inspectable runtime surfaces
- packaged runtime types make it easier to expose only the lane you actually want
- the foreground orchestrator only accepts and routes; background workers own direct and planner execution below the chat lane

## Agentic Runtime Model

The intended agentic vocabulary is:

- `User`
- `Orchestrator`
- `Task Queue`
- `Direct Worker`
- `Planner Worker`
- `Runtime Resources`

The orchestrator stays in the foreground and never executes user work directly. It accepts intent, assigns a durable task id, and routes work into the task queue. The workers consume that queue below the foreground lane with isolated, complementary awareness.

## Implemented Runtime Surfaces

This repository currently implements:

- `assistant.runtime`
  - umbrella entrypoint for runtime inspection and program loop workflows
- `assistant.runtime.loop`
  - truth-ladder planning and bounded execution loops backed by `.runtime/programs/programs.json`
- `assistant.runtime.host`
  - runtime boot planning, host posture, and POST checks backed by `.runtime/runtime/` and `.runtime/desktop/`
- `assistant.runtime.os`
  - direct OS passthrough for provable host command execution

These runtime types are declared in the manifest but intentionally not implemented in this build:

- `assistant.runtime.conversation`
- `assistant.runtime.governance`
- `assistant.runtime.registry`

The runtime also declares a resource model for:

- capabilities
- skills
- tools
- MCPs
- prompt objects
- utilities
- agent-facing runtime tooling

Those are treated as runtime-managed resources. The contract is that they should be cataloged and mounted on demand instead of becoming ambient project context by default.

## Governed Runtime Contract

This repo publishes a governed-runtime contract at [governed-runtime.json](/home/scoady/git/loops/assistant-runtime/governed-runtime.json).

It is deliberately one layer above the raw agent surfaces. The runtime owns:

- declared runtimes
- declarative skill mounts
- declarative tool mounts
- declarative MCP mounts
- capabilities and utilities
- queue-backed worker execution

Inspect it from the CLI:

```bash
assistant-runtime runtime governed-runtime
assistant-runtime runtime proof-metrics
assistant-runtime runtime benchmark
assistant-runtime runtime list-resources --class mcp
assistant-runtime runtime list-prompts
assistant-runtime runtime show-prompt assistant.runtime.core.system
```

## Proof And Benchmarking

`runtime proof-metrics` is the stable proof surface for the governed runtime model. It reports:

- visible vs relevant tokens
- irrelevant token exposure
- truth delivery ratio
- context amplification
- drift pressure per turn
- stable reference rate
- resume boundary rate

The comparison is explicit:

- `governed`: isolated lane context, queue-backed handoff, stable references
- `stock`: shared context accumulation across the whole loop

Use the repo-local benchmark script to run the governed and stock routes sequentially against the same 20-turn scenario:

```bash
./assistant-runtime-benchmark
```

What it does:

- runs the same 20-turn query set for both routes
- runs them sequentially so one route uses local CPU at a time
- emits comparable metrics for drift and stable truth delivery

Key metrics:

- `visible_tokens`: total tokens exposed to the route
- `relevant_tokens`: tokens that directly support the active truth delivery path
- `irrelevant_tokens`: extra token exposure that can contribute to drift
- `truth_delivery_ratio`: relevant / visible tokens
- `context_amplification`: visible / unique truth tokens
- `drift_pressure_per_turn`: irrelevant token exposure per turn
- `stable_reference_rate`: fraction of turns with durable task references
- `resume_boundary_rate`: fraction of turns that preserve a resumable boundary

The prompt objects are the right place for the non-negotiable behavior contract:

- proof is required when the answer depends on execution or state
- never lie to the user
- never claim completion when completion did not happen
- never fabricate outputs, files, commands, or observations
- never answer beyond confidence; inspect, verify, or say not proven
- preserve the boundary between chat ideation and durable execution

## Sandbox Direction

The governed runtime is intended to be sandboxed by contract.

Security direction:

- external skills: denied
- external tools: denied
- external prompts: denied
- external MCPs: denied
- ambient session imports: denied
- undeclared runtime imports: denied
- dynamic tool creation over OS primitives: denied
- raw OS access: only through `assistant.runtime.os`

This is the core trust model: the runtime should know exactly which governed resources are mounted, and nothing outside that declaration should become active execution context.

## Planning Status

The default planning status format is:

```text
[planning_decision: no] [execution_path: direct]
```

or:

```text
[planning_decision: yes] [execution_path: planner]
```

This keeps the outcome explicit:

- `planning_decision` says whether durable planning was triggered
- `execution_path` says where the task was routed

You can render it directly:

```bash
assistant-runtime planning status
assistant-runtime planning status --multi-step
assistant-runtime planning status --mutates-real-state --has-dependencies
```

## Durable Truth

The runtime can print its own durable-truth contract:

```bash
assistant-runtime runtime durable-truth
assistant-runtime runtime implementation-plan
assistant-runtime runtime managed-resources
assistant-runtime runtime list-resources
assistant-runtime runtime list-resources --class skill
assistant-runtime runtime show-resource assistant.runtime.skills
assistant-runtime runtime provenance
```

This keeps the boundary model executable and inspectable:

- what must remain true after implementation
- which parts of the implementation plan are already done
- which parts are still explicitly deferred
- which resource types should eventually be mounted by the runtime instead of carried ambiently

The resource substrate now persists under `.runtime/resources/`:

- `catalog.json`
- `mounts.json`
- `provenance.json`

Mounting is not implemented yet, but the durable state and inspection surfaces are real.

## Chat Handoff Model

Foreground chat does not execute direct work itself.

Current flow:

```text
Chat -> assistant.runtime.conversation -> assistant.runtime.task_queue -> worker -> normal flow
```

Durable state:

- `.runtime/chat/state.json`
- `.runtime/queue-lane/tasks.json`
- `.runtime/workers/direct/tasks.json`
- `.runtime/workers/planner/tasks.json`

The task-queue id is the reference id for later inspection.

## Path Examples

Direct path:

![Direct path](docs/images/direct-path.png)

This is the cheap path. No background task is created, and the turn remains direct:

```text
[planning_decision: no] [execution_path: direct]
```

Chat lane planner path:

![Chat lane planner path](docs/images/chat-lane-planner-path.png)

This is the foreground orchestrator path. The chat interface stays available while planner-routed work is accepted and enqueued in the background:

```text
[planning_decision: yes] [execution_path: planner]
```
