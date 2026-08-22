# NSR3-B4E2D7R5 -- private outer-AL integration contract

Status: `FROZEN / NOT_RUN / PRIVATE_TRANSACTION_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r5-private-outer-al-integration|v1|parent=3d7bf83ce177ccd2161dada39caa48c6172277ab5c37f0c2e3c6036676d4d96a:77f7afc7e9a936ba19e183da07175b64e20beb8f3db939f868d0a578b3ffa289:975da3f5adc13bba6c5fec3cfe08f0bc395f8fac58882b841ba5026774d6e886|solver=step-norm-trust-inner-candidate;outer=d7r-unchanged|prefix=first8-root:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2;state:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e|outer-cap=14;beta=1226.25;gates=primal1e-8,stationarity1e-8,complementarity1e-9,abs-dual1e-8J,pressure8e-5Pa,position1e-8dx,lambda-nonnegative,primal-monotone|confirmation=two-consecutive-admissible;warm-holdout=one-private-update;same-gates|controls=inactive;reset;forced-rollback;parent-bytes|routes=pressure-state-confirmation-candidate;outer-cap-or-nested-accuracy-research;inner-policy-insufficient|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;private-confirmation<=1;public-commit=none;physics-mutation=none|credit=dense-al-holdout-contract-research-only
```

Identity SHA-256:
`b9f0e532e064f2316b1f3133fdd3fb7877a7da431c1284d8233f3e3cc5be48b9`.

## Required command

Add `--nonlocal-al-step-norm-private-outer`. It must:

1. execute a separate D7R outer/confirmation transaction whose only changed
   dependency is the D7R4 candidate inner;
2. reproduce the exact first-eight D7 outer/state roots;
3. emit every outer record with all existing primal/dual/pressure/position/KKT
   values plus inner trial/accept/reject/HVP/step-norm-update counts;
4. require monotone primal violation, finite values and unchanged 14-update
   cap;
5. on two consecutive admissible updates, run one private warm holdout update
   under the same gates;
6. retain inactive/reset/forced rollback controls and zero public commits;
7. emit exactly one route.

## Routes

1. `PRESSURE_STATE_CONFIRMATION_CANDIDATE` requires two consecutive admissible
   updates, one admissible warm holdout, exact prefix/controls and exactly one
   private confirmation candidate.
2. `OUTER_CAP_OR_NESTED_ACCURACY_RESEARCH` requires all inner solves pass,
   primal remains monotone, the cap is exhausted, no confirmation exists and
   the last three primal violations are nonincreasing. It authorizes only a
   cap-versus-inner-accuracy discriminator.
3. `INNER_POLICY_INSUFFICIENT` requires exact prefix/controls but a later
   finite inner solver failure. It authorizes no second policy change without
   new research.

Prefix, gate, nonfinite, rollback, parent-byte or control mismatch is hard FAIL
with no route. PASS classifies the private outer transaction. It grants no
public pressure state, trajectory, coefficient/tolerance/cap change,
performance, runtime, GPU/PhysX or production authority.
