# Proof And Benchmarking

This repo publishes a governed runtime contract at [governed-runtime.json](/home/scoady/git/loops/assistant-runtime/governed-runtime.json), plus CLI proof and benchmark surfaces.

Useful commands:

```bash
assistant-runtime runtime governed-runtime
assistant-runtime runtime proof-metrics
assistant-runtime runtime benchmark
assistant-runtime runtime durable-truth
assistant-runtime runtime implementation-plan
```

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

## Benchmark Script

Use the repo-local benchmark script to run governed and stock routes against the same 20-turn scenario:

```bash
./assistant-runtime-benchmark
```

## Proof Artifacts

To regenerate the proof-oriented showcase artifacts:

```bash
./scripts/generate-proof-artifacts.sh
./scripts/capture-proof-screenshots.sh
```
