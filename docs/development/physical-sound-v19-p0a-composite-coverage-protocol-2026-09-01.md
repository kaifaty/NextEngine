# Physical sound V19 P0a — composite coverage protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V19 P0a–C0](../plans/physical-sound-synthesis-roadmap-v19.md) |
| Research basis | [V19 composite coverage research](physical-sound-v19-composite-coverage-research-2026-09-01.md) |
| Closed predecessor | [V18 O0](physical-sound-v18-o0-intrinsic-coverage-result-2026-09-01.md), repeat-exact reject |
| Allowed claim | One deterministic capability certificate for structural completeness and intrinsic coverage on bounded synthetic meshes |
| Product effect | None; no real/protected access, public schema, runtime model, cooker or fallback change |

## Question and stop rule

Can a deterministic composite distinguish complete, intrinsically covered
contact context from incomplete, disconnected, clustered or ambiently
misleading context without rejecting fresh valid meshes?

C0 runs only after this protocol and implementation/tests are separate Git
checkpoints. The test band is generated and evaluated once only after the
implementation checkpoint. A repeat-exact rejection closes V19 before P0b/F0
and I0. No threshold, reason precedence, row, grid, mutation or control changes
after test opening.

Passing C0 proves only coverage-certificate capability. It does not prove that
the later modal-gain model works or that synthetic truth matches real sound.

## Isolation and access ledger

- Development uses only `n=1101…1112`; test uses only `n=1201…1212`.
  No V16–V18 row, prediction, score, report or external run is an input.
- Test row formulas are frozen here, but test geometry, distances, thresholds,
  scores and decisions remain ungenerated until C0 implementation and focused
  tests are committed.
- Candidate code receives only its row, canonical mesh and coverage record. It
  cannot read a later field truth, B0 artifact, real waveform/force, source
  body, generator-real value, protected calibration, method holdout, admission
  shadow or network response.
- Every forbidden category and `test_rows_generated_before_commit` has an
  explicit access counter frozen at zero.
- Outputs publish atomically only to a fresh empty directory below
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`. A symlink,
  nonempty root or path escape fails before generation.
- Authored clips remain the complete fallback. Runtime inference and product
  contracts remain unauthorized.

The pinned environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`,
float64 CPU and one numerical-library thread. C0 uses no PyTorch, GPU, random
seed, optimizer or checkpoint.

## Exact corpus enumeration

Orders are fixed:

```text
materials = [Steel, Wood, Glass]
topologies = [Plate, Cylinder, Bowl, RolledSheet]
supports = [Free, BaseClamped]
c = material_index*4 + topology_index, c=0..11
```

For positive `n`, `H_b(n)` is the exact radical inverse from
[V17 P0a](physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md),
SHA-256 `39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e`.

| Role | `n` | Support index | Grid `(u,v)` | Count |
| --- | --- | --- | --- | ---: |
| development/calibration | `1101+c` | `c mod 2` | `(22+c mod 2, 18+c mod 3)` | 12 |
| one-shot test | `1201+c` | `(c+1) mod 2` | `(24+c mod 2, 19+c mod 3)` | 12 |

For both roles:

```text
L = 0.19 + 0.31*H_2(n)
aspect = 0.68 + 0.88*H_3(n)
slenderness = wall/L = 0.0035 + 0.0050*H_5(n)
wall = L*slenderness
physical_group_id =
  "v19-c0-{role}-{material}-{topology}-{support}-{n}" in lowercase
object_id = physical_group_id + "-primary"
```

`role` is exactly `development` or `test`. Canonical JSON retains `c`, `n`,
all IEEE-754 float64 row values, grid dimensions and IDs. Numeric arrays are
canonical little-endian float64/int64.

The UV, Plate/Cylinder/Bowl/RolledSheet embeddings, triangle winding, periodic
rules and unique metric edge graph are exactly V17 P0a. In particular,
RolledSheet has coincident ambient seam vertices but open intrinsic U
connectivity. Mesh identity hashes the canonical row, vertices and faces.

## Frozen development reachability

Only the development role was generated before this freeze. Its canonical
mesh-record root is
`b3af64b6ed2b9c4f7bafbbcd16a614a15db9609763731898d05046c8db5d8b52`.
It has 12 unique object/mesh identities, `396…460` vertices and `50…58`
required context vertices. Scalar ranges are:

| Quantity | Minimum | Maximum |
| --- | ---: | ---: |
| `L` | `0.2022607421875` | `0.4831982421875` |
| aspect | `0.7069593049839964` | `1.5435025148605395` |
| slenderness | `0.0038856000000000003` | `0.007885600000000001` |
| valid intrinsic fill / diameter | `0.060876848924084424` | `0.10656841357125353` |
| valid intrinsic mesh ratio | `1.9172846170127238` | `2.0` |

The 12 development mesh hashes in cell order are:

```text
29e5c434f246682a5eaa3cf374a51208397a9d3928a00c9c8639d4e79a7ae8ef
8ae090b0d076a74fe8bd963d7f601451e926375032ed106db981d6771f8762b5
1638f9c5ea9259aff1e893e4f0ff9da03ef6c4f2e830089f0cfae913833e1f34
1a071f694491d4e45a828bac6f868f1ac4fea97361f82d0994f17454476a1182
5c73fb313e4d0657af64f9f4283a3b81bbfaf9c4ccbc4aa4a78a39a89b23893f
44cf05e811445a5e57d777404ca97df236c68f758e026b62dbb69b5391904159
9e2d8310540c23b27035a73e03e1743a28ded0b04873b26af1bba8f498b42aac
494a31c9d83c7250ea07ca509c4a38cc2428769ca8a5559a02081c0a2a5bbfa1
590dce3c53471c59e1216877f1fdb24e96ed62b62f124e91c455a2399c4122a8
ee63418f0b06a1a59bb97189d56a3d37d693f01f9101225d6d6415f99399dfd7
519e6afc6e02168733a7c281d9ec17ca9d1d9a1f0f807d5ec9238313837b2a49
943af29fcc9629f3e6e4fa5d2a186d0e77f7a9392d1bdd081f947fc13e85b229
```

These values prove evaluator reachability only. No test geometry or candidate
quality value was produced.

## Context and structural contract

For a valid mesh with `N` vertices:

```text
K = max(16, ceil(N/8))
```

Context is deterministic intrinsic farthest-point sampling. It starts at the
lexicographically first `(x,y,z,vertex_index)` and repeatedly selects the
vertex with greatest nearest selected graph distance, breaking ties by the
same lexical rank. Query is the remaining vertex set in increasing index
order.

`CoverageInputV0` contains schema/study revision, physical-group/object/mesh
identity, row and graph hashes, `N`, edge count, derived and declared `K`,
ordered context/query indices and their canonical hashes. Structural closure
requires:

- exact schema, study, object, mesh, row and graph identity;
- declared `K` equals the certificate-derived formula;
- declared counts/hashes equal their payloads;
- context/query are nonempty, unique, in range and disjoint;
- unique context count is at least `K`.

Any failure returns `OOD_CONTEXT_BUDGET`; numerical distance is not evaluated.
The detail code records the first stable cause but does not change the public
reason.

## Intrinsic statistics and frozen calibration

The authoritative distance is exact SciPy sparse Dijkstra on the unique
triangle-edge graph. `D` is the maximum finite all-pairs distance of the valid
unmodified connected mesh.

For context `S` and vertex/query `q`:

```text
d(q,S) = min_{s in S} graph_distance(q,s)
local_score(q) = d(q,S)/D
global_fill(S) = max_{v in V} d(v,S)/D
separation(S) = 0.5*min_{s!=t in S} graph_distance(s,t)/D
mesh_ratio(S) = global_fill(S)/separation(S)
```

Development freezes:

```text
local_threshold = max(0.05, 1.25*p99(valid development local_score))
                = 0.11869598258542362
global_threshold = max(0.05, 1.25*max(valid development global_fill))
                 = 0.1332105169640669
```

The Euclidean negative control uses the same formulas and freezes
`local=0.16904819773036295`, `global=0.19098768258200008`.
Separation and mesh ratio are diagnostics, never alternate accept paths.

## Decision precedence

For every query, exactly the first applicable outcome is emitted:

1. structural failure: `OOD_CONTEXT_BUDGET`;
2. query unreachable from context: `OOD_DISCONNECTED`;
3. finite object `global_fill > global_threshold`: `OOD_INTRINSIC_FILL`;
4. finite query `local_score > local_threshold`: `OOD_INTRINSIC_GAP`;
5. otherwise: `ACCEPT`.

NaN is forbidden and infinity is never serialized as JSON. An unreachable
numeric value is represented by its reason plus a null score/status. Invalid
or rejected records never reach a learned model.

## Frozen mutations

Every development/test topology has four numerical mutations:

1. **intrinsic cap:** from the lexical seed, context is the nearest `ceil(0.40N)`
   vertices and query is the farthest `ceil(0.30N)` vertices;
2. **component isolation:** remove graph edges crossing the median V row,
   choose the lexical-seed component, rerun intrinsic FPS inside that component
   to exactly `K` context vertices and query every vertex in the other component;
3. **thinning:** retain `valid_context[::4]` and query the intrinsically
   farthest `ceil(0.25N)` vertices on the original graph;
4. **ambient shortcut:** RolledSheet uses context `u<=-0.75` and query
   `u>=0.75`; other topologies use intrinsic cap as balanced controls.

Each valid record also has four structural mutations, all scored over its
unchanged valid query set:

1. **duplicate context:** replace the last context index with the first while
   retaining the declared count;
2. **out of range:** replace the last context index with `N`;
3. **minimum tamper:** declare `K-1` while retaining the payload;
4. **identity mismatch:** replace the mesh hash with 64 zeroes.

All structural mutations must stop before Dijkstra and return
`OOD_CONTEXT_BUDGET` for every query.

## Controls and successful pre-freeze result

Frozen controls are:

- V18-style local intrinsic distance only;
- local Euclidean distance only;
- graph-only disconnection/global-fill/local-gap with structural identity and
  minimum checks disabled except memory-safe index bounds;
- structural-only closure with every graph statistic disabled.

On the development role, the composite rejects `100%` of all four numerical
mutation/topology cells. The local-intrinsic and Euclidean controls reject
`0.9720985455624814` and `0.7914811516770555` of pooled numerical mutation
queries. Every thinning decision is `OOD_CONTEXT_BUDGET`, every component
query is `OOD_DISCONNECTED`, and every cap/ambient decision is
`OOD_INTRINSIC_FILL`. All four structural mutations reject before distance.

Removing structural closure makes identity mismatch acceptable; removing
graph checks makes intrinsic cap, component isolation and ambient shortcut
acceptable. These successful ablations prove both layers are reachable. They
do not predict the unopened test outcome.

## C0 gates

C0 passes only if both complete runs satisfy every item:

- valid false-OOD fraction is `<=0.10` in every topology aggregate and at
  least `11/12` valid objects have per-object false OOD `<=0.10`;
- each numerical mutation/topology aggregate rejects `>=0.95` of queries and
  at least `11/12` objects reject `>=0.95` across their four mutations;
- thinning is `100% OOD_CONTEXT_BUDGET`, component isolation is
  `100% OOD_DISCONNECTED`, and RolledSheet ambient shortcut is at least
  `95% OOD_INTRINSIC_FILL|OOD_INTRINSIC_GAP`;
- all 48 structural mutation records reject `100%` of queries as
  `OOD_CONTEXT_BUDGET` before any distance counter increments;
- composite numerical utility is no lower than local intrinsic and Euclidean
  utility; RolledSheet ambient rejection is strictly greater than Euclidean;
- structural-only and graph-only ablations each accept at least one complete
  mutation family that the composite rejects, with zero composite regression;
- every row/object/mesh/case/query identity is unique and complete; all finite
  score records are finite and no NaN/infinity is serialized;
- all forbidden access counters and premature test-generation counter are zero;
- manifests, corpus, geometry, calibration, decisions, scores, report and
  access ledger match byte-for-byte across two independent fresh roots.

For a method, mutation rejection is the unweighted mean of the 16
mutation/topology query aggregates. Valid false rejection is the unweighted
mean of four topology aggregates. Utility is rejection minus false rejection.
Structural mutations are reason gates and do not inflate numerical utility.

The utility rule was changed from Roadmap V19's provisional strict aggregate
win before this freeze: a bounded score can saturate at `1.0`, so strict
superiority could reject a semantically stronger exact certificate when a
control ties accidentally. Non-inferiority plus layer-specific ablation gates
proves the added capability without an impossible saturated comparison. No
test value informed this change.

## Artifact and compute boundary

Each run emits exactly these eight files:

```text
manifest.json
corpus.json
geometry.npz
calibration.json
decisions.jsonl
scores.npz
report.json
access-ledger.json
```

The runner processes one mesh at a time; test `N<=525`, context `K<=66`, and
no unbounded queue exists. All-pairs arrays and geometry remain external. Wall
time is diagnostic because host load is not a capability property; GPU,
network and learned-model work are forbidden.

## Implementation order

1. Commit this protocol and its roadmap/task-state transition.
2. Implement exact rows, mesh/hash closure, certificate, controls, serializers
   and focused successful/failure tests using development only.
3. Commit implementation. The commit must record zero generated test rows.
4. Generate/evaluate test exactly once into run A, then independently run B.
5. Record repeat-exact Pass or Reject without repair. Only Pass unseals P0b.
