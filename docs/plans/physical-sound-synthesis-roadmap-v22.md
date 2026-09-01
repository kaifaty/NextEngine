# Roadmap V22: resource-bounded continuous physical-sound tournament

| Field | Value |
| --- | --- |
| Rebaseline date | `2026-09-01` |
| Status | `ADOPTED / V21_F1A_RESOURCE_REJECT / QUALITY_UNOBSERVED / P1R_PROTOCOL_NEXT / FRESH_VALUES_SEALED / SOURCE_GROWTH_PARALLEL / ADMISSION_BLOCKED / RUNTIME_NOT_AUTHORIZED` |
| Replaces | [Roadmap V21](physical-sound-synthesis-roadmap-v21.md), closed before artifact publication by its F1a 30-minute resource gate |
| Evidence basis | [F1a result](../development/physical-sound-v21-f1a-continuous-field-result-2026-09-01.md), [P1a](../development/physical-sound-v21-p1a-continuous-residual-field-protocol-2026-09-01.md), [F0 conformance audit](../development/physical-sound-v21-f0-protocol-conformance-audit-2026-09-01.md), [P0d](../development/physical-sound-v21-p0d-counterfactual-owner-correction-protocol-2026-09-01.md) and [M0b result](../development/physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Mandatory fallback | Existing authored/recorded clip for every reject, OOD, absent source, unsupported material or tooling failure |

## Outcome sought

Recover the unanswered V21 scientific question without learning from its spent
roles: can the frozen small learned prior plus continuous topology-native
residual recover signed contact fields consistently across valid remeshes?

V22 changes execution shape, not scientific scope. It first proves that a
batched/precomputed implementation is algebraically faithful on synthetic
metadata-only controls, then runs the unchanged six-candidate tournament on
fresh identities. Only a repeat-exact pass may open one-shot test and
reintegration. Real-data admission, automatic validation and deterministic clip
cooking retain their prior independent gates.

## What V21 established

- Protocol, implementation and metadata tests were committed in the required
  order; the strict per-topology gradient gate remains `<=0.50`.
- Official F1a run A exceeded `1,800 s` before publication. Staging was removed,
  no quality metric or model artifact was exposed, and run B was not started.
- The observed worker remained CPU-active at roughly `1.0 GiB RSS`; elapsed time,
  not memory, was the blocking gate.
- Bands `1801…1912` are spent. Test/integration `2001…2112` remain unopened but
  belong to the closed V21 protocol and are not inherited by V22.

## Immutable scientific boundary

P1r must copy from V21 P1a without semantic change:

- `ContinuousResidualOperatorV1`, six exact `(q,lambda)` candidates;
- `61 -> 128 -> 128 -> 128 -> 8` float64 prior, seeds `210001/210002`;
- analytic mesh-independent features and topology-native tensor basis;
- equally weighted views, differentiable Cholesky residual, 49 pair probes;
- full-batch AdamW, exactly 1,500 updates and the same cosine schedule/loss;
- exact F0 and paired-harmonic controls, compatible diagnostics and deterministic
  remesh-first selection;
- every absolute, ratio, mutation, coverage, remesh, serialization, access,
  30-minute, 4-GiB and 100-MiB gate, including topology `<=0.50`;
- nine-file atomic external output and complete authored-clip fallback.

V22 may not prune candidates, reduce updates, subsample rows/vertices/probes,
change arithmetic precision, alter thresholds, inspect V21 internal state or
use any quality observation from spent roles. There is no V21 quality
observation to inherit.

## Allowed execution-only successor

P1r may freeze these bounded transformations before new meshes or values:

1. build analytic vertex/probe features, bases, penalties, Cholesky factors,
   active edges and slice ledgers once outside the update loop;
2. concatenate vertex features into one prior forward per optimizer update,
   then restore exact view slices before equally weighted losses and solves;
3. concatenate canonical-probe features into one prior forward per update while
   retaining one independently solved coefficient matrix per view;
4. retain candidate isolation, identical initialization, optimizer state,
   update order and float64 deterministic CPU execution;
5. prove batched versus reference forward outputs, scalar loss and parameter
   gradients on a small handcrafted finite fixture before implementation commit.

The equivalence fixture contains no F1 mesh, row or truth value and cannot tune
the model. Exact equality is required where operation order is unchanged;
otherwise P1r must preregister a bounded float64 tolerance before the fixture is
run. Official output reproducibility remains byte-exact.

## Fresh role reservation

P1r owns new disjoint bands and must freeze exact row roots before generating a
mesh or truth value:

| Role | Reserved Halton indices | State |
| --- | --- | --- |
| train | `2201…2224` | `METADATA_ONLY / VALUES_SEALED` |
| development | `2301…2312` | `METADATA_ONLY / VALUES_SEALED` |
| one-shot test | `2401…2412` | `METADATA_ONLY / VALUES_SEALED` |
| reintegration | `2501…2512` | `METADATA_ONLY / VALUES_SEALED` |

The V21 row construction, primary/twin ordering and context formulas remain the
template, but hashes and exact identities belong to P1r and cannot be inferred
as capability evidence from this planning document.

## Work packages and gates

| ID | Package | State | Observable exit criterion |
| --- | --- | --- | --- |
| R0 | V21 resource attribution | `COMPLETE / NO_QUALITY_OBSERVED` | [F1a result](../development/physical-sound-v21-f1a-continuous-field-result-2026-09-01.md) records the single resource reject, absent outputs, spent roles and elapsed-time attribution. |
| P1r | Execution-equivalent successor protocol | `NEXT / VALUES_SEALED` | Freeze fresh row roots, exact equivalence fixture/tolerance, batched computation, unchanged P1a science, outputs, gates and stop rules before code may create a V22 mesh/value. |
| E1r | Batched runner and metadata/equivalence tests | `BLOCKED_BY_P1R` | Focused tests prove identity guards, sealed roles, resource accounting and reference/batched forward-loss-gradient agreement; implementation is committed before fresh values. |
| F1r | Train/development tournament | `BLOCKED_BY_E1R` | Two independent runs finish under every resource ceiling, emit byte-identical nine-file trees and select one candidate passing every unchanged P1a development gate. Any resource or quality reject closes the family before test. |
| P1b | One-shot test freeze | `BLOCKED_BY_F1R` | Freeze winner/model/tree identities and unchanged test gates while `2401…2412` values remain unopened. |
| F1b | One-shot capability | `BLOCKED_BY_P1B` | Two executions on fresh test pass every unchanged hard, quality, remesh, corruption, resource and byte-exact gate. Reject closes the family. |
| P1c | Reintegration freeze | `BLOCKED_BY_F1B_AND_P0D` | Freeze exact B0+C0+winner identities, `2501…2512`, corrected counterfactual owners and M0b gates. |
| I2 | Phase-consistent reintegration | `BLOCKED_BY_P1C` | Two byte-identical runs pass hard, physical, remesh, acoustic, counterfactual, access and resource gates. |
| S1 | Published-source growth | `PARALLEL / SOURCE_INSUFFICIENT` | Each internet source gains exact revision/member/axis/provenance evidence or a machine-readable closure reason; no local recording. |
| M1–A0 | Metal generator, validator, holdout and shadow | `BLOCKED_BY_I2_AND_SOURCE_ROLES` | Preserve immutable `4/1/1/1/1` source roles, independent validator and one untouched protected decision. |
| K0–D0 | Deterministic cooker and one demo prop | `BLOCKED_BY_A0_PASS` | Bake byte-identical dry clips with provenance and complete fallback, then use them presentation-only through the existing audio path. |
| G2 | Wood then Glass | `AFTER_METAL_A0` | Repeat source roles, validator and protected admission independently for each material. |
| P2 | Production promotion | `POST_RESEARCH / ADR_REQUIRED` | Concrete consumer, admissible real evidence, measured Linux budget and a separate Accepted ADR authorize any public/runtime contract. |

## Ordered implementation queue

1. Commit this roadmap and the V21 resource-reject evidence.
2. Freeze P1r, row roots and equivalence contract before generating any V22
   mesh or truth value.
3. Implement batching/precomputation plus metadata-only/equivalence tests and
   commit that boundary.
4. Run F1r A/B on train/development only. Close the continuous family on any
   resource, reproducibility or quality reject; do not optimize from values.
5. On pass only, freeze P1b and run the one-shot test twice; then freeze P1c
   and run reintegration twice.
6. Continue internet-source growth independently. Do not open real generator,
   validator, holdout or protected roles before their immutable certificates.
7. Bake and demonstrate only after one protected Metal `Pass`; authored clips
   remain the authority for every other state.

## Stop rules

1. Never reuse V21 `1801…2112`, inspect abandoned staging, reconstruct missing
   F1a metrics or present the resource reject as evidence for/against quality.
2. Do not reduce 1,500 updates, remove candidates/views/probes, change float64,
   loosen `0.50`, `0.10`, acoustic or resource thresholds, or select an
   optimization using V22 quality values.
3. If F1r misses the 30-minute gate, close this execution family. A smaller F2
   model requires fresh research, protocol and identities, not another nearby
   performance retry.
4. If F1r or F1b fails quality, do not choose another seed, basis, lambda,
   checkpoint, contact or threshold from the opened role.
5. Do not let prompted audio, training loss or the generator itself serve as
   the independent validator.
6. No local microphone/hammer work, per-sound human approval queue, runtime
   learning/inference, raw PhysX callback mixing or fallback removal.

## Definition of done

The research-to-product vertical remains complete only when fresh F1/I2
synthetic evidence passes twice exactly, disclosed internet sources satisfy
immutable Metal roles, an independent validator and one untouched shadow pass,
and the accepted record cooks deterministic clips with complete fallback for a
demo prop. Wood and Glass repeat the same admission discipline or remain
fallback-only. None of those outcomes implies runtime ML; promotion still
requires a separate architecture decision.
