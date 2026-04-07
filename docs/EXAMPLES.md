# Demo And Showcase

## Demo Application

`assistant-runtime-demo` installs the current runtime into a disposable demo repo and exercises the runtime surfaces.

It shows:

- runtime manifest and managed resources
- foreground orchestrator status
- task queue ids
- direct worker vs planner worker handoff state
- occasional long-running tasks that appear over time

Example:

```bash
cargo run --bin assistant-runtime-demo -- init
cargo run --bin assistant-runtime-demo -- status
cargo run --bin assistant-runtime-demo -- run --ticks 8 --sleep-ms 100
```

## Interactive Showcase

`assistant-runtime-showcase` is a fake but high-fidelity version of the chat/runtime experience.

It renders:

- a 20-turn scripted user scenario
- explicit routing state
- an always-open foreground orchestrator lane
- task queue ids and queue pressure
- separate direct and planner worker lanes
- mounted skill/tool resource state

Example:

```bash
cargo run --bin assistant-runtime-showcase -- summary
cargo run --bin assistant-runtime-showcase -- play
cargo run --bin assistant-runtime-showcase -- play --auto --delay-ms 80
```
