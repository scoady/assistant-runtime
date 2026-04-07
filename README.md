# assistant-runtime

`assistant-runtime` is a small native runtime package for local assistant execution boundaries.

Its job is simple: keep ideation separate from execution, keep execution state durable, and make the active runtime lane explicit when work is actually happening.

## Observed Outcomes

Observed in limited testing:

- materially less conversational drift over longer sessions
- fewer technical corrections needed from the user
- clearer separation between ideation and execution
- more stable references to active work through task ids and queue boundaries
- stronger proof posture because runtime-backed claims are expected to include repeatable commands, scripts, or artifact paths

## Why a Runtime

The high-level idea is simple:

- keep ideation separate from execution
- keep execution state durable under `.runtime/`
- route real work through explicit runtime lanes instead of letting it blur into chat state
- make proof inspectable when the answer depends on state, files, commands, or prior execution

If you want the deeper model, loop semantics, queue handoff layout, planning status, durable truth surfaces, and proof metrics, see [LOOPS.md](LOOPS.md).

## Build

```bash
cargo build
```

This repository also includes a demo binary:

```bash
cargo run --bin assistant-runtime-demo -- help
cargo run --bin assistant-runtime-showcase -- help
```

## Package

```bash
cargo run -- runtime package
```

That produces a distributable runtime bundle under `dist/assistant-runtime/`.

## Proof

This repo publishes a governed runtime contract at [governed-runtime.json](governed-runtime.json), plus proof and benchmark surfaces in the CLI.

Useful commands:

```bash
assistant-runtime runtime governed-runtime
assistant-runtime runtime proof-metrics
assistant-runtime runtime benchmark
assistant-runtime runtime durable-truth
```

The detailed proof model, benchmark metrics, and loop-specific behavior are in [LOOPS.md](LOOPS.md).

## Install Into a Target Repository

Preferred install flow for an agent:

```bash
./scripts/install-and-activate-runtime.sh /absolute/path/to/target-repo
```

If you run it from inside the target repo, the path is optional:

```bash
cd /absolute/path/to/target-repo
/absolute/path/to/assistant-runtime/scripts/install-and-activate-runtime.sh
source ./.assistant-runtime/activate.sh
assistant-runtime runtime manifest
assistant-conversation-runtime status
```

What the script does:

- builds the local `assistant-runtime` binary
- packages the runtime bundle into a temporary directory
- installs the bundle into `<target-repo>/.assistant-runtime/`
- writes `<target-repo>/.assistant-runtime/activate.sh`
- prints the exact activation command for the current shell

Manual package/install flow is still available:

```bash
./dist/assistant-runtime/install.sh /absolute/path/to/target-repo
```

Then run from the target repo root:

```bash
./.assistant-runtime/bin/assistant-runtime runtime manifest
./.assistant-runtime/bin/assistant-runtime program broad-plan
./.assistant-runtime/bin/assistant-os-runtime ls
```

## Demo Application

`assistant-runtime-demo` is a small example CLI that installs the current runtime into a disposable demo repo and exercises the runtime surfaces.

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

`assistant-runtime-showcase` is a fake but high-fidelity version of this chat experience.

It opens a long-lived terminal process and renders:

- a 20-turn scripted user scenario
- explicit `[planning_decision: x] [execution_path: y]` routing
- the always-open foreground orchestrator lane
- task queue ids and queue pressure
- separate direct and planner worker lanes
- mounted skill/tool resource state

Run it after one build:

```bash
cargo run --bin assistant-runtime-showcase -- summary
cargo run --bin assistant-runtime-showcase -- play
cargo run --bin assistant-runtime-showcase -- play --auto --delay-ms 80
```

## When To Use Which Runtime

See [RUNTIMES.md](RUNTIMES.md) for the per-runtime guide.

## Warnings

- `assistant.runtime.os` executes real host commands. Treat it like direct shell access.
- `assistant.runtime.loop` mutates durable local state under `.runtime/`.
- `assistant.runtime.host` writes runtime posture files used by the installed runtime.
- conversation, governance, and registry are not implemented here yet. Their wrappers should not be presented as working features.

For the detailed loop model and examples, see [LOOPS.md](LOOPS.md).

## License

This repository is licensed under the GNU General Public License v3.0 only.

See [LICENSE](LICENSE).
