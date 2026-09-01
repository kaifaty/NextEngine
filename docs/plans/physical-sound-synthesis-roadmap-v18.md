# Roadmap V18: deterministic modal scaffold, learned surface residual

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `CLOSED_AT_O0 / B0_REPEAT_EXACT_PASS / O0_REPEAT_EXACT_REJECT / F0_I0_NOT_RUN / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V17](physical-sound-synthesis-roadmap-v17.md) after its frozen G0 rejection |
| Evidence basis | [V18 O0 result](../development/physical-sound-v18-o0-intrinsic-coverage-result-2026-09-01.md), [V18 B0 result](../development/physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md), [V18 P0b protocol](../development/physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md) and [V15 source insufficiency](../development/physical-sound-v15-s0c-source-sufficiency-role-freeze-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, unsupported material, missing source or tooling failure |

## Goal

Build the smallest falsifiable offline physical-sound pipeline that uses
closed-form/deterministic methods for the easy global structure and neural
operators only for contact-dependent surface behavior that simple methods
cannot recover.

V18 must prove, in order:

1. an unchanged scale-separated low-order global baseline generalizes to a
   fresh object/scale set;
2. deterministic intrinsic coverage reliably detects unsupported contacts;
3. a masked graph operator adds measurable value over classical interpolation
   for signed modal-gain fields;
4. the frozen pieces remain accurate when rendered together on fresh objects;
5. disclosed internet audio and an independent protected validator support a
   narrow cooked vertical.

No step authorizes runtime inference, runtime training, public contracts or
claims that synthetic truth equals real material acoustics.

## Why the roadmap changes

V17 G0 passed every absolute, scale-transfer, mutation, OOD and repeat gate,
but missed the required ten-percent improvement over degree-two ridge:
`0.9497x` frequency and `0.9239x` damping. The representation worked; the
neural complexity was not justified.

Therefore V18 promotes no neural global head. It treats the unchanged
scale-separated ridge as a provisional research scaffold, verifies it once on
fresh rows, and reserves learning for intrinsic surface transport. A neural
global residual may return only after disclosed real residuals demonstrate a
specific failure that the deterministic scaffold cannot express.

## Candidate pipeline

```text
object/acoustic metadata
  -> declared dimensional scales
  -> frozen degree-two dimensionless modal scaffold

mesh + sparse contact observations
  -> deterministic graph-geodesic coverage certificate
  -> masked intrinsic diffusion/message-passing residual

global modes + contact gains
  -> deterministic modal renderer
  -> offline dry clip + provenance + fallback map

reject/OOD/missing evidence
  -> authored clip fallback
```

Acoustic material parameters remain synthesis-profile inputs, never physics
authority. Rolling, scraping, fracture, rooms, arbitrary force transfer and
radiation/listener fields remain separate claims.

## Evidence lanes

| Lane | Question | Output | Promotion rule |
| --- | --- | --- | --- |
| B — baseline | Does the unchanged low-order dimensionless scaffold survive fresh scales/objects? | `GlobalBaselineV0` | Opens O/F work; no real credit. |
| O — coverage | Can intrinsic geometry reject unsupported contacts without model confidence? | `CoverageCertificateV0` | Frozen input to F/I. |
| F — field | Does a graph operator beat compatible classical contact interpolation? | `FieldCapabilityV0` | Opens integration only after B/O pass. |
| I — integration | Do frozen B+O+F reproduce complete rendered endpoints? | `IntegratedCapabilityV0` | May open disclosed-real work if sources are ready. |
| S — sources | Are enough internet groups complete, fresh and role-eligible? | source/role certificates | Independent prerequisite; no user recording. |
| R/V/A | Does the pipeline improve disclosed audio and pass independent protected validation? | candidate, validator and admission certificates | One-shot shadow; rejection selects fallback. |
| C/D/P | Can admitted clips cook, demo and justify product promotion? | atlas/demo, then ADR | Blocked until admission. |

All lanes exchange immutable hashes and bounded records. Training state never
crosses into validator or protected roles.

## P0b protocol freeze

[P0b](../development/physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md)
is frozen before implementation. It defines:

- exactly `72` fresh B rows in Halton bands `801…824`, `901…924` and
  `1001…1024`, split into development, interpolation-test and
  scale-transfer-test roles with no complete V17 G overlap;
- the unchanged `72` V17 training rows, `11 -> 78` degree-two design,
  `lambda=1e-6`, 16 log-dimensionless targets and canonical serialized ridge;
- absolute mode/damping gates on the full test and on each test stratum,
  `72/72` material/support/scale mutation rejects, valid-OOD and exact repeat;
- predictor/scorer separation and zero access to every opened V17 G artifact;
- unchanged reuse of the V17 P0a O/F/I corpus, models, controls and gates.

The O/F/I reuse was admissible because those roles had never been implemented,
evaluated or opened. B0 was committed before opening fresh rows and passed
twice byte-exactly. O0 was then committed before its test-role evaluation but
rejected twice exactly; F0 and I0 therefore remain unopened and V18 closes.

## Milestones

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V18 rebaseline | `COMPLETE` | G0 evidence yields the deterministic-global/neural-surface split and explicit non-claims. |
| P0b | Hybrid protocol | `COMPLETE / FROZEN_BEFORE_IMPLEMENTATION` | Fresh B roles, exact ridge, inherited unopened O/F/I identities, gates, successful controls, access ledger and compute are frozen before code. |
| B0 | Deterministic global baseline | `COMPLETE / REPEAT_EXACT_PASS` | All 19 single-run gates and the full repeat gate pass: `4.434/15.891 cents`, `0.001226/0.004701` damping, `0/48` valid OOD and `72/72` mutations rejected. |
| O0 | Intrinsic coverage certificate | `COMPLETE / REPEAT_EXACT_REJECT` | Valid OOD, disconnection, ambient shortcut and utility pass, but thinning reaches only `65.02%` Plate and `53.73%` RolledSheet versus `>=95%`; `8/10` gates pass. |
| F0 | Masked intrinsic field operator | `NOT_RUN / CLOSED_BY_O0` | O0 prerequisite rejected; no model, seed, test prediction or parameter artifact exists. |
| I0 | Hybrid truth tournament | `NOT_RUN / CLOSED_BY_O0` | Integration identities remain sealed because the O0 prerequisite rejected. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each bounded source gains exact identity/member/axis/freshness evidence or a machine-readable closure reason. |
| S2 | Metal role freeze | `BLOCKED_BY_S1` | Five generator-ready and three fresh evaluation-complete groups satisfy unchanged `4/1/1/1/1`. |
| R1 | Disclosed-real tournament | `BLOCKED_BY_I0_AND_S2_GENERATOR_PREREQUISITE` | Frozen hybrid candidate beats real compatible controls without protected access. |
| V0/V1 | Independent validator | `SCAFFOLD_AFTER_I0 / RELEASE_BLOCKED_BY_R1_S2` | Calibrated ensemble/OOD policy freezes independently from generator training. |
| A0 | Metal shadow admission | `BLOCKED_BY_V1` | One untouched protected shadow is opened once and emits immutable Pass/Reject/FallbackOOD. |
| C0/D0 | Cooker/demo | `BLOCKED_BY_A0_PASS` | Byte-identical clips and fallback map feed one presentation-only demo prop through the existing audio path. |
| G1 | Wood then Glass | `AFTER_METAL_A0` | Each material repeats unchanged source, validator and admission policy; missing domains stay fallback. |
| N1 | Optional neural global residual | `DEFERRED / EVIDENCE_TRIGGERED` | A new preregistered family addresses a clustered disclosed-real residual on fresh data and beats B0 by a declared margin. |
| P1 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | A concrete consumer, protected evidence and separate Accepted ADR justify contracts and ProductChecks. |

## Closure at O0

O0 demonstrates that graph geodesics are the correct topology boundary:
valid-test false OOD is zero, every disconnected query rejects, RolledSheet
ambient shortcuts reject at `100%`, and intrinsic utility is `0.95061` versus
Euclidean `0.72063`. It also falsifies nearest-context distance as a complete
coverage certificate: thinning can reduce the context set fourfold while many
Plate/RolledSheet queries remain inside the ordinary valid fill radius.

No O0 threshold or mutation is repaired on opened rows. A successor requires a
new preregistered context-density/fill family and fresh test identities. Source
growth may continue independently, but no learned F0 or integrated I0 work is
authorized by V18.

## Stop rules

1. Do not retry the V17 global neural head, its opened rows or nearby
   capacity/seed/update/threshold variants.
2. B0 freezes the already declared ridge method before fresh verification; a
   failed B0 closes this scaffold rather than selecting another regression on
   its test.
3. O0, F0 and I0 execute only in dependency order. A rejection stops the
   downstream chain and records the fallback state.
4. F0 must beat compatible classical interpolation and non-graph attention;
   absolute pleasantness or one good WAV cannot substitute for the field gate.
5. Synthetic, disclosed-real and protected-admission evidence remain separate.
6. No local microphone/hammer capture, per-sound human queue, prompt-to-wave,
   raw PhysX callback mixing, runtime learning/inference or fallback removal.
7. Datasets, arrays, weights, reports, WAVs and caches remain external. Only
   small protocols, code, tests and evidence summaries enter Git.

## Commit sequence

1. `R0` — record G0 repeat-exact rejection and adopt V18.
2. `P0b` — freeze fresh B/O/F/I protocol and successful controls.
3. `B0a/B0b` — serialize, test and execute deterministic global baseline twice.
4. `O0a/O0b` — implement and execute intrinsic coverage twice.
5. `F0a/F0b` — implement and execute masked graph field twice.
6. `I0a/I0b` — integrate only exact passing component hashes and execute twice.
7. `S1a…` — continue bounded published-source adapters in parallel.
8. `S2/R1/V1/A0` — proceed only when capability and source prerequisites pass.
9. `C0/D0` — cook/demo only after one-shot admission pass.
10. `N1/P1` — revisit global neural residual or production only from new
    evidence and, for product promotion, a separate Accepted ADR.
