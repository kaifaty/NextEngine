# NR1-RC1 — Deterministic accumulation reclosure

Status: `EXECUTED / NR1_RECLOSED_GATHER_DIRECTED / NR2_UNBLOCKED / REPORT_ONLY`

Research basis:
[accumulation reclosure research](../../development/nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md).

Execution evidence:
[NR1-RC1 reclosure report](../../development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md).

## Outcome

Test one separately identified owner-only accumulation path that preserves the
NR1 Nonlocal/SISSM mathematical workload while removing run-dependent reverse
endpoint `f32 atomicAdd` ordering. Passing this stage reopens NR2; it does not
change SPEC-38, replace DFSPH, authorize GPU authority or receive W2 credit.

The implementation identity is `nuv-gather-directed-r0`. The failed
`source-atomic-v0` binary, evidence hashes and timings remain immutable.

## Frozen and changed dimensions

The candidate MUST retain:

- every `nuv-*.v0` fixture record, coefficient, term set and fixed iteration
  count;
- the current uniform-grid build and exact symmetric CSR membership/order;
- CUDA `f32` storage/compute, current compiler flags and local `3x3` solve;
- the current density, energy, position handoff and velocity reconstruction;
- all absolute/relative, finite-value, capacity and momentum gates;
- existing explicit clears during RC1, even where owner overwrite makes them
  redundant.

It changes only accumulation ownership:

- one CUDA thread owns output particle `i`;
- that thread traverses the existing `N(i)` order;
- it reconstructs both the outgoing local term and the incoming reverse term
  that `source-atomic-v0` would scatter from `j -> i`;
- it is the only writer of `source[i]` and `matrix[i]`;
- no floating atomic, shared endpoint write, unordered container or pair
  fragment is permitted in the RC1 kernels.

The implementation report binds base profile hash, accumulation identity,
binary hash, CUDA/CCCL/driver/device identity and exact command. A report that
omits accumulation identity is invalid.

## Mathematical obligations

For the frozen symmetric directed relation `E`, RC1 implements per owner:

```text
source[i] = sum_(j in N(i)) (local(i,j) + reverse(j,i))
matrix[i] = sum_(j in N(i)) (local_matrix(i,j) + reverse_matrix(j,i))
```

The incompressibility incoming term MUST use `density_ratio[j]`. Viscosity and
surface expressions MUST initially evaluate both named endpoint contributions;
replacing them with a multiplied simplified contribution belongs to a later
adjacent optimization. The skipped self/radius-epsilon rule remains unchanged.

The CPU `f64` gather oracle and CUDA `f32` gather implementation MUST be
independently transcribed. They may share fixture/report records but not one
pair-contribution helper.

## Ordered gates

| Gate | Command shape | Required result | On failure |
|---|---|---|---|
| `RC1-CPU-ALGEBRA` | CPU scatter and independent CPU gather over all eleven tiny cases | every existing normalized field/momentum bound passes | `GATHER_FORMULA_MISMATCH`; stop |
| `RC1-CUDA-TINY` | existing CUDA self-test under `nuv-gather-directed-r0` | 11/11 PASS with unchanged tolerances | `GATHER_NUMERIC_MISMATCH`; stop |
| `RC1-SURFACE-I2` | surface-16k, fixed 2 iterations, 10 cold repeats | same profile/input/CSR hashes; finite; exact ordered output-state digest across repeats; momentum PASS | `ORDER_HYPOTHESIS_FALSIFIED` or numeric mismatch; stop |
| `RC1-SURFACE-I20` | surface-16k, fixed 20 iterations, 10 cold repeats | exact repeated digest plus all current field/momentum gates | `GATHER_NUMERIC_MISMATCH`; stop |
| `RC1-FULL-CONTROLS` | water-16k/48k at 5; viscous-16k at 20 | current correctness/capacity gates pass | stop; no timing campaign |
| `RC1-ADJACENT-TIMING` | atomic then gather, warm-up 5/runs 50, passing profiles only | complete stage/total/memory report; no speed threshold for reclosure | record cost; transition below |

The exact repeated digest covers ordered final positions and velocities plus
the last-iteration density, source and matrix. It does not include timers,
allocation addresses or GPU completion metadata.

The same-device bitwise rule applies only to the frozen Linux RTX 3080
environment. Cross-GPU/toolkit correspondence is not tested by RC1 and no
cross-target determinism claim is allowed.

## Resource and measurement bounds

RC1 allocates no `O(P)` pair-fragment, owner-key, sort or segmented-reduction
buffer. Device memory may differ from `source-atomic-v0` only by bounded report
state and an optional per-particle density-ratio array declared before code;
any such array is reported separately and justified against the no-array
variant. Pair count, degree and CSR hashes must remain equal.

Timing starts only after all correctness gates pass. RC1 reports adjacent
atomic/gather times to expose cost, but it does not reorder O1–O6 or claim an
NR2 retained speedup. NR2 may later count the admitted accumulation change in
HN-3 while preserving the atomic denominator only for the two denominator
profiles where the atomic implementation passed correctness.

## Exit states

### `NR1_RECLOSED_GATHER_DIRECTED`

All gates pass. Update the roadmap/task state, make `nuv-gather-directed-r0`
the correctness-valid NR1 successor, and unblock the ordered NR2 ladder.

### `GATHER_FORMULA_MISMATCH`

CPU real-arithmetic reconstruction does not match the existing oracle within
the frozen bounds. Preserve the report and stop before CUDA changes.

### `ORDER_HYPOTHESIS_FALSIFIED`

The owner-only CUDA path varies on the known two-iteration surface control with
identical inputs/CSR. Inspect the earliest differing producer; do not jump
directly to a larger timing run or tolerance change.

### `GATHER_NUMERIC_MISMATCH`

The path is repeatable but violates independent CPU, field or conservation
gates. Preserve both implementations and consider a separately specified
stable segmented reduction only after the mismatch is localized.

### `GATHER_COST_REJECTED`

Correctness passes, but the adjacent report shows no credible bounded route to
the NR2 resource/performance gates. Proceed to NR4 stop or separately specify
the segmented fallback; do not integrate the candidate.

## Rollback

The RC1 kernels are selected only by explicit accumulation identity. Reverting
the candidate leaves `source-atomic-v0`, its hashes and NR1 mismatch evidence
unchanged. No runtime, Cargo workspace, schema, public contract or accepted
architecture document is touched.

## Non-goals

Segmented fragments, fixed-point accumulation, `f64` atomics, algebraic
factor-of-two simplification, term fusion, clear removal, CUDA Graphs,
adaptive convergence, line search, Pairwise Descent, engine integration,
Windows execution and production/cross-target authority.
