# Runtime Governance

Use this skill when work should be shaped by the governed runtime instead of by ambient chat context.

Principles:
- treat the foreground assistant as the orchestrator
- route accepted work into the task queue first
- keep direct and planner workers isolated from the full chat transcript
- mount only the skills, tools, MCPs, and utilities needed by the active runtime lane
- record provenance for mounts and worker handoffs

When to use:
- designing or extending runtime-owned skills and tools
- deciding whether a capability should be ambient or explicitly mounted
- defining queue, worker, or orchestrator behavior
- translating agent-facing features into governed runtime contracts
