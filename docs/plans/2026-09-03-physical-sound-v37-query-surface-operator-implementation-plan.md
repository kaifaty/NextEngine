# Physical sound V37 query-surface operator implementation plan

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `ACTIVE / R0_COMPLETE / A0_F0_C0_X0_E0_T0_D0R_REPEAT_EXACT_PASS / D0_ONE_SHOT_NEXT / ZERO_OFFICIAL_TARGET_VALUES_EVALUATED` |
| Research decision | [QSO-v0 successor research](../development/physical-sound-v37-query-surface-operator-research-2026-09-03.md) |
| Parent roadmap | [Roadmap V37](physical-sound-synthesis-roadmap-v37.md) |
| Current evidence | [D0R complete-entry result/seal](../development/physical-sound-v37-d0r-complete-entry-readiness-result-2026-09-03.md), [D0R research](../development/physical-sound-v37-d0r-complete-entry-readiness-research-2026-09-03.md), [T0 exact truth/composite-seal result](../development/physical-sound-v37-t0-exact-truth-protocol-result-2026-09-03.md), [E0 full-count rehearsal/seal result](../development/physical-sound-v37-e0-full-surrogate-seal-result-2026-09-03.md), [X0 complete-owner/terminal result](../development/physical-sound-v37-x0-complete-owner-terminal-result-2026-09-03.md), [C0 full-shape structural/cost result](../development/physical-sound-v37-c0-query-surface-structural-cost-result-2026-09-03.md), [F0 fresh QSO science/role result](../development/physical-sound-v37-f0-fresh-query-surface-operator-result-2026-09-03.md), corrected [A0 query-surface contract](../development/physical-sound-v37-a0-query-surface-contract-result-2026-09-03.md) |

## Outcome

Test a small query-conditioned graph/surface operator as the successor to the
V36 scalar hybrid. The work first makes the experiment contract reusable and
eliminates reject/freeze ambiguity, then freezes fresh synthetic roles and
truth, proves cost and complete-path conformance, and only then opens one D0
and optional H0.

This plan changes no production contract. All code lives in `lab/` until a
real generator, independent validator, joint admission, cooker and product
consumer pass the later Roadmap V37 gates.

## Commit boundaries

### Commit 1 — A0 typed query-surface contract — `COMPLETE`

Deliver:

- `SurfaceFieldSetV1`: explicit field/probe/edge counts, CSR offsets, stable
  IDs, immutable float64 probe values and immutable integer indices;
- `SurfaceQueryBatchV1`: explicit row count, row-to-field indices, query
  positions/normals/barycentrics and typed role identity;
- capability/lifecycle states that separate structural, surrogate D0/H0 and
  future official providers;
- `CandidateDispositionV1` and publisher rules:
  `Pass -> candidate freeze`, scientific reject -> rejected evidence, fault ->
  neither;
- canonical structural hashes that are independent of input enumeration;
- positive tests plus field/CSR/query/permutation/disposition mutations.

Exit:

- Ruff, compile and strict typing pass;
- two fresh processes reproduce the same positive report and mutation matrix;
- every test is target-free and creates no official capability;
- V36 reject artifacts cannot validate as a QSO H0 candidate.

### Commit 2 — F0 fresh operator truth and roles — `COMPLETE`

Deliver:

- new role/truth namespace with disjoint materials, geometries, contacts,
  surface functions, operator mixtures and seeds;
- exact zero intersections with V32–V36 identities;
- three hidden operator components: local anisotropic integral, two-hop graph
  diffusion and global low-rank branch/query interaction;
- frozen candidate width/layers/steps, controls, ablations, metrics, thresholds
  and resource ceilings;
- zero-target role commitment roots and scientific-projection diff.

Exit: repeat-exact manifests with zero target/model/prior-value access.

### Commit 3 — C0 structural and cost owner — `COMPLETE`

Deliver:

- provider constructs every typed field/query batch without targets;
- canonical probe graph, query support, topology reachability, field/query
  ablation reachability and equivalent-remesh roots close for all roles;
- full-shape QSO tensor/caching path executes with artificial zeros;
- complete candidate/control cost oracle runs twice inside the frozen envelope.

Exit: every structural row is supported, no OOD/overlap exists and A/B reports
are exact. Cost failure stops before target access.

### Commit 4 — X0 complete owner and terminal publisher — `COMPLETE`

Deliver:

- one owner function owns materialization, fit, predict, controls, metrics,
  hard/resource gates and atomic terminal publication;
- all D0/H0 stage/callable identities and access receipts are explicit;
- container/lifecycle/access-order/trace/failpoint mutations;
- natural Pass plus metric, hard, resource, pre-access contract and post-access
  owner-fault fixtures;
- H0 validator binds both the Pass terminal root and freeze document.

Exit: two processes repeat every terminal exactly; no reject/fault tree contains
a freeze document or candidate bundle.

### Commit 5 — E0 full surrogate rehearsal and execution seal — `COMPLETE`

Deliver:

- full row counts and frozen training steps for candidate and all controls;
- D0 then zero-training H0 through the exact shared owner;
- A/B recursive byte comparison, external wall/RSS measurements and dependency
  closure;
- checked-in seal binding owner, profile, dependencies, environment, topology,
  both rehearsal roots and zero forbidden access.

Exit: full exact-path A/B Pass under `300 s / 1 GiB / 64 MiB`, or no official
capability may exist.

### Commit 5a — T0 exact truth protocol closure — `COMPLETE`

Pre-access review found that F0 froze seeds and qualitative truth families but
not the exact coefficient PRF, numerical normalizations, reduction order or
canonical row-to-mixture binding. T0 closes that omission without evaluating
an official target:

- domain-separated SHA-256 derives 72 immutable binary64 coefficients from the
  already frozen seeds;
- every local, diffusion, global, decay and gain operation is executable and
  ordered;
- all `15,120` canonical rows receive target-free mixture metadata roots;
- a four-row artificial fixture proves permutation exactness, bounds and
  mode/topology/mixture mutation sensitivity;
- composite seal `7bae65cc…61573` binds T0 to E0 seal `608f9019…1d7f` and its exact environment.

Exit: A/B trees and stdout are exact, stderr is empty and every official target
counter remains zero. Neither F0 nor E0 is rewritten; official capability
creation now requires the composite seal.

### Commit 5b — D0R complete-entry readiness — `COMPLETE`

The [bounded audit](../development/physical-sound-v37-d0r-complete-entry-readiness-research-2026-09-03.md)
found a second pre-access exactness defect: control, optimizer, metric and hard
gate names still leave executable choices unspecified. D0R must freeze those
details and run the actual future D0 owner twice over every structural row with
nonzero artificial targets. The readiness seal binds that exact owner/profile,
environment, E0/T0 seals, full terminal paths and byte-identical output.

Exit: the complete measurement procedure is repeat-exact and every official
target/capability counter remains zero. Only the D0R-sealed owner may later
receive an official D0 provider.

Result: both complete `6,480` train + `4,320` development executions returned
natural `Pass` with byte-identical four-file trees. All 22 metric, 11 hard and
five resource gates passed; replacement seal `a2c3c6ce…b18c5` binds the exact
provider-ready target-aware owner/profile and frozen environment without
opening any official value.

### Commit 6 — D0/H0 one-shot evidence — `NEXT`

Deliver:

- seal-verifying official providers which materialize only their authorized
  fresh role transaction;
- predetermined D0 A/B with exact tree/stdout comparison;
- only after D0 Pass, frozen candidate plus predetermined zero-training H0 A/B;
- compact terminal reports; all generated values/weights remain external.

Exit: admit QSO only on repeat-exact D0 and H0 Pass. Any scientific reject,
resource reject, owner fault or divergence closes the family without retry.

## Fixed scientific comparison

Candidate QSO must beat all applicable frozen controls:

- nearest and continuous local interpolation;
- V36-shaped pointwise MLP;
- fixed RBF integral ridge;
- query-only, field-only and no-topology QSO ablations.

Primary contact gates are aggregate plus contact-only, geometry-only and joint
transfer. Contribution gates require the field branch, query trunk, explicit
branch/query interaction and topology propagation to each matter in their
declared strata. Decay/global-gain gates and every P1 hard invariant remain
unchanged unless F0 documents a new fresh scientific question before values.

## Verification routing

- A0–E0: Ruff/compile, strict typing, focused unit/property/state-machine tests,
  process A/B exact comparison, focused xtask physical-sound registry tests and
  SPEC-45 boundary scan.
- D0/H0: the same plus exact official access receipts, the E0 execution seal,
  the T0 composite truth/input seal and all sealed dependency identities.
- No Cargo ProductCheck applies until cooker/demo code exists.
- K0 later runs affected `content-package`; P0 runs affected `play` checks.

The known `SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`
is reported but is outside this plan's source layout unless an edited file
touches that boundary.

## Immediate action

Implement Commit 5b only. Freeze the complete D0 measurement profile, build
the actual target-aware D0 owner and run its full-count nonzero artificial A/B
plus terminal/failpoint mutations. Publish a readiness seal binding E0
`608f9019…1d7f`, T0 `7bae65cc…61573`, the exact environment and owner. Do not
implement an official provider or evaluate any official target until this
readiness seal passes.
