# assistant-runtime

![Runtime lanes explicit](docs/images/runtime-lanes-badge.svg)

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

## Docs

- build and packaging: [docs/BUILD.md](docs/BUILD.md)
- install and activation: [docs/INSTALL.md](docs/INSTALL.md)
- proof and benchmarks: [docs/PROOF.md](docs/PROOF.md)
- demos and showcase: [docs/EXAMPLES.md](docs/EXAMPLES.md)
- runtime model and loop behavior: [docs/LOOPS.md](docs/LOOPS.md)
- per-runtime guide: [RUNTIMES.md](RUNTIMES.md)

## Proof

This repo publishes a governed runtime contract at [governed-runtime.json](governed-runtime.json), plus proof and benchmark surfaces in the CLI.

Useful commands:

```bash
assistant-runtime runtime governed-runtime
assistant-runtime runtime proof-metrics
assistant-runtime runtime benchmark
assistant-runtime runtime durable-truth
```

The detailed proof model, benchmark metrics, and loop behavior are in [docs/PROOF.md](docs/PROOF.md) and [docs/LOOPS.md](docs/LOOPS.md).

## Warnings

- `assistant.runtime.os` executes real host commands. Treat it like direct shell access.
- `assistant.runtime.loop` mutates durable local state under `.runtime/`.
- `assistant.runtime.host` writes runtime posture files used by the installed runtime.
- conversation, governance, and registry are not implemented here yet. Their wrappers should not be presented as working features.

## License

This repository is licensed under the GNU General Public License v3.0 only.

See [LICENSE](LICENSE).
