# NSR3-B4E2D7R -- dimensionally stable AL commit contract

Status: `FROZEN / NOT_RUN / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r-al-stable-commit|v1|parent=daea8b078ceebbc971b830ab030f6790725d53f1860341f14a219199329378c2:0453794038d8788e4ae8e6a77a691619a6e0769aa910b82291031c7d042cbab4:e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11|fixture=nsr3b2-corner-box-2x2x2:8:176;fixed-support;rest-feasible|identity=nuv-al-pressure-r0+dense-vector-stable-commit-v1|objective=inertia-mass0.125-dt1/240;active-prediction=isotropic0.99;inactive-prediction=isotropic1.01;constraint8=rho/rho0-1<=0;lambda8>=0;beta=1226.25|al-gradient=max(0,lambda+beta*c)*J;al-hvp=beta*JtJ+max(0,lambda+beta*c)*H|reproduction=d7-first8-state:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e;outer-json-nolf:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2;records8-exact|inner=unchanged-steihaug-trust;outer-total<=14;beta-fixed;no-penalty-update|admission=primal<=1e-8;stationarity<=1e-8;complementarity<=1e-9;dual-feasible;absolute-dlambda<=1e-8J;equivalent-pressure-change<=8e-5Pa;position-update-rms/dx<=1e-8|commit=private-provisional;next-private-confirmation;two-consecutive-admissible;commit-confirmed-only;public-unchanged-before-confirmation|controls=derivatives-d7-exact;inactive-exact;zero-lambda-reset-positive;forced-confirmation-reject-rollback-exact;multiplier-mutation-sensitive;fixed-boundary|route=al-dense-stable-commit-if-confirmed;semismooth-primal-dual-required-if-inner-exact-and-monotone-cap-exhausted;fail-otherwise|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;nominal=none|credit=tiny-multistep-al-transaction-contract-research-only
```

Identity SHA-256:
`86427f686fd719655497f7171cace1ccaaea9353a5e0d919766a6309f11dfe2d`.

## Required command

Add `--nonlocal-al-dense-vector-stable-commit`. It must be standalone from the
penalty trajectory and preserve the old D7 command bytes. It emits:

1. the unchanged fixture and all D7 derivative/dense/reaction facts;
2. the exact first-eight state and no-final-LF outer-array roots;
3. every outer record through confirmation, adding absolute multiplier
   change, equivalent pressure change and normalized position change;
4. provisional and confirmation indices/state roots, public precommit root,
   committed root and exact commit count;
5. inactive, reset, forced-confirmation rejection/rollback and multiplier
   mutation controls;
6. exactly one frozen route.

## Frozen gates

- D7 gradient/HVP/dense symmetry/dense product/reaction limits remain
  `1e-7/2e-6/2e-12/2e-12/1e-12`; topology/active set and fixed support remain
  exact;
- the first eight records equal D7 bit-for-bit, their state root is
  `04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e`
  and their exact JSON-array/no-final-LF SHA is
  `9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2`;
- inner trust gates remain unchanged; every accepted trial has positive actual
  and model reduction, at most 64 trials/eight rejects per inner solve;
- primal violation is monotone, total updates including confirmation are at
  most 14, `beta` never changes and no fallback retry occurs;
- both provisional and immediately following confirmation records have
  primal `<=1e-8`, stationarity `<=1e-8`, complementarity `<=1e-9`,
  nonnegative multipliers, maximum absolute multiplier update `<=1e-8 J`,
  equivalent pressure update `<=8e-5 Pa` and RMS position update
  `<=1e-8 dx`;
- no state is public before confirmation; the confirmed state commits exactly
  once. Forced confirmation rejection leaves the prior public position and
  multipliers bit-exact;
- inactive, reset and mutation controls retain D7 semantics.

## Route and authority

`AL_DENSE_STABLE_COMMIT` requires every gate and authorizes only research and
contract design for a tiny multi-step AL transaction. If all inner solves are
exact and primal violation remains monotone but 14 updates cannot produce a
confirmed pair, select `SEMISMOOTH_PRIMAL_DUAL_REQUIRED`. All other failures
are hard failures.

Build Release twice and run two fresh processes without timing. Require
identical executables, exit zero, empty stderr and byte-identical stdout. No
nominal trajectory, runtime pressure schema/persistence, performance,
GPU/PhysX or production authority is granted.
