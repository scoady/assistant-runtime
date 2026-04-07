# Install And Activate

## Preferred Flow

Use the bootstrap script:

```bash
./scripts/install-and-activate-runtime.sh /absolute/path/to/target-repo
```

If you are already in the target repo:

```bash
/absolute/path/to/assistant-runtime/scripts/install-and-activate-runtime.sh
source ./.assistant-runtime/activate.sh
```

What it does:

- builds the local `assistant-runtime` binary
- packages the runtime bundle into a temporary directory
- installs it into `<target-repo>/.assistant-runtime/`
- writes `.assistant-runtime/activate.sh`

## Manual Flow

If you already have a packaged bundle:

```bash
./dist/assistant-runtime/install.sh /absolute/path/to/target-repo
```

Then from the target repo root:

```bash
./.assistant-runtime/bin/assistant-runtime runtime manifest
./.assistant-runtime/bin/assistant-runtime program broad-plan
./.assistant-runtime/bin/assistant-os-runtime ls
```

## Activation Notes

Run runtime commands from the target repository root so the runtime uses that repo's `.runtime/` state.
