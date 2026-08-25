# NSR3-B4E2D7R20R1 phase-I feasibility discriminator contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Immutable parent

- implementation `5be5b059`;
- R20 manifest semantic `420031730236445859426960bc5540ae8b8f9522d4b9263fe5e86cf5f13f85b4`;
- R20 preflight semantic `16a24ef3040411ef0bd29294f0887b29a1cce7690c7d8a8bee511ea58880b1f6`;
- R20 oracle semantic `885dc7bdb628e4d435b3cd6966c87b6866dd9ca33fec3d8b9718b8477328c60f`;
- exact eight preflight problem roots and stable case order;
- oracle route `ORACLE_UNRESOLVED`; candidate iterations zero.

Any mismatch rejects before the discriminator.

## Frozen phase-I problem

```text
rho(s) = 0.5 * ||max(c+A*s,0)||^2,  s in D
D      = contact box intersect global ball of radius 0.0625
L(l)   = l^T*c - 0.5*||l||^2 - sigma_D(-A^T*l),  l>=0
```

Use the frozen sparse R64 row action/transpose. The generator is a
primal-dual method with zero primal/dual initialization, a conservative
Frobenius operator-norm bound, fixed power-of-two checkpoints through `2^16`
and no tolerance-selected stop. All eight cases are reported; cycle-zero cases
may close before generator work.

## Independent audits

At every checkpoint:

1. recompute all density rows from the frozen sparse entries;
2. compute a binary128 gamma-style outward upper for the maximum residual;
3. recompute `A^T*l`, the exact box-ball support KKT bracket and a binary128
   outward lower bound for `L(l)`;
4. require `l>=0`, finite values, weak duality `L(l)<=rho(s)` within the
   outward envelopes and exact source/problem roots.

Certificate routes are mutually exclusive:

- `PHASE1_FEASIBLE_CERTIFICATE`: every residual upper is `<=0`;
- `PHASE1_INFEASIBLE_CERTIFICATE`: the dual lower bound is `>0`;
- otherwise `PHASE1_UNRESOLVED` at the fixed cap.

Dense feasible, dense infeasible, support/bracket, multiplier-sign,
weak-duality, source-bit, problem-root and lifecycle corruptions must reject.

## Work and prohibitions

Report generator row actions/transposes/projections and certificate sparse
terms separately. No wall timing, oracle rerun/depth extension, candidate
iteration, geometry edit, nonlinear trial, runtime state, GPU path or
production authority is admitted.

If either blind holdout is certified infeasible, reclassify it without editing
its bytes and design a new source-frozen feasible blind corpus before any
candidate execution. If both are feasible, research a different oracle rather
than increasing cyclic-Dykstra depth.
