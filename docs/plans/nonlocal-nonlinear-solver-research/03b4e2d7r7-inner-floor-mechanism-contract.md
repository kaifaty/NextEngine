# NSR3-B4E2D7R7 -- inner-floor mechanism discriminator contract

Status: `CLOSED / PASS / TRUST_MODEL_OR_DERIVATIVE_RECLOSURE / PRIVATE_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r7-inner-floor-mechanism-discriminator|v1|parent=65f1a01ccc110a4e110c2e8b9907d9a29f0ecb58fd1e09a4f3b3a9af0fc5ee0a:9e93beb38632b10d9a7a1b8473edcb65d7d5ed672d268597071aa5bbbda00dfa:6979ebf9f0b88fb57cbcbbaed6721951cb7fa05acb8f7d4b8c39962f257b9f6f|replay=post-d7-prefix;eta1e-8-fail-outer58;eta1e-9-fail-outer13;eta1e-10-fail-outer11;eta1e-11-and-1e-12-common-state-exact|solver=step-norm-trust-inner-unchanged;beta=1226.25;raw-admission-unchanged;no-acceptance-change|trace=all-failed-inner-trials;state-root;trust-radius;step-norm;radius-owner;hvp;negative-curvature;gradient-step;predicted;raw-actual;direct-actual;raw-ratio;direct-ratio;current-trial-ulp;reduction-ulp-ratios;active-root;fluid-root;boundary-root;phr-margin;horizon-margin;would-accept;merit-floor-bound=8*max-current-trial-ulp|controls=d7r6-complete-bytes;three-pre-failure-state-roots;common-tight-state;inactive;reset;forced-rollback|routes=common-phr-kink;common-binary64-merit-floor;mixed-active-and-merit;trust-model-or-derivative-reclosure|precedence=phr,merit,mixed,model|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;public-commit=none;physics-mutation=none|credit=one-failure-mechanism-research-only
```

Identity SHA-256:
`f29336079dcb1a54db81fe5c8913130c3cbf72c272336c80fcad6d02c68eb47e`.

## Required command

Add `--nonlocal-al-inner-floor-mechanism-discriminator`. It must:

1. reproduce the complete D7R6 report at its expected FAIL bytes;
2. reproduce exact pre-failure states for the `1e-8`, `1e-9` and `1e-10`
   lanes and prove that `1e-11/1e-12` share the third state;
3. replay every live failed-inner trial using the unchanged step-norm policy,
   raw admission, reject cap and minimum trust radius;
4. emit all trace values listed in the identity projection;
5. compare raw subtraction with fixed-order factored/direct reduction and an
   explicit binary64 ULP scale without using either to accept a trial. The
   frozen unresolved-merit bound is
   `8 * max(ulp(current_total), ulp(trial_total))`; a trial is merit-floor
   evidence only when topology is exact, model/direct reductions are
   positive, raw admission rejects, and both actual reductions have magnitude
   no greater than that bound;
6. compare exact active and compact-support pair roots at every proposal;
7. retain inactive/reset/forced rollback and zero public commits;
8. emit exactly one route under the frozen precedence.

## Routes

1. `COMMON_PHR_KINK_RESEARCH`: every unique failure contains an active-root
   change while fluid/boundary pair roots remain stable, and no certified
   topology-stable merit-floor explanation covers every failure.
2. `COMMON_BINARY64_MERIT_FLOOR_RESEARCH`: every unique failure keeps all
   topology roots exact, has a positive model and positive direct reduction,
   and raw admission is unresolved at the emitted binary64 error scale.
3. `MIXED_ACTIVE_AND_MERIT_RESEARCH`: no preceding route; at least one unique
   failure meets the PHR condition and another meets the merit-floor
   condition.
4. `TRUST_MODEL_OR_DERIVATIVE_RECLOSURE`: all replay/control values are finite
   and exact, but none of the preceding mechanisms explains all failures.

Parent, state reproduction, common-tight-state, nonfinite, work-count,
unchanged-admission, rollback or route-precedence mismatch is hard FAIL.
PASS is classification only. It grants no trial acceptance, cap/tolerance,
pressure gate, `beta`, semismooth/primal-dual selection, trajectory,
performance, runtime, GPU/PhysX or production authority.
