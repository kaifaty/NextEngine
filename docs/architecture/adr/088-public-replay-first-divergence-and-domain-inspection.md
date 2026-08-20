# ADR-088: Public replay first-divergence and bounded domain inspection

| Field | Value |
|---|---|
| ID | ADR-088 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-20 |
| Last verified | 2026-08-20 |
| Normative dependencies | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-074](074-systemic-strategic-agent-owner-vertical.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md), [ADR-084](084-public-creator-run-and-project-package-vertical.md), [ADR-087](087-public-creator-runtime-scenario-and-prefix-minimization.md) |
| Supersedes | Narrowly supersedes ADR-087 and SPEC-09/15 statements that public replay/domain inspection is unimplemented. Replay V10 wire semantics, scenario semantics and existing creator report families remain unchanged. |
| Superseded by | not superseded |

## Context

R6e can preserve and minimize a failing creator scenario, while exact
production replay still required Rust verification code. `ReplayManifestV10`
already records the current ten-owner closure, exact ingress/query/outcome
facts and per-tick compare points, and application already re-executes it
through Runtime and World Services. The missing product boundary is a safe
public consumer that identifies the first divergent tick/subsystem and exposes
enough bounded evidence to debug it without dumping private owner state.

Creating a second tool-only replay format or executing a simplified CLI model
would undermine the determinism proof. Promoting arbitrary snapshot queries,
recording/editing, branching or every future domain would violate ADR-046.

## Decision

### Public commands and exact project input

R6f adds exactly:

```text
next replay validate --replay <file> (--project <authoring-directory> | --package <creator-package-directory> | --content-store <cooked-content-store>)
next replay inspect --replay <file> (--project <authoring-directory> | --package <creator-package-directory> | --content-store <cooked-content-store>) --tick <u64> --domain <runtime|world-services|physics|owners>
```

The input is the existing canonical current-only `ReplayManifestV10`, not a
new creator replay schema. A regular non-link file is limited to 16 MiB and one
to 4,096 contiguous ticks. Replay V9 and earlier are rejected by the bounded
outer version probe before nested decode. Unknown fields, non-canonical JSON,
invalid owner closure, hashes, batches or queries fail closed.

Authoring input is cooked and activated in a private temporary content store.
A creator package receives its existing full inventory/NOTICE/activation and
run-proof revalidation before replay use. `--content-store` activates the
immutable output of `next project cook` directly. Exactly one source is
required and no source path appears in the report.

Replay compatibility must match the activated project ID, authoring/build,
schema registry, content manifest, mechanics lock, runtime determinism profile
and tick settings. This comparison occurs before replay restore/mutation. It
closes the previous application-runner gap where a package was supplied but
the manifest compatibility block was not compared with it.

### Validate, verify and inspect

`validate` completes current Replay V10 decode, owner/tick/compare-point
validation, project activation and exact compatibility binding. It executes no
replay tick and returns `validated-not-run`.

`inspect` requires one existing tick and one domain. It first executes the
complete manifest through
`next_application::replay::run_replay_manifest_v10`, which restores the
recorded ten-owner state and uses ordinary Runtime/World Services/Physics,
query and physical-animation paths. A projection is returned only after every
recorded compare point passes; selecting an early tick does not weaken the
full-replay oracle.

The four V1 projections are deliberately bounded:

- `runtime`: application state root, command-ledger and ingress/outcome hashes,
  plus direct-command/result/event counts;
- `world-services`: streaming operation/target stable ID, mapping,
  interaction and targeting counts/hashes;
- `physics`: substeps, accepted-intent/contact/query counts and exact
  step/contact/query hashes;
- `owners`: the sorted compare-point descriptors (owner/schema/segment stable
  IDs, version, length and content hash), never canonical owner bytes.

Each success contains exactly one tick projection, so output size does not
scale with replay duration. Creator Replay Report V1 is a seventh independent
report family; no prior report bytes or semantics change.

### First divergence and failure safety

Production replay stops at the first failing tick. Compare-point failure names
the first ordered stage and owner, including application state root,
command-ledger, owner closure, ingress/outcome, Physics step/contact/query,
targeting trace or interaction availability. Runtime replay-driver mismatches
retain their precise stage instead of collapsing into one generic query phase.

The public failure is `NONDETERMINISTIC_RESULT` with
`first_divergent_tick`, stable `stage`, stable `owner` and expected/actual roots
when available. Invalid replay, unsupported version, exact-project mismatch,
absent requested tick and unsafe source are separate stable diagnostics. No
failure emits a partial domain projection or mutates source/project/package
bytes.

## Product impact

A creator or CI job can take an exact Replay V10 produced by a runtime/check,
prove that it belongs to the intended project, reproduce it without a private
verification harness and ask a small machine-readable question about one
tick. When determinism breaks, the report points directly to the first tick
and subsystem instead of returning only a final-root mismatch.

This completes R6f and materially reduces B-09. R6 remains open for the
remaining external SDK/documentation workflow and consumer-driven inspector
breadth; R6f is not a live editor/debugger or replay authoring system.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused `next_cli` | Argument, retired/malformed/link replay and exact filters | Typed path-free rejection before project access/replay | Input remains unchanged |
| `persistence-replay` | Generate current production Replay V10, call public validate and all four domains | Exact project identity; every inspection is `replayed-exact` | Fail the ProductCheck |
| first divergence | Replace the first compare-point state root without changing tick order | `NONDETERMINISTIC_RESULT` at the first tick, `application-state-root`, owner `application` | No projection |
| compatibility/tick negatives | Bind another project ID or request an absent tick | Stable project-mismatch/tick-not-found failure before replay | No partial result |
| Linux `host-check` | Shared contracts/application/tooling/verification path | Workspace formatting, clippy, tests, doctests and boundary scan pass | R6f remains incomplete |

## Considered alternatives

- **Add a Creator Replay V1 format.** Rejected: Replay V10 is already the
  current production authority and a second format would create drift.
- **Inspect recorded fields without replaying.** Rejected for `inspect`: it
  could describe a corrupt trace as valid. Static validation remains the
  explicit cheaper `validate` operation.
- **Return complete decoded snapshots.** Rejected: it exposes private storage,
  expands reports with world size and creates a mutable-inspector precedent.
- **Return all ticks by default.** Rejected: output would scale with input and
  encourage ad-hoc trace dumping. One exact tick/domain is sufficient for the
  first consumer.
- **Support Replay V9 migration.** Rejected under ADR-025/046: no public v1
  predecessor exists and the current owner closure cannot be projected through
  a retired replay generation.

## Consequences and next boundary

- The public creator surface now has twelve operations and seven separately
  versioned report families.
- Production Replay V10 now enforces exact project compatibility at the runner
  boundary as well as in tooling.
- The generated persistence replay fixture governs the public consumer without
  making verification a production dependency or checking in private owner
  bytes.
- Linux is the active implementation host. Windows/THOTH remain
  `NotRun(WindowsHostDeferred)` under ADR-082.
- R6/B-09 remain open for the remaining SDK workflow and only concrete future
  domain consumers may extend the fixed projection set.

## Supersession

Changing accepted replay generation, project binding, size/tick bounds,
full-replay-before-projection rule, stage/owner failure identity, domains,
fields or report semantics requires an explicit successor. Recording,
editing, migration, live queries, capture or branching requires a separate
consumer-driven decision and equivalent positive/failure evidence.
