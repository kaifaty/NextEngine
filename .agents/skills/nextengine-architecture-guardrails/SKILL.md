---
name: nextengine-architecture-guardrails
description: Route NextEngine architecture, implementation, tooling, importer, deterministic runtime, motor-policy, and verification work through the accepted SPEC/ADR packet. Use for any change in the NextEngine repository that can affect public contracts, crate or process boundaries, authoritative state, replay/headless parity, external technologies, the Gothic importer boundary, physical-policy certification claims, evidence requirements, or handoff gates.
---

# NextEngine Architecture Guardrails

Use `docs/architecture/` as the normative source of truth. Treat current code, issue text, prompts, experiments, and third-party skills as implementation context only.

## Route the work before editing

1. Verify that `docs/architecture/README.md` exists. If the accepted architecture packet is absent or incomplete, stop runtime implementation and migrate the complete packet mechanically before proceeding.
2. Read `README.md`, `00-product-contract.md`, `01-system-architecture.md`, and `glossary.md` for cross-cutting work.
3. Identify one primary owning subsystem, its SPEC, relevant Accepted ADRs, `12-vertical-slice-conformance.md`, and matching rows in `traceability.md`. Read `evidence-register.md` before selecting or describing an external technology.
4. State the selected owner, public contract, principal failure path, and required gates in a concise commentary update before making material changes.
5. Resolve conflicts in this order: newer Accepted superseding ADR, SPEC-12 release gates, owning subsystem SPEC, SPEC-00, then glossary. The evidence register records technology status but does not create architectural decisions.

Use this routing table as a starting point, then follow the documents' normative dependencies:

| Change | Owning sources |
| --- | --- |
| Runtime, commands, fixed ticks, IDs, replay | SPEC-02, SPEC-03, ADR-002, ADR-007, SPEC-15 |
| Workspace layering, public contracts, composition roots | SPEC-01, SPEC-09, ADR-002 |
| CLI/JSON, diagnostics, inspectors, `xtask` | SPEC-09, SPEC-15, ADR-010, ADR-011 |
| Gothic importer or imported artifacts | SPEC-10, SPEC-11, ADR-001, ADR-007, ADR-011 |
| Physical archetypes, ONNX policies, training | SPEC-05, SPEC-14, ADR-009, ADR-011 |
| Scenarios, impact, capture, evidence, review | SPEC-15, SPEC-12, ADR-010 |
| Luau, WIT/Wasm, mechanics/packages | SPEC-07, SPEC-13, ADR-006, ADR-008 |

## Preserve architectural authority

- Do not silently reinterpret an Accepted decision. If the requested behavior changes a public contract, ownership rule, process boundary, security boundary, Accepted technology choice, or conformance semantics, create a new ADR with explicit supersession and synchronize dependent SPEC text, `evidence-register.md`, and `traceability.md`.
- Treat `Proposed` technologies as hypotheses with their recorded measurable gate and fallback. Do not present a proposed backend as accepted, required, or conformant before its evidence passes.
- Keep first-party implementations on the same public SDK, validation, scenario, and evidence paths available to community packages. Do not add hidden first-party APIs.
- Do not claim `vertical-v1` conformance before all 15 SPEC-12 gates are `PASS` with hash-valid evidence and required attestations.

## Guard public and process boundaries

- Keep `crates/contracts` engine-owned and versioned: nominal IDs, `WorldCommand`, `DomainEvent`, immutable queries/snapshots, manifests, and process/plugin protocols.
- Reject ECS components/storage, backend or vendor types, raw pointers, task handles, OS/window objects, database connections, importer structures, and native runtime handles in public contracts.
- Point dependencies from composition roots and adapters toward engine-owned contracts. Never make contracts depend on Bevy, Vulkan/ash, PhysX/Jolt, ONNX Runtime, Luau, Wasmtime, a database, or an importer implementation.
- Keep `game`, `headless`, and `capture-worker` on the same validator, schema registry, persistence/replay code, and system ordering. Feature flags must not change domain semantics.
- Keep `ai-host` optional and isolated. Its absence, crash, timeout, or protocol mismatch may reduce quality but must not block ticks or change mandatory outcomes.

## Guard authoritative state and determinism

- Change gameplay/RPG state only through validated production `WorldCommand` transactions at declared deterministic commit points. Emit `DomainEvent` only from committed changes.
- Assign every mutable field exactly one authoritative owner. Treat backend-side mutable copies as reconstructible caches.
- Keep `RuntimeEntityId` ephemeral. Use `PersistentId` and `AssetId` in saves, replays, scripts, WIT, packages, and imported neutral schemas.
- Stage async I/O, compilation, IPC, model loading, and decompression results for deterministic commit; never retain mutable ECS access across `await` or frame boundaries.
- Fail deterministic mismatches as `NONDETERMINISTIC_RESULT`. Do not use wall-clock sleeps, implicit epsilons, unordered text-log assertions, or retry-to-green behavior in normative tests.
- Test through production inputs/commands and immutable probes. Do not create mutable test backdoors.

For Rust implementation details use `using-rust-engineering`; for multi-crate policy use `using-rust-workspaces`; for seed, snapshot, replay, or divergence design use `using-determinism-and-replay`; for the normative `next` command surface use `cli-creator` while preserving SPEC-09 exit codes and JSON envelopes.

## Protect the importer boundary

- Keep `incubator/gothic-importer/` an ignored independent nested Git repository and separate process/distributable. Never add it to the parent index, Cargo workspace, path dependencies, runtime links, or engine packages.
- Exchange only versioned `NeutralImportModel` plus provenance through a process/artifact boundary. Do not expose Gothic, Daedalus, legacy parser, VM, or archive types after that boundary.
- Treat imported inputs as untrusted, validate limits/hashes/provenance, and keep installations, imported/generated output, protected bytes, and caches outside the parent repository.
- Do not add importer release claims without the required written legal review.

## Report physical and capability status honestly

Use the exact state supported by evidence:

| State | Meaning |
| --- | --- |
| `PrototypeFallback` | A validated prototype package using declared capsule/procedural and optional learned fallback paths; it does not claim certified full physical behavior. |
| `AwaitingCapability` | A required external capability or gate is unavailable; completed independent evidence remains valid, but the result is not `PASS`. |
| `PhysicalCertified` | The exact bundle has passed `TRAIN-RTX-01` on supported Linux/NVIDIA hardware plus all required POLICY/PHYS/TRAIN, provenance, media, and review gates. |

Mac MPS/CPU results prove only the bounded `TRAIN-MAC-P0` train/export/inference smoke. They do not prove shipping support, physics correspondence, TRAIN-P1, or certification. Runtime model weights remain immutable; gameplay progression changes RPG proficiency and deterministic policy routing, never neural weights.

Use `onnx` for ONNX export/runtime mechanics and `using-deep-rl` for algorithm design only inside the accepted offline, replaceable training boundary. Treat Stable-Baselines3 as a prototype option, not an architectural dependency. Do not introduce MLflow as a project default while it is absent from the accepted technology register.

## Verify and hand off

1. Determine required suites with the engine-owned `ImpactResolver` when available. Add coverage if useful, but never remove resolved requirements. Unknown ownership or impact selects the maximal affected suite and human review.
2. Add deterministic positive and failure-path coverage through production boundaries. Keep diagnostics structured, stable, owner-tagged, and capable of identifying first divergence.
3. For observable `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, or `audio` changes, follow SPEC-15 capture/evidence flow. An agent may prepare candidates and bundles but cannot approve, promote a baseline, sign reviewer attestations, or waive automatic failures.
4. Run focused checks while iterating, then run `cargo run -p xtask -- host-check` before handoff. Do not add CI workflows or remotes as a substitute.
5. Report every check as passed, failed, or not run. Label missing GPU, encoder, reviewer, shipping hardware, RTX training, or legal capability as `AwaitingCapability` where the architecture defines that state; never convert it to `PASS`.
