# Roadmap V23: closed-form learned field and complete evaluator boundary

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ADOPTED / V21_RESOURCE_REJECT / V22_IMPLEMENTATION_REJECT / QUALITY_UNOBSERVED / FIXED_FEATURE_RIDGE_SELECTED / P2A_FROZEN / E2A_IMPLEMENTATION_NEXT / FRESH_VALUES_SEALED / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V22](physical-sound-synthesis-roadmap-v22.md), closed by its single-run implementation-conformance reject before publication |
| Evidence basis | [V22 result](../development/physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md), [V23 research](../development/physical-sound-v23-closed-form-field-research-2026-09-01.md), [V21 result](../development/physical-sound-v21-f1a-continuous-field-result-2026-09-01.md), [P0d](../development/physical-sound-v21-p0d-counterfactual-owner-correction-protocol-2026-09-01.md) and [M0b](../development/physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, absent source, unsupported material or tooling failure |

## Outcome sought

Answer the still-unobserved contact-field question with a much smaller learned
method: deterministic fixed nonlinear features, closed-form ridge modal readout
and continuous topology-native residual. Eliminate long neural optimization and
prove the entire evaluator callable boundary before fresh values.

A repeat-exact synthetic pass only establishes bounded representation
feasibility. Real internet-source roles, independent validation, protected
admission, deterministic cooking and one demo prop remain separate later gates.

## Lessons carried forward

- V21 and V22 published no quality values. Their train/development roles are
  spent, but they cannot justify a quality-oriented hyperparameter choice.
- V21 disproved the many-small-call seven-network runner's 30-minute claim.
- V22 passed reference/batched numeric equivalence and stayed near the resource
  boundary, but failed because the wrapper omitted one evaluator callback.
- Numeric unit tests alone are insufficient: the whole evaluator must execute
  on a value-independent miniature fixture through corruption and serialization.
- F0 remains control-only; per-topology gradient stays `<=0.50`; P0d and M0b
  owner/metric corrections stay immutable.

## Technical strategy

`FixedFeatureRidgePriorV1` keeps P1a's 61 analytic, mesh-independent channels.
P2a will freeze a small deterministic sine/cosine random-feature bank and solve
eight modal readouts using equally view-weighted float64 ridge. Each query then
uses the unchanged independent q2/q3 continuous residual, exact context and C0
fallback.

The candidate grid must be small and fixed before values. It may vary only
feature dimension, bandwidth, readout ridge, basis order and residual ridge.
Every candidate is closed-form; there is no optimizer, epoch, checkpoint or
early stopping. Analytic-linear, prior-free continuous, exact F0, graph harmonic
and fixed RBF paths remain causal controls.

## Fresh role reservation

| Role | Reserved Halton indices | State |
| --- | --- | --- |
| train | `2601…2624` | `METADATA_ONLY / VALUES_SEALED` |
| development | `2701…2712` | `METADATA_ONLY / VALUES_SEALED` |
| one-shot test | `2801…2812` | `METADATA_ONLY / VALUES_SEALED` |
| reintegration | `2901…2912` | `METADATA_ONLY / VALUES_SEALED` |

P2a must freeze exact row roots. No V21/V22 row, model state, staging or internal
metric can be reused.

## Work packages and gates

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V22 attribution | `COMPLETE / IMPLEMENTATION_REJECT` | [Result](../development/physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md) records exact missing API, absent artifact, spent roles and unopened test/integration. |
| R1 | Smaller-family research | `COMPLETE / DIRECTION_SELECTED` | [Research](../development/physical-sound-v23-closed-form-field-research-2026-09-01.md) compares four hypotheses and selects fixed-feature ridge without quality inheritance. |
| P2a | F2 protocol and metadata freeze | `COMPLETE / FROZEN_BEFORE_VALUES` | [P2a](../development/physical-sound-v23-p2a-fixed-feature-ridge-protocol-2026-09-01.md) freezes fresh roots, eight closed-form candidates, unchanged gates, 5-minute budget and twice-exact full evaluator API smoke. |
| E2a | Closed-form runner and complete-boundary tests | `NEXT / VALUES_SEALED` | Metadata tests plus miniature no-F2-value tournament invoke every required API, corruption and serialization path; implementation is committed before fresh values. |
| F2a | Train/development tournament | `BLOCKED_BY_E2A` | Two runs finish within frozen resources, emit byte-identical complete trees and select one candidate passing all unchanged quality/remesh/control gates. Any reject closes F2 before test. |
| P2b | One-shot test freeze | `BLOCKED_BY_F2A` | Freeze winner/tree and unchanged test gates while `2801…2812` values remain unopened. |
| F2b | One-shot capability | `BLOCKED_BY_P2B` | Two fresh test runs pass every hard, quality, remesh, mutation, resource and byte-exact gate. |
| P2c | Reintegration freeze | `BLOCKED_BY_F2B_AND_P0D` | Freeze B0+C0+winner, `2901…2912`, corrected counterfactual owners and M0b gates. |
| I2 | Phase-consistent reintegration | `BLOCKED_BY_P2C` | Two byte-identical runs pass hard, physical, remesh, acoustic, counterfactual, access and resource gates. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Internet sources gain exact revision/member/axis/provenance evidence or machine-readable closure; no local recording. |
| M1–A0 | Metal generator, validator, holdout and shadow | `BLOCKED_BY_I2_AND_SOURCE_ROLES` | Preserve immutable `4/1/1/1/1`, independent validator and one untouched protected decision. |
| K0–D0 | Cooker and one demo prop | `BLOCKED_BY_A0_PASS` | Bake byte-identical clips with provenance and complete fallback, then use them presentation-only through existing audio. |
| G2 | Wood then Glass | `AFTER_METAL_A0` | Repeat source roles, validator and protected admission independently per material. |
| P3 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | Concrete consumer, admissible real evidence, measured Linux cost and separate Accepted ADR authorize a public/runtime contract. |

## Ordered implementation queue

1. Commit V22 result, V23 research and this roadmap.
2. Freeze P2a, exact fresh row roots, candidate grid, feature normalization,
   closed-form solves, complete API smoke, outputs and stop rules.
3. Implement E2a and run the full no-F2-value mini-tournament; commit before any
   `2601…2712` mesh or truth.
4. Run F2a A/B on train/development only. Stop on resource, exception,
   reproducibility or quality reject; never tune from the opened role.
5. On pass only, freeze/run one-shot test and reintegration in order.
6. Continue internet-source growth independently; generator, validator, holdout
   and protected roles stay disjoint and frozen in that order.
7. Cook and demonstrate only after protected Metal pass. Wood/Glass repeat the
   same admission or remain explicit fallback.

## Stop rules

1. Never repair/replay V21 or V22, inspect abandoned staging, reconstruct
   missing metrics or infer quality from elapsed time/exception position.
2. Do not introduce iterative neural training, GPU/compile backend, large graph
   operator or prompted waveform model inside F2.
3. Do not alter strict `0.50`, remesh `0.10`, acoustic, mutation, F0-control,
   access or byte-exact gates.
4. P2a's candidate grid and API smoke are immutable after implementation commit.
   Any F2a exception spends the role and closes the family.
5. No local microphone/hammer work, human per-sound approval queue, runtime ML,
   raw PhysX callback mixing or fallback removal.

## Definition of done

The vertical remains complete only after fresh F2/I2 synthetic evidence passes
twice exactly, disclosed internet sources satisfy immutable Metal roles, an
independent validator and untouched shadow pass, and accepted records cook
deterministic clips with complete fallback for a demo prop. Wood and Glass
repeat that admission independently or remain fallback-only. Runtime promotion
still requires a separate architecture decision.
