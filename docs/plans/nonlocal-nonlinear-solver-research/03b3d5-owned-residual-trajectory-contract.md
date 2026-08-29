# NSR3-B3D5 -- owned-gradient residual trajectory contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent D4 selects `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED`, semantic
SHA-256 `bdf5b7eaa1f215e14ebb05500ffa2495b8cb3fccd19aa4b369514464086aae34`.
D3, D1 and B3 remain failed.

## Complete owned inertia state

On the candidate path only, evaluate inertia from the owned quantities:

```text
e = delta-delta*
I = M/(2h^2) ||e||^2
grad I = M/h^2 e
H_I p = M/h^2 p.
```

Pressure position, gradient and HVP remain unchanged. The HVP therefore has no
new operator term. Prediction, accepted trust corrections, contact, velocity
and position retain D1 ownership.

## Bounded floor phase

Use the ordinary energy trust path until the unchanged active energy-floor
predicate fires. Then evaluate the already proposed owned-gradient trial. If
topology is exact, state is finite and reaction residual strictly decreases,
accept it under merit `0.5||R||^2`, rebuild the fully owned gradient and begin
the next outer iteration at the same trust radius.

Limits per smooth solve are frozen before execution:

- at most four accepted floor-merit trials;
- no residual backtracking or line search;
- any topology change, negative curvature, non-finite state or non-decrease
  fails closed;
- the solve succeeds only at the unchanged reaction mixed limit;
- reaching the fifth required merit trial reports
  `FLOOR_STATIONARITY_ITERATION_LIMIT`.

Every floor support/gradient evaluation and every HVP is charged. Inactive
states retain the D1 exact path and certificate.

## Trajectory gates

Run face/corner fixed `96/192/384` traces for four frames. Reuse all D3 gates:

- full completion, active/contact presence and exact split ordering;
- inactive cumulative `B_fp <= 1e-10*M*c`;
- maximum active stationarity/reaction-limit ratio `<=1`;
- translation/contact and cumulative/direct momentum gates;
- final difference from old non-aborting B3D trajectory `<=1e-5 dx/c`;
- at least one floor-merit accept per row, all conditions exact;
- maximum merit accepts in any smooth solve `<=4`;
- candidate HVP calls
  `<=floor(2.5*ordinary_HVP + 4*active_steps)`.

Publish total and maximum-per-solve floor work, residual ratios, HVPs, final
state hashes and correspondence errors.

## Parent and exit

Require exact D4 JSON-without-newline SHA-256
`36f91802d114ba87124d00303c6ac413fbc09d4cd792f4262636e9816a5680ca`.
Two reports must be byte-identical. D4, D3, D2, D1, B3D, B3 and B2 remain
exact.

PASS selects `OWNED_RESIDUAL_TRAJECTORY_CANDIDATE` and authorizes only a
separately frozen B3R retry. Failure preserves all earlier failures.

No physical corpus, CUDA, performance, runtime or production authority is
granted.
