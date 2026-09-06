# Physical sound V21 P1a — continuous residual field protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_F1A_IMPLEMENTATION / METADATA_ONLY / TRAIN_DEVELOPMENT_VALUES_SEALED / TEST_INTEGRATION_VALUES_SEALED` |
| Candidate family | `ContinuousResidualOperatorV1`, six preregistered variants |
| Trigger | [V21 remesh research](physical-sound-v21-remesh-consistency-research-2026-09-01.md), SHA-256 `7d389d5ad578666602de365d06dcadaabb731e43e94afabf54171a35c54c15d0` |
| Predecessor audit | [F0 protocol conformance reject](physical-sound-v21-f0-protocol-conformance-audit-2026-09-01.md), SHA-256 `78fa857cdc4ca643e451bc756c37cbf72f584a86bde615b321b25d722d35abb4` |
| Integration correction | [P0d](physical-sound-v21-p0d-counterfactual-owner-correction-protocol-2026-09-01.md), SHA-256 `3513d3f0f9230af989be25f00541c9ac5fda997b46209e74349805d3f60d937a` |
| Product effect | None; F1a can select only a synthetic field artifact and cannot open test, integration, real data, clips, demo or runtime ML |

## Question

Can one small learned continuous prior plus a topology-native continuous
residual fit recover fresh signed modal-gain fields while remaining stable
when the same physical object is represented by two valid meshes?

P1a freezes identities, representation, candidate grid, optimizer, controls,
metrics, ranking, corruption behavior, access and output before code may build
an F1 mesh or evaluate truth. The opened V20 I1 values are attribution-only and
cannot select anything below.

## Immutable dependencies

| Dependency | Identity |
| --- | --- |
| V20 I1 result document | `f072dd206fc497240121800c05913d156f987860f6fe954ad8f0594ead1da404` |
| V18 B0 complete tree | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| V19 C0 complete tree | `ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace` |
| V19 F0 complete tree, control-only | `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718` |
| V20 M0b complete tree | `7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22` |
| F0 common / model / oracle | `84b742502f7be48742a5ee7d42d7073fd2f0061cb9fc43d8283bbbe7125942e7` / `ce48434d14a445949ad8e8d0516ac34306f3fd4d1bcbc623ba14daad27e884be` / `7a9107f292bfe33a3bf14de900adec667a95ac8c9d16fb9784441aeb81cf57f4` |
| C0 common / oracle | `1224038462ba54e94986419f5db0510125474fbb9487d964066a6b3200548024` / `be586995efd17efda30bf6631526a6315a2c04b65228d046d8218cb46b6c5e88` |
| Geometry helper | `baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad` |
| M0b common / metric | `8f5ad986847f971e32e1c7e814cfb7c162143ad2a869824b6dc5cc5fb624fcce` / `b74208de60fa1889cd035b177919244b5a50e7d7416eaa1eaf7b347d232c4bcb` |

F0's exact artifact is a reproducible causal control, not a passing
prerequisite. B0, C0, analytic truth, M0b code and all dependency bytes are
read-only. F1a performs no global-mode fit, coverage fit, acoustic metric
calibration or threshold selection.

## Fresh role ledger

There are twelve cells with material order `[Steel,Wood,Glass]`, topology
order `[Plate,Cylinder,Bowl,RolledSheet]` and support order
`[Free,BaseClamped]`. Every physical group has one primary and one remesh twin.

| Role | Physical groups | Views | Halton indices | Row root |
| --- | ---: | ---: | --- | --- |
| train | 24 | 48 | `1801…1824` | `3d7be93c39b1f29be178a814e863f5894b93644a81d0f479c822abc3f7c3c6b2` |
| development | 12 | 24 | `1901…1912` | `9eecedcc541c32d394dc036f26be6e29f976b9d28e86de36ab9a51b0f458b5af` |
| one-shot test | 12 | 24 | `2001…2012` | `4537153ade7a4e9804142e2c7f431ebda22c786ba307aed65e4fdb84b62d990e` |
| one-shot integration | 12 | 24 | `2101…2112` | `a1bc5aa4bb185d069892fc6a0e24d2b82a890856bfd9a4e27d27a077dc7f793f` |

Canonical compact JSON of the ordered ledger
`{role:{row_count,row_root}}`, with sorted keys and a final newline, has SHA-256
`9324c04d9abb538bbe2e102323ce576f9c109ea31f38af4ad69296a5981ba433`.
Only metadata records were used to derive these roots; no mesh or value exists.

For train cell `c=0…11` and replicate `r in {0,1}`:

```text
n        = 1801 + 2*c + r
support  = (Free,BaseClamped)[(c+r) mod 2]
grid     = (15 + 2*r + c mod 2, 13 + r + c mod 3)
```

For a non-train cell:

```text
role          n         support index   primary grid
development   1901+c    (c+1) mod 2     (19+c mod 2,16+c mod 3)
test          2001+c    c mod 2         (21+c mod 2,17+c mod 3)
integration   2101+c    (c+1) mod 2     (23+c mod 2,18+c mod 3)
```

Every row then uses:

```text
material    = (Steel,Wood,Glass)[floor(c/4)]
topology    = (Plate,Cylinder,Bowl,RolledSheet)[c mod 4]
length_m    = 0.21  + 0.27  * radical_inverse(n,2)
aspect      = 0.72  + 0.76  * radical_inverse(n,3)
slenderness = 0.004 + 0.004 * radical_inverse(n,5)
wall_m      = length_m * slenderness
group       = "v21-f1-{role}-{material}-{topology}-{support}-{n}" lowercase
```

The primary object suffix is `-primary`. Its twin suffix is `-twin` and grid is
`(primary_u+1,primary_v-1)`. Row order is cell, replicate when present, then
primary before twin. Roles never share a physical group, mesh or object ID.

## Geometry, truth, context and coverage

Reuse the exact frozen surface domains, embeddings, faces, normals, curvatures,
eight-mode signed-gain truth and C0 composite decision order. Primary/twin
truth is the same analytic physical function evaluated at each mesh's UV.

Context is signal-blind farthest-first selection with the frozen lexicographic
tie break:

- train primary count is `max(16,ceil(N/3))`; train twin adds two;
- every non-train primary uses `max(16,ceil(N/8))`; its twin adds two;
- query is the complement; C0 evaluates every query before inference;
- every train query must be accepted; every other view must pass global fill
  and have local false OOD fraction `<=0.05`;
- each C0 reject selects exactly one authored fallback and never enters field
  quality or residual fitting.

Test and integration row records may be validated, but their meshes, contexts,
truth, predictions and metrics remain forbidden during F1a.

## Mesh-independent prior features

The prior remains a `61 -> 128 -> 128 -> 128 -> 8` float64 SiLU network, but
its local XYZ channels no longer use sampled mesh bounding boxes. For
`x=(u+1)/2`, `y=(v+1)/2`, define analytic dimensionless XYZ:

```text
Plate:       [u, v, 0]
Cylinder:    [cos(2*pi*x), sin(2*pi*x), v]
Bowl:        [sin(pi*y/2)*cos(2*pi*x),
              sin(pi*y/2)*sin(2*pi*x), -cos(pi*y/2)]
RolledSheet: [cos(2*pi*x), sin(2*pi*x), v]
```

Concatenate UV `2`, analytic XYZ `3`, analytic normal `3`, curvature times
`length_m` `2`, the exact F0 40 positional sine/cosine channels, and material
`3` + topology `4` + support `2` + `log(aspect)` + `log(slenderness)` `11`.
This is exactly 61 channels and is evaluated by the same formula at vertices
and arbitrary canonical probes. No sampled min/max, vertex count, edge index,
mesh hash or object ID enters the prior.

Per-mode gain scale is the square root of the equally weighted mean of each
train view's accepted-query mean square. Thus primary/twin views and mesh
resolutions receive equal weight rather than weight proportional to vertices.

## Continuous residual operator

Let `x=(u+1)/2`, `y=(v+1)/2`. For an open axis define ordered features
`[1,cos(pi*z),sin(pi*z),...,cos(q*pi*z),sin(q*pi*z)]`. Cylinder and Bowl use a
periodic U axis `[1,cos(2*pi*x),sin(2*pi*x),...,cos(2*q*pi*x),sin(2*q*pi*x)]`.
Plate/RolledSheet U and every V axis are open.

`B_q(u,v)` is the lexicographically ordered tensor product of U and V features,
with `25` columns for `q=2` and `49` for `q=3`. Each column carries axis
frequencies `(j_u,j_v)` and penalty
`P=(1+j_u^2+j_v^2)^2`.

For normalized observed residual
`r_C = observed_gain_C/gain_scale - prior_C`, solve independently for all eight
modes:

```text
A = B_C^T B_C / |C| + lambda * diag(P)
b = B_C^T r_C / |C|
coef = cholesky_solve(A,b)
prediction_Q = gain_scale * (prior_Q + B_Q*coef)
prediction_C = observed_gain_C exactly
```

The float64 Cholesky solve must be finite, positive definite and have
2-norm condition number `<=1e12`; otherwise the query selects fallback. Residual
evaluation uses UV only and has no mesh adjacency, interpolation or nearest
neighbour step.

## Six candidates and two causal controls

Candidate order is exact:

| ID | `q` | `lambda` |
| --- | ---: | ---: |
| `continuous-q2-l1e-6` | 2 | `1e-6` |
| `continuous-q2-l1e-4` | 2 | `1e-4` |
| `continuous-q2-l1e-2` | 2 | `1e-2` |
| `continuous-q3-l1e-6` | 3 | `1e-6` |
| `continuous-q3-l1e-4` | 3 | `1e-4` |
| `continuous-q3-l1e-2` | 3 | `1e-2` |

Each candidate trains its own prior from the same initialization seed `210001`.
The two causal controls are:

1. `frozen-f0-harmonic`: exact F0 model/gain scale and graph-harmonic residual;
   it is control-only and receives no predecessor-pass credit;
2. `paired-harmonic-v1`: the new analytic-feature prior trained on all paired
   train views with query/gradient loss, followed by the old graph-harmonic
   residual. Its seed is `210002` and it has no pair-probe loss.

Context mean, nearest intrinsic, each trained prior-only output, raw graph
harmonic, raw continuous basis and fixed geodesic/Euclidean RBF residuals are
reported compatibility diagnostics. They cannot be selected.

## Training contract

For every continuous candidate, build its full prediction through the
differentiable Cholesky solve. Average train views equally:

```text
L_query = mean accepted-query normalized gain squared error
L_edge  = mean active-edge normalized gradient squared error,
          excluding edges with two context endpoints
L_pair  = mean over 24 groups and 49 canonical probes of
          squared(primary_prediction - twin_prediction)
L = mean_views(L_query + 0.15*L_edge) + 0.10*L_pair
```

Canonical probes are the Cartesian product of seven values linearly spaced on
`[-0.75,0.75]` for U and V. Candidate outputs are evaluated directly there;
no mesh interpolation occurs. Query truth prevents pair agreement from being a
self-sufficient objective.

Use full-batch AdamW, 1,500 updates, weight decay `1e-6` and half-cosine
learning rate `0.002 -> 0.00001`. The paired-harmonic control uses the same
optimizer and updates but loss `mean_views(L_query+0.15*L_edge)`. No early
stopping, checkpoint choice, dropout, augmentation, mixed precision or
candidate-specific seed is allowed. Every network remains 41,992 parameters,
335,936 raw float64 parameter bytes and at most 400,000 parameter bytes.

## Metrics and eligibility gates

Gain NRMSE, edge-gradient p99, topology aggregation, mutations and structural
corruptions use the frozen P0b formulas. Canonical remesh disagreement uses
direct candidate evaluation at the 49 probes; frozen F0/legacy diagnostics use
their historical interpolation and are labelled accordingly. Relative
primary/twin gain-metric drift retains its exact asymmetric formula:

```text
abs(twin_gain_nrmse-primary_gain_nrmse)
  / max(primary_gain_nrmse,1e-12)
```

A continuous candidate is development-eligible only if all conditions pass:

1. all train/development rows, identities, contexts, C0 decisions, residual
   solves, model round-trips and scalar serializations are complete and finite;
2. gain NRMSE mean/max is `<=0.25/0.40` overall and topology mean `<=0.30`;
3. edge-gradient p99 mean/max is `<=0.45/0.80` overall and every topology mean
   is `<=0.50`; the erroneous F0 implementation literal `0.55` is forbidden;
4. candidate mean gain and gradient are each `<=0.80x` the best of context
   mean, nearest intrinsic, its own prior-only, raw graph harmonic, raw
   same-basis continuous, raw geodesic RBF and raw Euclidean RBF; they are also
   `<=0.75x` its own prior-only output and `<=0.60x` raw graph harmonic;
5. primary/twin direct disagreement mean/max is `<=0.13/0.18`, and every
   relative gain-metric drift is `<=0.10`;
6. maximum gain-metric drift is `<=0.90x` frozen F0 on the same development
   rows, while gain and gradient means are each `<=1.05x` frozen F0;
7. it beats its prior-only and prior-plus-Euclidean residual on both gain and
   gradient in at least `10/12` development primary groups;
8. query cyclic shift 17, context-value cyclic shift 17 and alternating
   context mode-sign each quality-reject at least `11/12` primaries;
9. duplicate/out-of-range/missing context, mesh/topology/role mismatch,
   unknown basis ID, changed lambda, nonfinite coefficient and insufficient
   budget reject before returning a learned prediction;
10. fallback set equality is exact and every forbidden-access counter is zero.

If frozen F0 has zero maximum drift, condition 6 requires candidate drift
`<=1e-12`; otherwise the ratio uses its observed positive value. Quality reject
means gain NRMSE `>0.40` or edge-gradient p99 `>0.80`, unchanged from P0b.

## Deterministic selection

If no candidate is eligible, F1a is `Reject` and closes before test. Otherwise
select the lexicographic minimum of this tuple, using unrounded development
values:

```text
(maximum gain-metric drift,
 maximum direct probe disagreement,
 maximum topology edge-gradient mean,
 gain NRMSE mean,
 edge-gradient p99 mean,
 q,
 lambda,
 candidate ID)
```

No weighted score, tolerance tie, listening choice or post-hoc material slice
is allowed. Two complete independent F1a executions must select the same ID and
emit the same nine files and bytes. The winner remains trained on train only;
development never becomes supervision.

## Frozen one-shot test gates

P1b may pin only the selected model/basis identity and exact F1a tree; it cannot
change any gate. On unopened `2001…2012`, the winner must repeat every
eligibility condition using test instead of development, including the strict
topology `<=0.50`, `<=0.10` per-group drift, direct disagreement, `<=0.90x` F0
drift improvement and `<=1.05x` F0 quality non-regression. All three numerical
mutations reject at least `11/12`; all ten structural/basis corruptions reject
`12/12`; two executions are byte-identical. A reject closes F1 and cannot
select a retry from test values.

## Access, resources and output

F1a may generate exactly 24 train groups/48 views and 12 development groups/24
views after its implementation commit. It must record zero test/integration
rows, I1 artifact bytes, real waveform/force/source body, external dataset or
checkpoint, generator-real, protected calibration, method holdout, shadow and
network access.

Each F1a run must finish below 30 minutes, 4 GiB RSS and 100 MiB output and
publish atomically under the external experiment root exactly:

```text
access-ledger.json
corpus.json
geometry.npz
manifest.json
metrics.jsonl
models.npz
predictions.npz
report.json
selection.json
```

Environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, PyTorch
`2.13.0+cu130`, deterministic float64 CPU and one numerical-library thread.
No dataset, model, array, report, audio or cache enters Git.

## Commit and stop boundaries

1. Commit this protocol before F1a code may generate a mesh or value.
2. Commit implementation and metadata-only tests before opening train or
   development values.
3. Do not generate test/integration meshes or values in F1a; their row roots
   are metadata commitments only.
4. Do not use I1 values, F0's erroneous `0.55`, a new threshold, extra
   candidate, seed, contact, checkpoint or listening preference.
5. Do not open real/protected roles, bake clips, integrate a demo or infer at
   runtime regardless of F1a outcome.
