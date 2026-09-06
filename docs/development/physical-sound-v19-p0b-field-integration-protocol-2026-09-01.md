# Physical sound V19 P0b — field and integration protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_OFFICIAL_F0_IMPLEMENTATION / F0_TEST_AND_I0_VALUES_SEALED` |
| Candidate | `ResidualHarmonicOperatorV0` |
| Scope | Synthetic signed contact-gain capability and frozen B0/C0/F0 integration only |
| Product effect | None; no real-material, cooker, demo, public-contract or runtime-inference credit |

## Decision boundary

V19 C0 passed and is immutable. P0b therefore tests the next unresolved claim:
whether a small learned surface prior plus context-conditioned intrinsic
residual transport can recover signed modal gains on fresh object identities.

The development tournament rejected a direct multiscale-kernel MLP because its
coordinate-only control was better. The selected candidate instead separates
the two observable parts:

```text
object + local geometry
  -> learned topology-conditioned prior field

observed context gains - prior at context
  -> graph harmonic residual extension

prior + residual
  -> signed gains only at C0-accepted queries

any C0 reject
  -> authored clip fallback; learned code is not invoked for that query
```

This is an offline capability experiment. It does not establish that the
synthetic law is real material acoustics, authorize neural runtime inference,
or weaken fallback.

## Immutable inputs

The implementation must fail before training if any dependency changes:

| Dependency | SHA-256 / identity |
| --- | --- |
| V18 B0 result document | `c275dc49664e91064f01a9123727a8f1cc5c61b8a2b47a95f1b942f56680ce75` |
| B0 complete tree digest | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| B0 coefficients | `3dddb1b00b55055fb6c78ae34638df20c03428c8b2b620295b827a5f4805b7ea` |
| B0 normalization | `44c1312b2805207422c967d15d01b7ed54c85462547d390647da8308d7022b01` |
| V19 C0 result document | `fc3141a354d900d5cdda57ece99ce4b45ed9c9334db4fdbadc5ffb9aefb30d42` |
| C0 complete tree digest | `ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace` |
| C0 common implementation | `1224038462ba54e94986419f5db0510125474fbb9487d964066a6b3200548024` |
| C0 oracle implementation | `be586995efd17efda30bf6631526a6315a2c04b65228d046d8218cb46b6c5e88` |
| Frozen geometry helper | `baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad` |

C0 reason precedence and thresholds are reused exactly. F0 cannot refit,
replace, or learn confidence around them.

## Identity ledger

There are twelve cells
`c = material_index*4 + topology_index`, with material order
`[Steel,Wood,Glass]`, topology order
`[Plate,Cylinder,Bowl,RolledSheet]`, and support order
`[Free,BaseClamped]`.

| Role | Physical groups | Halton index | Primary grid `(u,v)` | Support |
| --- | ---: | --- | --- | --- |
| train | 24 | `1301+2*c+r`, `r in {0,1}` | `(13+2*r+c%2,12+r+c%3)` | `(c+r)%2` |
| development | 12 | `1401+c` | `(17+c%2,15+c%3)` | `(c+1)%2` |
| one-shot F0 test | 12 | `1501+c` | `(19+c%2,16+c%3)` | `c%2` |
| one-shot I0 integration | 12 | `1601+c` | `(21+c%2,17+c%3)` | `(c+1)%2` |

Every non-train physical group has a same-role remeshed twin with grid
`(primary_u+1,primary_v-1)`. Primary and twin share material, topology,
support, Halton index, dimensions, physical-group identity and analytic truth,
but have distinct object and mesh identities. A twin never crosses a role.

Exact row-record roots, using C0 canonical compact JSON, are:

| Role | Mesh-view row count | Row root |
| --- | ---: | --- |
| train | 24 | `273d1bd08b2a35d7a27116a01ce5f4deec34ff286b50966bbcfad6bd802293cf` |
| development | 24 | `825d0aa5de9bda2ffefc0737cfd8221f8e55585d3d6431b19bf5ef4db33806a2` |
| test | 24 | `94e66ca654a6bfa51525803ec578894348a358011ae209d5ac7d1b87bdfa6283` |
| integration | 24 | `2532b99e6332ab42c502720f4abf6fd6d46918a5636716ce5ffe23f7219b5a0b` |

The row root freezes metadata only; no test/integration mesh, truth, prediction,
metric, waveform, or model-selected value was generated before this protocol.
The successful train and development records, including mesh, context, query
and analytic-gain hashes, are respectively
`39d517f2d467059e162f694a6b700fc3bc73c16cd8a73cc32329fefe72969718`
and
`f21db879bdf497cb37d52a5dfd3fcc92d488c30d9ad9e013a93d348898f0785e`.

Every row uses:

```text
L            = 0.21 + 0.27*H_2(n)
aspect       = 0.72 + 0.76*H_3(n)
slenderness  = 0.004 + 0.004*H_5(n)
wall         = L*slenderness
```

Physical-group ID is
`v19-f0-{role}-{material}-{topology}-{support}-{n}` in lowercase.
Object ID appends `-primary` or `-twin`.

## Geometry and C0 context

UV, embedding, faces and metric-length graph edges are exactly the frozen V17
surface construction carried by the pinned V18 geometry helper:

- Plate is endpoint-inclusive in U/V and intrinsically flat;
- Cylinder is periodic in U and inclusive in V;
- Bowl is periodic in U and cell-centred in V;
- RolledSheet is endpoint-inclusive with an intrinsically open U seam;
- quads use triangles `(a,b,c),(a,c,d)`; only Cylinder/Bowl wrap U.

Normals are analytic. Plate uses `[0,0,1]`; Cylinder/RolledSheet use the radial
normal. For Bowl with radial semiaxis `a=L/2`, axial semiaxis
`z=L*aspect/2`, polar angle `alpha`, the normal is the normalized vector
`[sin(alpha)cos(theta)/a, sin(alpha)sin(theta)/a, -cos(alpha)/z]`.

The two local curvature channels are `(0,0)` for Plate,
`(2*pi/L,0)` for Cylinder/RolledSheet, and for Bowl:

```text
d = sqrt(a*a*cos(alpha)^2 + z*z*sin(alpha)^2)
k_meridional = a*z/d^3
k_azimuthal  = z/(a*d)
```

Context uses intrinsic farthest-first traversal and the exact C0
lexicographic XYZ tie break:

- train context is `max(16,ceil(N/3))`; this denser split makes every train
  query C0-accepted and prevents learning from unsupported supervision;
- primary development/test/integration context is the C0 minimum
  `max(16,ceil(N/8))`;
- each non-train twin uses that minimum plus two further farthest-first points;
- query is the complement of context.

The declared C0 minimum remains `max(16,ceil(N/8))`; extra observations are
legal, hashed context members rather than a changed threshold. Every object
must pass global fill. Per-query local C0 reject fraction must be `<=0.05` on
every mesh view; rejected queries select fallback and are excluded from field
quality metrics with their count and reason still reported.

## Signed-gain truth

There are eight modes. Define:

```text
k  = [1,1,2,2,3,3,4,5]
l  = [1,2,1,3,2,4,3,2]
rk = [1,1,2,1,2,2,2,2]
rl = [1,2,1,2,1,2,1,2]
phase = 0 for Free, pi/17 for BaseClamped
amplitude_m = (1+0.06*log(density/650))/sqrt(m+1)
```

The shared base field is unchanged from V17:

```text
Plate:
  phi = cos(k*pi*u/2+phase)*cos(l*pi*v/2)
        +0.12*sin((k+l)*pi*u*v/2)
Cylinder:
  phi = cos(k*pi*u+m*pi/9+phase)*cos(l*pi*v/2)
Bowl:
  phi = cos(k*pi*u+m*pi/12+phase)*cos(l*pi*v/2)
        *(1-0.10*(u*u+v*v))
RolledSheet:
  phi = sin(k*pi*(u+1)/2+phase)*cos(l*pi*v/2)
        +0.10*cos((k+1)*pi*u*v)
```

V19 adds a smooth, object-specific intrinsic residual so context is necessary:

```text
hidden_m = 2*pi*H_7(n+13*(m+1))
mix_m    = 0.25+0.20*H_11(n+17*(m+1))
residual = mix_m
           *sin(rk*pi*(u+1)/2+hidden_m+phase)
           *cos(rl*pi*(v+1)/2-0.5*hidden_m)
directional = 0.72+0.28*abs(dot(normal,normalize([0.27,-0.91,0.31])))
gain = amplitude_m*(phi+residual)*directional
```

The residual has no candidate input containing `n`, `hidden` or `mix`; it can
be recovered only through context observations. Primary/twin truth is evaluated
analytically on each mesh. There is no noise, waveform residual or random draw.

## Candidate: `ResidualHarmonicOperatorV0`

Train-only per-mode RMS normalizes signed gain targets. The learned prior input
has exactly 61 channels:

- UV `2`, bbox-normalized XYZ `3`, analytic normal `3`, dimensionless
  curvatures `2`;
- for frequencies `j=1..5`, sine and cosine of
  `j*pi*u`, `j*pi*v`, `j*pi*(u+v)`, `j*pi*(u-v)`: `40`;
- material one-hot `3`, topology one-hot `4`, support one-hot `2`,
  `log(aspect)` and `log(slenderness)`: `11`.

The float64 prior is `61 -> 128 -> 128 -> 128 -> 8`, with SiLU after each
hidden layer. Seed is exactly `190001`. Full-batch AdamW runs 1,500 updates,
weight decay `1e-6`, with half-cosine learning rate `0.002 -> 0.00001`. No
early stopping, checkpoint selection, dropout, augmentation or test-based
choice is allowed.

For each train object, loss is mean normalized query gain MSE plus `0.15` times
mean normalized edge-gradient MSE. Edges with two context endpoints are
excluded; for the gradient term only, context endpoints are overwritten by
their exact observed normalized gains. Object losses are averaged equally.

At inference, let `p` be the learned prior and
`r_C = observed_gain_C-p_C`. Construct symmetric adjacency conductance
`w_ij=1/max(edge_length_ij,1e-12)` and graph Laplacian `L=D-W`. The only
candidate residual is the deterministic Dirichlet solution:

```text
r_Q = solve(L_QQ, -L_QC*r_C)
prediction = p+r
prediction_C = observed_gain_C exactly
```

Singular, nonfinite or ill-conditioned solves fail closed to fallback. The
model has 41,992 float64 parameters (`335,936` raw parameter bytes); an
artifact above `400,000` parameter bytes is invalid.

## Frozen controls

All controls use the same train/development rows and gain normalization:

1. context mean;
2. nearest intrinsic context;
3. raw graph harmonic extension of observed gain;
4. raw graph-geodesic Gaussian RBF, bandwidth `0.20*diameter`;
5. raw Euclidean Gaussian RBF, bandwidth `0.20*diameter`;
6. the exact learned prior with no context correction;
7. learned prior plus graph-geodesic RBF residual, bandwidth `0.20*diameter`;
8. learned prior plus Euclidean RBF residual, bandwidth `0.20*diameter`.

Controls 6–8 reuse the exact candidate prior weights. Thus any candidate win
isolates the intrinsic harmonic residual rather than extra neural capacity.

The final development-only successful control produced:

| Method | Gain NRMSE mean/max | Edge-gradient p99 mean/max |
| --- | --- | --- |
| candidate harmonic residual | `0.183119080 / 0.290613097` | `0.363671119 / 0.768889353` |
| learned prior only | `0.409977683 / 0.525623506` | `0.644208751 / 0.895018716` |
| prior + geodesic RBF residual | `0.311879715 / 0.417451421` | `0.504785807 / 0.890602600` |
| prior + Euclidean RBF residual | `0.291131821 / 0.414121145` | `0.482213804 / 0.833501086` |
| raw graph harmonic | `0.557767986 / 0.691950216` | `0.826091897 / 1.147796660` |

Candidate primary/twin canonical-probe disagreement was
`0.108165418 / 0.167543158` mean/max; maximum relative per-object NRMSE drift
was `0.078109309`. Query shift, context shift and alternating
context-mode sign each rejected `12/12` development objects at the frozen
quality boundary.

## F0 metrics and gates

Gain NRMSE is RMS prediction error divided by truth RMS over C0-accepted query
vertices. Edge-gradient error is per-edge eight-mode RMS prediction-minus-truth
delta divided by full-object truth RMS; its per-object p99 excludes edges with
an OOD query endpoint. Canonical remesh probes are the Cartesian product of
seven U and V values linearly spaced over `[-0.75,0.75]`; periodic U copies are
included before deterministic linear interpolation.

One complete F0 test run passes only if all conditions hold:

1. all 24 primary/twin mesh views pass C0 global fill, local false OOD is
   `<=0.05` per view, and every fallback decision carries the exact C0 reason;
2. candidate gain NRMSE mean/max is `<=0.25/0.40` overall and mean is `<=0.30`
   in every topology;
3. candidate edge-gradient p99 mean/max is `<=0.45/0.80` overall and every
   topology mean is `<=0.50`;
4. candidate mean gain and gradient are each `<=0.80x` the best compatible
   control, `<=0.75x` prior-only, and `<=0.60x` raw harmonic;
5. candidate beats both prior-only and prior-plus-Euclidean residual on both
   gain and gradient in at least `10/12` paired primary physical groups;
6. canonical primary/twin disagreement mean/max is `<=0.13/0.18`, and relative
   primary/twin gain-metric drift is `<=0.10` for every physical group;
7. query cyclic shift by 17, context-value cyclic shift by 17 and alternating
   context mode-sign flip each quality-reject at least `11/12` primary groups;
8. duplicate/out-of-range/missing context, mesh/topology identity mismatch and
   insufficient declared budget each reject `12/12` before model inference;
9. every scalar is finite, model round-trip preserves all predictions, no
   forbidden role is read, and two complete independent executions are
   byte-identical.

Quality reject means gain NRMSE `>0.40` or edge-gradient p99 `>0.80`.
Development fixes these gates; F0 test cannot change a threshold, seed,
feature, context, truth term, solver, model width or update count.

## Output and access boundary

F0 writes only under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/` to a caller-chosen
new absolute directory. Publication is staged then atomically renamed. The
exact root contains only:

```text
access-ledger.json
corpus.json
decisions.jsonl
geometry.npz
manifest.json
model.npz
predictions.npz
report.json
```

Canonical JSON forbids NaN/infinity. NPZ members are sorted, uncompressed, have
fixed ZIP metadata and little-endian arrays. No symlink, pre-existing output,
path escape, undeclared file or model pickle is accepted.

Every forbidden counter is zero: network/source body, external dataset,
external checkpoint, real waveform, force, generator-real, method holdout,
protected calibration, admission shadow, opened V16/V17/V18 artifacts, B0
external artifacts during F0, and integration values. The premature-test
counter is zero until implementation and tests are committed.

Pinned environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`,
PyTorch `2.13.0+cu130`, float64 CPU, deterministic algorithms and one numerical
library thread. One run is capped at ten minutes, 4 GiB RSS and 100 MiB output.

## I0 freeze

I0 remains sealed until two exact F0 passes freeze protocol, implementation,
model, normalization, prediction and report hashes. It trains and calibrates
nothing. It loads only the exact B0 ridge, C0 implementation/result identity
and passing F0 model, then generates integration rows `1601…1612` plus twins.

Global frequencies/damping come from the frozen B0 scale-separated ridge.
Signed gains come from the frozen F0 operator only at C0-accepted queries.
Each reject has a complete authored-fallback decision and no synthesized clip.
Accepted endpoints render eight zero-phase damped sinusoids at 16 kHz for
8,192 samples.

I0 gates, fixed now, are:

- B0 frequency median/p95 `<=20/60 cents`, damping median/p95 `<=0.08/0.20`;
- C0 global pass on every view and local false OOD `<=0.05` per view;
- gain mean/max `<=0.25/0.40`, gradient p99 mean/max `<=0.45/0.80`;
- first-1,024-sample joint waveform NRMSE `<=0.35`;
- median multiresolution spectrum RMSE `<=2.5 dB`, envelope NRMSE p95
  `<=0.15`, and every truth modal peak matched within `60 cents`;
- F0 control ratios, corruption rejection and remesh gates remain true;
- negative damping, unordered frequency, missing fallback and dependency hash
  mutations hard reject `100%`;
- two complete independent executions are byte-identical with zero forbidden
  access.

I0 rejection closes V19 capability work. It cannot refit B0/F0, regenerate C0,
open real protected roles or select a nearby model.

## Stop and promotion rules

1. Commit the protocol before official F0 implementation.
2. Commit implementation and development-only tests before generating any F0
   test mesh, gain, prediction or metric.
3. F0 runs exactly twice. A reject seals I0 and requires a new roadmap/protocol
   family, not a test-driven retry.
4. I0 runs exactly twice only after F0 passes and its exact hashes are frozen.
5. Synthetic pass opens disclosed-real work only when the independent source
   lane also satisfies its unchanged prerequisites.
6. No local microphone/hammer capture, prompt-to-waveform substitution,
   per-sound human queue, raw physics callback, runtime training/inference or
   fallback removal is authorized.
7. Cooker/demo work still waits for protected admission; production still
   needs a concrete consumer, Accepted promoting ADR and relevant ProductChecks.
