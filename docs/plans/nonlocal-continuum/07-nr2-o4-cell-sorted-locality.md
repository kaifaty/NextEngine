# NR2-O4 — Stable cell-sorted storage and neighbor locality

Status: `EXECUTED / O4_RETAINED_STABLE_SAMPLE / NR2_FIXED_WORK_COMPLETE / REPORT_ONLY`

Prerequisite:
[NR2-O3 evidence](../../development/nonlocal-continuum-nr2-o3-evidence-2026-08-20.md)
exits `O3_SEGMENTED_NUMERIC_MISMATCH` and retains the exact O2 stack. O4 does
not retry O3 or inherit its endpoint-fragment association.

## Outcome

Measure whether storing particle state in stable cell order improves memory
locality without changing the retained gather arithmetic. The adjacent
baseline is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
term kernels  = nuv-terms-specialized-o2
storage       = stable-sample-v0
```

The only retainable O4 candidate is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
term kernels  = nuv-terms-specialized-o2
storage       = cell-sorted-o4
```

`--storage` selects this fourth identity explicitly in checks, repeatability
and reports. Commands without it preserve `stable-sample-v0`. The candidate
is rejected at construction with atomic or segmented accumulation, copy
handoff or runtime term kernels. It changes no production or public default.

## Stable storage identity

The frozen reference positions define the storage permutation. Grid origin,
dimensions and packed key remain the existing neighbor-grid construction:

```text
cell[d] = clamp(floor((reference[d] - origin[d]) / horizon), 0, dims[d]-1)
key     = cell.x + dims.x * (cell.y + dims.y * cell.z)
```

Every multiplication/addition is range-checked before the packed key is
admitted to signed 32-bit CUB input. `DeviceRadixSort::SortPairs` sorts by the
complete 32-bit key. Input values are stable sample IDs `0..N-1`; equal-key
values MUST emerge in ascending stable sample-ID order. The implementation
validates that property rather than relying on an unordered hash or an
unstated secondary order.

The resulting immutable maps are:

```text
storage_to_sample[storage_id] = stable sample_id
sample_to_storage[sample_id]  = storage_id
```

They MUST be a bounded bijection and exact inverses. The candidate stores
solver-facing reference, initial velocity, fixed flags and all evolving
particle fields by `storage_id`. Capture, fixture comparison and JSON report
order remain stable sample-ID order through the inverse mapping.

The map is constructed and immutable-input fields are gathered once during
instance setup. This setup time is reported separately and excluded from one
execution. Each execution still performs the same measured CUB grid sort and
CSR reconstruction as the baseline. Its sorted map is checked against the
frozen storage map before the CSR can be consumed; mismatch is a hard error,
not a rebuild, retry or fallback.

## CSR and arithmetic preservation

The baseline visits neighbor cells with the exact loop nesting:

```text
for dz in -1..=1
  for dy in -1..=1
    for dx in -1..=1
      for sorted slot in cell_begin[key]..cell_end[key]
```

O4 retains that 27-cell order and the ascending stable sample-ID order inside
each cell. It constructs CSR segments in `storage_id` owner order, but the
logical segment for a stable owner contains exactly the baseline neighbor
sequence. Neighbor entries contain mapped `storage_id` values. Self remains
present once and at the same logical position.

Consequently every density and directed-gather owner performs the same
floating operations in the same neighbor order over the same physical
values. O4 permits address changes only. It does not permit tolerance-based
acceptance of a changed trajectory: after remapping, every captured field and
logical CSR entry MUST be bit-exact against `stable-sample-v0`.

Reports reconstruct logical CSR as:

```text
logical owner sample i
  -> physical owner sample_to_storage[i]
  -> physical neighbor storage ids
  -> storage_to_sample[neighbor]
```

They include ordered digests for both maps, physical CSR and reconstructed
logical CSR, plus map/secondary-order validation results. A generic unordered
hash table is forbidden in mapping or neighbor construction.

## Frozen dimensions

O4 retains:

- every fixture/profile hash, coefficient, tolerance, term and fixed
  iteration count;
- CUDA `f32`, compiler flags, O1 pointer ownership and O2 specialization;
- directed gather, the full nine-entry matrix and unregularized local inverse;
- prediction, density, term, update, velocity and diagnostic expression order;
- cell size, grid bounds, pair membership, one self slot, maximum degree,
  directed capacity and all overflow behavior;
- explicit clears, passes, launches and stage boundaries;
- default atomic/copy/runtime behavior and every NR1/RC1/O1/O2 identity;
- the failed O3 identity as a selectable diagnostic that cannot combine with
  O4.

O4 does not evaluate unique pairs, pre-add endpoints, fuse passes, remove
clears, cache neighbors across executions, change precision, use textures or
shared-memory tiling, introduce adaptive work or alter the nonlinear method.

## Capacity contract

The candidate adds two `i32` maps, sorted immutable `float3` reference and
velocity arrays, sorted `u8` fixed flags and one `i32` storage error flag:

```text
O4 bytes = N * (4 + 4 + 12 + 12 + 1) + 4 = 33*N + 4
```

No second evolving state, CSR copy or extra CUB scratch allocation is admitted.

| Profile scale | O4 bytes | Total device-memory ceiling |
|---|---:|---:|
| 16k | `528,004 B` | `11,442,058 B` |
| 48k | `1,584,004 B` | `34,309,770 B` |

One retained and one candidate 48k instance must coexist below
`70,000,000 B`. Reports include total/candidate bytes, exact map and sorted
field bytes, setup time, directed capacity/actual count, maximum degree and
all map/CSR digests. A 48k allocation failure or exceeded bound exits before
timing.

## Ordered correctness gates

Run in order and stop before retention timing on the first candidate failure:

| Gate | Required result |
|---|---|
| `O4-BUILD` | clean external CMake/Ninja build PASS under unchanged CUDA flags |
| `O4-LEGACY` | default atomic/copy/runtime and retained gather/swap/specialized tiny each PASS 11/11; invalid O4 combinations reject; failed O3 remains selectable only under its frozen identity |
| `O4-TINY` | candidate PASS 11/11 against CPU; three reused executions exact; remapped candidate/baseline fields and logical CSR bit-exact; both maps, packed keys, secondary order and bounded inverse pass |
| `O4-SURFACE-I2/I20` | ten cold candidate repeats at both counts exact; retained correspondence, logical CSR, momentum, finiteness and local solve bit-exact/pass |
| `O4-FULL` | water-16k/48k i5 and viscous-16k i20 pass topology/capacity/momentum; two reused executions exact; retained/candidate fields and logical CSR bit-exact |
| `O4-STORAGE` | keys nondecreasing, equal-key stable IDs ascending, maps bijective, measured sort reproduces the immutable map, physical degree bound and declared memory exact |

The candidate output digest is taken after stable-ID remapping. Any numeric or
logical-CSR difference stops O4; the surface tolerances are not widened and a
different deterministic trajectory is not accepted.

## Same-process locality tournament

After correctness, run one fixed command for water-16k, water-48k and
viscous-16k:

```text
nonlocal-feasibility --locality-tournament <profile> --warmup 32 --runs 96
```

The command owns one retained and one candidate instance in one process. Each
warm-up and measured round alternates exact order:

```text
stable-sample -> cell-sorted
cell-sorted   -> stable-sample
```

Ninety-six measured rounds give each identity each position 48 times. CUDA
events report minimum, median, p95, p99 and mean for every existing stage and
total. One-time map/gather setup is reported but excluded from total. The
command checks before/after output, logical CSR, mapping and memory identities
and fails instead of emitting retention timing if any check changes.

Reports additionally include mean and p95 absolute neighbor-storage distance
as diagnostic evidence of locality. This metric cannot select retention by
itself; timings and correctness remain decisive.

## Retention decision

Retain `cell-sorted-o4` only if:

- every correctness, storage and capacity gate passes;
- total p95 improves on both HN-3 denominator profiles, water-48k and
  viscous-16k;
- their adjacent total-p95 geometric-mean speedup is at least `1.05x`;
- water-16k total p95 does not regress by more than `2%`;
- the sum of density, incompressibility, viscosity and surface p95 does not
  regress on any profile;
- candidate and co-resident allocations remain within the declared ceilings.

The `1.05x` floor pays for a new storage identity, inverse mapping and extra
memory while remaining below O3's larger layout threshold. Stage values are
not added to O1/O2 percentages or compared across binaries.

After O4, apply the parent NR2 early-stop rules using one same-process
aggregate comparison of the final retained stack against the original HN-3
`source-atomic-v0 + copy-v0 + nuv-terms-runtime-v0` denominator on its two
correctness-valid profiles. O3 is excluded. Only if the fixed-work ladder
remains open may O5 receive the retained O4 result.

Execution closure:
[O4 evidence](../../development/nonlocal-continuum-nr2-o4-evidence-2026-08-20.md)
passes every exactness/storage/capacity gate but records a `0.938546x`
denominator geometric-mean candidate speedup. O4 exits
`O4_RETAINED_STABLE_SAMPLE`. The final retained NR2 stack reaches `3.27688x`
against HN-3 and water-48k p95 `4.019520 ms`; final profiling leaves no
untested O5 stage at `20%`, so fixed-work research closes for NR4.

## Exit states

### `O4_RETAINED_CELL_SORTED`

All exactness, storage, capacity and retention rules pass. Cell-sorted storage
becomes the input to O5; stable-sample remains its rollback comparator.

### `O4_RETAINED_STABLE_SAMPLE`

Correctness passes but memory/performance retention fails. Preserve the
selectable locality candidate and its evidence, retain the O2 stack, then
apply the NR2 early-stop rules without crediting O4 speedup.

### `O4_STORAGE_IDENTITY_MISMATCH`

Packed-key range, stable secondary order, map inverse, measured-map equality
or logical CSR identity fails. Retain stable-sample and stop before timing.

### `O4_NUMERIC_MISMATCH`

Any remapped field differs from retained stable-sample output. Preserve the
first failing fixture/iteration/digest; do not widen tolerances or reorder the
logical neighbor sequence.

### `O4_CAPACITY_REJECTED`

Candidate exceeds a declared byte/capacity bound or cannot run 48k. Retain
stable-sample and stop before timing.

## Non-goals

O3 remediation, pair deduplication, pass fusion, clear/launch removal, CUDA
Graphs, warp/shared-memory tiling, cached topology, adaptive particles,
precision changes, adaptive exit, warm start, line search, Pairwise Descent,
runtime integration, public contracts, Windows execution and NR4 selection.
