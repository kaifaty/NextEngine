# Physical sound: neural acoustic field strategy

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Status | `ROADMAP_V4 / R2_LISTENER_FIELD_REJECTED / GEOMETRY_AWARE_MODAL_CONTACT_FIELD / R3A_NEXT / RESEARCH_ONLY / NO_RUNTIME_MODEL` |
| Architecture boundary | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Roadmap | [Physical sound synthesis roadmap](../plans/physical-sound-synthesis-roadmap.md) |
| Implementation plan | [Neural acoustic field implementation plan](../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md) |

## Decision summary

Use an offline neural acoustic field as the primary successor to the frozen
modal plus coloured-residual baseline. The model does not generate an opaque
runtime waveform. It predicts a bounded, inspectable acoustic representation:

- object-global modal frequencies and damping;
- contact-position-conditioned modal gains at one canonical listener;
- a compact coloured residual descriptor;
- an explicit coverage/OOD estimate.

The result is cooked into sorted, bounded coefficients and rendered by the
existing deterministic modal reference path. Model inference, training data,
checkpoints and validator inference stay outside the game runtime and outside
the repository. An authored clip remains the mandatory fallback.

This is a research strategy, not a production promotion. It adds no public
contract, content role, runtime dependency or ProductCheck. A concrete
consumer and a later ADR-046 promotion decision remain prerequisites.

## Why the strategy changes

The classical program established valuable boundaries but did not close real
generalization:

- fixed-point modal rendering and force/position controls repeat exactly;
- synthetic FEM/BEM controls can produce physically coherent examples;
- real multi-listener experiments reject a rank-one spatial transfer model;
- real metal holdouts reject the current shared residual representation;
- the DCT coloured-residual successor closes its intended synthetic controls,
  but real admission is still unopened.

Another sequence of hand-selected residual variants would repeat the same
local search pattern. R2 also shows that predicting detailed listener
radiation from a coarse coordinate grid is not the smallest product task. The
remaining first unknown is a conditional field over geometry and contact
position at a declared canonical listener, not one more global material
coefficient. Listener radiation remains a later independent claim. A learned
surrogate is therefore the smaller falsifiable next step, provided its output
remains bounded and independently validated.

## External evidence

The selected shape follows several complementary primary sources:

- [Objects as Audio-Visual Modal Sound Fields](https://arxiv.org/abs/2608.05145)
  represents an object through global frequencies/damping, a spatial neural
  gain field and residual noise, and reconstructs that field from multiple
  views plus a small number of impact recordings. It is the closest prior art
  for a few-shot object-specific candidate. Its REALIMPACT benchmark selects
  one nearest microphone and varies among five impact positions, matching the
  V4 separation of contact modeling from listener radiation.
- [NeuralSound](https://arxiv.org/abs/2108.07425) separates vibration and
  acoustic radiation learning and predicts compact far-field transfer rather
  than treating every output sample as unrelated. It motivates a shared
  geometry-to-modal/radiation surrogate candidate.
- [DiffSound](https://arxiv.org/abs/2409.13486) makes high-order modal synthesis
  differentiable and estimates physical parameters from audio. It remains a
  useful inverse-physics baseline and teacher, but its parameters alone do not
  solve the observed real residual and spatial-transfer gaps.
- [REALIMPACT](https://openaccess.thecvf.com/content/CVPR2023/html/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.html)
  publishes impact sound fields with object, impact, listener and force
  information. It is suitable for object/listener-disjoint tests and for
  measuring whether a learned field interpolates rather than memorizes. Its
  authors also show that the 20-degree listener grid aliases high-frequency
  radiation, so it cannot alone authorize an arbitrary continuous field.
- [Physics-Driven Diffusion Models for Impact Sound Synthesis](https://openaccess.thecvf.com/content/CVPR2023/html/Su_Physics-Driven_Diffusion_Models_for_Impact_Sound_Synthesis_From_Videos_CVPR_2023_paper.html)
  combines a physical prior with a learned residual. It supports the hybrid
  design, while direct waveform diffusion remains only a report-only
  perceptual upper bound in this program.
- [Rigid-Body Sound Synthesis with Differentiable Modal Resonators](https://arxiv.org/abs/2210.15306)
  shows that modal resonators can be predicted and optimized with an
  audio-domain objective. It supports learning the cooked representation
  rather than making a neural decoder the production renderer.

These sources establish feasibility and useful representations. They do not
prove a Next Engine domain, deterministic runtime, acceptable risk or broad
material generalization.

## Selected representation

### Inputs

Every training or evaluation row must bind available claims explicitly:

- source, object and geometry revision;
- normalized geometry features or a hash-bound geometric artifact;
- material and support evidence, with absent fields remaining absent;
- impact point, normal and a bounded force/impulse/energy descriptor;
- canonical listener condition for the first task; any variable listener axis
  requires a separate capability claim and split;
- recording and preprocessing revision.

Object names and material labels may be features only when the source actually
publishes them. They cannot fabricate missing geometry, support or force axes.

### Outputs

The first bounded output record is conceptually:

```text
NeuralAcousticCookResultV0
  modes[0..N): frequency_hz, decay_rate, global_gain
  gain_field: bounded coefficients over contact coordinates
  residual: bounded log-magnitude/DCT coefficients and optional low-rank covariance
  coverage: domain distance, uncertainty and OOD reason
  lineage: data/split/model/code/environment/seed/checkpoint/output hashes
```

This is an external research artifact, not a proposed public Rust schema. The
runtime candidate receives only the cooked mode/gain/residual coefficients
after validation and quantization.

### Runtime boundary

The first vertical has no neural runtime inference. Cooking must:

1. validate finiteness, bounds, counts and coordinate domains;
2. sort modes canonically and resolve duplicate/unstable modes;
3. quantize the accepted representation into the deterministic reference
   profile;
4. reproduce the same canonical 48 kHz PCM for the same cooked record and
   excitation;
5. select the authored clip on invalid, unavailable or OOD input.

Runtime inference can be reconsidered only if a measured product need cannot
be met by the cooked representation and a separate architecture decision
defines the artifact, budget, fault and fallback boundaries.

## Candidate ladder

| Candidate | Purpose | Admission role |
|---|---|---|
| Authored clip | Production fallback and real-audio reference | Never displaced outside admitted coverage |
| Frozen Q30 modal plus DCT residual | Deterministic classical baseline and negative control | Must remain reproducible |
| Object-specific few-shot modal field | Test whether a small set of impacts can interpolate new contact coordinates on one object at a canonical listener | First neural feasibility candidate |
| Shared geometry-conditioned surrogate | Test transfer to object- and family-disjoint geometry | Second candidate, only after few-shot controls work |
| Direct waveform model | Estimate a perceptual upper bound and expose representation loss | Report-only; cannot pass runtime admission |
| DiffSound/FEM/BEM teacher | Produce structured synthetic supervision and physics controls | Teacher/baseline, never treated as real identity evidence |

The few-shot candidate comes first because it separates representation
feasibility from zero-shot material generalization. Failure of a universal
model must not erase a successful exact-object field.

## Data and split policy

Freeze five non-overlapping roles before training:

1. `train` — optimization only;
2. `development` — architecture and ablation choices;
3. `calibration` — thresholds, uncertainty and selective policy;
4. `method_holdout` — one-shot comparison of the frozen candidate with
   baselines on unseen objects/contact positions;
5. `admission_shadow` — untouched until Validator Release V1 and the candidate
   are both frozen.

Grouping is by object, family, source project, recording parent and mutation
parent as applicable. Near-duplicate recordings and derived crops remain in
one group. A generator checkpoint cannot train on validator calibration,
holdout or shadow rows. A learned validator cannot train on generator
method-holdout or admission-shadow rows.

## Neural feasibility benchmark (`PS-2N`)

The first benchmark is deliberately smaller than domain admission.

### Required tasks

- reconstruct held-out impact positions for known objects;
- keep one declared canonical listener/source condition for the first task and
  report detailed radiation as an unsupported independent axis;
- preserve monotonic bounded energy scaling;
- preserve silence/rest/separation and invalid-input controls;
- cook the prediction and reproduce canonical PCM exactly;
- detect object/contact/excitation conditions outside declared coverage.

### Required comparisons

All candidates use identical rows and preprocessing:

- authored reference/clip;
- current frozen Q30 plus DCT baseline;
- object-specific few-shot neural field;
- shared surrogate when available;
- report-only waveform upper bound when reproducible.

No candidate advances because of a single aggregate similarity score.
Hard/causal failures reject first. Acoustic metrics, specialist heads and
learned representations remain separate, and the report publishes per-object
and per-contact distributions. Listener distributions are required only by a
future radiation branch.

### Go criteria

The frozen neural candidate must:

- beat the classical baseline on every preregistered primary aggregate;
- introduce no hard/causal regression;
- improve unseen-contact reconstruction rather than only training-row
  reconstruction;
- remain useful on a fully object-disjoint method holdout or explicitly narrow
  its claim to few-shot exact-object use;
- produce a bounded cooked record with exact repeat;
- publish calibrated OOD behavior before admission-shadow access.

Threshold values belong in the future hash-closed benchmark manifest, not in
this strategy document.

## Stop rules

- If the candidate improves training rows but not held-out contact positions,
  classify it as memorization and reject it.
- If a compact representation oracle cannot beat the preregistered target
  baseline, do not train a neural field over that representation.
- If published listener density cannot resolve the requested radiation field,
  narrow to a canonical listener/exact grid instead of selecting a larger MLP.
- If the shared surrogate fails object-disjoint transfer while the few-shot
  field succeeds, retain the exact-object route and reject zero-shot support.
- If direct waveform generation sounds better but fails controls, cooking or
  repeatability, retain it only as a reference or authored-asset generator.
- After the frozen DCT baseline is exercised once, do not start another manual
  residual family unless a preregistered neural ablation identifies one exact
  missing statistic.
- If the validator cannot achieve bounded risk with useful coverage, keep the
  entire domain fallback-only regardless of model quality.
- If published data lacks an axis, do not infer that axis and do not require
  the user to record new impacts.

## Roadmap consequence

The old sequence `validator -> autonomous formula search` is replaced by:

```text
frozen classical boundary
  -> new corpus + compact representation oracle
  -> PS-2N exact-object neural feasibility benchmark
  -> PS-3 independent Validator Release V1
  -> PS-4 neural cooker and one-shot admission
  -> PS-5 Neural-Cooked Formula Base V1
  -> PS-6 production rigid-impact consumer
```

PS-2N may use development, calibration and method-holdout partitions, but it
cannot inspect the admission shadow. PS-3 freezes the independent decision
rule. PS-4 is the first place where a frozen generator and frozen validator
meet the untouched admission shadow.

## Remaining uncertainty

- whether public data has enough synchronized geometry, force and contact-
  position coverage for exact-object and later object-disjoint learning;
- whether a compact modal/residual record retains the perceptual advantage of
  the learned model;
- whether force profiles across sources can be normalized without fabricating
  mechanics;
- whether the validator can obtain useful selective coverage at a bounded
  false-pass risk;
- whether any admitted research record later fits a player-visible whole-mixer
  budget.

PS-2N0, the classical controls and the 600-position dense representation are
now frozen. The first joint separable complex field is also complete and
rejected: both data-only and Helmholtz variants collapse toward silence. The
[R2C failure diagnostic](physical-sound-listener-field-r2c-result-2026-08-30.md)
shows that rank-96 context capacity is sufficient to retain `99.64%` energy,
while the trained objective remains worse than a zero predictor.

R2D V1 freezes that profile and repeats byte-identically, but misses only
small/full log-energy convergence under a fixed learning rate. R2D V2 changes
only to a half-cosine `0.05 -> 0.00001` schedule and passes every unchanged
gate twice. The separately frozen R2E field then fits context essentially
exactly but loses every held-listener endpoint. A repeated post-reject query
oracle also shows rank-96 representation loss (`0.2760` NRMSE, `92.38%`
retained energy) and misses the mean-spectrum gate. See the
[R2E result and V4 research](physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md).

Roadmap V4 therefore retires the opened listener family and requires a new
internet-only multi-impact corpus plus a query-seeing compact-representation
oracle before model training. The smallest next action is the R3A/N0.4A source,
split and representation preflight. This pivot adds no quality, admission or
runtime result.
