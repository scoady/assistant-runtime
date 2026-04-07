# Build And Package

## Build

Use Cargo directly:

```bash
cargo build
```

To build the release binary:

```bash
cargo build --release --bin assistant-runtime
```

## Package

Package the runtime bundle into `dist/assistant-runtime/`:

```bash
cargo run -- runtime package
```

Or package from the release binary:

```bash
target/release/assistant-runtime runtime package --output dist/assistant-runtime
```

That bundle includes:

- the `assistant-runtime` binary
- runtime wrapper scripts under `bin/`
- `assistant-runtime-manifest.json`
- `governed-runtime.json`
- `install.sh`

## Validation

Run the test suite before handoff or release:

```bash
cargo test
```
