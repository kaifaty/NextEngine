# Nonlocal corrected CUDA neighborhood/index correspondence — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NCGA1` revision 1 |
| Status | `FROZEN / IMPLEMENTATION_PENDING / REPORT_ONLY` |
| Architecture snapshot | `7d368b69e8a443cdace20473451a570e3c79ef4d`; SPEC-38 and ADR-076 remain `Proposed`; ADR-081 remains `Accepted` |
| Engineering consumer | Decide whether the corrected CUDA lineage may proceed from reviewed pair terms to separately frozen local assembly |
| Claim class | Exact finite-set neighborhood and stable-index correspondence |
| Review budget | One independent initial review and at most one repair/re-review |

## Question and competing hypotheses

The historical CUDA solver agrees with its source-shaped CPU path on eleven
tiny cases, but that evidence does not isolate spatial indexing from formulas,
assembly or the solver. NCGA1 asks only whether a GPU cell index can reconstruct
the exact V1 water support graph from canonical sample state.

- `H1`: exact signed-cell construction and stable `SampleId` projection on the
  GPU reproduce an independent all-pairs CPU oracle under input permutation.
- `H2`: a radius boundary error changes membership even when every position is
  an exact canonical micrometre value.
- `H3`: array-position identity or non-floor signed cell division changes the
  cache identity under adversarial ordering/negative coordinates.

The first failed mandatory gate ends the positive claim. No fixture, bound,
algorithm identity or expected result may change after observing execution.

## Exact input and neighborhood semantics

The profile is the current SPEC-38 V1 water profile only:

```text
support_radius = cell_size = 100000 micrometres
maximum_samples = 256
maximum_neighbors_per_sample = 256
```

Each immutable fixture record contains a unique `u32 SampleId` and three signed
`i64` micrometre coordinates. Coordinates are finite by representation and MUST
lie in `[-1000000000, 1000000000]`. Empty fixtures, duplicate `SampleId`, excess
sample/cell/pair capacity and arithmetic overflow reject before a result is
published.

For samples `i,j`, membership is the exact integer predicate

```text
(xi-xj)^2 + (yi-yj)^2 + (zi-zj)^2 <= 100000^2.
```

Self is present exactly once. Every row is owned in ascending `SampleId` order;
neighbor IDs inside each row are also strictly ascending. Input array position
has no semantic meaning.

The signed cell coordinate is mathematical floor division

```text
cell_axis(x) = floor(x / 100000).
```

The frozen packed cell key biases each axis by `2^20` and concatenates three
21-bit unsigned fields as `(x,y,z)` from most to least significant. The admitted
coordinate range keeps every cell in `[-2^20, 2^20-1]`. The reconstructed cache
is exposed in stable `(packed_cell_key, SampleId)` order as well as canonical
CSR so cell mapping and pair membership are independently observable.

## Independent oracle and CUDA candidate

The host oracle is a separate C++ translation unit. It performs direct
`O(N^2)` checked integer distance tests, an independent floor-division/key
implementation, explicit stable sorts and no call into CUDA candidate helpers.

The CUDA candidate is a separate target for NVIDIA RTX 3080 (`sm_86`) and CUDA
13.3. It MUST:

1. compute signed cell keys on device;
2. use a device radix sort to construct stable `(cell key, SampleId)` storage;
3. visit exactly the 27 adjacent signed cells through device range lookup;
4. build row counts/offsets and neighbor IDs on device;
5. emit canonical ascending `SampleId` rows and neighbors; and
6. return explicit work counts and a failure code.

Host-side fixture admission, allocation and final byte comparison are allowed.
Host construction or canonicalization of the candidate cell index/CSR is not.
No source from `cuda_baseline.cu`, `oracle.cpp` or the NCGA0 force oracles may
be linked into either NCGA1 evaluator.

## Frozen fixtures

The ordered corpus is:

1. `isolated_nonzero_id` — one nonzero ID and self membership;
2. `support_edge` — an exact-radius pair plus one sample one micrometre outside;
3. `negative_cell_floor` — negative coordinates on both sides of exact cell boundaries;
4. `duplicate_positions_distinct_ids` — coincident samples with distinct IDs;
5. `adjacent_and_diagonal_cells` — face/edge/corner adjacent cells with inside/outside pairs;
6. `scrambled_cloud` — non-contiguous IDs and a fixed non-spatial input order;
7. `scrambled_cloud_reversed` — the same semantic cloud in reverse input order; and
8. `water_lattice_4x4x4_scrambled` — spacing `50000` micrometres with a fixed affine permutation.

The two cloud fixtures MUST have byte-identical canonical cache and CSR payloads.
All eight candidate payloads MUST equal the independent oracle byte-for-byte.

## Mandatory negative controls

The same executable contains deliberately wrong CUDA identities. A control
passes only when the common comparator rejects the named fixture:

1. `strict_radius` uses `<` and MUST reject `support_edge`;
2. `truncate_signed_cell` uses C/C++ truncation toward zero and MUST reject
   `negative_cell_floor` through the cache-order/key gate;
3. `array_index_identity` publishes input positions instead of `SampleId` and
   MUST reject `scrambled_cloud`; and
4. `same_cell_only` omits adjacent cells and MUST reject
   `adjacent_and_diagonal_cells`.

No control may use a special expected output path. It is evaluated by the same
oracle comparator as the positive candidate.

## Exact gates and work receipt

Every positive fixture requires exact equality of:

- ordered `(packed_cell_key, SampleId)` vectors;
- owner `SampleId` vector;
- CSR offsets and neighbor `SampleId` vector;
- sample, directed-pair and maximum-degree counts;
- self count, symmetry and strict row-order predicates; and
- a canonical binary payload SHA-256.

The result also records exact counts for device key evaluations, radix-sort
items, owner-sort items, 27-cell probes, range binary-search comparisons,
distance predicates, count writes, fill writes and emitted directed pairs.
Repeated executions MUST report the same work receipt. Counts are evidence for
this isolated finite run, not a performance claim.

Ten cold allocation/execution runs MUST produce byte-identical positive
payloads and receipts. Fresh Release runs must be byte-identical. CUDA
`memcheck`, `initcheck` and `synccheck` must report zero errors.

## Resolution and claim ceiling

- `SUPPORTED_BOUNDED`: eight exact positives, four rejected negatives, cold
  repeatability, sanitizers and retained NCGA0/old tiny controls all pass, then
  an independent reviewer returns `GO`.
- `REFUTED`: a valid admitted positive fixture differs from the oracle. Preserve
  the first mismatch and do not proceed to local assembly.
- `INCONCLUSIVE`: environment/identity closure fails, the oracle shares
  candidate logic, a load-bearing defect survives the single repair/re-review,
  or required evidence cannot be reproduced.

This contract can establish only exact neighborhood/cache/index correspondence
for the frozen finite integer fixtures on one GPU/toolchain. It establishes no
float solver arithmetic, local energy/source/matrix assembly, factorization,
SISSM recurrence, trajectory, stability, visual water quality, throughput,
runtime integration, canonical GPU authority or product-ready water.

## Evidence and review protocol

1. Commit this contract before implementation.
2. Build the isolated target in a fresh directory outside Git and run it twice.
3. Run ten cold positive executions, all negative controls and all sanitizers.
4. Re-run NCGA0 and the retained historical CUDA tiny control without changing
   their sources or expected results.
5. Freeze contract/source/binary/stdout/payload roots, repository snapshot,
   parent-to-snapshot diff and exact commands in one evidence report.
6. Give a fresh reviewer only the frozen contract, exact candidate snapshot and
   neutral review request. Candidate files remain read-only during review.

## Stop and reconsider

Stop before implementation if the canonical integer state, stable identity or
support predicate cannot be kept distinct from historical float/index behavior.
Stop after the first valid positive mismatch; do not widen the support radius,
sort on the host, drop a boundary fixture or replace stable IDs with array
positions. Even a reviewed positive result authorizes only a new separately
frozen local-assembly audit.
