# Agent instructions

## Admission and architectural authority

- Treat `docs/architecture/` as the normative source of truth. Do not infer product or subsystem contracts from scaffolding code, experiments, issue text, or agent prompts.
- This repository is not admitted for implementation while the accepted architecture packet is absent. Migrate the complete packet into `docs/architecture/` before adding runtime code.
- Architecture migration must be mechanical: preserve document IDs, ADR history, relative links, and recorded provenance. Do not change architectural decisions in the migration commit.
- Read `docs/architecture/README.md`, `00-product-contract.md`, `01-system-architecture.md`, and `glossary.md` before cross-cutting work. Then read the owning subsystem SPEC, relevant ADRs, `12-vertical-slice-conformance.md`, and the matching traceability rows.
- Resolve conflicts in this order: a newer Accepted superseding ADR, then SPEC-12 release gates, the owning subsystem SPEC, SPEC-00, and finally the glossary.
- Do not silently reinterpret an Accepted decision. A semantic change requires a new ADR with explicit supersession plus synchronized RFC, evidence-register, and traceability updates.
- `Proposed` technologies are hypotheses, not defaults. Preserve their declared measurable gate and fallback; do not describe them as Accepted or conformant before evidence passes.

## Product and platform baseline

- Next Engine is an independent, AI-first open-source engine and toolchain for systemic single-player RPGs. It is not an OpenGothic port or a general-purpose engine.
- Rust is the portable core language. Use the repository-pinned Rust 1.93.0 toolchain and keep workspace `unsafe_code` forbidden unless a future Accepted decision explicitly creates a reviewed boundary. Any future exception must be an ADR-named backend/FFI crate in `[workspace.metadata.nextengine.ffi].allowed_crates` and pass `FFI-01`; the initial allowlist is empty.
- Windows x86_64 and Linux x86_64 are the v1 shipping targets. Apple Silicon macOS is a developer-host tier for portable core/tooling and bounded training smoke only; never present it as game, renderer, packaging, or shipping conformance.
- Required v1 composition roots are `game`, deterministic `headless`, `tools`, and displayless `capture-worker`. `ai-host` is optional and isolated. A full editor, multiplayer, consoles, mobile, and macOS shipping support are outside v1.
- Keep the game correct and playable offline. Network, LLM, `ai-host`, renderer frame rate, and optional plugins must not determine authoritative simulation correctness.

## Architecture and dependency boundaries

- Keep versioned engine-owned public contracts in `crates/contracts`: nominal IDs, `WorldCommand`, `DomainEvent`, immutable queries/snapshots, manifests, and process/plugin protocols.
- Never expose ECS storage/components, raw pointers, task handles, OS/window objects, database connections, importer structures, or vendor/backend types through public contracts.
- Dependencies point from composition roots and backend adapters toward engine-owned contracts, never from contracts toward a vendor. Put replaceable technology behind an engine-owned API.
- Preserve bounded-context ownership. Core runtime, RPG, presentation, physical embodiment, agent intelligence, gameplay extensibility, asset/tooling, verification/evidence, and the external importer must not share mutable objects or a common mutable database.
- Every mutable field has exactly one authoritative owner. A backend-side mutable copy is only a reconstructible cache, never a second source of truth.
- First-party mechanics, physical archetypes, policies, scenarios, and tools use the same public SDK and validation paths available to community packages. Do not add hidden first-party APIs.

## Runtime, state, and determinism

- Gameplay/RPG state changes only through validated production `WorldCommand` transactions. Scripts, AI, plugins, tests, tools, and inspectors must not mutate ECS or domain state directly.
- Emit `DomainEvent` from committed changes and expose presentation/inspection through immutable projections. Presentation state is never gameplay authority.
- Use `PersistentId` and `AssetId` in durable/public data. `RuntimeEntityId` is ephemeral and must not appear in saves, replays, scripts, WIT, or package contracts.
- Authoritative work occurs at declared fixed simulation stages and deterministic commit points. Async I/O, model/shader compilation, AI IPC, and decompression return through staging queues; never hold mutable ECS access across `await` or frame boundaries.
- `game`, `headless`, and `capture-worker` must share command validation, schema registry, persistence, replay, and system ordering. Compile-time feature differences must not alter domain semantics.
- A deterministic mismatch is a failure (`NONDETERMINISTIC_RESULT`), not a flaky retry. Do not add wall-clock sleeps, implicit epsilons, unordered text-log assertions, or retry-to-green behavior to normative tests.
- Corrupt or incompatible authoritative data fails closed before partial mutation. Optional service/backend failures use a declared bounded fallback and a stable structured diagnostic.

## Scripting, plugins, AI, and physical policies

- Luau gameplay and Wasm plugins operate only through declared capabilities and validated command/proposal boundaries. Denial, timeout, or overrun must not partially commit state or crash the host simulation.
- Treat `AgentIntent`, LLM output, mechanic proposals, and model output as untrusted proposals. Deterministic validators and safety layers retain final authority.
- Every AI role needs a deterministic in-process fallback. `ai-host` crash, timeout, absence, or protocol mismatch may reduce quality, but must not block ticks or change mandatory outcomes.
- Runtime gameplay never trains or mutates neural weights. Policies and model assets are immutable, content-addressed, provenance-bound inputs selected through deterministic compatibility and supervisor rules.
- Never claim `PhysicalCertified` until `TRAIN-RTX-01` and all required POLICY/PHYS/TRAIN gates pass. Mac MPS/CPU smoke proves only the bounded train/export/inference toolchain. Use `PrototypeFallback` or `AwaitingCapability` honestly.

## Assets, importer, security, and repository hygiene

- Never commit game installations, protected/imported data, generated imported output, datasets, checkpoints, training runs, model outputs, evidence/captures, caches, credentials, signing material, or secrets.
- `incubator/gothic-importer/` is an ignored, independent nested Git repository and process boundary. It must never become a parent-repository tracked path, Cargo workspace member, path dependency, linked runtime library, or distributed engine component by accident.
- Importer integration is process/artifact-only through versioned `NeutralImportModel` plus provenance. Runtime, SDK, and packages must not contain Gothic/Daedalus/legacy parser or VM types.
- Treat importers, parsers, scripts, plugins, AI IPC, models, packages, saves, and evidence media as untrusted input. Validate schema/version, bounds, hashes, capabilities, provenance, licenses, and resource budgets before mutation or publication.
- Do not add a remote or `.github/workflows/` during local bootstrap. Future CI may only project the same project-owned local commands; CI configuration is not architectural authority.

## Verification and evidence

- Testability is a production architecture property. Scenarios drive production input/command paths, probes read documented immutable projections, and capture replays the exact run. Test-only mutable backdoors are forbidden.
- Let the engine-owned `ImpactResolver` determine the required suites, profiles, long gates, capture jobs, and review category. An author or agent may add verification but may not remove resolved requirements.
- Observable `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, or `audio` changes require the SPEC-15 capture/evidence flow and hash-bound human review after automatic gates pass.
- Agents may create changesets, baselines candidates, evidence bundles, and diagnostics, but cannot create human approval, access reviewer credentials, self-certify, promote a baseline, or waive an automatic gate.
- Missing GPU, encoder, reviewer, shipping hardware, or RTX training capability is `AwaitingCapability`, never `PASS`. Preserve completed CPU evidence and do not substitute an interactive showcase.
- Do not claim `vertical-v1` conformance until SPEC-12 has 15/15 PASS with complete hash-verified evidence and required attestations.

## Change workflow and handoff

1. Identify the authoritative owner, public contract, relevant requirements/failure paths, and required gates before editing.
2. Keep the change inside that owner. If a new cross-context contract, public parser/capability/IPC surface, backend choice, or security boundary is needed, update the architecture through the ADR process first.
3. Implement through production boundaries, with deterministic positive and failure-path coverage and stable machine-readable diagnostics.
4. Update affected schemas, migrations, traceability, provenance/license records, examples, and architecture documentation together when their contract changes.
5. Once the bootstrap workspace and `xtask` exist, run focused tests while iterating and run `cargo run -p xtask -- host-check` before handoff. `host-check` covers formatting, clippy with warnings denied, workspace tests, documentation validation, and boundary scanning.
6. Report exactly which checks passed, failed, or could not run. Keep developer-host, prototype, shipping, training, certification, and vertical-conformance claims separate.

## Admission

- Treat `docs/architecture/` as the normative source of truth.
- Run `cargo run -p xtask -- host-check` before handing off a change.
- Use public engine-owned contracts; do not leak OS, vendor, ECS-backend, or importer types into `crates/contracts`.
- Do not add CI workflows or remotes during local bootstrap.

## Data and repository boundaries

- Never commit game installations, imported or protected assets, datasets, checkpoints, training runs, model outputs, captures, or secrets.
- `incubator/gothic-importer/` is an ignored nested Git repository. It must not become a Cargo member, path dependency, or parent-repository tracked path.
- Runtime state changes only through production command boundaries; test-only mutable backdoors are forbidden.

## Physical and evidence claims

- Mac MPS/CPU smoke proves only the bounded train/export/inference toolchain.
- Do not claim `PhysicalCertified` until `TRAIN-RTX-01` and all required POLICY/PHYS/TRAIN gates pass. Use `PrototypeFallback` or `AwaitingCapability` honestly.
- Observable changes require the scenario/capture/evidence workflow from SPEC-15. An agent cannot create human approval or waive an automatic gate.
