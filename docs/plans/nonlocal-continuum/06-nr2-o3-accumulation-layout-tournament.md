# NR2-O3 — Accumulation-layout tournament

Status: `SPECIFIED / IMPLEMENTATION_NOT_STARTED / REPORT_ONLY`

Prerequisite:
[NR2-O2 evidence](../../development/nonlocal-continuum-nr2-o2-evidence-2026-08-20.md)
exits `O2_RETAINED_TERM_SPECIALIZATION` and admits O3 from the exact
gather/swap/specialized identity.

## Outcome

Measure whether evaluating every non-self symmetric pair once plus a stable
owner reduction improves the retained directed-gather implementation enough to
justify its fragment memory and extra launches. O3 does not change equations,
active terms, iteration count, term order, local-system shape or neighbor
membership.

The retained baseline is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
term kernels  = nuv-terms-specialized-o2
```

The only retainable O3 candidate is:

```text
accumulation = nuv-unique-pair-segmented-o3
handoff       = pointer-swap-o1
term kernels  = nuv-terms-specialized-o2
```

The existing source-shaped path remains the third tournament observation:

```text
accumulation = source-atomic-v0
handoff       = copy-v0
term kernels  = nuv-terms-runtime-v0
```

`source-atomic-v0` is historical/diagnostic only. It cannot win O3 because its
stiff-surface trajectory is already `BASELINE_MISMATCH`, and its handoff/term
identities predate retained O1/O2. It is timed only on the water and viscous
profiles where its existing correctness contract passes.

`nuv-unique-pair-segmented-o3` is rejected at construction unless
`pointer-swap-o1` and `nuv-terms-specialized-o2` are selected. Commands without
an accumulation selector retain their legacy atomic default; O3 changes no
production or public default.

## Candidate layout

The frozen symmetric CSR is already segmented by output owner. O3 therefore
does not add a global owner-key sort merely to reproduce the existing segment
order.

For every directed slot `s = (i -> j)`, preflight constructs an exact reverse
slot `r(s) = (j -> i)`. A term pair kernel assigns one thread to owner `i`,
walks its frozen CSR range and evaluates only `i < j`. That evaluation writes:

```text
fragment[s]    = complete endpoint contribution owned by i
fragment[r(s)] = complete endpoint contribution owned by j
```

Self slots receive a zero fragment. After the pair kernel completes, one owner
reduction thread walks the original `offsets[i]..offsets[i+1]` order and adds
each fragment to `source[i]` and the full nine-entry `matrix[i]`.

One fragment is exactly:

```text
3 x f32 source + 9 x f32 matrix = 48 B
```

One reverse-slot index is `4 B`, giving `52 B` per admitted directed
capacity slot. The fragment buffer is reused between incompressibility,
viscosity and surface passes. O3 retains explicit source/matrix clears and one
pair plus one reduction launch per active term; clear/pass/launch fusion
belongs to O5.

The reverse map is constructed once from the immutable reference CSR during
baseline setup. Its device setup time is reported separately and excluded from
the per-execution timeline. Every execution rebuilds the frozen CSR as before
and validates reverse-slot range, owner, neighbor and involution in the
neighbor stage before term use. A mismatch fails the run; it never silently
rebuilds, retries or changes pair order.

## Mathematical and work obligations

Let the directed relation contain exactly one self slot per sample. The
candidate MUST report:

```text
nonself_directed = directed_pairs - samples
unique_pairs     = nonself_directed / 2
pair evaluations per active term = unique_pairs
physical endpoint fragments      = nonself_directed
```

The directed baseline reports `nonself_directed` pair evaluations per active
term. Odd non-self count, missing reverse slot, duplicate endpoint ownership
or a non-involutive reverse map is a hard correctness failure.

For one unordered pair `{i,j}`, the candidate independently forms the same two
named directed endpoint contributions that gather would assign to each owner.
It may add those two values into one endpoint fragment before the owner
reduction. That changes `f32` association and is tolerance-compared against
gather; it does not authorize factor-of-two algebraic simplification, packed
matrices, coefficient changes or term fusion.

Bulk-only, shear-only and bulk+shear viscosity remain compile-time closed as
in O2. Diagnostic energy remains the independent runtime-mask comparator and
is outside timed term accumulation.

## Frozen dimensions

O3 retains:

- every profile, fixture/input hash, coefficient, tolerance and fixed iteration
  count;
- CUDA `f32`, compiler flags, O1 pointer ownership and O2 term masks;
- density construction, prediction, update, velocity and energy diagnostics;
- the full `3x3` local inverse and all source/matrix/error clears;
- exact CSR membership/order, cell traversal, term order and report ordering;
- default atomic/copy/runtime behavior and all NR1/RC1/O1/O2 identities.

O3 does not sort particles, change cell locality, remove pair terms, fuse
passes, use cooperative kernels, change precision or introduce adaptive work.

## Capacity contract

Performance profiles reserve `samples * 123` directed slots. With one
`48 B` fragment and one `4 B` reverse index per slot, the predeclared candidate
allocation is:

| Profile scale | Slot capacity | O3 layout bytes | Total device-memory ceiling |
|---|---:|---:|---:|
| 16k | `1,968,000` | `102,336,004 B` including one layout error flag | `113,250,058 B` |
| 48k | `5,904,000` | `307,008,004 B` including one layout error flag | `339,733,770 B` |

The candidate may allocate less but not more. Tournament co-residency of the
historical, retained and candidate instances is capped at `420,000,000 B`.
CUB sort/reduce scratch, a second fragment copy, owner keys, packed-matrix
experiments and capacity growth are not admitted by O3.

Reports include candidate/baseline device bytes, layout bytes, bytes per
capacity slot, setup time, directed/non-self/unique counts and evaluations per
active term. A 48k allocation failure or exceeded bound exits before timing.

## Ordered correctness gates

Run in order and stop before tournament timing on candidate failure:

| Gate | Required result |
|---|---|
| `O3-BUILD` | external CMake/Ninja build PASS under unchanged CUDA flags |
| `O3-LEGACY` | default atomic/copy/runtime tiny PASS 11/11; retained gather/swap/specialized tiny PASS 11/11; invalid atomic, copy or runtime-term + segmented combinations reject |
| `O3-TINY` | segmented tiny PASS 11/11 against CPU; gather correspondence passes; bulk-only, shear-only and bulk+shear execute; three reused candidate executions are exact; CSR and reverse map pass |
| `O3-SURFACE-I2/I20` | ten cold segmented repeats at both counts are exact; gather correspondence and momentum pass; CSR/reverse-map identity is exact |
| `O3-FULL` | water-16k/48k i5 and viscous-16k i20 pass finite/topology/capacity/momentum; two reused candidate executions are exact; gather/candidate full-field correspondence passes |
| `O3-WORK` | every profile reports one self slot per sample, exact symmetric reverse mapping, `unique_pairs=(directed_pairs-samples)/2`, one physical endpoint fragment per non-self directed slot and the declared memory bound |
| `O3-ATOMIC-CONTROL` | source-atomic water-16k/48k i5 and viscous-16k i20 remain tolerance-valid; exact repeatability is neither required nor inferred |

Gather/candidate ordered output digests are reported. Candidate repeats MUST be
exact on the frozen Linux/CUDA/RTX 3080 environment; gather correspondence uses
the existing absolute-or-relative field contract because endpoint pre-addition
changes floating association.

## Same-process tournament timing

After correctness, run one fixed command per water-16k, water-48k and
viscous-16k profile:

```text
nonlocal-feasibility --layout-tournament <profile> --warmup 32 --runs 96
```

The command holds all three instances in one process. Each warm-up/measured
round executes all three layouts, rotating the order exactly:

```text
atomic -> gather -> segmented
gather -> segmented -> atomic
segmented -> atomic -> gather
```

Ninety-six rounds give every layout every position 32 times. Device-event
timings report complete stage/total minimum, median, p95, p99 and mean for each
identity. One-time segmented reverse-map setup is reported but excluded from
the execution total. The command fails unless before/after correctness,
topology, candidate exactness and co-resident memory gates pass.

Historical atomic timing is diagnostic. The retention comparison is only the
adjacent-in-process retained gather versus segmented candidate from the same
binary, fixture and rotation.

## Retention decision

Retain `nuv-unique-pair-segmented-o3` only if:

- every candidate correctness/work/capacity gate passes;
- total p95 improves on both HN-3 denominator profiles, water-48k and
  viscous-16k;
- the geometric-mean total-p95 speedup across those two profiles is at least
  `1.10x`, paying for the roughly tenfold solver-memory increase;
- water-16k total p95 does not regress by more than `2%`;
- the result remains inside the declared candidate and co-resident memory
  ceilings.

No source-atomic result can override this rule. Stage movement is reported but
not added across O1/O2/O3 or non-adjacent binaries.

## Exit states

### `O3_RETAINED_UNIQUE_PAIR_SEGMENTED`

All correctness and retention rules pass. The segmented identity becomes the
input to O4; gather remains its rollback comparator.

### `O3_RETAINED_GATHER_DIRECTED`

Candidate correctness passes but memory/performance retention fails. Preserve
the selectable segmented evidence path, retain gather/swap/specialized for O4
and do not count O3 as speedup.

### `O3_SEGMENTED_NUMERIC_MISMATCH`

Candidate repeatability, reverse-map, CPU or gather correspondence fails.
Preserve the failed identity and exact first boundary; retain gather. Do not
widen tolerances, change expression association again or attempt a second
segmented family without a new specification.

### `O3_CAPACITY_REJECTED`

Candidate exceeds a declared byte/capacity bound or cannot execute 48k. Retain
gather and proceed to O4 without timing the rejected candidate.

## Non-goals

Production CUDA abstractions, CUB owner-key sorting, matrix packing, pass
fusion, clear removal, CUDA Graphs, cell sorting, precision changes, adaptive
exit, warm start, line search, Pairwise Descent, runtime integration, public
contracts, Windows execution and NR4 selection.
