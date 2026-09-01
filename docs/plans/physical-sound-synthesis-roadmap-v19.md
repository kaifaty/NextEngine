# Roadmap V19: integrity-first intrinsic field learning

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ADOPTED / R0_COMPLETE / P0A_FROZEN / C0_REPEAT_EXACT_PASS / P0B_PENDING / F0_I0_NOT_RUN / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V18](physical-sound-synthesis-roadmap-v18.md), closed by its repeat-exact O0 rejection |
| Evidence basis | [V19 C0 result](../development/physical-sound-v19-c0-composite-coverage-result-2026-09-01.md), [V19 P0a protocol](../development/physical-sound-v19-p0a-composite-coverage-protocol-2026-09-01.md), [V19 coverage research](../development/physical-sound-v19-composite-coverage-research-2026-09-01.md), [V18 B0 result](../development/physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md) and [V15 source insufficiency](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material, absent source or tooling failure |

## Goal

Prove the smallest offline pipeline that can turn object/mesh/contact inputs
into bounded modal impact clips without asking the user to record or manually
approve every sound:

1. keep the already passing deterministic global modal scaffold unchanged;
2. reject incomplete or spatially unsupported context before ML inference;
3. learn only the contact-dependent signed modal-gain field and beat compatible
   deterministic interpolation;
4. integrate the frozen pieces on fresh synthetic identities;
5. earn real-audio and independent-validator credit only from disclosed
   internet sources with complete provenance and untouched protected roles;
6. bake accepted output to deterministic clips while preserving fallback.

V19 does not authorize runtime inference/training, public contracts, removal of
clip fallback or a claim that synthetic truth equals real material acoustics.

## Why V19 changes the decomposition

V18 B0 passed all `19/19` gates twice byte-exactly, so its scale-separated
degree-two ridge remains the frozen provisional global scaffold. V18 O0 then
proved graph geodesics useful for disconnection and topology, but falsified raw
nearest-context distance as a complete thinning certificate.

V19 makes coverage a layered precondition rather than model confidence:

```text
object/acoustic metadata
  -> frozen B0 dimensionless modal scaffold

mesh + declared context manifest + sparse observations
  -> structural context closure
  -> set-level graph-geodesic fill
  -> local graph reachability / intrinsic gap
  -> masked intrinsic field operator

global modes + accepted contact gains
  -> deterministic modal renderer
  -> offline dry clip + provenance + fallback map

any reject / unsupported input / missing evidence
  -> authored clip fallback
```

## Frozen research boundaries

- Impact comes first. Rolling, scraping, fracture, radiation/listener fields,
  rooms and arbitrary-force transfer remain separate claims.
- Acoustic material values are synthesis-profile inputs, not physics authority.
- The user records nothing and does not become a per-sound validation queue.
- Real evidence must be found on the internet and retain source identity,
  revision, member completeness, provenance and redistribution disposition.
- Synthetic capability, disclosed-real candidate quality and protected
  admission are separate certificates.
- All datasets, arrays, weights, WAVs, reports and caches remain external.

## Fresh identity ledger

P0a must enumerate and hash exact rows/grids before implementation. These
Halton bands are reserved and disjoint from every opened V16–V18 identity:

| Role | Reserved indices | Rows | May be generated when |
| --- | --- | ---: | --- |
| C0 development/calibration | `1101…1112` | 12 | P0a successful controls |
| C0 one-shot test | `1201…1212` | 12 | After C0 code/tests are committed |
| F0 train | `1301…1324` | 24 | P0b after C0 pass |
| F0 development | `1401…1412` | 12 | P0b successful controls |
| F0 one-shot test | `1501…1512` | 12 | After F0 code/model recipe is committed |
| I0 one-shot integration | `1601…1612` | 12 | After B0/C0/F0 hashes are frozen passing inputs |

P0a/P0b must freeze exact material/topology/support cells, mesh resolution,
context identity and successful controls. No candidate may read a later role,
opened V16–V18 prediction, real waveform or protected value. A failed stage
seals every downstream role.

## Evidence lanes

| Lane | Question | Output | Promotion rule |
| --- | --- | --- | --- |
| B — global scaffold | Do object-global frequencies/damping remain bounded? | frozen V18 `GlobalBaselineV0` | Reuse exact passing hash; no refit or fresh claim. |
| C — coverage | Is the context structurally complete and intrinsically adequate? | `CompositeCoverageCertificateV0` | Must pass before any field inference. |
| F — field | Does masked intrinsic learning beat compatible classical interpolation? | `FieldCapabilityV0` | Opens integration only after repeat-exact pass. |
| I — integration | Do frozen B/C/F reproduce complete rendered endpoints? | `IntegratedCapabilityV0` | May open disclosed-real work if sources are ready. |
| S — sources | Are enough internet physical groups complete and role-eligible? | source/role certificates | Independent prerequisite; no local recording. |
| R/V/A — real validation | Does the candidate improve disclosed audio and pass an independent protected policy? | candidate, validator and admission certificates | One-shot shadow; reject/OOD selects fallback. |
| K/D — product evidence | Can admitted output cook byte-exactly and serve one demo consumer? | clip atlas, fallback map and demo | Still research-only until a promoting ADR/ProductChecks. |

## C0 certificate contract

`CompositeCoverageCertificateV0` has ordered, immutable failure reasons:

1. `OOD_CONTEXT_BUDGET`: wrong mesh/context identity, duplicate or out-of-range
   index, incomplete declared minimum or hash closure;
2. `OOD_DISCONNECTED`: context cannot reach the query component;
3. `OOD_INTRINSIC_FILL`: normalized set-level graph covering radius exceeds the
   development-frozen limit;
4. `OOD_INTRINSIC_GAP`: a reachable query exceeds the local intrinsic limit;
5. `ACCEPT`: every structural and geometric check passes.

Intrinsic separation and mesh ratio are recorded as diagnostics. Count-only,
raw V18 distance, global-fill-only and Euclidean variants are fixed controls.
A spectral proxy is report-only unless a later preregistered field-bandlimit
experiment first justifies its premise.

P0a freezes the exact gates, but they must include:

- valid false OOD `<=10%` per topology and at least `11/12` valid objects;
- `>=95%` rejection for every mutation/topology cell and at least `11/12`
  mutation objects;
- `100%` rejection of duplicate/out-of-range/incomplete-budget successful
  controls with the declared structural reason;
- `>=95%` component isolation and RolledSheet ambient-shortcut rejection with
  topology-aware reasons;
- composite utility no lower than raw V18-distance and Euclidean controls,
  plus a strict layer-specific ablation win for structural and graph checks;
- complete finite records, zero forbidden access and byte-exact repeat.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V19 rebaseline | `COMPLETE` | Competing count/fill/spectral hypotheses, primary sources, development diagnostic and selected composite are recorded without opening a new test. |
| P0a | Composite coverage protocol | `COMPLETE / FROZEN_BEFORE_IMPLEMENTATION` | [Exact C0 rows/grids, structural manifest, formulas, reason codes, controls, gates, access ledger and compute ceiling](../development/physical-sound-v19-p0a-composite-coverage-protocol-2026-09-01.md) are frozen before implementation. |
| C0 | Composite coverage capability | `COMPLETE / REPEAT_EXACT_PASS` | [All 15 gates pass twice byte-exactly](../development/physical-sound-v19-c0-composite-coverage-result-2026-09-01.md): valid false OOD `0.0`, every mutation/reason cell `1.0`, composite utility `1.0` versus raw intrinsic `0.961706` and Euclidean `0.760927`. |
| P0b | Field/integration protocol | `PENDING / UNSEALED_BY_C0` | Exact F0/I0 identities, signed-gain truth, masks, classical controls, renderer endpoints and gates freeze before learned implementation. |
| F0 | Masked intrinsic field operator | `NOT_RUN / BLOCKED_BY_P0B` | A bounded graph operator beats harmonic/geodesic interpolation and non-graph controls on every required field/topology stratum with calibrated OOD and exact repeat. |
| I0 | Frozen hybrid tournament | `NOT_RUN / BLOCKED_BY_P0B_F0` | Exact B0+C0+F0 hashes reproduce held modal endpoints and dry renders on `1601…1612` twice exactly. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each bounded source gains exact identity/member/axis/freshness/provenance evidence or a machine-readable closure reason. |
| M1 | Metal role freeze | `BLOCKED_BY_S1` | Five generator-ready and three evaluation-complete groups satisfy unchanged `4/1/1/1/1`; no exposed alias receives protected credit. |
| R1 | Disclosed-real tournament | `BLOCKED_BY_I0_AND_M1_GENERATOR_PREREQUISITE` | Frozen hybrid candidate beats compatible deterministic and authored controls without protected access. |
| V0/V1 | Independent validator | `SCAFFOLD_AFTER_I0 / RELEASE_BLOCKED` | Hard checks, embedding/acoustic ensemble and OOD policy calibrate independently of generator training and freeze before shadow. |
| A0 | Metal shadow admission | `BLOCKED_BY_R1_V1_M1` | One untouched protected shadow opens once and emits immutable `Pass`, `Reject` or `FallbackOOD`. |
| K0/D0 | Cooker and demo | `BLOCKED_BY_A0_PASS` | Byte-identical clips plus fallback map feed one presentation-only demo prop through the existing audio path. |
| G1 | Wood then Glass | `AFTER_METAL_A0` | Each material repeats the unchanged source/validator/admission policy; unavailable domains remain fallback. |
| N1 | Optional neural global residual | `DEFERRED / EVIDENCE_TRIGGERED` | Only a clustered disclosed-real B0 residual may open a new preregistered family; no synthetic architecture search. |
| P1 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A concrete consumer, protected evidence, bounded Linux cost and separate Accepted ADR authorize contracts and ProductChecks. |

## Execution order

1. Freeze P0a, including successful malformed-record and geometry controls.
2. Implement C0 and focused tests; commit before opening the C0 test.
3. Execute C0 twice into independent empty external roots. Reject closes V19
   before learned field work; pass freezes the exact certificate hash.
4. Freeze P0b on all-new F0/I0 identities and successful development controls.
5. Implement and commit F0 before opening its test; execute twice.
6. If F0 passes, integrate only exact passing B0/C0/F0 hashes and execute I0
   twice. No component refit is allowed in I0.
7. Continue source growth independently. R1 waits for both I0 and generator
   source prerequisites.
8. Freeze the independent validator, then open one protected Metal shadow once.
9. Cook/demo only after A0 pass. Wood and Glass repeat the same policy.
10. Consider product promotion only through a separate ADR and real consumer.

## Stop rules

1. Do not tune O0 or reuse its opened test meshes as V19 validation.
2. Do not select C0 thresholds, reason precedence or ablations from C0 test.
3. A C0 reject seals P0b/F0/I0. A F0 reject seals I0. A disclosed-real or
   protected reject selects fallback rather than a nearby retry.
4. B0 is reused by exact hash. Do not refit it or revive the V17 global neural
   head without the N1 evidence trigger.
5. F0 must beat compatible classical interpolation; one pleasant WAV or lower
   training loss is insufficient.
6. No local microphone/hammer capture, prompt-to-waveform shortcut, per-sound
   human queue, raw PhysX callback mixing or runtime learning/inference.
7. Unknown or incompatible redistribution terms exclude an artifact from a
   distributed product; scientific observation may be cited without silently
   promoting the bytes.

## Definition of done

V19 succeeds as a research roadmap only when:

- C0, F0 and I0 pass twice exactly in dependency order;
- enough disclosed Metal sources satisfy the unchanged role contract;
- a frozen candidate and independent validator process one untouched Metal
  shadow exactly once;
- a passing candidate cooks byte-identical dry clips plus a complete fallback
  map for one demo prop;
- all unsupported cases remain playable through authored clips.

That still does not ship runtime ML. Production promotion remains a separate
architecture decision with a concrete consumer and measured product checks.
