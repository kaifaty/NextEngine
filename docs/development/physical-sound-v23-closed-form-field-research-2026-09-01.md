# Physical sound V23 — closed-form field research

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `BOUNDED_RESEARCH_COMPLETE / FIXED_FEATURE_RIDGE_SELECTED / VALUES_UNOPENED` |
| Trigger | V21 resource reject plus [V22 implementation reject](physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md) left field quality unobserved after two coherent cycles |
| Scope | Select one genuinely smaller synthetic contact-field family; no threshold, real-data, test, integration, clip or runtime change |

## Falsifiable problem

The current question is not whether more neural capacity can fit the synthetic
truth. It is whether a small mesh-independent learned prior plus a continuous
residual can be evaluated honestly within the offline research envelope and
survive fresh remesh/quality gates.

V21 spent almost 30 minutes on seven 1,500-step networks and failed its resource
gate. V22 removed repeated small forwards, passed a strict numeric equivalence
fixture, then failed after roughly 26 minutes because its wrapper did not expose
one evaluator callback. Neither run published quality. A successor must reduce
algorithmic work rather than optimize the same long iterative loop again, and
must close the evaluator API before values.

## External evidence

- Rahimi and Recht map a shift-invariant kernel into an explicit low-dimensional
  random feature space so fast linear methods can replace a full kernel machine;
  they report competitive regression/classification speed and accuracy
  ([NeurIPS 2007](https://proceedings.neurips.cc/paper_files/paper/2007/hash/013a006f03dbc5392effeb8f18fda755-Abstract.html)).
- Avron et al. analyze random Fourier features for kernel ridge regression from
  a spectral approximation perspective and connect the approximation to
  statistical guarantees ([ICML 2017](https://proceedings.mlr.press/v70/avron17a.html)).
- Lingsch et al. show that a limited spectral basis can be evaluated directly on
  arbitrary non-equispaced point distributions, avoiding a regular-grid FFT
  assumption ([ICML 2024](https://proceedings.mlr.press/v235/lingsch24a.html)).
- DiffSound keeps modal physics explicit and differentiates a high-order finite
  element plus damped-oscillator synthesizer for inverse problems
  ([SIGGRAPH 2024](https://jiajunwu.com/papers/diffsound_siggraph.pdf)).
- NeuralSound demonstrates that learned acceleration can be useful for modal
  vibration/radiation, but its sparse-convolution, iterative eigensolver and GPU
  radiation scope is substantially larger than the present signed contact-field
  question ([paper](https://arxiv.org/abs/2108.07425)).

These sources support feasibility and decomposition choices, not a NextEngine
pass. The decision below is an inference combining them with local failures.

## Competing hypotheses

### H1 — another optimized neural tournament

Reduce width, steps or candidates and retain AdamW.

- For: closest to V21's intended nonlinear prior.
- Against: width/update/candidate changes would be selected from resource
  failures after two spent roles; iterative training remains the dominant cost,
  and quality was never observed to justify that complexity.
- Decision: reject for V23. Reconsider only if a closed-form family passes
  resources but causally fails fresh quality while a preregistered neural
  discriminator has a specific representational advantage.

### H2 — fixed nonlinear features plus closed-form ridge prior

Map P1a's analytic mesh-independent channels through a deterministic Fourier
feature bank, then solve the eight modal readouts with weighted ridge. Apply the
same topology-native continuous residual independently per view.

- For: nonlinear, resolution-independent prior; one bounded linear solve rather
  than 10,500 optimizer steps; direct UV evaluation; deterministic bytes; retains
  explicit modal gains and C0 fallback.
- Against: fixed features may underfit material/topology interactions; random
  feature approximation quality depends on dimension and regularization.
- Discriminator: a small preregistered dimension/bandwidth/ridge grid on fresh
  train/development, with exact analytic-linear and prior-free controls.
- Decision: select as `FixedFeatureRidgePriorV1`.

### H3 — large graph/geometric neural operator or GPU path

- For: strongest capacity for arbitrary meshes in cited operator work.
- Against: introduces new backend/hardware/determinism/resource confounds before
  the small-field hypothesis is answered; V21/V22 did not publish evidence that
  capacity was insufficient.
- Decision: reject. Reconsider only after F2 passes execution/reproducibility but
  fails a fresh, causally attributed representation gate.

### H4 — no learned prior, continuous residual only

- For: cheapest and maximally deterministic.
- Against: P1a already treats raw continuous interpolation as a compatibility
  control; promoting it would abandon the object/material prior hypothesis.
- Decision: retain as control, not the sole candidate.

## Selected family

`FixedFeatureRidgePriorV1` will:

1. reuse the exact 61 analytic P1a features and normalize them only by constants
   frozen before values;
2. add a fixed seeded sine/cosine feature bank with a small preregistered grid of
   dimensions and bandwidths;
3. fit all eight normalized modal-gain outputs using one equally view-weighted
   float64 ridge solve per candidate;
4. combine the prior with the exact q2/q3 topology-native residual and C0
   fallback from P1a;
5. retain exact F0, analytic-linear, prior-free continuous, graph-harmonic and
   fixed-kernel controls;
6. keep strict `0.50`, remesh, mutation, F0 non-regression, byte-exact and
   independent later test/integration gates unchanged.

P2a must bound feature dimension, condition number, model bytes, candidate count
and a much shorter resource ceiling before implementation. It must also freeze a
closed-world evaluator API manifest: every required symbol/signature is listed,
and a synthetic no-F2-value smoke invokes the complete tournament through
structural mutation and serialization boundaries.

## Rejected shortcuts

- No V21/V22 role reuse, exception repair/retry, reconstructed staging or
  threshold inference.
- No prompted waveform generator, listening selection, local microphone or
  generator-as-validator.
- No GPU/compile backend, stochastic training, mixed precision, model checkpoint
  or runtime inference.
- No claim that synthetic ridge recovery establishes real material acoustics;
  disclosed internet evidence and independent protected admission remain
  separate.

## Smallest next action

Create Roadmap V23, then freeze P2a on fresh metadata bands with exact random
feature construction, candidate grid, ridge weighting, controls, API smoke,
gates, outputs and stop rules before any new mesh or truth value.
