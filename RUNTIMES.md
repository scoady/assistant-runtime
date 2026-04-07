# Runtime Guide

## assistant.runtime

Purpose:
The umbrella runtime. Use this when you want one stable entrypoint for runtime inspection, packaging, and loop-oriented execution.

Use when:
- you want to inspect the runtime manifest
- you want to boot or package the runtime
- you want to run the program loop without thinking about a narrower wrapper

Primary commands:

```bash
assistant-runtime runtime manifest
assistant-runtime runtime governed-runtime
assistant-runtime runtime list-types
assistant-runtime runtime managed-resources
assistant-runtime runtime list-prompts
assistant-runtime runtime show-prompt assistant.runtime.core.system
assistant-runtime runtime list-resources --class skill
assistant-runtime runtime list-resources --class mcp
assistant-runtime runtime show-resource assistant.runtime.skills
assistant-runtime runtime provenance
assistant-runtime runtime boot
assistant-runtime program broad-plan
assistant-runtime chat status
```

The umbrella runtime can keep the foreground orchestrator ready while direct or planner-routed work is accepted into background queue state.
It is also the inspection surface for runtime-managed capabilities, skills, tools, utilities, and agent-facing runtime tooling.

## assistant.runtime.loop

Purpose:
A bounded execution loop over durable program state.

Use when:
- you want a fresh broad plan from the current truth ladder
- you want to advance one rung at a time
- you want a bounded while-loop instead of open-ended execution

Primary commands:

```bash
assistant-runtime program broad-plan
assistant-runtime program loop <program-id>
assistant-runtime program while-loop 3 <program-id>
assistant-loop-runtime broad-plan
```

Warning:
This runtime mutates `.runtime/programs/programs.json`.

## assistant.runtime.host

Purpose:
Host posture, boot planning, and runtime state.

Use when:
- you need to inspect the active runtime image
- you want to boot a desktop or server profile
- you want a POST-style validation of the installed runtime state

Primary commands:

```bash
assistant-runtime runtime boot --dry-run
assistant-runtime runtime boot
assistant-runtime runtime status
assistant-runtime runtime post
assistant-host-runtime status
```

Warning:
This runtime writes `.runtime/runtime/state.json` and may also write `.runtime/desktop/state.json`.

## assistant.runtime.conversation

Purpose:
Foreground orchestrator lane that stays ready while accepted work is routed into the task queue below it.

Use when:
- you want the chat interface to remain available
- you want to accept a task without blocking on execution in the same foreground lane
- you want the orchestrator to hand off direct and planner paths below the chat lane
- you want to inspect the task queue ids and worker handoffs

Primary commands:

```bash
assistant-runtime chat status
assistant-runtime chat accept "answer a direct question"
assistant-runtime chat accept "implement a multi-step change" --plan
assistant-runtime chat queue
assistant-conversation-runtime status
```

Warning:
This runtime keeps the foreground orchestrator ready, but accepted work mutates task-queue and worker handoff state under `.runtime/queue-lane/` and `.runtime/workers/`.

## assistant.runtime.os

Purpose:
Direct, provable operating system execution on the user's machine.

Use when:
- you need a real OS command result instead of a synthesized answer
- you want byte-for-byte passthrough of stdout and stderr from the host process
- you want an explicit OS lane instead of silently mixing shell work into the umbrella runtime

Primary commands:

```bash
assistant-runtime '\ls'
assistant-runtime run os_runtime pwd
assistant-os-runtime ls -ltr
```

Warning:
This is real command execution. It is not simulated, summarized, or sandboxed by the runtime itself.

## Declared But Not Implemented

These runtime types are part of the manifest contract but are not implemented in this repository today:

- `assistant.runtime.governance`
- `assistant.runtime.registry`

Treat them as reserved contract surfaces, not working user commands in this release.

## Resource Direction

Durable direction from here:

- capabilities become the catalog of what the runtime can mount
- skills become runtime-mounted procedures instead of per-project baggage
- tools become explicit runtime-owned surfaces instead of ambient assumptions
- MCPs become explicit runtime-owned connectors instead of ambient session setup
- utilities become reusable runtime bundles rather than ad hoc repo files
- agent tooling moves into the runtime layer so active execution context comes from mounts and state, not prompt spillover

Current substrate:

- `.runtime/resources/catalog.json` seeds the known runtime resources
- `.runtime/resources/mounts.json` is the durable mount store for future skill/tool activation
- `.runtime/resources/provenance.json` is the durable provenance ledger for future mount and release events

## Runtime Topology

Canonical agentic vocabulary for this runtime:

- `User` sends intent into the foreground lane
- `Orchestrator` accepts and routes without executing the work itself
- `Task Queue` assigns durable task ids and buffers accepted work
- `Direct Worker` handles bounded execution
- `Planner Worker` handles multi-step execution
- `Runtime Resources` are mounted explicitly for the lane that needs them

Target flow:

```text
User -> Orchestrator -> Task Queue -> Direct Worker
User -> Orchestrator -> Task Queue -> Planner Worker
```

## Prompt Objects

The governed runtime also carries prompt objects as explicit runtime assets.

Use them when:
- you need a stable system contract for a runtime lane
- you want proof, honesty, confidence, and completion rules to be inspectable
- you do not want those rules to depend on ambient chat memory

Core prompt surfaces:
- `assistant.runtime.core.system`
- `assistant.runtime.conversation.system`
- `assistant.runtime.loop.system`
- `assistant.runtime.host.system`
- `assistant.runtime.os.system`
- `assistant.runtime.governance.user`

## Sandbox Contract

The governed runtime should be treated as sandboxed.

Required contract:
- no external skills
- no external tools
- no external prompts
- no external MCP mounts
- no ambient session imports
- no undeclared runtime imports
- no dynamic tool creation over OS primitives
- raw OS access only through `assistant.runtime.os`

This gives the operator a bounded trust surface: only governed, declared runtime assets are available for execution.
