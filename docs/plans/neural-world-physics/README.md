# Neural-assisted world-physics roadmap

| Field | Value |
|---|---|
| Status | `PLANNED / NOT_ACTIVE` |
| Architecture | [SPEC-44](../../architecture/44-neural-assisted-world-simulation.md), [ADR-080](../../architecture/adr/080-neural-assistance-as-bounded-proposals.md) |
| Model/data authority | [SPEC-34](../../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-053](../../architecture/adr/053-engine-native-model-training-and-immutable-artifact-boundary.md) |
| Current checkpoint | `N0 ARCHITECTURE COMPLETE / N1 TARGET NOT_SELECTED` |
| Activation gate | One promoted classical owner plus `WORLD-NEURAL-DATAPLANE-P1` and `WORLD-NEURAL-SHADOW-P1` |
| v1 impact | None; optional post-v1 performance research |

This roadmap cannot select a solver authority or repair an incomplete physical
profile. It starts only after a classical owner has reference correctness,
exact persistence and a representative production performance baseline.

## Critical path

```text
N0 Proposal-only architecture (complete)
 └─ N1 Select promoted owner + measured bottleneck
     └─ N2 Hash-closed instrumentation and dataset generation
         └─ N3 Report-only model and shadow evaluation
             └─ N4 Exact non-regression + cross-target evidence
                 └─ N5 End-to-end performance gate
                     └─ N6 Optional runtime promotion
```

No neural package is on a classical owner's critical path. A water warm start,
for example, begins only after the independent W1-W6 program and does not
change CPU authority or GPU correspondence status.

## Package status

| Package | Status | Exit |
|---|---|---|
| N0 Architecture | `COMPLETE` | Model is stateless bounded advice; owner, correction, no-retry, persistence and fallback rules are explicit. |
| N1 Target selection | `BLOCKED ON EVIDENCE` | One promoted owner exposes a stable iterative bottleneck and frozen baseline/corpus. |
| N2 Data plane | `NOT_STARTED` | `WORLD-NEURAL-DATAPLANE-P1 = PASS`; teacher and split lineage are exact. |
| N3 Shadow proposal | `NOT_STARTED` | `WORLD-NEURAL-SHADOW-P1 = PASS`; model remains report-only. |
| N4 Non-regression | `NOT_STARTED` | Exact roots/failures and Windows/Linux applied proposals pass required continuations. |
| N5 Performance | `NOT_STARTED` | `WORLD-NEURAL-PERFORMANCE-P1 = PASS` with inference/gate/correction costs included. |
| N6 Promotion | `NOT_STARTED` | Production consumer and Accepted successor ADR introduce only the needed current contracts. |

## N1 selection criteria

Select no target until all are true:

- classical reference, coupling and exact persistence checks pass;
- the production solver has a frozen authority/numeric/schedule profile;
- instrumentation identifies one repeated cost rather than an assumed model
  opportunity;
- the proposal has a bounded fixed-point shape and deterministic default;
- a shadow evaluator can compare exact roots and failure classes;
- end-to-end THOTH budget can show material value after inference overhead.

Prefer the smallest warm-start or diagnostic proposal. Learned forces,
constitutive laws, topology, representation selection and end-to-end world
evolution are excluded from N1.

## Evidence rules

- Dataset generations bind exact owner, profile, teacher, project, numeric,
  input and result roots. Changing any component creates a new generation.
- Stable scenario groups fix train/evaluation splits before training.
- Heavy datasets, checkpoints and runs stay in the external training store.
- Model bundles are immutable and hash-bound; runtime never trains or updates
  weights.
- Missing/invalid output is rejected before solve and uses the same classical
  default initialization. Failure after an admitted proposal rejects the
  step; it is not retried from default.
- Shadow tolerance metrics are diagnostic. Runtime promotion requires exact
  canonical roots and failure classification.
- Inference readiness, worker order and GPU completion cannot select an
  authoritative branch.

## Stop condition

After two evidence-backed optimization cycles without exact non-regression and
a predeclared end-to-end p95/p99 improvement, stop the branch in research
status. Do not reduce the product workload, accept approximate roots, enlarge
the budget or grant model/GPU authority without a separate decision.

## Promotion boundary

N0/N1 diagnostics add no runtime schema. N2/N3 are external lab work. N6 may
introduce a proposal/profile/model-lane contract only when a production
consumer exercises it. The authoritative checkpoint remains the classical
owner's segment and restore without the optional bundle must remain correct.
