# Physical sound V22 P1r — resource-bounded continuous-field protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_AND_EQUIVALENCE_ONLY / TRAIN_DEVELOPMENT_VALUES_SEALED / TEST_INTEGRATION_VALUES_SEALED` |
| Scientific family | Exact V21 `ContinuousResidualOperatorV1`, execution-only successor |
| Trigger | [V21 F1a resource reject](physical-sound-v21-f1a-continuous-field-result-2026-09-01.md), SHA-256 `7a9695ede548968074c8355cb037dbbd8e0193ab42103320e83e87ac2d1a4318` |
| Planning boundary | [Roadmap V22](../plans/physical-sound-synthesis-roadmap-v22.md), SHA-256 `af71d7e0bfea764c2d23ecc91db59dd35b9c647d1c8e8bd8c1d794c04f29158d` |
| Predecessor protocol | [V21 P1a](physical-sound-v21-p1a-continuous-residual-field-protocol-2026-09-01.md), SHA-256 `18465f95b423bc619d2987106332b753de77a65091210b9625b52235277e4bae` |
| Product effect | None; P1r cannot open real/test/integration data, bake clips, authorize runtime ML or replace authored fallback |

## Question

Can algebraically equivalent batching and precomputation make the unchanged V21
continuous-residual tournament finish below its 30-minute resource ceiling,
without learning from V21's spent roles or altering its scientific result?

V21 run A published no model, metric or selection. P1r therefore freezes only a
new execution layout, fresh identities and a value-independent equivalence
test. It does not treat the resource failure as evidence for or against model
quality.

## Immutable lineage

| Dependency | Identity |
| --- | --- |
| V21 F1a implementation commit | `95886ce06a7806511337a5314961c8d4be462313` |
| V21 common / model / tournament / tests | `0f1a0b428ae2420e98ae422eae68a1a5654a196793e15d0f0c9d8ddc7b1eb99a` / `760d4856446aa372681a7a06988bf4cc82316a1c5b975f2c6ff00bbbb3c80190` / `6ab9d8e31a3caa1b668a4db9a795d9bf3ed3d5aded6068f2c36c1678ba8a22b1` / `9fc1eb3f084a6f29d308e097065492d9997bb66d459d4ded50ac0ce1a4eeb242` |
| B0 complete tree | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| C0 complete tree | `ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace` |
| F0 complete tree, control-only | `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718` |
| M0b complete tree | `7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22` |

V21 bands `1801…2112` are spent and forbidden. Abandoned staging is not an
artifact and must not be searched, reconstructed or used. F0 remains a
reproducible causal control with no predecessor-pass credit because its runner
used `0.55` instead of the frozen per-topology gradient `0.50`.

## Fresh role ledger

There are twelve material/topology cells ordered exactly as V21: materials
`[Steel,Wood,Glass]`, topologies `[Plate,Cylinder,Bowl,RolledSheet]` and support
`[Free,BaseClamped]`. Every physical group has primary and remesh-twin views.

| Role | Physical groups | Views | Halton indices | Row root |
| --- | ---: | ---: | --- | --- |
| train | 24 | 48 | `2201…2224` | `4fd6d5b42777ffe5e7eed22c34045bae0319dcb4a895ccec4de5b4c475467e77` |
| development | 12 | 24 | `2301…2312` | `f5365e82c86e644e68ae60f5a2ad307c1b05d3e68ae95a8768ead55fe8425d9d` |
| one-shot test | 12 | 24 | `2401…2412` | `1f45f83b881387429a1fb640df28b6c1c90084d0239d2fa39fa50168f31dce72` |
| integration | 12 | 24 | `2501…2512` | `61dc577e44fd9557bd4de8b39210cba8dee92cfc824196ae123598987899ea62` |

Canonical compact JSON of the ordered ledger
`{role:{row_count,row_root}}`, sorted keys plus final newline, has SHA-256
`08098820b99388243f3d78940943747a1361c9a24e1e1e9ade68b56967137e09`.
No mesh, context, truth or prediction was generated to derive these roots.

P1r copies V21 row equations exactly except base and group revision. Train uses
`n=2201+2*c+r`; development/test/integration use `2301+c`, `2401+c`,
`2501+c`. Supports, primary grids, Halton geometry parameters and twin grid
offset `(+1,-1)` are exactly P1a. Group is lowercase
`v22-f1r-{role}-{material}-{topology}-{support}-{n}`; suffix order is primary,
then twin.

## Unchanged scientific contract

P1r incorporates V21 P1a's geometry, analytic truth, contexts, C0 coverage,
features, model, basis, candidates, controls, training and gates without value
change:

- train primary context is `max(16,ceil(N/3))`, other primaries use
  `max(16,ceil(N/8))`, every twin adds two, and all selection is signal-blind;
- analytic prior features are exactly 61 channels and carry no sampled bounds,
  mesh count/hash/index or object identity;
- the float64 prior is `61 -> 128 -> 128 -> 128 -> 8`, 41,992 parameters and
  335,936 raw bytes;
- candidates remain exact ordered `(q,lambda)` values
  `(2,1e-6),(2,1e-4),(2,1e-2),(3,1e-6),(3,1e-4),(3,1e-2)`;
- candidate seed stays `210001`; paired-harmonic seed stays `210002`;
- loss remains equally weighted per view,
  `mean(L_query+0.15*L_edge)+0.10*mean(L_pair)`, with 49 direct probes;
- each model uses full-batch AdamW, exactly 1,500 updates, weight decay `1e-6`
  and half-cosine learning rate `0.002 -> 0.00001`;
- exact F0 and paired-harmonic causal controls plus all nine compatibility
  methods retain P1a formulas and cannot be selected;
- every absolute, compatible-ratio, prior-ratio, raw-harmonic-ratio, paired-win,
  coverage, fallback, corruption, remesh, F0 non-regression and serialization
  gate is unchanged, including topology gradient `<=0.50`, per-pair drift
  `<=0.10`, direct disagreement mean/max `<=0.13/0.18` and F0 drift `<=0.90x`;
- deterministic lexicographic selection retains the exact P1a tuple and uses
  unrounded development values.

No batch may change view weighting, solve independently aggregated
coefficients, share residual coefficients between views, combine candidate
optimizer state or turn rejected queries into supervision.

## Resource-equivalent implementation

The V22 implementation may make only these transformations:

1. Before updates, compute each authorized view's analytic features, normalized
   truth, topology-native basis/penalty, Cholesky factor, active edges, context
   and output slice exactly once.
2. Concatenate all vertex features in canonical view/vertex order and perform
   one prior forward per update. Slice its output back to each view before the
   unchanged independent Cholesky solve and equally weighted view loss.
3. Concatenate all 49-probe features in canonical view/probe order and perform
   one prior forward per update. Slice before applying the independently solved
   coefficient matrix and primary/twin pair loss.
4. For paired-harmonic control, concatenate vertex features once and slice the
   single forward output before the unchanged view losses.
5. Cache immutable CPU tensors only. Candidate models still train sequentially,
   each from seed `210001`; optimizer construction, update ordering, cosine
   schedule, backward call and AdamW step remain unchanged.

No compilation backend, multiprocessing, mixed precision, GPU, early stop,
checkpoint, accumulation approximation or persistent cache is allowed.

## Metadata-only equivalence fixture

Implementation tests must compare reference-many-call and batched paths without
building any F1 row, mesh or truth. The fixture uses float64 CPU, one thread and
seed `220001` with four views in two ordered pairs `(0,1)` and `(2,3)`.

For view `i=0…3`, vertex counts are `[7,9,8,10]`. With zero-based vertex `j`,
feature `k=0…60` and mode `m=0…7`:

```text
F[i,j,k] = sin((i+1)*(j+1)*(k+1)/113)
           + 0.25*cos((i+3)*(j+1)*(k+2)/79)
T[i,j,m] = 0.5*sin((i+2)*(j+1)*(m+1)/37)
           + 0.2*cos((i+1)*(j+3)*(m+2)/43)
```

Context is ordered `[0,floor(N/2),N-1]`; accepted query is its ordered
complement; edges are the chain `(j,j+1)` excluding edges whose two endpoints
are context. The q2 basis has 25 columns:

```text
B[i,j,b] = cos((i+1)*(j+1)*(b+1)/17)
           + 0.1*sin((i+2)*(j+1)*(b+3)/29)
P[b]     = (1 + (b mod 5)^2 + floor(b/5)^2)^2
```

Probe features and basis use the same equations with `j` replaced by `j+101`
for 49 ordered probes. Gain scale is P1a's equal-view accepted-query RMS.
Use q2, lambda `1e-4`, candidate loss weights `0.15/0.10`, initial learning rate
`0.002` and AdamW weight decay `1e-6`.

Starting from identical state dictionaries, compare reference and batched:

- every vertex/probe prior output and per-view residual coefficient;
- each view loss, pair loss and final scalar loss;
- every parameter gradient after one backward call;
- every parameter after one AdamW step.

For each scalar/array pair require finite, same shape, and both maximum absolute
difference and `abs(a-b)/max(1,abs(a),abs(b))` `<=5e-12`. Forward outputs that
are bit-identical are additionally recorded as such, but bit identity is not a
separate gate. Fixture failure blocks implementation commit and cannot change
the tolerance.

## F1r execution, output and decision

After P1r commit, implementation and tests must be committed cleanly before
generating a train/development mesh or value. Official run A targets
`v22-f1r-continuous-field-run-a`; run B targets the corresponding `-run-b`
under the external physical-sound experiment root.

Each run must finish in `<1,800.0 s`, remain below 4 GiB RSS and 100 MiB output,
and atomically publish exactly the unchanged nine logical files:

```text
access-ledger.json  corpus.json      geometry.npz
manifest.json       metrics.jsonl    models.npz
predictions.npz     report.json      selection.json
```

Schemas receive V22/F1r revision identities. The manifest pins protocol and
implementation hashes, fresh row ledger, B0/C0/F0/M0b trees and the exact F0
control. Access records zero V21 artifact/staging bytes, test/integration
meshes, real/protected/waveform/force/source-body, dataset/checkpoint and network
access.

F1r passes only if run A and B each select the same eligible candidate and all
nine files are byte-identical. Any timeout, RSS/output violation, exception,
missing file, byte mismatch or candidate/gate reject closes the continuous
execution family before test. Run B is not started when A cannot publish a
complete passing artifact.

## Commit and stop boundaries

1. Commit P1r before V22 implementation may generate a mesh or truth value.
2. Commit implementation plus metadata/equivalence tests before opening
   `2201…2312` values.
3. Never generate `2401…2512` meshes/values during F1r.
4. Never reuse or inspect V21 roles/staging, change the fixture tolerance,
   candidate, update count, seed, loss, threshold, precision or resource gate.
5. A resource or quality reject closes this execution family. A successor must
   research a genuinely smaller model on new identities.
6. Do not open real/protected roles, bake clips, integrate a demo or infer at
   runtime regardless of F1r outcome.
