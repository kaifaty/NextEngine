# Physical sound V23 P2a — fixed-feature ridge field protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_AND_API_SMOKE_ONLY / TRAIN_DEVELOPMENT_VALUES_SEALED / TEST_INTEGRATION_VALUES_SEALED` |
| Candidate family | `FixedFeatureRidgePriorV1`, eight preregistered closed-form variants |
| Trigger | [V23 research](physical-sound-v23-closed-form-field-research-2026-09-01.md), SHA-256 `771081265840eaf17e8741df2d246130db1cf22670ac9674e319454be6197057` |
| Predecessor result | [V22 F1r](physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md), SHA-256 `a99f4d32053ce45c53bde5d8f821943bb3dfa6531f416ca2b362e32839164885` |
| Planning boundary | [Roadmap V23](../plans/physical-sound-synthesis-roadmap-v23.md), SHA-256 `2149fcc4eb6ca337e04846202e3a92c3d93ebdb0fab5311455b4bad9a7c6f90b` |
| Product effect | None; P2a cannot open real/test/integration data, bake clips, authorize runtime ML or replace authored fallback |

## Question

Can a deterministic fixed nonlinear feature map plus one equally view-weighted
ridge solve learn enough of the object/material contact prior for the unchanged
continuous residual to pass strict quality and remesh gates, while completing a
full reproducible tournament below five minutes?

P2a freezes all identities, transforms, candidates, controls, gates, API smoke,
resources and output before an F2 mesh or truth value may exist. V21/V22
published no quality and cannot select anything below.

## Immutable lineage

| Dependency | Identity |
| --- | --- |
| V22 implementation commit | `e0c44d87a08dff69677b2dbdb9128f3d6f16a562` |
| B0 complete tree | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| C0 complete tree | `ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace` |
| F0 complete tree, control-only | `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72edb4eceb42d608718` |
| M0b complete tree | `7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22` |
| P0d counterfactual correction | `3513d3f0f9230af989be25f00541c9ac5fda997b46209e74349805d3f60d937a` |

All V21 `1801…2112` and V22 `2201…2512` roles/staging/internal state are
forbidden. F0 remains an exact control without pass credit. B0, C0, analytic
truth and M0b are immutable read-only dependencies.

## Fresh role ledger

Twelve cells retain material order `[Steel,Wood,Glass]`, topology order
`[Plate,Cylinder,Bowl,RolledSheet]`, supports `[Free,BaseClamped]` and one
primary/remesh-twin pair per physical group.

| Role | Physical groups | Views | Halton indices | Row root |
| --- | ---: | ---: | --- | --- |
| train | 24 | 48 | `2601…2624` | `09a79ea5dd20ac3003b56f80482d214cb728f8911c09a940eb45ff5e0a855be8` |
| development | 12 | 24 | `2701…2712` | `a29a16a135c44d93f80496bacabf76ac08879234188a904a5ac16491a0decda1` |
| one-shot test | 12 | 24 | `2801…2812` | `93effeeab47652146a5b96a35da97a5ebd291edb31def2246517dfa81ad38214` |
| integration | 12 | 24 | `2901…2912` | `1eb174ef3562cf6bba33d8f6537fab07c1ec6d7afc1a0ae4b72288ebe1e49476` |

Canonical compact JSON of ordered `{role:{row_count,row_root}}`, sorted keys
plus final newline, has SHA-256
`bea16c3c957c0a7feb23779e2f18b67360edef7cd3699ad1927056c37b4253bc`.
Only metadata produced these roots.

Row equations copy P1a exactly with bases `2601/2701/2801/2901` and lowercase
group `v23-f2-{role}-{material}-{topology}-{support}-{n}`. Train uses
`n=2601+2*c+r`; other roles use their base plus `c`. Primary grids, support
alternation, Halton geometry and twin offset `(+1,-1)` remain exact.

## Geometry, truth, context and coverage

Reuse exact P1a surface domains, embeddings, normals, curvatures, eight-mode
signed analytic truth and C0 composite decision order. Primary/twin truth is the
same physical function evaluated on each UV mesh.

Context is signal-blind frozen farthest-first: train primary
`max(16,ceil(N/3))`, other primary `max(16,ceil(N/8))`, twin plus two. Train
queries must all accept. Development must pass global fill, local false OOD
`<=0.05` and exact authored-fallback set equality. Test/integration meshes and
values remain forbidden in F2a.

## Normalized analytic input

Start from exact P1a 61 channels: UV `2`, analytic dimensionless XYZ `3`,
normal `3`, curvature-times-length `2`, positional sin/cos `40`, and material
`3` + topology `4` + support `2` + `log(aspect)` + `log(slenderness)` `11`.

Normalize without corpus statistics:

- divide the two curvature channels by `2*pi`;
- map `log(aspect)` using center `0.5*log(0.72*1.48)` and half-range
  `0.5*log(1.48/0.72)`;
- map `log(slenderness)` using center `0.5*log(0.004*0.008)` and half-range
  `0.5*log(0.008/0.004)`;
- leave every other already bounded or one-hot channel unchanged.

No sampled mean/range, mesh count/hash/index, object ID, target or context value
enters the feature map.

## Fixed nonlinear features

Create one maximum bank with NumPy `Generator(PCG64(230001))` in the pinned
environment. Draw `W_max = standard_normal((128,61),dtype=float64)` followed by
`phase_max = uniform(0,2*pi,128)`. Candidate dimension `D` uses the exact prefix.
For normalized analytic row `x` and bandwidth `s`:

```text
z_D,s(x) = [1,
            x[0:61],
            sqrt(2/D)*cos((W_max[0:D] @ x)/s + phase_max[0:D])]
```

Feature width is `126` for `D=64` and `190` for `D=128`. The bank is serialized
once and hash-checked; no draw occurs during fitting or inference.

## Equally view-weighted ridge prior

Per-mode gain scale is exact P1a equal-view accepted-query RMS. For each of 48
train views, evaluate `Z_i` and normalized target `Y_i` only at accepted query.
Fit all eight readouts together:

```text
G = mean_i (Z_i^T Z_i / |Q_i|) + 1e-4 * diag([0,1,...,1])
H = mean_i (Z_i^T Y_i / |Q_i|)
coef = cholesky_solve(G,H)
prior(x) = z(x) @ coef
```

The intercept is unregularized; all other columns use ridge `1e-4`. Cholesky,
coefficients and predictions must be finite; `cond_2(G) <= 1e12`. Otherwise the
candidate rejects. Maximum serialized bank plus coefficient bytes are `<=128
KiB` per candidate.

The analytic-linear control uses `[1,x]` and the same weighted ridge. It cannot
be selected.

## Continuous residual and candidates

At each view, subtract the candidate prior at context, solve the exact P1a
topology-native q2/q3 tensor residual with fixed lambda `1e-4`, evaluate directly
at query/probe UV, and restore observed context exactly. Condition limit remains
`1e12`; C0 rejects select fallback before candidate inference.

Candidate order is the full lexicographic factorial `D`, bandwidth, q:

| ID | D | bandwidth | q | prior ridge | residual ridge |
| --- | ---: | ---: | ---: | ---: | ---: |
| `ffr-d64-s0p5-q2` | 64 | 0.5 | 2 | `1e-4` | `1e-4` |
| `ffr-d64-s0p5-q3` | 64 | 0.5 | 3 | `1e-4` | `1e-4` |
| `ffr-d64-s1p0-q2` | 64 | 1.0 | 2 | `1e-4` | `1e-4` |
| `ffr-d64-s1p0-q3` | 64 | 1.0 | 3 | `1e-4` | `1e-4` |
| `ffr-d128-s0p5-q2` | 128 | 0.5 | 2 | `1e-4` | `1e-4` |
| `ffr-d128-s0p5-q3` | 128 | 0.5 | 3 | `1e-4` | `1e-4` |
| `ffr-d128-s1p0-q2` | 128 | 1.0 | 2 | `1e-4` | `1e-4` |
| `ffr-d128-s1p0-q3` | 128 | 1.0 | 3 | `1e-4` | `1e-4` |

Controls are exact frozen F0, analytic-linear ridge plus graph harmonic,
analytic-linear prior-only, candidate prior-only, prior-free same-basis
continuous, raw graph harmonic, context mean, nearest intrinsic and fixed
geodesic/Euclidean RBF. No control can be selected.

## Metrics, gates and selection

F2a retains every P1a development gate without loosening:

1. complete finite identities, C0 decisions, solves, serialization and zero
   forbidden access;
2. gain NRMSE mean/max `<=0.25/0.40`, topology mean `<=0.30`;
3. gradient p99 mean/max `<=0.45/0.80`, every topology mean `<=0.50`;
4. candidate gain/gradient each `<=0.80x` best compatible control, `<=0.75x`
   its prior-only and `<=0.60x` raw graph harmonic;
5. direct remesh disagreement mean/max `<=0.13/0.18`, every relative gain drift
   `<=0.10` and maximum drift `<=0.90x` frozen F0;
6. gain/gradient mean each `<=1.05x` frozen F0;
7. both gain/gradient beat prior-only and prior-plus-Euclidean in at least
   `10/12` primary groups;
8. all three numerical mutations quality-reject at least `11/12`; all ten
   structural/identity/budget corruptions reject `12/12` before learned output;
9. exact fallback equality, model roundtrip and scalar serialization.

Selection is the exact P1a remesh-first tuple followed by `D`, bandwidth, q and
candidate ID. If no candidate passes all gates, F2 closes before test.

## Complete evaluator API smoke

Implementation freezes an explicit `REQUIRED_MODEL_API` containing:

```text
candidate_specs  fit_candidates   validate_request
predict_candidate  predict_direct_probes  compatible_predictions
encode_models  decode_models  parameter_bytes
```

Before implementation commit, a no-F2-value miniature tournament must:

- assert every symbol exists with its exact callable signature;
- build two handcrafted primary/twin pairs from finite arrays unrelated to any
  F2 row/mesh/truth generator;
- fit all eight candidate identities and analytic-linear control;
- invoke candidate, direct-probe and every compatibility prediction;
- execute all three numerical and ten structural corruptions through actual
  validators, including `validate_request`;
- encode/decode, compare prediction roundtrip, serialize finite report/metrics,
  enforce the exact nine-file member set and exercise staging abandon;
- call the inherited gate/selection functions through the same adapter used by
  the official runner.

The smoke must pass twice byte-exactly. Static provenance or numeric unit tests
alone are insufficient. It cannot call `generate_rows` or `generate_objects` and
cannot change this API after commit.

## Resources, output and access

Each official F2a run must finish in `<300.0 s`, remain below 2 GiB RSS and 100
MiB output, and atomically publish exactly:

```text
access-ledger.json  corpus.json      geometry.npz
manifest.json       metrics.jsonl    models.npz
predictions.npz     report.json      selection.json
```

Environment remains CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, PyTorch
`2.13.0+cu130` only for frozen control compatibility, float64 CPU and one
numerical thread. F2 fitting itself uses no autograd/optimizer.

Record zero V21/V22 bytes, test/integration meshes, real/protected/waveform/
force/source-body, external dataset/checkpoint and network access. All artifacts
remain under the external experiment root and never enter Git.

Two complete independent F2a executions must select the same candidate and emit
the same nine files byte-for-byte. Run B is not started if A fails to publish a
complete passing artifact.

## Commit and stop boundaries

1. Commit P2a before F2 implementation may generate any mesh or truth value.
2. Commit implementation and twice-exact complete API smoke before opening
   `2601…2712` values.
3. Never generate `2801…2912` meshes/values during F2a.
4. Do not change candidates, bank, normalization, solve, control, gate, API,
   resource or output after values.
5. Any exception, resource, reproducibility or quality reject spends the role
   and closes F2 before test; no nearby repair/retry.
6. Do not open real/protected roles, bake clips, integrate a demo or infer at
   runtime regardless of F2a outcome.
