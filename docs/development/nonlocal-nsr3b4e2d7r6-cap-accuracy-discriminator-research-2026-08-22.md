# NSR3-B4E2D7R6 cap versus nested-accuracy research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / PRIVATE_DIAGNOSTIC_ONLY`

## Question

D7R5 removes the local trust-globalization failure but reaches 14 outer
updates without a stable pressure state. The next experiment must distinguish:

1. the unchanged inner accuracy is sufficient and only the outer observation
   horizon is too short;
2. the original 14-update horizon is sufficient once the inner subproblem is
   solved more accurately;
3. both bounds matter; or
4. neither bounded change produces a confirmed pressure state.

This is a discriminator, not permission to increase a production cap or lower
a production tolerance.

## Dimensional bracket

For the frozen fixture, both absolute gates imply the same multiplier bound:

```text
abs(delta lambda) <= 1e-8 J
delta pressure = delta lambda * 1000 / 0.125 <= 8e-5 Pa
```

With `beta=1226.25 J`, a positive constraint update must be at most
`8.1549439347604491e-12`. D7R5 outer 10 exits its inner solve at scaled
stationarity `6.006e-9` while the constraint is `8.335e-11`; its dual update
is still `10.22x` too large. Later zero-work exits accumulate multiplier until
a corrective inner step overshoots the same pressure-state gate.

There is no proven algebraic equivalence between scaled stationarity and
constraint error, so the next stage must observe a ladder rather than infer a
new tolerance from that one ratio.

## Experiment design

Fork every lane from the exact private post-outer-7 state. Preserve those
first eight records as immutable parent evidence. Run five diagnostic lanes
whose only changed input is the inner scaled-stationarity stop:

```text
eta = 1e-8  (unchanged control)
eta = 1e-9
eta = 1e-10
eta = 1e-11
eta = 1e-12
```

Each lane may observe outer indices 8 through 63. Index 13 remains the
original cap boundary. Every PHR formula, `beta`, trust update, reject cap,
raw acceptance rule and dimensional confirmation gate remains unchanged.
Stop a lane only after two consecutive admissible updates plus one private
warm holdout, or after the bounded observation cap.

The unchanged `eta=1e-8` lane must reproduce D7R5 through outer 13 exactly.
Tighter lanes are counterfactual diagnostics and may diverge only after the
shared post-outer-7 fork.

## Decision rule

Use fixed precedence:

1. unchanged `eta=1e-8` confirms after the original cap but by outer 63:
   `OUTER_CAP_SUFFICIENT`;
2. otherwise the loosest tighter lane confirms by outer 13:
   `NESTED_ACCURACY_SUFFICIENT`;
3. otherwise a tighter lane confirms only after outer 13:
   `COUPLED_CAP_AND_ACCURACY_REQUIRED`;
4. otherwise all lanes remain finite and monotone without confirmation:
   `PRESSURE_STATE_FORMULATION_RESEARCH`.

Any parent/prefix/control mismatch, nonfinite value, negative multiplier,
nonmonotone primal sequence or new inner failure is hard FAIL. The selected
route authorizes only a separate implementation contract or further
formulation research; it does not itself change the solver.

## Publication boundary

All lane states, confirmations and holdouts remain private. No candidate may
commit, start Dam/Hydro, claim runtime ownership or collect performance
timings. This isolates convergence mechanics from the shared-host performance
stop and from production readiness.
