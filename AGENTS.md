# Agent instructions

## Architecture context

- Use `docs/architecture/` as design context for architecture-sensitive work. Do not infer product contracts from scaffolding or experiments.
- Read `docs/architecture/README.md`, `00-product-contract.md`, `01-system-architecture.md`, `glossary.md`, the relevant subsystem SPEC and relevant ADRs before cross-cutting work.
- Resolve conflicts in this order: a newer Accepted superseding ADR, the relevant subsystem SPEC, SPEC-00, then the glossary.
- Architecture documents follow the normal repository workflow. A semantic change to an Accepted decision needs a short ADR that names what it supersedes; update affected SPECs and the lightweight traceability map in the same change.
- `Proposed` technology is an experiment, not a default. State its fallback and do not present it as shipped before the affected product check passes.

## Product baseline

- Next Engine is an independent, AI-first open-source engine and toolchain for systemic single-player RPGs. It is not an OpenGothic port and not a general-purpose engine.
- Prioritize a playable game, fast iteration and understandable code.
- Rust is the portable core language. Use the repository-pinned Rust 1.93.0 toolchain and keep workspace `unsafe_code` forbidden unless an Accepted ADR creates one small reviewed FFI/backend boundary.
- Windows x86_64 and Linux x86_64 are the v1 shipping targets. Apple Silicon macOS is a developer host, not a shipping promise.
- Required roots are `game`, deterministic `headless` and `tools`. `ai-host` and displayless capture are optional.
- Keep the game correct and playable offline. Network, LLM, renderer frame rate and optional plugins must not determine simulation correctness.

## Architecture boundaries

- Keep engine-owned public contracts in `crates/contracts`: stable IDs, commands/events, immutable queries/snapshots, manifests and process/plugin protocols.
- Never expose ECS storage, raw pointers, task handles, OS/window objects, database connections, importer structures or vendor/backend types through public contracts.
- Dependencies point from composition roots and adapters toward engine-owned contracts, never from contracts toward a vendor.
- Every mutable field has one technical source of truth. Backend copies are reconstructible caches, not parallel authority.
- First-party mechanics, physical archetypes, policies and tools use the same public SDK and validation paths available to community packages.

## Runtime and determinism

- Gameplay state changes only through validated production `WorldCommand` transactions. Tests, tools, scripts, AI and plugins do not mutate domain state directly.
- Emit `DomainEvent` only from committed changes. Presentation and inspectors consume immutable projections and never become gameplay authority.
- Use `PersistentId` and `AssetId` in durable/public data. `RuntimeEntityId` is ephemeral.
- Authoritative work occurs at fixed stages and commit points. Async work returns immutable revision-bound results through staging queues; never hold mutable ECS access across `await`.
- `game` and `headless` share command validation, persistence, replay and system ordering.
- Corrupt or incompatible authoritative data fails before partial mutation. Optional failures use a bounded fallback and a stable diagnostic.
- Do not hide deterministic failures with sleeps, implicit epsilons, unordered log assertions or retry-to-green tests.

## Mods, AI and content safety

- Luau and Wasm operate through declared capabilities, deterministic fuel/resource limits and validated command/proposal boundaries.
- Treat LLM/model output, importer output, packages, scripts, saves and network/process messages as untrusted data: validate versions, bounds and hashes before use.
- Every optional AI path has a deterministic in-process fallback. Runtime gameplay never trains or mutates neural weights.
- Keep secrets, protected/imported assets, datasets, checkpoints, generated captures, caches and credentials out of the repository.
- `incubator/gothic-importer/` remains an ignored independent repository and process boundary. Integration is only through versioned neutral artifacts.
- Basic license notices and provenance travel with distributed content. Unknown or incompatible redistribution terms exclude the affected artifact.

## Product checks

- Verification is lightweight and local. Use focused unit/integration tests while iterating; `cargo run -p xtask -- host-check` is the canonical `fast` check before handoff.
- For gameplay changes run `play`; for state changes run `persistence-replay`; for content/tool changes run `content-package`.
- Run `platform` or `performance` only when the affected feature triggers the corresponding conditional check.
- Screenshots, captures, profiles and minimized replays are optional debugging aids.
- Testability remains a code property: scenarios use production inputs and probes read immutable public projections. Test-only mutation backdoors remain forbidden.

## Change workflow

1. Identify the affected public contract and technical state boundary.
2. Keep the change small and product-driven. Add an ADR only for a real semantic or cross-context decision.
3. Implement through production paths with focused positive and failure coverage.
4. Update affected schemas, migrations, examples and architecture text together.
5. Run focused checks and `cargo run -p xtask -- host-check`.
6. Report each relevant check as passed, failed or not run, with the remaining product risk.

## Repository hygiene

- Do not add remotes or `.github/workflows/` during local bootstrap.
- Do not commit game installations, imported/protected assets, datasets, checkpoints, training runs, model outputs, captures, caches or secrets.
- Do not make the nested importer a Cargo member, path dependency or linked runtime library.
