# Agent instructions

## Architecture context

- Use `docs/architecture/` as design context for architecture-sensitive work. Do not infer product contracts from scaffolding or experiments.
- Read `docs/architecture/README.md`, `00-product-contract.md`, `01-system-architecture.md`, `glossary.md`, the relevant subsystem SPEC and relevant ADRs before cross-cutting work.
- Use `docs/architecture/agent-routing.md` as the deterministic task-to-document routing table: find the matching row(s), read the listed SPEC/ADRs in full and run the mapped product checks.
- Resolve conflicts in this order: a newer Accepted superseding ADR, the relevant subsystem SPEC, SPEC-00, then the glossary.
- Architecture documents follow the normal repository workflow. A semantic change to an Accepted decision needs a short ADR that names what it supersedes; update affected SPECs and the lightweight traceability map in the same change. Adding a new SPEC/ADR or changing a document status (including supersession) must update the routing table in `docs/architecture/agent-routing.md` in the same change.
- `Proposed` technology is an experiment, not a default. State its fallback and do not present it as shipped before the affected product check passes.

## Documentation retrieval

- For architecture-sensitive, roadmap-sensitive or cross-cutting work, start from the deterministic routing table in `docs/architecture/agent-routing.md`: find the matching row(s) and read the listed SPEC/ADRs in full before making claims or edits.
- Read complete documents, not search snippets. Snippets and matches are discovery aids, not authority, and do not replace the mandatory direct reads or the precedence rules above.
- Use `rg` and direct file reads for exact source-code symbols, known paths and verification, and for fuzzy lookup across `docs/plans/`, `docs/reviews/` and `docs/development/` — those are working materials, not normative architecture.

## Roadmap context

- Use `docs/roadmap.md` as planning context when a task affects product scope, implementation order, stage dependencies, a roadmap blocker, an exit criterion or the reported state of a subsystem. Routine local fixes that do not change those facts do not require reading or editing the roadmap.
- Treat the roadmap as a living planning document, not normative architecture. Accepted SPEC/ADR and the precedence rules above remain authoritative if they conflict with roadmap wording.
- Before roadmap-sensitive implementation, identify the affected stage or work package and its stated prerequisites, blockers, success criteria and scope guard. Do not expand a bounded task merely to close unrelated roadmap work.
- Update `docs/roadmap.md` in the same change when completed work materially changes its facts: subsystem implementation status, stage status, an exit criterion, blocker state, a recorded decision or the near-term implementation queue. Keep unrelated roadmap text stable.
- Mark a stage or blocker complete only when the documented observable criteria and relevant ProductCheck actually pass. `NOT_RUN`, a public contract, an Accepted SPEC, a compiling adapter or partial implementation is not completion.
- If implementation changes Accepted semantics, follow the ADR/SPEC workflow independently of the roadmap update. A priority or sequencing change by itself normally updates only the roadmap.

## Product baseline

- Next Engine is an independent, AI-first open-source engine and toolchain for systemic single-player RPGs. It is not an OpenGothic port and not a general-purpose engine.
- Prioritize a playable game and fast iteration.
- Rust is the portable core language. Use the repository-pinned Rust 1.97.1 toolchain and keep workspace `unsafe_code` forbidden unless an Accepted ADR creates one small reviewed FFI/backend boundary.
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

- Verification is lightweight, local and proportional to executable risk.
- A Git commit is a checkpoint, not a validation gate. Never run checks solely because a commit is about to be created, and never require passing checks as a precondition for committing. An unverified commit is allowed; run and report the relevant checks before final handoff, readiness/completion claims, or when the user/plan explicitly requests them.
- A documentation-only change qualifies for the cheap path when every changed file is human-readable documentation or agent guidance and the change touches no Rust/Python/C++, build/configuration, schema, generated fixture, package manifest or runtime-consumed data. Run `git diff --check` and directly validate changed links, paths and identifiers. Do not run Cargo or `host-check` for this path unless the user explicitly asks.
- For a localized code change, run formatting/static analysis and focused tests for the affected package or boundary. `cargo run -p xtask -- host-check` is the broad workspace `fast` command; use it for cross-cutting changes, public contracts, workspace/build configuration, changes whose affected package set is uncertain, or when a plan/user explicitly requires it. It is not a default for simple edits.
- For gameplay changes run `play`; for state changes run `persistence-replay`; for content/tool changes run `content-package`.
- Run `platform` or `performance` only when the affected feature triggers the corresponding conditional check.
- Screenshots, captures, profiles and minimized replays are optional debugging aids.
- Testability remains a code property: scenarios use production inputs and probes read immutable public projections. Test-only mutation backdoors remain forbidden.

## Change workflow

1. Identify the affected public contract, technical state boundary and, when relevant, roadmap stage or work package.
2. Keep the change small and product-driven. Add an ADR only for a real semantic or cross-context decision.
3. Implement through production paths with focused positive and failure coverage.
4. Update affected schemas, migrations, examples, architecture text and material roadmap facts together.
5. When implementing an approved plan, create the commits specified by that plan as each commit boundary is completed. If the plan does not define commit boundaries, create coherent commit(s) for the completed in-scope work before handoff; do not wait for checks or a separate commit reminder, and never include unrelated user changes.
6. Before final handoff or a readiness/completion claim, run the minimum risk-scoped checks above. Documentation-only work uses the cheap path; localized code uses focused package checks; broad `host-check` is conditional, not automatic. Commit creation itself never triggers checks.
7. Report each relevant check as passed, failed or not run, with the remaining product risk.

## Persistent-problem research escalation

- If the same blocker survives two coherent remediation cycles, or several
  variants move symptoms without closing the stated criterion, pause the next
  similar implementation attempt and run a bounded research cycle.
- Restate the problem as falsifiable competing hypotheses. Inspect failure
  clustering and causal order, question assumptions at adjacent layers, and
  deliberately consider non-local explanations rather than only tuning the
  component where the symptom appears.
- Use primary sources when external tool, backend or scientific semantics may
  matter. Prefer small counterfactual experiments with successful controls
  before another full or expensive run.
- Record evidence for and against each serious hypothesis, rejected options,
  remaining uncertainty and the decision criterion for resuming implementation.
  Resume with the smallest evidence-backed change and an explicit rollback or
  non-regression check.
- Research and brainstorming do not weaken an existing requirement, authorize
  a later stage, or justify indefinite analysis. If the cycle cannot
  discriminate between options, state the missing evidence and choose the
  cheapest safe experiment that can.

## Repository hygiene

- Do not add remotes or `.github/workflows/` during local bootstrap.
- Do not commit game installations, imported/protected assets, datasets, checkpoints, training runs, model outputs, captures, caches or secrets.
- Do not make the nested importer a Cargo member, path dependency or linked runtime library.
