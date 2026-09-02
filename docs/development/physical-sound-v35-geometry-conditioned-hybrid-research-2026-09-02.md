# Physical sound V35 — geometry-conditioned hybrid successor research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / HYPOTHESIS_SELECTED / NO_NEW_ORACLE_MODEL_OR_REAL_VALUES_OPENED` |
| Trigger | [V34 H0](physical-sound-v34-h0-method-holdout-result-2026-09-02.md) geometry-only contact is `1.756639x` nearest |
| Selected successor | `coverage-gated-geometry-contact-hybrid-v1` |
| Claim | Bounded method selection only; no synthetic admission, real-material quality, validator, cooker, runtime or ProductCheck credit |

## Question

Why does the V34 spectral candidate beat every aggregate, new-contact and joint
control yet lose unseen-geometry/known-contact transfer to nearest retrieval,
and what is the smallest fresh experiment that can distinguish a missing
geometry representation from the legitimate advantage of local interpolation?

This is the required bounded research escalation after V32 and V34 both lost a
contact comparison to nearest retrieval. It does not reopen either spent role
and cannot select a seed, width, basis, step count, loss or threshold for V34.

## Exact local evidence

V34 H0 is a repeat-exact scientific reject, not a protocol or resource fault:

- all nine P1 hard gates and every resource gate pass;
- aggregate, decay, global gain, contact aggregate, contact-only and joint
  comparisons pass;
- geometry-only contact is `0.001173994` for the candidate versus
  `0.000668319` for nearest, a `1.756639x` ratio against `<=0.98x`;
- the same nearest control is much weaker on aggregate contact (`0.02116958`),
  so the advantage is localized to known contacts on unseen geometries;
- V34's contact head sees modal coordinates, family/support, contact `u/v`, P1
  participation, fixed Fourier coordinates and a local P1 stencil. It does not
  see the three explicit geometry multipliers used by the gain head.

The missing geometry coordinates are a plausible cause, not a proven one. The
disclosed V34 synthetic contact oracle can be written entirely from fields
already present in the contact input. A sufficiently trained global model could
therefore represent it. The result instead shows that this global
parameterization does not generalize as well as local reuse in one fresh
stratum. V35 must test locality and geometry conditioning separately.

## Primary prior art inspected

1. Jin et al., [NeuralSound: Learning-based Modal Sound Synthesis With Acoustic
   Transfer](https://arxiv.org/abs/2108.07425), revised 2022. NeuralSound learns
   from object surface vibration and predicts far-field acoustic-transfer maps;
   it does not ask a coordinate-only MLP to recover all geometry dependence.
   The bounded lesson is to expose geometry/field structure while retaining the
   modal solver, not to adopt its sparse 3D and radiation networks now.
2. Jin et al., [DiffSound: Differentiable Modal Sound Rendering and Inverse
   Rendering for Diverse Inference Tasks](https://arxiv.org/abs/2409.13486),
   SIGGRAPH 2024. DiffSound binds implicit shape, high-order FEM and a
   differentiable modal synthesizer, and evaluates geometry and impact
   inference. It supports treating shape and contact as causal inputs. It does
   not prove that the V35 synthetic teacher represents real Steel or Glass.
3. Lu, Jin and Karniadakis, [DeepONet: Learning nonlinear operators for
   identifying differential equations](https://arxiv.org/abs/1910.03193),
   Nature Machine Intelligence 2021. Its branch/trunk split separates an input
   function from output coordinates and reports lower generalization error than
   fully connected baselines in its PDE studies. It supports an operator-shaped
   escalation if compact geometry/local evidence fails, but it is larger than
   the present discriminator and optimization/generalization are not guaranteed
   by the approximation theorem.
4. Moscovich, Jaffe and Nadler, [Minimax-optimal semi-supervised regression on
   unknown manifolds](https://proceedings.mlr.press/v54/moscovich17a.html),
   AISTATS 2017. The paper proves an optimal finite-sample bound for a geodesic
   k-nearest-neighbor regressor under its manifold assumptions. Those
   assumptions are not claimed for our corpus; the bounded counterexample is
   enough: a strong local retrieval control is scientifically credible and
   should become part of the candidate rather than be dismissed as a toy.

No external source code, model, dataset, checkpoint or audio was downloaded or
executed. The architectural conclusion below is an inference from these papers
and the exact local V34 result.

## Competing hypotheses

| ID | Hypothesis | Evidence for | Evidence against | Fresh discriminator |
| --- | --- | --- | --- | --- |
| H1 | Explicit physical geometry coordinates are missing from the contact head. | The failure appears only when geometry changes; V34 omits the geometry vector from this head; NeuralSound and DiffSound condition on shape/field structure. | Every disclosed oracle term is recoverable from existing modal/stencil inputs, so geometry is not mathematically absent. | Add frozen dimensionless geometry coordinates while keeping P1, optimizer and other heads fixed; require a geometry-removal ablation. |
| H2 | Contact transfer is locally smooth and a local interpolator is the right inductive bias. | Nearest wins only for known-contact unseen geometry; local-regression prior art makes this plausible. | One nearest sample is noisy/discontinuous and fails badly for new contacts. | Compare nearest, fixed continuous local interpolation and the neural expert in all three transfer strata. |
| H3 | A value-independent coverage gate can combine local and global experts. | Local retrieval and spectral ML have complementary failure clusters: geometry-only versus new-contact/joint. | A gate could merely infer benchmark strata and hide two weak models. | Gate may use only continuous train-support distances, never role/stratum IDs or targets; require continuity, permutation and boundary probes plus both-expert ablations. |
| H4 | V34 only needs more capacity, training or another seed. | The oracle is representable and the absolute error is small. | The proposal is selected from spent H0 values, does not explain nearest's localized advantage and would be an unfalsifiable nearby retry. | Rejected without execution. |
| H5 | A full mesh neural operator is already required. | NeuralSound, DiffSound and DeepONet show appropriate large-scale representations. | P1 already supplies exact modes and local fields; V34 fails one narrow residual stratum. Cost and attribution would expand sharply. | Defer unless the compact hybrid fails fresh roles while structural and local controls pass. |
| H6 | The geometry-only split can be solved by exact-contact detection. | It would route the failed stratum to retrieval. | It encodes the benchmark partition, is discontinuous and would not generalize to continuous real impact locations. | Explicitly forbidden; gate probes must include near-but-not-equal contacts and lattice perturbations. |
| H7 | Synthetic work is no longer the critical bottleneck. | Real role power remains six exact-Steel and 27 non-Metal groups short. | A real lane cannot qualify a generator family that already fails known truth. | Continue source scouting independently; require both H0 and real-role power before real training. |

## Decision

Select one `coverage-gated-geometry-contact-hybrid-v1` family. P1 remains the
sole owner of frequency, modal order, nodes, signs, impulse scaling, support and
remesh identity. V34's successful decay and global-gain heads remain unchanged.
Only the bounded contact residual changes:

```text
P1 modal/local field + explicit dimensionless physical geometry
  -> compact spectral neural expert

train-only family/support/material/geometry/contact/modal keys
  -> fixed continuous local interpolator

continuous target-free train-support distances
  -> deterministic coverage gate

gate * local expert + (1 - gate) * neural expert
  -> bounded contact multiplier -> unchanged P1 composition
```

F0 must freeze the exact causal key, normalization, neighborhood rule, kernel,
neighbor count, zero-distance behavior and gate before official targets exist.
Bandwidths and support radii may be derived only from train geometry/contact
coordinates by a declared deterministic rule. They may not be selected using
train targets, development metrics or knowledge of role/stratum membership.

The candidate is hybrid by design: retrieval handles regions with close
structural support, while the neural expert provides a smooth learned prior
away from that support. It is not allowed to hard-code exact contact equality,
object IDs, geometry-cell IDs, role labels, stratum labels or oracle
coefficients. The local expert must be continuous and permutation-invariant
over the train set. OOD remains a terminal fallback, not extrapolation by fiat.

## Mandatory controls and ablations

The one fresh tournament compares the candidate against:

- unchanged P1 identity;
- one-nearest train row;
- a frozen continuous non-neural local interpolator;
- raw ridge and spectral ridge;
- the V34-shaped spectral MLP with no explicit geometry or local expert;
- the geometry-conditioned neural expert alone;
- the hybrid without geometry, without the local expert and without the neural
  expert as preregistered causal ablations.

The candidate must beat the best non-neural control, not merely old V34. A
passing aggregate cannot hide a failed geometry-only, contact-only or joint
stratum. F0 freezes minimum directional ablation margins; C0 proves each expert
is structurally reachable in every evaluation role before targets open.

## Fresh-evidence and anti-leakage rules

- V32/V33/V34 train, development and holdout identities, oracle coefficients,
  targets, predictions, weights and metrics are forbidden inputs.
- V35 freezes fresh material constants, geometry cells, contact sets, truth
  coefficients and train/development/method-holdout commitments.
- The three evaluation strata remain unseen-geometry/known-contact,
  known-geometry/unseen-contact and joint transfer, but no model or gate input
  may expose those names.
- Local interpolation for train-time losses uses leave-one-case-group-out
  predictions; a row may never retrieve itself or another remesh view of the
  same case.
- C0 may inspect only structure and P1 witnesses. The complete owner must prove
  target access receipts and atomic terminal publication before D0.
- D0 opens once. Only a complete pass freezes one candidate and authorizes H0.
  H0 opens once without retraining. Either reject permanently closes V35.

## Falsification and stop rule

V35 succeeds only if the hybrid beats nearest and the frozen continuous local
control on geometry-only contact while retaining V34's gains on contact-only,
joint, aggregate, decay and global gain. All P1 hard gates, coverage-gate
continuity/provenance, repeat-exact and resource bounds also pass.

Interpretation is preregistered:

- geometry expert passes and hybrid fails: coverage gate/local composition
  closes; do not tune the gate on opened evidence;
- local control wins: the neural hybrid closes and deterministic local
  interpolation becomes the next scientific baseline, not a failed ML result
  relabeled as success;
- all compact methods fail: escalate only under a new roadmap to a topology or
  operator representation such as a surface-field branch/trunk model;
- D0 passes but H0 fails: close all V35 candidates and roles; no retry;
- H0 passes: this is synthetic method evidence only. Real training remains
  blocked until the independent internet-source role and validator gates pass.

## Smallest next action

Freeze the V35 F0 profile: fresh roles/truth, exact continuous local expert and
coverage gate, unchanged successful branches, candidate/control identities,
witness and anti-leakage predicates, metrics, thresholds and resource bounds,
without materializing any official target or training a model.
