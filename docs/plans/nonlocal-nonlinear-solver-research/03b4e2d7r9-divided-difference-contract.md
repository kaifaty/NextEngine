# NSR3-B4E2D7R9 -- divided-difference reduction contract

Status: `FROZEN / NOT_RUN / PRIVATE_DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r9-divided-difference-discriminator|v1|parent=dd7357db48445a0d68cb88b3a9d7171c069faac2:dbdfcf009a44ddaf36cd643d750a0859f93678904a27668a020de9c21a245cac:42ce10541b29dd03b589e1e8699436e65cb5a794a45b8644fc040990dd4648b1|states=eta1e-8:21ad77e22bdba4520ca231bb78d51947a1b67e4263e08dae33af13b0aafabd05;eta1e-9:075442656934aeed156642091e9fc1ed41bc739513b5710990cdfb231d8aabc2;eta1e-10:299e4ce372a8a3d418ac6f354c70c362fd772acdf278464fb603a3881527901c|oracle=d7r8-resolved-only;11-positive;0-negative;12-unresolved;long-double-for-scoring-only|candidate=binary64-only;compensated-current-density;stable-squared-radius-delta;sqrt-delta=delta-r2/(trial-r+current-r);piecewise-cubic-kernel-delta;same-segment-factorization;cross-segment-knot-telescope;density-delta;branch-aware-phr-square-delta;inertia=2d-dot-step+step2;compensated-reduction|ablations=parent-direct;compensated-absolute;divided-difference|gate=all-11-resolved-signs;relative-error<=0.5;repair-at-least-one-causative;finite;branch-ledger-exact|unresolved=reported-not-scored-not-accepted|controls=d7r8-complete-bytes;state-roots;work-acceptance;forced-rollback|routes=compensated-absolute-candidate;divided-difference-candidate;branch-reclosure;stronger-arithmetic|precedence=absolute,divided,branch,stronger|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;trial-acceptance=none;public-commit=none;physics-mutation=none|credit=one-binary64-actual-reduction-candidate-only
```

Identity SHA-256:
`c53abac70daab09c9a006d73c79d595eace232e81e30e620a91ba4b30b8fd13a`.

## Required command

Add `--nonlocal-al-divided-difference-discriminator`. It must:

1. reproduce the complete D7R8 parent bytes, all three state roots and the
   frozen `11/0/12` resolved-positive/resolved-negative/unresolved ledger;
2. retain all 23 D7R7 current/trial binary64 positions and acceptance facts;
3. independently evaluate compensated absolute binary64 current/trial energy;
4. evaluate the binary64 divided difference with stable radius, piecewise
   cubic kernel, density, PHR and inertia delta propagation;
5. report compact-support, kernel-segment and PHR branch crossings;
6. score only D7R8-resolved trials against exact sign and maximum relative
   error `0.5` to the compensated extended reduction;
7. require at least one repaired causative false-ascent trial;
8. preserve forced rollback, zero accepted replay trials and zero public
   commits;
9. emit exactly one route under the frozen precedence.

## Routes

1. `COMPENSATED_ABSOLUTE_REDUCTION_CANDIDATE`: the absolute compensated
   ablation passes the complete scored gate.
2. `DIVIDED_DIFFERENCE_REDUCTION_CANDIDATE`: no preceding route; the divided
   candidate passes the complete scored gate.
3. `BRANCH_RECLOSURE_REQUIRED`: neither candidate passes, and every divided-
   difference mismatch crosses a kernel segment, compact support or PHR
   branch.
4. `STRONGER_ARITHMETIC_REQUIRED`: the controls close but none of the
   preceding conditions holds.

Parent, identity, state-root, oracle-ledger, binary64-only candidate,
nonfinite value, work/acceptance, rollback or route-precedence mismatch is a
hard FAIL. PASS is diagnostic classification only. It grants no formula
integration, trial acceptance, cap/tolerance, pressure gate, `beta`, kernel,
state-precision, solver-family, trajectory, performance, GPU/runtime or
production authority.

