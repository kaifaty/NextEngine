# Physical sound V17 — factorized operator successor research

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `BOUNDED_RESEARCH_COMPLETE / ROADMAP_INPUT / NO_MODEL_SELECTED_BY_TEST` |
| Trigger | [V16 L0b](physical-sound-v16-l0b-known-truth-neural-oracle-result-2026-09-01.md) repeat-exact rejection after several earlier coordinate/residual variants |
| Question | What is the smallest falsifiable successor that addresses the observed global, surface and coverage failures independently? |
| Product effect | None; SPEC-45 remains `Proposed`, authored clips remain complete fallback and no real/protected role opens |

## Observed failure, not a generic ML verdict

V16 provides evidence for and against its representation:

- geometry-aware gain NRMSE `0.3489` beats the best classical control `0.6432`
  and equal-budget geometry-agnostic model `0.8120`;
- global frequency median/p95 `164.1/431.9 cents`, damping median/p95
  `0.1807/0.3098`, continuity `0.8592` and coverage-collapse rejection
  `52.08%` fail independently;
- two executions are byte-identical, so seed noise is not a viable explanation;
- training loss is low but failure clusters by unseen object, which is
  consistent with wrong inductive structure or insufficient object-level
  support rather than optimizer instability.

The next experiment must therefore separate three hypotheses. Widening the
same network would not discriminate them and is forbidden by the opened V16
test.

## Primary-source evidence

### H-G — scale-separated global modes

[DiffSound](https://hellojxt.github.io/DiffSound/) keeps modal sound inside a
differentiable physical pipeline with implicit shape, high-order finite-element
analysis and a differentiable synthesizer; its inverse tasks estimate physical
parameters, geometry and impact position through that structure. Similarly,
[NeuralSound](https://hellojxt.github.io/NeuralSound/) couples a learned sparse
3D representation to LOBPCG rather than replacing modal analysis with an
unstructured output regressor. These works support solver-shaped factorization,
not the claim that their exact implementations or datasets fit NextEngine.

The Journal of Computational Physics study on
[Buckingham-Pi physics-informed learning](https://www.sciencedirect.com/science/article/pii/S0021999122008737)
reports that dimensionless variables can turn a dimensional extrapolation into
interpolation under dynamic similarity and reduce the learned relation. That
directly matches V16's failure mode: the MLP received `log(L)`, `log(wall)` and
`log(wave_speed)` separately and had only nine object labels from which to
learn their multiplicative scale law.

**Hypothesis:** compute declared dimensional scales outside the network and
learn only ordered dimensionless modal ratios and damping multipliers. Do not
hard-code hidden topology eigenvalues or truth coefficients.

**Counterevidence/limit:** dimensionless scaling is helpful only inside the
declared similarity family. Real acoustic material proxies remain uncertain and
cannot become physics-material authority. A successful synthetic control still
earns only capability credit.

### H-F — intrinsic masked surface operator

[MeshGraphNets](https://arxiv.org/abs/2010.03409) reports that mesh-space
positions and edge message passing are crucial on irregular meshes; a larger
MLP-based graph baseline did not cure the missing relational structure. The
[DiffusionNet](https://arxiv.org/abs/2012.00888) experiments use learned
diffusion plus spatial gradients and remain comparatively stable under
remeshing/resampling. [GINO](https://arxiv.org/abs/2309.00583) and
[NORM](https://arxiv.org/abs/2302.08166) further show that function-to-function
operators can be defined across varying geometries/discretizations. These are
stronger priors for a surface gain field than one global mean/max context pool.

[Set Transformer](https://proceedings.mlr.press/v97/lee19d.html) establishes
attention as a permutation-invariant way to retain interactions among set
elements, but attention alone supplies no surface metric. It is therefore a
control/ablation, not the primary successor.

**Hypothesis:** inject `(observed-mask, signed modal gains)` at context vertices
and propagate them with a small intrinsic diffusion/message-passing operator;
decode every query vertex without collapsing context identity. Train an
explicit edge-gradient/continuity term in addition to pointwise gain error.

**Counterevidence/limit:** the cited surface/operator papers do not demonstrate
modal gain recovery from sparse impacts. Full GINO/NORM machinery is too large
for the first discriminator. The first candidate is a bounded masked diffusion
operator; cross-attention and larger neural operators remain preregistered
fallback hypotheses, not automatic retries.

### H-O — deterministic intrinsic coverage

The [Heat Method](https://www.cs.cmu.edu/~kmcrane/Projects/HeatMethod/) computes
single- or multi-source intrinsic distance on triangle meshes and explicitly
supports geodesic farthest-point sampling and Voronoi regions. V16 instead used
ambient Euclidean nearest-context distance. On a periodic Cylinder/Bowl, the
declared UV collapse can remain physically close across the seam, so requiring
`95%` rejection was partly a mutation-definition error rather than solely a
learned-uncertainty failure.

**Hypothesis:** coverage is a deterministic certificate based on the complete
multi-source intrinsic distance field, its maximum/quantiles, connected
components and boundary/topology descriptors. A collapse mutation must select
the intrinsically farthest region after contexts are fixed; it cannot use a UV
half-space that contradicts periodic topology.

**Counterevidence/limit:** intrinsic distance proves sampling support, not
semantic/material or model-error OOD. Ensemble disagreement and static-envelope
checks remain separate components; none may borrow protected outcomes.

## Decision

Adopt a factorized successor with four gates in order:

1. `G0` scale-separated global modes/damping;
2. `O0` model-independent intrinsic coverage certificate;
3. `F0` masked intrinsic surface operator;
4. `I0` integrated renderer/waveform tournament.

Each gate receives a new disjoint synthetic truth partition and its own simple
controls. A failed gate closes its hypothesis before integration. Only `I0`
repeat-exact pass can reopen disclosed-real R1, and source sufficiency remains
an independent prerequisite.

No external code, weights or datasets were copied or executed during this
research. The sources constrain hypotheses; they do not supply product
authority, redistribution permission or admission evidence.

## Smallest next action

Freeze `P0a`, a new unopened synthetic protocol with:

- enough cheap object-level samples to distinguish dimensional interpolation
  from extrapolation without reusing V16 object rows;
- separate mesh/contact partitions for the field operator;
- intrinsic, topology-valid coverage mutations;
- predeclared candidate/control budgets and two-run exact stop rules;
- zero real/source/protected access and no runtime/product effect.
