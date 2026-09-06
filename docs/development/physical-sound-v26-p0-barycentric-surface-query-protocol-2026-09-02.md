# Physical sound V26 P0 — deterministic barycentric surface-query protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Protocol revision | `m0b-v1.0` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / MODEL_VALUES_UNOPENED / EXECUTION_EQUIVALENT_SURFACE_REPAIR_ONLY` |
| Roadmap package | V26 `P0`, prerequisite of `I0` and `M0b-E` |
| Trigger | [M0a-E preprocessing conformance reject](physical-sound-v25-m0a-official-evaluation-result-2026-09-02.md) |
| Research | [Off-vertex field-transfer research](physical-sound-v26-off-vertex-field-transfer-research-2026-09-02.md) |
| Inherits | [M0a v1.1](physical-sound-v25-m0a-causal-material-neural-student-protocol-2026-09-02.md) except for the explicit delta below |
| Product effect | None; external feasibility evidence only, with authored clips authoritative |

## Question and allowed claim

Can the frozen M0a experiment evaluate the same physical surface query on
coarse and refined triangle meshes without requiring that query to be a vertex,
while changing no model, optimization, evidence-role or quality-gate choice?

P0 may establish only deterministic point-on-surface and per-vertex-field
evaluation. I0 may establish implementation conformance. M0b-E may then answer
the still-unopened M0a representation question. None of these establishes real
material quality, automatic admission, a public content record or runtime use.

## Frozen M0b delta

M0b is an execution-equivalent successor. The only semantic changes from M0a
are:

1. replace `nearest_vertex(mesh, query)` for T0 gain lookup with the surface
   evaluator below and linearly interpolate the refined per-vertex gain field;
2. validate the same query on the coarse mesh, but do not require coarse and
   refined teacher samples to be byte-equal at an off-vertex point;
3. retain the coarse object/contact descriptors as the unchanged input to the
   existing model remesh gate;
4. add the official-profile structural fixture and its canonical report before
   official model execution.

The following remain exactly M0a v1.1 and may not change:

- model ID `m0a-contact-modal-field-v1`, layers, activations, residual atoms,
  parameter limit and float32 CPU inference;
- 38-value object input, 24-value contact input, physical-material table,
  unknown X0 masks and support semantics;
- seed `3101`, deterministic settings, batch sizes, `1500 + 500` AdamW steps,
  learning rates, weight decay and gradient clipping;
- every loss, weight, ridge/control/ablation definition, renderer and nuisance
  gain policy;
- evidence hashes, role access order, candidate freeze, one-shot method
  holdout, disclosed real query and sealed admission/REALIMPACT row `2407`;
- every hard, causal, quality and ratio threshold, including model remesh gain
  difference `<= 1e-5` at the identical physical query;
- canonical tensor format, atomic publication, local diagnostic MLflow and
  `1800 s / 4 GiB / 256 MiB` per-run limits.

No M0a source file is edited. The M0b implementation root includes the exact
bytes of every inherited module it imports. The frozen inherited hashes are:

| M0a module | SHA-256 |
| --- | --- |
| `physical_sound_v25_m0a_common.py` | `592b38137eb088e4ffc8087c3c68c0c19b0a8ceed8d049e7e9b2f6a002573e89` |
| `physical_sound_v25_m0a_evaluate.py` | `e5c48a74f5df89d60433d2ef1605bce648250836809e04721a2aa1fefe3e552a` |
| `physical_sound_v25_m0a_model.py` | `6075b4121c5eb45aca2934308fb4e73c078c467669e458c0d36b65b066f4009b` |
| `physical_sound_v25_m0a_train.py` | `86ada58126ad901021cdf297cef2fc46059dc33763e960dbdc21ebf3972630b5` |

Dynamic monkey-patching of the spent M0a runner is forbidden. M0b owns a new
manifest identity, entry point and implementation root while reusing named
frozen helpers normally through imports.

## Surface evaluator V1

The evaluator accepts one parsed `NEMESH01` mesh, one finite float64 XYZ query
in metres and, when field evaluation is requested, one finite float64
`vertex_count × channel_count` array bound to that exact mesh. M0b uses exactly
ten gain channels.

Let `D` be the float64 Euclidean diagonal of the mesh axis-aligned bounding
box. Reject unless `D` is finite and positive. Freeze:

```text
surface_tolerance_m = max(1e-12, D * 2^-40)
barycentric_tolerance = 2^-40
degenerate_squared_normal_limit = D^4 * 2^-80
```

Before any query, validate every triangle:

- three distinct in-range vertex indices;
- unique canonical face key equal to its three vertex indices sorted ascending;
- squared cross-product normal strictly greater than
  `degenerate_squared_normal_limit`;
- finite vertices, edges and derived values.

One invalid or duplicate face rejects the complete mesh. Faces are never
silently skipped, repaired, welded or retriangulated.

For each triangle `(a,b,c)` in float64:

1. compute `e0=b-a`, `e1=c-a`, normal `n=e0×e1` and `n2=n·n`;
2. compute signed plane offset `s=(q-a)·n`, plane residual
   `abs(s)/sqrt(n2)` and projected point `p=q-n*(s/n2)`;
3. reject this candidate if plane residual exceeds `surface_tolerance_m`;
4. compute triangle coordinates using the dot-product form with denominator
   `(e0·e0)(e1·e1)-(e0·e1)^2`;
5. retain the candidate only when all three weights lie in
   `[-barycentric_tolerance, 1+barycentric_tolerance]`;
6. verify finite weights, absolute sum-to-one error `<= 2^-40` and reconstructed
   point residual `<= surface_tolerance_m`.

If no triangle remains, return `FallbackOutOfDomain`. For a vertex/edge query,
multiple incident faces may remain. Select the lexicographically smallest
canonical face key. Return that sorted key and weights reordered to the same
ascending vertex-index order. Triangle enumeration, cyclic vertex order and
winding therefore cannot change canonical output.

Interpolate a field as the float64 weighted sum of the three selected vertex
rows. Reject non-finite output, wrong vertex/channel count or a field whose
mesh binding differs. M0b casts the resulting ten gains to float32 only when
constructing the existing training example; the surface evaluator report stays
float64.

Complexity is `O(F + C)` time per uncached query for `F` faces and `C` field
channels, with `O(1)` working memory beyond the output. The current largest X0
mesh is below 100,000 faces and has four queries, so a custom bounded linear
scan is simpler than a new spatial-index dependency. Implementations may cache
validated immutable face terms by mesh SHA-256, but cache order/identity cannot
affect output.

## M0b preprocessing semantics

For every T0 row opened by the existing role owner:

1. evaluate the query on the refined mesh and interpolate the ten refined gain
   channels; these values replace exact-vertex gain lookup as the T0 gain target;
2. evaluate the identical query on the coarse mesh and verify the coarse gain
   artifact binding/finiteness, without treating a piecewise-linear coarse
   interpolation as identical teacher truth;
3. compute refined and coarse object/contact descriptors exactly as M0a, using
   the original query point rather than projected or snapped coordinates;
4. retain the original synthetic waveform, modes, renderer, losses and model
   remesh comparison unchanged.

For X0, the evaluator validates each disclosed REALIMPACT query against the
exact scanned mesh. No gain field exists and none is fabricated. Identified
recordings remain material-only and have no surface query.

Projection is validation arithmetic, not a query rewrite. The original query
bytes remain the descriptor/provenance input. Any off-surface residual over the
frozen tolerance returns `FallbackOutOfDomain` before training.

## Official-profile structural fixture

The value-independent `surface-query-official-shape-v1` fixture uses no teacher
gain, audio, model, optimizer or protected role value. It freezes:

```text
coarse grid = 17 × 13
refined grid = 33 × 25
plate dimensions metres = [0.137, 0.083, 0.0027]
beam dimensions metres = [0.311, 0.041, 0.0043]
contacts =
  [(1/8,1/8), (7/8,1/8), (1/8,7/8), (7/8,7/8),
   (1/2,1/4), (1/4,1/2), (3/4,1/2), (1/2,3/4),
   (3/8,3/8), (5/8,3/8), (3/8,5/8), (5/8,5/8)]
```

Plate maps `(u,v)` to `(L*u, W*v, T/2)`; beam maps it to
`(L*u, W*(v-1/2), T/2)`. The fixture evaluates both families, both grids and
all contacts: exactly 48 query evaluations and 24 coarse/refined pairs.

Its ten-channel affine control field is:

```text
[1, x, y, z, x+y, x-y, x+z, y+z, 2x-3y+z, -x+4y-2z]
```

Every legal query must resolve. Canonical face/weight output must repeat
byte-for-byte. Maximum reconstruction residual must not exceed the per-mesh
surface tolerance; maximum affine coarse/refined difference must be `<= 2^-48`.

The owning I0 fixture additionally exercises:

- exact vertex, shared edge, triangle interior and every official fraction;
- full face-array reversal, deterministic face permutation, cyclic index order
  and reversed winding with identical canonical query/field output;
- duplicate canonical face, repeated/out-of-range index and degenerate face;
- query NaN/Inf, field NaN/Inf, wrong field shape/channel count and stale mesh
  binding;
- on-plane outside query displaced by `D*2^-12` and off-plane query displaced
  by `2*surface_tolerance_m`, both rejected;
- tolerance mutations by one representable step on each side of the declared
  boundary;
- interrupted/occupied output and canonical report corruption.

The fixture emits a canonical JSON report containing protocol/implementation
identity, fixture constants, evaluation/pair counts, maximum residuals,
canonical query hashes, mutation counts and authority flags. It emits no
teacher, real, model or protected-role value.

## M0b identity, artifacts and access

The new manifest schema is
`nextengine.experimental-physical-sound-v26-m0b.manifest.v1`; the preprocess
and final report schemas use the same prefix with `preprocess.v1` and
`report.v1`. The manifest pins this protocol SHA-256, the complete M0b
implementation root, exact inherited-module hashes and the unchanged official
combined/T0/X0/tool references.

The owning CLI first produces and validates the surface-query structural report
in staging. The official canonical output is the M0a eight-file set plus:

```text
surface-query-report.json
```

All nine files must repeat byte-for-byte between complete A/B executions.
MLflow remains diagnostic and excludes run IDs/timestamps from canonical
identity. Admission shadow and REALIMPACT row `2407` never materialize.

## Stop rules

- P0 freezes before implementation. A semantic change requires a new protocol
  revision before any official M0b value.
- I0 must pass the full M0a entry fixture and the official-profile structural
  fixture twice before an official manifest is built.
- M0a is spent and is never rerun or edited.
- One failed official M0b execution spends M0b; B is not started after A fails.
- Opened M0b values cannot select a tolerance, face tie-break, interpolation,
  model, seed, loss, schedule, threshold, role or retry.
- A pass authorizes only V0/Metal research sequencing. Runtime inference,
  public schemas, material admission and removal of authored fallback remain
  unauthorized.
