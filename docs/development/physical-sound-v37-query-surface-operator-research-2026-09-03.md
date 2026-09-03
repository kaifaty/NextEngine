# Physical sound V37 query-surface operator successor research

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `SELECTED_PRE_VALUES / QSO_V0 / A0_TYPED_CONTRACT_NEXT / NO_NEW_ROLE_OR_TARGET_ACCESS` |
| Trigger | [V36 D0 repeat-exact MetricReject](physical-sound-v36-d0-fresh-development-result-2026-09-03.md) |
| Parent plan | [Roadmap V37](../plans/physical-sound-synthesis-roadmap-v37.md) |
| Claim | `SUCCESSOR_REPRESENTATION_AND_PREFLIGHT_DESIGN_ONLY / NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Research question

What is the smallest materially different neural representation that can learn
contact-conditioned modal correction from a surface field and a query point,
while preserving P1's exact physics ownership, deterministic offline cooking,
fresh-role discipline and the current CPU envelope?

The answer must not be another V36 seed, width, scalar gate, local-kernel
threshold or postfilter. All V36 train/development values are spent, H0 is
retired unopened, and no successor implementation may load a V36 target,
prediction, weight or metric artifact.

## What V36 actually falsified

V36 did not fail on execution or on every learned branch:

- all 20 hard and both resource gates pass;
- decay is `0.371116x` nearest and global gain is `0.440647x` nearest;
- removing the local expert hurts geometry-only transfer;
- removing the neural expert hurts contact-only transfer.

It fails where those signals must interact. The final contact prediction is a
fixed convex blend:

```text
contact(q) = w(distance(q, train)) * local(q)
           + (1 - w(distance(q, train))) * pointwise_neural(features(q))
```

The weight does not see the surface field or the disagreement structure of the
two experts. The pointwise MLP receives a flattened local stencil but does not
represent a mapping from a surface function to an independently evaluated
query. The result is `2.394927x` nearest overall, with best-control ratios
`1.344382x / 4.195670x / 3.550940x` on contact-only, geometry-only and joint
strata. The joint ablation improves when the neural expert is removed. This is
evidence against scalar expert blending, not against bounded offline ML.

## Primary research basis

- [DeepONet](https://arxiv.org/abs/1910.03193) separates an input-function
  encoder (branch) from an output-location encoder (trunk), then evaluates the
  learned operator at independent query locations. That separation directly
  matches `P1 surface mode field -> contact query correction`.
- The [Graph Kernel Neural Operator](https://arxiv.org/abs/2003.03485)
  approximates function-space mappings with learned kernel integrals computed
  by graph message passing and is designed to transfer across discretizations.
- [MeshGraphNets](https://arxiv.org/abs/2010.03409) demonstrates that mesh
  adjacency and message passing are effective physical inductive biases and
  can transfer across mesh resolution, but its temporal rollout machinery is
  unnecessary for a static modal query.
- [Geo-FNO](https://arxiv.org/abs/2207.05209) and
  [GINO](https://arxiv.org/abs/2309.00583) support arbitrary geometries and
  irregular point/mesh inputs. Their learned deformation or graph-to-latent-
  grid/Fourier stack is useful evidence for geometry-aware operators, but is
  larger than needed for one bounded scalar correction under the current CPU
  budget.
- Deep operator networks have also been applied to parameterized source and
  receiver queries in [3D acoustic wave propagation](https://arxiv.org/abs/2308.05141),
  supporting the branch/query decomposition as an acoustic representation.

These papers motivate the representation; none supplies evidence that it will
pass the NextEngine gates or that it models impact synthesis specifically.

## Candidate comparison

| Candidate | Addresses V36 failure | Remesh/discretization story | Current fit |
| --- | --- | --- | --- |
| Larger pointwise MLP | No; same flattened query representation | None beyond current features | Rejected as a nearby V36 capacity retry |
| Learned scalar mixture-of-experts gate | Partly, but still combines already-collapsed scalars | Depends on local-kernel identity | Rejected as direct tuning of the failed gate family |
| DeepONet with fixed Euclidean sensors | Yes; explicit branch/trunk interaction | Fixed sensors can be mesh/parameterization dependent | Useful control, not the selected final representation |
| Full Geo-FNO/GINO | Yes; strong arbitrary-geometry operator | Explicit irregular-to-latent mapping | Escalation-only because cost/complexity is not yet justified |
| Temporal MeshGraphNet | Yes, but solves a larger dynamics problem | Mesh-native | Rejected for unnecessary rollout state and error accumulation |
| Query-conditioned graph/surface operator | Yes; integrates the field at the query | P1-owned canonical probes plus graph/quadrature invariants | Selected smallest falsifiable successor |

## Selected hypothesis: QSO-v0

QSO-v0 learns only the contact multiplier. V36's bounded decay and global-gain
heads are copied as controls and unchanged branches; P1 continues to own modal
frequency, mode order, node/sign structure, impulse scaling and rendering.

For each `(case, mode)` P1 supplies one immutable canonical surface-probe graph:

```text
F = { probe position, normal, area weight, P1 mode value, adjacency }
q = { stable surface location, position, normal, contact descriptor }
c = { material, support, geometry and mode descriptors }
```

The learned contact path is:

```text
h_j^0 = NodeEncoder(F_j, c)
h_j^l = h_j^(l-1) + CanonicalGraphIntegral(h^(l-1), edges, weights)
b(q)  = CanonicalQueryIntegral(q, F, h)
t(q)  = QueryEncoder(q, c, P1(q))
contact_multiplier = bound * tanh(Head[b, t, b*t, P1(q)])
```

The `b*t` interaction is structural: the model must combine information from
the field branch and query trunk before it can emit a scalar. There is no
learned or fixed blend with nearest/local predictions. Nearest, continuous
local and the V36-shaped pointwise MLP remain frozen controls only.

This is a graph-kernel/DeepONet hybrid, not a PDE solver. It predicts no
waveform, frequency, topology, mode shape or physics state.

## P1-owned representation boundary

Raw importer meshes are not a second authority. P1 derives a canonical probe
graph and stable probe identities as part of the external research record.
Equivalent remesh fixtures must expose the same probe identities, mode values,
query identity and graph root before learning. The ML batch may contain only:

- immutable float64 probe matrices and explicit integer CSR offsets;
- stable field/query/row identities;
- canonical adjacency and quadrature weights;
- row-to-field indices and query descriptors;
- the same three bounded target axes during a role-authorized run.

It may not contain object labels, role labels, target-derived distances,
development strata, raw ECS/backend handles or importer-owned structures.
Variable-length fields use typed CSR containers with explicit counts; no
`Any`, implicit `len()` semantics or unbounded nested dictionaries cross the
owner/provider boundary.

## Determinism and remesh rules

1. Probe, edge and query identities are canonical and sorted before tensor
   construction.
2. CSR offsets, edge endpoints and row-to-field indices are range-checked and
   immutable.
3. Every reduction uses declared canonical edge/probe order and float64 CPU;
   input permutation is normalized before arithmetic.
4. Padding, duplicate-edge, reversed-edge, missing-quadrature and noncanonical
   ordering mutations reject before target access.
5. Equivalent-remesh pairs must produce identical P1 probe graph roots. A
   model is never asked to learn remesh equivalence from approximate meshes.
6. QSO output remains a bounded multiplier over the P1-owned modal response,
   preserving nodal zeros, sign and frequency/order by construction.

## Fresh known-truth discriminator

The successor needs a new identity namespace and new unopened roles. The truth
family must not repeat the disclosed V36 trigonometric expression. It will
generate contact correction through three independently seeded surface
operators:

1. a compact local anisotropic kernel integral around the query;
2. a two-hop topology diffusion term that cannot be reconstructed from the
   V36 five-value stencil;
3. a global low-rank field/query interaction modulated by geometry and mode.

All coefficients, kernels, bounds, role identities and truth seeds freeze
before official values. Development and H0 contain new geometries, contacts,
field functions and operator mixtures with zero overlap against V32–V36.

Mandatory frozen controls are nearest, continuous local, V36-shaped pointwise
MLP, fixed RBF integral ridge, query-only QSO, field-only QSO and QSO without
topology propagation. Candidate admission requires every transfer stratum to
beat its best control; aggregate averaging cannot hide a miss.

## Pre-access implementation ladder

| ID | Gate | Exit |
| --- | --- | --- |
| A0 | Typed surface-field/query contract and candidate disposition | Positive construction plus container, CSR, permutation and `Pass`/reject freeze mutations pass twice with zero target values. |
| F0 | Fresh role/truth/identity freeze | Scientific knobs, new operator truth, controls, seeds and all V32–V36 intersections close at zero without materializing a target. |
| C0 | Structural and resource census | Every role has valid canonical probe/query support; both query and topology paths are reachable; full-shape cost oracle fits the frozen CPU/RSS envelope. |
| X0 | Complete owner lifecycle and terminal conformance | Pass, every reject, owner fault and failpoint publish atomically; reject evidence can never validate as an H0 freeze. |
| E0 | Full-count/full-step discarded rehearsal and seal | The exact D0/H0 owner, controls and publisher repeat byte-for-byte on surrogate values within resources. |
| D0 | One-shot fresh development | QSO beats all frozen controls in aggregate and every transfer stratum and passes ablations/hard/resources, or the family closes. |
| H0 | One-shot method holdout | Frozen QSO loads without training and repeats the same gates on new operator mixtures. |

No new official role may be allocated before A0–E0 pass. Unlike V36, A0 must
make candidate disposition part of the publisher contract:

- `Pass` publishes `candidate-freeze.json` and returns a candidate bundle;
- metric/hard/resource reject may publish `rejected-candidate-evidence.json`
  but cannot publish a freeze document or return a bundle;
- owner fault publishes neither;
- H0 accepts only a `Pass` terminal root bound into the freeze document.

## Resource hypothesis

The first QSO is deliberately small: no latent FFT grid, no temporal rollout,
at most two graph-integral layers, one query-integral readout and a bounded
latent width selected before values. Field encodings are cached per
`(case, mode)` within one owner call and reused across contact rows.

A value-free full-shape cost oracle must prove the complete candidate plus all
controls fits the existing `300 s / 1 GiB / 64 MiB` per-process envelope. If it
does not, the next action is a value-independent execution redesign, not a
smaller model selected from target quality.

## Rejected shortcuts

- Retune the V36 local weight, distance scale, model width, seed or steps.
- Train a free waveform codec or prompt-to-audio model as the physical owner.
- Use raw mesh vertex order or importer triangles as public/durable identity.
- Let a learned OOD score replace structural coverage or P1 hard gates.
- Use V36 predictions/weights as distillation targets or initialization.
- Encode the disclosed V36 truth expression as fixed QSO Fourier features.
- Start with full GINO/Geo-FNO before the small graph/query operator fails a
  fresh discriminator and a cost oracle justifies escalation.

## Decision

Select QSO-v0 and proceed to A0. The first implementation commit is contract
and publication semantics only: typed immutable field/query containers,
canonicalization, explicit counts, lifecycle capability states and
conditional `Pass` freeze versus rejected-candidate evidence. It contains no
truth formula, training loop, official role, target value or model-quality
claim.

SPEC-45 remains `Proposed`. QSO is external/offline research and authored audio
remains the only production fallback.
