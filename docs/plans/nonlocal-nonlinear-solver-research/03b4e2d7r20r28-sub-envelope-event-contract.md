# NSR3-B4E2D7R20R28 sub-envelope event contract

Status: `FROZEN / REPORT-ONLY TWO-SIDED EVENT AUDIT AUTHORIZED`.

## Parent

- R27 implementation `5ac52d26`, semantic
  `6ef46e9ec490bfb40e4b7eecbf50898b3efb4d494fa1028fd501c1f14be6e003`;
- exact R26 shear root `1c9a0bf3...cf91` and 19 accepted steps;
- iteration 20 has 21 crossing trials, zero stable trials, unchanged ball and
  strictly signed negative margins through `2^-20`.

## Frozen audit

Replay one exact R26 shear execution and observe only rejected iteration 20.
Enumerate all fixed-current-face component-bound and ball-activity roots in
`(0, 2^-20]`; report coefficients, admissibility, multiplicity, scalar/bound
identity, alpha and separation from the next root.

If and only if the nearest root is a unique zero-bound component event with
nonzero derivative, derive the unchanged R25 sparse-entry forward bound and
construct exactly `alpha_old/alpha_new` from the formulas in the research
note. Require `0 < alpha_old < root < alpha_new <= 2^-20`, no second event in
the bracket, nonnegative multipliers, expected one-scalar old/new mask
lineage and unchanged ball status. Evaluate exactly those two shadow trials;
report rigorous Armijo margins, dual increase and full KKT tuples.

Classify, in priority, parent/reproduction failure, ambiguous or coupled event,
unbracketed/uncertified side, precision boundary, bidirectional acceptance,
old-side-only acceptance, new-side-only acceptance, both-side rejection or
unresolved. Hash all enumerated roots and both conditional trial roots into the
result.

No dyadic/ULP sweep, third trial, state update, cap/tolerance/event-policy
change, trajectory continuation, timing, runtime/GPU, generalization or
production authority.
