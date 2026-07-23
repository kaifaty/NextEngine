# NextEngine

NextEngine is the local bootstrap repository for an independent, AI-first open-source RPG engine. The product name is temporary. The normative architecture baseline is [docs/architecture/README.md](docs/architecture/README.md).

This repository is the local source of truth after the migration gate. It contains no OpenGothic runtime code, Gothic assets, CI workflow, or remote. Windows and Linux remain the v1 shipping targets; Apple Silicon macOS is currently a developer-host tier for portable Rust code, tooling, and bounded ML smoke tests.

## Local verification

The pinned toolchain is Rust 1.93.0. Run:

```text
cargo run -p xtask -- docs-check
cargo run -p xtask -- boundary-scan
cargo run -p xtask -- host-check
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

`host-check` runs formatting, clippy, workspace tests, documentation validation, and boundary checks. The lab smoke validates train → ONNX → inference on MPS or CPU. Neither command claims Windows/Linux package conformance or physical-policy certification; see [docs/development/training-capability.md](docs/development/training-capability.md).

An architecture promotion first runs `cargo run -p xtask -- architecture-review-preflight <target>` while its hash-bound review record is `Pending`. After the Repository Owner approves the exact candidate root, `docs-check` and `host-check` perform final admission. The preflight command never creates human approval.

## Workspace

```text
crates/contracts      engine-owned commands, events, manifests, IDs, and snapshots
crates/rpg            generic RPG aggregate owner and atomic domain transitions
crates/runtime        deterministic command admission and staged tick transaction
crates/assets         recoverable save generations and owner-segment persistence
crates/verification   state roots and deterministic headless replay
apps/headless         portable headless composition root
tools/xtask           local admission commands
lab/                  isolated training smoke lane (added separately)
```

The current generic RPG slice is intentionally bounded to engine-owned aggregate snapshots and
validated commands for dialogue/quest/relationship, item transfer, skill proficiency, and
interactive-object state. It does not admit a Luau/Wasm host, claim `RPG-01` or `vertical-v1`
conformance, or change any Proposed architecture packet.

The Gothic importer may be developed locally under ignored `incubator/gothic-importer/`, but it is a nested independent Git repository and never a Cargo member or path dependency. Integration is process/artifact-only through the Neutral Import Model and provenance contracts.

## Status and licensing

The engine bootstrap is Apache-2.0. Migrated architecture documents retain their recorded MIT origin; see [MIGRATION_PROVENANCE.md](MIGRATION_PROVENANCE.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Public release is blocked until naming and license/provenance review. No game data, imported output, datasets, checkpoints, or evidence media belong in Git.
