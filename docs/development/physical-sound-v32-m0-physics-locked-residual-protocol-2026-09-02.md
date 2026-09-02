# Physical sound V32 M0 — physics-locked residual protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION_AND_CANDIDATE_VALUES / SYNTHETIC_ONLY` |
| Roadmap package | V32 `M0`, prerequisite for one-shot `M1` |
| Research | [Physics-locked residual selection](physical-sound-v32-m0-physics-locked-residual-research-2026-09-02.md) |
| Profile | [`physical-sound-v32-m0-physics-locked-residual.v1.json`](../../lab/profiles/physical-sound-v32-m0-physics-locked-residual.v1.json) |
| Product effect | None; external known-truth experiment, authored clips authoritative |

## Question and bounded claim

Can one small branch-isolated network learn bounded corrections that P1 does
not own, while P1 retains exact frequency, mode ordering, contact nodes,
impulse scaling, support formula and remesh invariance?

A pass establishes only that this representation and automatic tournament work
on a disclosed synthetic oracle. It cannot establish naturalness, real material
quality, Steel/Glass/Wood validity, validator release, admission, cooking or
runtime authority.

## Frozen lineage and isolation

The profile binds the exact P0 profile; P1, T0 and V0a owners/results; the V28
representation reject; and the bounded V32 research result. M0 may import the
P1 analytic owner but may not decode T0 WAV samples, access a real/protected
role, call the network, use a pretrained model or reuse a V24–V28 checkpoint,
metric, prediction, optimizer state or generated cache.

Generated corpus, weights, predictions, reports and caches are external and
excluded from Git. V0a is a prerequisite proving validator mechanics; its
exact-P1 target is not misrepresented as a real or corrected-output validator.

## Synthetic corpus

M0 reconstructs legal P1 fixtures from the frozen P0 profile. It varies only
declared P1 inputs and validates every fixture through the production P1
validation/solve path.

### Object groups

Three synthetic materials are crossed with three family/support bindings and
eight geometry multiplier cells:

| Material | `E` Pa | density kg/m3 | Poisson | base loss s^-1 |
| --- | ---: | ---: | ---: | ---: |
| `synthetic-a` | `69000000000` | `2700` | `0.33` | `8` |
| `synthetic-b` | `200000000000` | `7850` | `0.29` | `12` |
| `synthetic-c` | `110000000000` | `8900` | `0.34` | `18` |

The family/support bindings are plate/simple-all-edges, beam/cantilever and
beam/simple-both-ends. Plate geometry multiplier components map to
`length_x,length_y,thickness`; beam components map to `length,width,height`.
The eight ordered cells are:

```text
0 (0.75, 0.85, 0.80)    4 (1.10, 0.90, 0.90)
1 (0.85, 1.15, 1.10)    5 (1.20, 1.10, 1.25)
2 (0.95, 0.75, 1.20)    6 (0.80, 1.20, 0.95)
3 (1.00, 1.00, 1.00)    7 (1.25, 0.80, 1.15)
```

Every object has eight contacts in frozen order:
`(0.125,0.25)`, `(0.25,0.5)`, `(0.375,0.75)`, `(0.5,0.5)`,
`(0.625,0.25)`, `(0.75,0.5)`, `(0.875,0.75)`, `(0.5,0.25)`.
Pickup stays P0-exact per family and impulse is exactly `1 N·s`.

The 72 object groups, 576 cases and 5,760 modal rows split only by geometry
cell, never by row or contact:

| Role | Cells | Objects | Cases | Modal rows |
| --- | --- | ---: | ---: | ---: |
| train | `0..4` | `45` | `360` | `3,600` |
| development | `5` | `9` | `72` | `720` |
| method holdout | `6..7` | `18` | `144` | `1,440` |

Development and holdout targets are materialized only in their access phase.
The method-holdout manifest commitment is visible before training; row values
and metrics are not.

## Frozen correction oracle

The oracle produces three log-space corrections, not audio. It uses explicit
binary64 formulae and coefficients from the canonical profile. Normalized
coordinates include mode ordinal/frequency/family indices, material constants,
family/support one-hot values, geometry log multipliers, contact `u/v` and P1
participation.

```text
d* = 0.20 tanh(0.55 loss + 0.25 logf
                + 0.20 support*ordinal + 0.15 youngs*density)

g* = 0.16 tanh(0.45 ordinal*geom0 + 0.35 geom2
                + 0.25 youngs*ordinal - 0.20 density
                + 0.15 family*geom1)

p* = 0.22 tanh(0.50 p1_contact
                + 0.35 sin(2*pi*u)*cos(pi*v)
                + 0.25 ordinal*support
                + 0.15 family*(2*u-1))
```

The candidate does not receive formula coefficients or targets outside the
current role. The oracle is intentionally not a claim about real materials; it
is a nonlinear known truth that distinguishes the candidate from linear and
retrieval controls.

## Physics-locked composition

P1 frequency, family indices and ordering pass through bit-exactly. Corrections
compose as:

```text
decay = p1_decay * exp(clamp(d, -0.25, 0.25))
contact = p1_contact * exp(clamp(p, -0.25, 0.25))
signed_gain = contact * p1_pickup * exp(clamp(g, -0.20, 0.20))
render = 0.01 * impulse * sum(signed_gain * sin(2*pi*f*t) * exp(-decay*t))
```

The network cannot change frequency, mode identity, pickup sign or impulse.
Multiplicative positive corrections preserve every exact contact node and sign.
No mesh resolution, vertex index, object ID or role label enters a feature.

## Frozen candidate

`m0-physics-locked-residual-v1` contains three independent MLP heads:

| Head | Exact input | Width/topology | Output bound |
| --- | --- | --- | ---: |
| decay | 4 mode + 4 material + 3 support = `11` | `11→16→16→1`, SiLU/SiLU/tanh | `0.25` |
| global gain | 4 mode + 4 material + 2 family + 3 geometry = `13` | `13→16→16→1`, SiLU/SiLU/tanh | `0.20` |
| contact | 4 mode + 2 family + 3 support + 2 contact + P1 participation = `12` | `12→16→16→1`, SiLU/SiLU/tanh | `0.25` |

The exact parameter count is `1,491`. Cross-branch inputs are absent by
construction, not regularized. There is no frequency head, shared trunk,
object embedding, waveform/codec decoder, attention, recurrence, graph model,
noise, pretrained encoder or runtime export.

Parameters are binary64 CPU tensors. Seed `3201` drives a NumPy `PCG64`
Xavier-uniform initializer in lexicographic tensor-name order; biases are zero.
PyTorch default initialization is never consulted.

## Training and controls

Train exactly one candidate for `1,200` full-batch AdamW steps over the 3,600
train rows: learning rate `0.003`, weight decay `1e-6`, gradient-norm clip
`1.0`, no scheduler, checkpoint selection, early stop, resume or retry.
Python/NumPy/PyTorch seeds, deterministic algorithms and one CPU intra/inter-op
thread are mandatory. CUDA, TF32, AMP, compilation and workers are forbidden.

The loss is the arithmetic mean of three normalized mean-square errors:
`MSE(d)/0.20^2`, `MSE(g)/0.16^2`, `MSE(p)/0.22^2`. No development or holdout
value enters gradients.

All controls use the same branch inputs and role closure:

1. `p1-identity-v1`: all three corrections are zero;
2. `fixed-feature-ridge-v1`: branch-local float64 ridge with `lambda=1e-6`;
3. `nearest-train-row-v1`: branch-local normalized Euclidean nearest row,
   lexicographically smallest row ID on an exact tie;
4. `neural-full-v1`: the sole candidate.

Two fixed inference ablations zero material inputs or contact inputs in the
already trained candidate. They do not retrain and cannot select parameters.

## Access order and M1 gates

M0 implementation conformance runs formula/shape/hash/mutation fixtures and a
three-step non-official smoke only. It cannot publish an official metric.

M1 executes two fresh complete processes A/B:

1. verify profile, protocol, parents, environment and zero real/network access;
2. materialize train role, fit controls and the candidate;
3. materialize development once and evaluate every gate;
4. on any development miss, publish `REPRESENTATION_REJECT` without candidate
   freeze or holdout values;
5. on pass, hash-freeze canonical weights, then materialize method holdout once;
6. publish `M1_KNOWN_TRUTH_PASS` or `METHOD_HOLDOUT_REJECT`; never retry.

Development requires candidate aggregate normalized RMSE `<=0.85x` every
control, every branch RMSE `<=0.90x` both ridge and nearest, and absolute RMSE
`<=0.035` for decay/gain and `<=0.040` for participation. The material-zero
ablation must worsen mean decay+gain RMSE by `>=5%`; contact-zero must worsen
participation RMSE by `>=5%`.

Method holdout requires every branch `<=0.95x` the best non-neural control and
aggregate `<=0.90x` the best control. A zero denominator requires exact zero;
no epsilon is added.

Every phase also requires:

- P1 frequencies/order bit-exact under prediction and rendering;
- exact impulse ratios at `0.5/1/2`;
- exact preservation of P1 contact zeros and gain signs;
- exact coarse/fine common-contact outputs and no mesh-resolution feature;
- correction bounds, positive decay, finite output and render peak `<0.95`;
- branch feature isolation and no role/label/object-ID leakage;
- all seven T0 mutation reason identities and V0a parent identities unchanged;
- A/B canonical artifacts, decision and stdout byte-identical.

## Artifacts, resources and failure

The external owner atomically emits at most:

```text
corpus-manifest.json
control-report.json
candidate-weights.bin
development-predictions.bin
candidate-freeze.json          # development pass only
method-holdout-report.json     # development pass only
evidence.json
report.json
```

Canonical tensor containers use sorted names, explicit little-endian float64
shape/data and no pickle, `torch.save`, NPZ/ZIP timestamp or MLflow authority.
The two-run limit is `300 s`, `1 GiB` peak RSS and `64 MiB` output per process.

Contract probes corrupt parent/profile/protocol hashes, corpus roles, teacher
coefficients, normalization, branch features, tensor names/order/dtype/shape,
seed/step/device/thread settings, correction bounds, frequency bypass, node
preservation, impulse scaling, remesh identity, control denominators, holdout
access and occupied/unsafe output paths. Failure removes staging and publishes
no partial candidate or freeze.

One official M1 execution spends the family even on conformance, resource or
representation failure. Opened values cannot select another seed, width, step,
loss, coefficient, bound, split, threshold or checkpoint. A pass sequences only
real source/validator work; authored clips remain authority in every outcome.
