# NSR3-B4E2D6 -- augmented-Lagrangian path-oracle contract

Status: `FROZEN / NOT_RUN / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d6-al-path-oracle|v1|parent=5e3fc734c04294bd2705ce9ad96c371bb6423523521479f71dd6fba44049b9f8:1e7f856f35c4f684fae49487cd7419325ffee4ae500974e152346c006348cee8:f21020e2497befb6b843eff930b1874978346510371405c238c0a27851783ed8|fixture=nsr3b2-corner-box-2x2x2:80a01b2ed0cf844841da322233b121b33e273c71eace8682795d0cad4e1dfb80:8:176:828;rest-error<=3e-15;q0.01-compression=0.002362375096118585..0.002362375096119|identity=nuv-al-pressure-r0|path=q;position=center+(1-q)*(x-center);base=0.5*(q-qstar)^2;qstar=0.01-active,-0.01-inactive;constraint=rho/rho0-1<=0;lambda8>=0;beta=1226.25|derivatives=analytic-density-first-second;central-h=2e-7;relative<=1e-7;topology-stable|inner=safeguarded-bisection;range=-0.02..0.02;iterations<=128;gradient<=1e-10|outer=phr-update;iterations<=8;primal<=1e-10;dual-change/beta<=1e-10;stationarity<=1e-10;complementarity<=1e-10;q-abs<=1e-9;dual-feasible;primal-monotone|controls=cold;warm<=2;q-lambda-correspondence<=1e-9;inactive-exact;zero-lambda-reset-positive;forced-rollback-exact;multiplier-mutation-sensitive|route=al-path-viable-if-converged;semismooth-primal-dual-required-if-inner-exact-and-monotone-cap-exhausted;fail-otherwise|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;nominal=none|credit=dense-al-oracle-contract-research-only
```

Identity SHA-256:
`997212cb24fdda96c43a3cfba516f42c62617550e22ed61924855a3442f1d16b`.

## Required command

Add `--nonlocal-al-path-oracle`. Reuse the immutable B2 kernel and corner
fixture. Do not call the nominal neighborhood/trajectory implementation.

The command must publish:

- fixture counts, pair count, rest error and active `q=0.01` density range;
- analytic/finite-difference first/second density derivative errors and exact
  topology stability;
- each cold outer iteration's private `q`, inner iterations/gradient, maximum
  positive constraint, scaled dual change, stationarity, complementarity and
  multiplier range;
- cold/warm/inactive/reset/rollback/mutation results and deterministic roots;
- one route: `AL_PATH_VIABLE` or, only under the frozen monotone-cap rule,
  `SEMISMOOTH_PRIMAL_DUAL_REQUIRED`.

## Frozen gates

- derivative relative errors `<=1e-7`;
- inner bisection range `[-0.02,0.02]`, at most 128 iterations and final
  absolute derivative `<=1e-10`;
- at most eight cold outer iterations;
- final primal violation, scaled dual change and stationarity `<=1e-10`;
- complementarity `<=1e-10`, `abs(q)<=1e-9`, all multipliers nonnegative;
- positive primal violation is monotonically nonincreasing;
- warm start takes at most two outer iterations and cold/warm `q` plus
  multiplier maximum differences are `<=1e-9`;
- inactive `q` differs from `-0.01` by `<=1e-10`, constraints are nonpositive
  and multipliers remain bit-exact zero;
- zero-multiplier reset yields positive `q` and compression above `1e-8`;
- forced rollback preserves public state bit-exact; one-multiplier mutation
  changes its result root.

## Execution and authority

Build Release twice and run two fresh processes without timing. Require exit
zero, empty stderr and byte-identical stdout. PASS authorizes only dense-vector
AL oracle research/contract design. It grants no complete solver, nominal
trajectory, persistent pressure schema, performance, GPU/runtime/PhysX or
production authority.
