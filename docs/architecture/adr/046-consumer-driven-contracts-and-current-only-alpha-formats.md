# ADR-046: Consumer-driven contracts and current-only alpha formats

| Field | Value |
|---|---|
| ID | ADR-046 |
| Status | Accepted |
| Version | 1.6 |
| Decision date | 2026-08-08 |
| Last verified | 2026-08-18 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-026](026-deterministic-work-resource-and-streaming-admission.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-034](034-player-targeting-replay-v5-and-mapping-provenance.md) |
| Supersedes | Partially supersedes ADR-008's unconsumed agent-authoring/CLI/MCP contract, ADR-020's pre-v1 N-2 RPG migration obligation and check, ADR-021's unconsumed population/calendar schemas and migration, ADR-025's generic N-2 support requirement, ADR-026's unconsumed generic jobs/resource-management subsystem, and ADR-034's legacy Replay V4/InputMappingReceipt V1 retention. Fully supersedes current implementation obligations introduced by ADR-029 and ADR-031; their product ideas become future Proposed intent. |
| Superseded by | Partially [ADR-052](052-derived-world-calendar-and-authored-routine-vertical.md): later replay/owner closures replace the Replay V5 designation. Partially [ADR-083](083-public-creator-project-cli-vertical.md) and [ADR-084](084-public-creator-run-and-project-package-vertical.md): four public creator operations now have independent production consumers; the wider consumer-driven/current-only policy remains Accepted. |

## Context

Next Engine has no published stable data-format v1 and no production consumer
for several Accepted future feature sets. Maintaining migration windows,
decoders and complete public schemas before a supported external format exists
adds code and test cost without protecting user data. In particular,
`ReplayManifestV4` and `InputMappingReceiptV1` duplicate the current V5/V2
path, while narrative-director and divine-standing contracts have no production
consumer or passing ProductCheck.

## Decision

An Accepted contract MUST be either a product invariant required by a current
root or a format/API exercised by at least one production consumer and its
mapped ProductCheck. A future API remains Proposed until that condition is met.
Scaffolding, a type listing or a test-only fixture is not a production consumer.

All pre-v1 alpha package, save and replay families are current-only. A reader
probes the outer version first and returns its typed `UNSUPPORTED_*` result for
anything other than the current version. It MUST NOT supply defaults, migrate,
auto-upgrade, rewrite or delete the source. Migration policy is introduced only
after the first format is explicitly declared publicly supported.

The current replay is `ReplayManifestV5` with `InputMappingReceiptV2`.
Replay V5 validates its own complete closure directly. Replay V4 types,
tick/compare types, codec, runner and fixtures, plus `InputMappingReceiptV1`,
are removed. `SaveManifestV2`, `WorldCheckpointV4`, `ReplayManifestV5`,
`CommandLedgerV2` and the physics contracts retain their existing wire
semantics.

ADR-008 no longer makes `AuthoringContextBundle`, `AgentChangeSet`, a broad
public creator CLI or MCP parity a current engine contract. ADR-083 later
admits `next project validate/cook` and Creator Command Report V1; ADR-084 adds
bounded `project run/package` and their separate reports for the independent
`creator-smoke` consumer. All wider creator automation must still arrive with
a real consumer and cannot gain a privileged gameplay mutation path.

ADR-020 no longer requires an adjacent N-2 migration chain or migration
ProductCheck for current pre-v1 RPG formats. Its RPG ownership, typed
operations, atomic transaction and causal-history invariants remain Accepted.
Any future migration obligation requires a publicly supported format with a
real successor and a new Accepted decision.

ADR-021 no longer makes `WorldCalendarStateV1`, population tiers, bulk time
advance or legacy calendar migration current contracts. Its durable-identity,
single-owner, no-wall-clock-authority and no-fabricated-outcome invariants
remain. Future population/calendar work is Proposed in SPEC-20.

ADR-026 no longer requires current `JobCoordinatorState`, a generic job class
system, cancellation tree, logical memory admission, pins/leases, global
eviction, archive credits or persisted generic work state. Its implemented
cross-cutting invariants remain: async work receives immutable bounded input,
never retains mutable world access across an async boundary, returns a
revision-bound immutable result, and becomes authoritative only at a canonical
validated commit point. R3a adds no wider framework before its first chunk
consumer proves a need.

ADR-029 and ADR-031 no longer create current implementation obligations for a
narrative director, generated quest graph or divine-standing feature set.
Their ideas may return as small Proposed documents when a concrete playable
vertical requires them.

## Consequences

- Runtime produces one mapping receipt representation and one replay format.
- Old alpha bytes remain untouched and fail before nested decode or mutation.
- Permanent schema identity and atomic publication invariants from ADR-025 stay
  Accepted; its speculative support window does not apply before public v1.
- RPG authority and atomic transaction invariants from ADR-020 stay Accepted;
  its pre-v1 N-2 migration obligation and check do not.
- Population/calendar and generic resource-management APIs return to Proposed;
  immutable staging and deterministic commit remain current invariants.
- Luau and Wasm extension paths, ledger semantics and physics backends are
  unchanged.

## Product checks

| Check | Expected |
|---|---|
| `persistence-replay` | Replay V5 round-trips and replays exact current receipts, query facts and authoritative roots. |
| `fast` | A recognizable V4 replay header returns `UNSUPPORTED_REPLAY_MANIFEST_VERSION` before nested decode; no V4/V1 source symbol remains. |
| `content-package` | Current Luau and Wasm packages continue to use the same validated SDK paths. |
