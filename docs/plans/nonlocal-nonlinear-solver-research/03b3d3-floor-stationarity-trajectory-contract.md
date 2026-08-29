# NSR3-B3D3 -- floor-stationarity trajectory contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B3_RETRY_BLOCKED`

Parent D2 selects `FLOOR_STATIONARITY_MERIT_CANDIDATE`, semantic SHA-256
`da1da7a0fcd4adf29c01a60a5d72f97c4d7a4c9794484bc90f575b9d271d4601`.
D1 and B3 remain failed. This contract tests a numerical globalizer over the
unchanged FCR2 objective and split boundary semantics.

## Candidate

Retain D1 owned displacement and the complete ordinary trust-region path. If
and only if an active reaction-aware solve reaches the inherited predicate

```text
predicted > 0
predicted <= 1024*eps*max(|F_current|,1)
scaled residual <= 1e-7
||p||/dx <= 1e-7,
```

evaluate the already proposed trial once. Accept it and finish the smooth solve
with `FLOOR_STATIONARITY_MERIT` only when all conditions hold:

1. current/trial pressure active-center and pair counts are identical;
2. trial state and impulse residual are finite;
3. `||R_trial|| < ||R_current||`;
4. `||R_trial|| <= reaction_mixed_limit(trial)` using the unchanged B3D/D1
   formula;
5. no rejected or negative-curvature trial is relabelled.

The trial is charged as one support/gradient evaluation. It may be accepted
despite unresolved or negative materialized endpoint energy because the
accepted merit is explicitly `0.5*||R||^2`, where `R=h*grad F`. It cannot start
a residual-only iteration: failure to reach the reaction limit preserves
`REACTION_BELOW_ENERGY_RESOLUTION`.

Inactive pressure states retain the exact D1 path and bound. Contact remains
`smooth -> swept contact -> velocity from owned displacement -> rebuild`.

## Trajectory matrix and gates

Run face and corner fixtures at fixed `96/192/384` substeps per frame for four
frames. For each row require:

- every substep completes, with active and contact steps both present;
- inactive per-step defects and cumulative `B_fp <= 1e-10*M*c` certify;
- maximum active stationarity/reaction-limit ratio `<=1`;
- translation and contact defects retain D1 limits;
- signed cumulative/direct terminal momentum gates pass;
- final position and velocity differ from the corresponding old non-aborting
  B3D trace by at most `1e-5 dx` and `1e-5 c`;
- at least one floor-merit trial is accepted and every such trial satisfies
  all five candidate conditions;
- total candidate HVP calls are at most
  `floor(2.5*ordinary_HVP + 2*active_steps)`; support/gradient floor trials are
  reported separately.

The report publishes total outer trials, HVPs, rejects, floor trials/accepts,
maximum floor residual ratio and the six final-state hashes.

## Parent and exit

Require exact D2 JSON-without-newline SHA-256
`573bf5a943d338bbbed4c05919bea2a0255b6d71e98d85c195a3ee38adf82f6e`.
Two D3 reports must be byte-identical; D2, D1, B3D, B3 and B2 remain exact.

PASS selects `FLOOR_STATIONARITY_TRAJECTORY_CANDIDATE` and authorizes only a
separately frozen B3R composition retry. Failure preserves D1/B3 failure at the
exact first gate.

No hydrostatic/dam-break corpus, CUDA, performance, runtime or production
authority is granted.
