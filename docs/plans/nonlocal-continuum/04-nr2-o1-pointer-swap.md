# NR2-O1 — Persistent state and pointer-swap handoff

Status: `SPECIFIED / IMPLEMENTATION_NOT_STARTED / REPORT_ONLY`

Prerequisite:
[NR1-RC1 reclosure evidence](../../development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md)
exits `NR1_RECLOSED_GATHER_DIRECTED` and admits O1.

## Outcome

Remove the per-iteration device-to-device position copy from the
correctness-valid gather baseline without changing any mathematical work,
fixture, accumulation, arithmetic, buffer capacity or observable output.

The frozen baseline identity is:

```text
accumulation = nuv-gather-directed-r0
handoff       = copy-v0
```

The only O1 candidate identity is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
```

The two identities remain orthogonal in every JSON report and exact command.
Commands without a handoff selector preserve `copy-v0`; O1 cannot silently
replace the RC1 binary's behavior.

## Frozen and changed dimensions

O1 retains:

- every profile, coefficient, active term, fixed iteration count and
  tolerance;
- CUDA `f32`, compiler flags, density and three gather kernels, full local
  matrix and unregularized `3x3` solve;
- uniform-grid/CUB construction and exact CSR membership/order;
- both allocated position buffers, every other allocation and the admitted
  device-memory capacity;
- explicit source/matrix/error resets, final density/velocity reconstruction,
  diagnostics and capture ordering.

O1 changes only the post-update handoff:

```text
copy-v0:
    update(current -> next)
    device_copy(next -> current)

pointer-swap-o1:
    update(current -> next)
    host_swap(current_pointer, next_pointer)
```

CUDA launch arguments bind pointer values at launch. The host swaps only the
two private pointer variables after enqueueing the update; it adds no device
operation or synchronization. The next iteration reads the newly produced
buffer. Final density, velocity and capture always read the pointer currently
named `current`.

Pointer orientation may alternate across repeated `execute` calls after an
odd iteration count. Prediction therefore MUST overwrite the full current
buffer on every execution, and the update MUST overwrite the full other
buffer. Destruction MUST free both distinct allocations exactly once,
regardless of their current names.

## Correctness gates

Run in order and stop before timing on any failure:

| Gate | Required result |
|---|---|
| `O1-BUILD` | external CMake/Ninja build PASS with unchanged CUDA flags |
| `O1-LEGACY` | default atomic/copy tiny 11/11 PASS; explicit gather/copy tiny 11/11 PASS |
| `O1-TINY-RESET` | gather/swap passes CPU correspondence on 11/11; three reused-instance executions per fixture have exact ordered output and CSR digests; copy/swap output digests match exactly |
| `O1-SURFACE-I2` | ten cold gather/swap repeats at two iterations are exact and match the gather/copy output/CSR digest |
| `O1-SURFACE-I20` | same at twenty iterations |
| `O1-FULL` | water-16k/48k at five and viscous-16k at twenty pass finite/topology/capacity/momentum gates; two reused candidate executions and copy/candidate output digests are exact |

The ordered output digest remains density, source, matrix, final position and
final velocity. Energy correspondence remains inside the existing full-field
comparison even though energy is excluded from the exact digest.

The reused-instance check is the stale-state discriminator. It is not replaced
by cold object construction or a single even-iteration profile.

## Adjacent timing and retention

Only after every correctness gate passes, run gather/copy immediately followed
by gather/swap on water-16k, water-48k and viscous-16k with five warm-ups and
50 measured runs. Report the complete stage/total statistics and unchanged
device memory from one binary/toolchain/device state.

O1 is retained only if:

- every correctness gate passes;
- device memory does not increase;
- `state_handoff_and_velocity` p95 decreases on every profile;
- total p95 improves on both HN-3 denominator profiles, water-48k and
  viscous-16k.

No minimum percentage is required for one isolated O1. If total p95 does not
improve on both denominator profiles, record `O1_REJECTED_NO_TOTAL_BENEFIT`,
restore `copy-v0` as the NR2 baseline and proceed according to the existing
early-stop rules.

RC1 atomic/gather timing is not the O1 denominator. O1 compares only
gather/copy against gather/swap.

## Exit states

### `O1_RETAINED_POINTER_SWAP`

All correctness and retention gates pass. Pointer swap becomes the retained
NR2 implementation input to O2, while gather/copy and the RC1 evidence remain
reproducible through explicit identity.

### `O1_STALE_STATE_MISMATCH`

A reused execution differs from its first execution or copy baseline. Preserve
the report and stop; inspect pointer parity and full-buffer initialization.

### `O1_NUMERIC_MISMATCH`

Cold/reused execution is stable but differs from gather/copy, CPU or an
existing finite/topology/momentum gate. Preserve both paths and stop without
timing.

### `O1_REJECTED_NO_TOTAL_BENEFIT`

Correctness passes, but the adjacent timing retention rule fails. Keep the
implementation selectable for evidence, retain copy as the NR2 baseline and
do not count O1 as an optimization.

## Non-goals

Term specialization, accumulation changes, clear removal/fusion, CUDA Graphs,
cell sorting, precision changes, convergence changes, warm start, profiler
capture, production integration, runtime/public contracts, Windows execution
and NR4 selection.
