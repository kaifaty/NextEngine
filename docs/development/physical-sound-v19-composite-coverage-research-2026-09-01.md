# Physical sound V19 — composite coverage research

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / SUCCESSOR_SELECTED` |
| Trigger | [V18 O0](physical-sound-v18-o0-intrinsic-coverage-result-2026-09-01.md) repeat-exact thinning rejection |
| Decision | Separate structural observation completeness, set-level intrinsic fill and local graph reachability before learned field inference |
| Product effect | None; SPEC-45 remains `Proposed`, runtime ML is unauthorized and authored clips remain the complete fallback |

## Question

V18 O0 proved that graph geodesics correctly distinguish a disconnected or
ambiently close surface from an intrinsically supported contact. It also proved
that a per-query nearest-context distance is not a complete coverage
certificate: fourfold thinning still left many Plate and RolledSheet queries
inside the calibrated valid radius.

The successor question is therefore narrow and falsifiable:

> Can a deterministic certificate reject incomplete or spatially inadequate
> context sets while preserving valid contacts, before any learned field model
> sees the input?

This research does not reopen O0 thresholds, select from its test rows or claim
that synthetic modal fields equal real material acoustics.

## Competing hypotheses

| ID | Hypothesis | Evidence that would support it | Main risk |
| --- | --- | --- | --- |
| H-count | Thinning is first an input-integrity failure: the received unique context set no longer satisfies its declared minimum observation budget. | A fixed count/identity contract rejects thinning without inspecting target values or test outcomes. | Count alone cannot detect clustered, disconnected or ambient-shortcut samples. |
| H-fill | O0 used the wrong aggregation: set-level intrinsic covering radius and separation expose a sparse sample set even when individual query distances overlap the valid distribution. | Development-only valid and thinned fill/mesh-ratio ranges are disjoint; fresh test retains the separation. | A geometry-only threshold may vary with mesh resolution/topology and still cannot prove source-member completeness. |
| H-spectral | A sampling set is adequate only relative to the bandwidth/uniqueness of the graph signal; spectral proxies should replace distance statistics. | A frozen proxy predicts recoverability and beats simpler controls on fresh field truth. | The eight modal-gain fields have not yet been shown graph-bandlimited; adding an eigensolver now would import an unsupported assumption and more complexity. |

These hypotheses are complementary at different boundaries. H-count is a
record-integrity invariant; H-fill is a spatial-support invariant; H-spectral is
a later signal-recovery hypothesis.

## Evidence

### Exact predecessor result

O0 passed valid false-OOD, component isolation, ambient shortcut, intrinsic
utility and exact-repeat gates. It failed only thinning-sensitive gates:
Plate rejected `65.0206%` and RolledSheet `53.7255%` versus `>=95%`, and only
`7/12` objects reached the per-object requirement. The opened O0 test rows are
diagnostic evidence only and cannot validate V19.

### Development-only counterfactual

The committed O0 development meshes (`n=501…512`) were recomputed without
reading O0 test values, external artifacts, real audio or protected roles. For
the valid FPS context and the frozen `context[::4]` thinning control:

| Statistic | Valid range | Thinned range | Observation |
| --- | ---: | ---: | --- |
| unique context count | `32…39` | `8…10` | Structural budget loss is explicit. |
| intrinsic global fill / graph diameter | `0.072246…0.129602` | `0.236748…0.459716` | The set-level maximum separates this development control. |
| intrinsic mesh ratio `fill / (separation/2)` | `1.8927…2.0000` | `4.2514…8.0592` | Thinning preserves some local proximity but degrades set uniformity. |

This is a successful development control, not a frozen test gate. V19 must
calibrate any numerical fill limit only on new development identities, commit
the implementation, then open a disjoint test once.

### Primary-source constraints

- Gonzalez's metric clustering construction selects the farthest point and
  gives a factor-two approximation under the triangle inequality. It supports
  farthest-first sampling as a bounded covering construction, but does not make
  the chosen sample cardinality optional: the result is stated for a fixed
  `k`. See [Gonzalez 1985](https://www.cs.columbia.edu/~verma/classes/uml/ref/clustering_minimize_intercluster_distance_gonzalez.pdf).
- Quasi-uniform design theory distinguishes fill distance from separation
  radius and uses their ratio to characterize a point set. That supports
  reporting both coverage and clustering rather than treating nearest distance
  as the whole certificate. See [Pronzato and Zhigljavsky](https://arxiv.org/abs/2112.10401).
- Graph-signal sampling theory permits perfect recovery only under a declared
  bandlimit and a suitable sampling operator. It does not justify a spectral
  gate before the target fields satisfy that assumption. See
  [Chen et al.](https://arxiv.org/abs/1503.05432).
- Graph spectral proxies can avoid an explicit frequency-basis computation and
  provide stable reconstruction for noisy or approximately bandlimited signals,
  but they remain a field-recovery method with a bandlimit premise. See
  [Anis, Gadde and Ortega](https://arxiv.org/abs/1510.00297).

## Decision

V19 will implement `CompositeCoverageCertificateV0` with three ordered layers:

1. **Structural closure.** Validate mesh identity, unique/in-range context
   indices, declared expected/minimum context count and exact record/hash
   closure. Failure returns `OOD_CONTEXT_BUDGET` before numerical inference.
2. **Set-level intrinsic coverage.** Measure normalized graph-geodesic covering
   radius over the complete mesh. Record intrinsic separation and mesh ratio as
   diagnostics. Failure returns `OOD_INTRINSIC_FILL`.
3. **Local intrinsic reachability.** Preserve O0's per-query geodesic distance,
   unreachable-component rejection and ambient-distance negative control.
   Failure returns `OOD_INTRINSIC_GAP` or `OOD_DISCONNECTED`.

The certificate is deterministic and model-independent. A learned surface
operator may run only after all three layers accept. Count-only, raw O0
distance, global-fill-only, Euclidean and optional spectral-proxy variants are
controls; none may silently replace the frozen composite after test opening.

## Why not jump directly to a neural validator

This failure is declared-data integrity plus geometric support, not perceptual
quality. Training a classifier to rediscover missing members would add
calibration, OOD and lineage risks while weakening an exact invariant. Neural
models remain useful for the harder signed modal-gain field and later acoustic
validation, where deterministic rules do not already settle the question.

## Complexity and bounds

- structural validation is `O(K)` time and `O(K)` bounded identity storage;
- multi-source graph fill is `O((V+E) log V)` with the current sparse Dijkstra
  implementation after graph construction;
- separation is `O(K^2)` over the small frozen context budget and is diagnostic;
- a dense spectral decomposition would be roughly `O(V^3)` and is therefore
  deferred until a field-bandlimit experiment justifies it.

All arrays, geometry dumps, scores, reports, checkpoints and audio stay under
the external experiment root. Git receives only protocol, small code/tests and
bounded evidence summaries.

## Rejected alternatives

- Lowering or raising the opened O0 threshold, percentile or thinning factor.
- Treating context count alone as spatial support.
- Treating global fill alone as source-member completeness.
- Promoting mesh ratio from one development diagnostic directly to a test gate.
- Introducing graph spectral proxies before a bandlimit/control experiment.
- Using O0 test identities, per-object thresholds or test-selected reason codes.

## Smallest next action

Freeze P0a with new coverage identities, exact structural metadata, successful
malformed-input controls, development-only fill calibration, immutable reason
codes and one-shot test gates. Only a repeat-exact C0 pass can unseal a new F0
field protocol.
