# NSR3-A2 -- outer-state Hessian coefficient tape contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Baseline: selected `hvp-workspace-stream-v1`.

Candidate: `outer-state-hessian-tape-v1`.

## Hypothesis

Every inner CG/model HVP at one trust outer state uses the same positions,
pressure active set, current/reference pairs and adjacency. Density,
Jacobian/radial coefficients and viscosity/surface radial operators can be
assembled once per outer state. Recomputing them for 3--6 HVPs is redundant.

## Representation

- active pressure centers in ascending sample order;
- center Jacobian plus neighbor Jacobians in the existing sorted adjacency
  order for the `J^T J` reduction/scatter;
- pressure geometric radial operator per directed active-center edge;
- immutable reference viscosity radial operator built once per solve;
- current surface radial operator built once per accepted outer state;
- inertia scale stored once.

Assembly must evaluate expressions in the same order as
`hvp-workspace-stream-v1`. Application must preserve center, participant,
directed-neighbor and pair accumulation order. No dense matrix is allowed.

## Accounting

Add `hessian_tape_build` as a timed bucket. It is included in total solve time,
never hidden in fixture generation or pair construction. Report active centers,
directed pressure records, viscosity/surface pair records and allocated bytes.
Additional tape storage must be bounded by:

```text
192 * maximum_current_pairs + 128 * particles bytes
```

Capacity is checked before allocation; failure publishes no candidate result.

## Exactness and tournament gates

- bit-exact HVP on all seven NSR2-B controls and at every visited outer state
  of 512/1000/1728/4096;
- bit-exact final state and operations against A1;
- one warmup and seven alternating A1/A2 pairs on logical CPU 4;
- combined `hessian_tape_build + HVP` median improves by at least `1.20x` on
  every size;
- total median improves by at least `1.10x` on 1728 and 4096;
- total median regresses by no more than 2% on 512 and 1000;
- all historical exact hashes remain unchanged.

No coefficient compression, f32 storage, term pruning, parallelism or Krylov
vector reuse may be bundled into A2. PASS selects the tape; FAIL retains A1.

