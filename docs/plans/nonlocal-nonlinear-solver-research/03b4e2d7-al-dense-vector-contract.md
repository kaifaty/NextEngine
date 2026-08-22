# NSR3-B4E2D7 -- augmented-Lagrangian dense-vector contract

Status: `FROZEN / NOT_RUN / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7-al-dense-vector|v1|parent=997212cb24fdda96c43a3cfba516f42c62617550e22ed61924855a3442f1d16b:6b05ed4cf6da4a6007a1e4557ef559b56ab9bffafeb5730841c15bc34853403f:7deec6fd7aeb596cb358657f2e900e8b150f9a82285dec76ac018e2d642d0cc8|fixture=nsr3b2-corner-box-2x2x2:8:176;fixed-support;rest-feasible|identity=nuv-al-pressure-r0+dense-vector-v1|objective=inertia-mass0.125-dt1/240;active-prediction=isotropic0.99;inactive-prediction=isotropic1.01;constraint8=rho/rho0-1<=0;lambda8>=0;beta=1226.25|al-gradient=max(0,lambda+beta*c)*J;al-hvp=beta*JtJ+max(0,lambda+beta*c)*H|derivatives=directional-gradient<=1e-7;hvp<=2e-6;dense-symmetry<=2e-12;dense-product<=2e-12;active-set-stable;reaction-closure<=1e-12|inner=steihaug-trust;outer<=64;reject<=8;scaled-stationarity<=1e-8;positive-actual-and-model|outer=phr-update<=8;primal<=1e-8;dual-change/beta<=1e-8;stationarity<=1e-8;complementarity<=1e-9;dual-feasible;primal-monotone|controls=cold;warm<=2;state-correspondence<=1e-8;inactive-exact;zero-lambda-reset-positive;forced-rollback-exact;multiplier-mutation-sensitive;fixed-boundary|route=al-dense-viable-if-converged;semismooth-primal-dual-required-if-inner-exact-and-monotone-cap-exhausted;fail-otherwise|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;nominal=none|credit=tiny-vector-transaction-contract-research-only
```

Identity SHA-256:
`daea8b078ceebbc971b830ab030f6790725d53f1860341f14a219199329378c2`.

## Required command

Add `--nonlocal-al-dense-vector-oracle`. It must be standalone from the
existing penalty transaction and emit:

1. AL energy, complete fluid/support gradient and matrix-free HVP over the B2
   fixture;
2. active-state directional gradient/HVP finite differences, explicit dense
   HVP matrix, symmetry/product error, branch/topology stability and support
   reaction closure;
3. every cold outer record: inner work and accept/reject facts, maximum
   positive constraint, scaled dual change, scaled stationarity,
   complementarity and multiplier range;
4. cold/warm/inactive/reset/forced-rollback/mutation state roots;
5. exactly one frozen route.

## Frozen gates

- directional gradient error `<=1e-7`, HVP error `<=2e-6`, dense symmetry and
  dense product errors `<=2e-12`;
- active-set/topology stable across derivative stencils, fixed support state
  bit-exact and total pressure reaction closure `<=1e-12`;
- inner at most 64 trust trials and eight rejects, final scaled stationarity
  `<=1e-8`; every accepted trial has positive actual/predicted reduction;
- outer at most eight updates; primal violation and scaled dual change
  `<=1e-8`, stationarity `<=1e-8`, complementarity `<=1e-9`, nonnegative
  multipliers and monotonically nonincreasing primal violation;
- warm start at most two outer updates; cold/warm RMS position normalized by
  spacing and maximum multiplier difference both `<=1e-8`;
- inactive prediction is returned bit-exact with zero multipliers;
- zero-multiplier reset has positive compression above `1e-8`;
- forced rollback preserves position/multiplier state bit-exact and a one-ULP
  multiplier mutation changes its bound result root.

## Execution and authority

Build Release twice and run two fresh processes without timing. Require exit
zero, empty stderr and byte-identical stdout. PASS authorizes only a tiny
multi-step AL transaction research/contract. It grants no nominal trajectory,
runtime pressure schema, performance, GPU/PhysX or production authority.
