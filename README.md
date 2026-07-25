# NextEngine

NextEngine is the local bootstrap repository for an independent, AI-first open-source RPG engine. The product name is temporary. The normative architecture baseline is [docs/architecture/README.md](docs/architecture/README.md).

This repository is the local source of truth after the bootstrap migration. It contains no OpenGothic runtime code, Gothic assets, CI workflow, or remote. Windows and Linux remain the v1 shipping targets; Apple Silicon macOS is currently a developer-host tier for portable Rust code, tooling, and bounded ML smoke tests.

## Local verification

The pinned toolchain is Rust 1.93.0. Run:

```text
cargo run -p xtask -- boundary-scan
cargo run -p xtask -- host-check
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

`host-check` is the canonical `fast` check and runs formatting, clippy,
workspace tests, and boundary checks. Affected gameplay, state, and content
changes additionally use `play`, `persistence-replay`, and `content-package`;
`platform` and `performance` are conditional. The lab smoke validates train →
ONNX → inference on MPS or CPU and does not replace those product checks; see
[docs/development/training-capability.md](docs/development/training-capability.md).

Architecture documents are edited through the normal repository workflow.
Semantic changes to Accepted decisions use a short superseding ADR and update
the affected SPECs and lightweight traceability map. See
[ADR-030](docs/architecture/adr/030-product-first-development-and-lightweight-validation.md).

## Workspace

```text
crates/contracts      engine-owned commands, events, manifests, IDs, and snapshots
crates/rpg            generic RPG aggregate owner and atomic domain transitions
crates/runtime        deterministic command admission and staged tick transaction
crates/assets         recoverable save generations and owner-segment persistence
crates/verification   state roots and deterministic headless replay
apps/headless         portable headless composition root
tools/xtask           local verification commands
lab/                  isolated training smoke lane (added separately)
```

The current generic RPG slice is intentionally bounded to engine-owned
aggregate snapshots and validated commands for dialogue/quest/relationship,
item transfer, skill proficiency and interactive-object state. It is a
bootstrap implementation, not a finished game.

The Gothic importer may be developed locally under ignored `incubator/gothic-importer/`, but it is a nested independent Git repository and never a Cargo member or path dependency. Integration is process/artifact-only through the Neutral Import Model and provenance contracts.

## Status and licensing

The engine bootstrap is Apache-2.0. Migrated architecture documents retain
their recorded MIT origin; see [MIGRATION_PROVENANCE.md](MIGRATION_PROVENANCE.md)
and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). No game data, imported
output, datasets, checkpoints, generated models, caches or secrets belong in
Git.
