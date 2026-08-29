# NSR3-B4E2D7R19R4 -- Krylov model-image discriminator contract

Status: `EXECUTED / PASS / KRYLOV_MODEL_IMAGE_RESIDUAL_CANDIDATE / REPLAY ONLY / NO TRIAL`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r4-krylov-model-image-discriminator|v1|parent=2ef63a8a306e4191b0f723d59d45b2201515f6ba:a1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771:156782481d783abc500c1a1b888b693d158f8cd30503412d41285f4c725df6d8|legacy=d7r19r2-stdout3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0|target=current-54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a;predicted-36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e;radius0x3f8999999999999a;prefix-f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd;recurrence33;forcing-converged|images=direct-H-step-oracle1;accumulated-sum-alpha-Hd;residual-derived-r-final-minus-g|model=predicted-reduction-negative-g-dot-p-minus-half-p-dot-Hp;no-trial-energy|bounds=image-l2-relative<=1e-10;component-scaled<=1e-10;quadratic-relative<=1e-10;predicted-relative<=1e-10;positive-signs|required=step-root-exact;direct-finite;recursive-residual-root;accumulated-image-root;oracle-image-root|preference=residual-derived,accumulated,direct|controls=r19r3-parent-bytes;r19r2-bytes;target-roots;prefix-root;static-binding;finite;work;rollback|routes=krylov-model-image-residual-candidate;krylov-model-image-accumulated-candidate;direct-model-hvp-required|precedence=residual,accumulated,direct|runs=2-clean-release-builds;1-process-each;byte-exact|work=parent-offline-hvp33;new-oracle-hvp1;workspace1;precision-audits0;trial-formation0;acceptance0|candidate-nominal-substeps=0;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;production-cap-change=none|credit=one-replay-only-model-image-discriminator
```

Identity SHA-256:
`467f253b2e1756caafc77aea36fa76a67911d3afa4418a44e81862d51a8421ae`.

## Required command

Add `--nonlocal-al-krylov-model-image-discriminator`. It must:

1. reproduce D7R19R3 stdout SHA
   `a1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771`
   and semantic result
   `156782481d783abc500c1a1b888b693d158f8cd30503412d41285f4c725df6d8`;
2. retain D7R19R2 stdout SHA
   `3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0`;
3. bind the exact R3 current/predicted/radius/prefix roots and require forcing
   convergence after exactly 33 HVPs;
4. passively expose the converged step, final recursive residual and
   `sum(alpha_k H(d_k))` accumulated image while keeping public R3 bytes
   exact;
5. rebuild one static sparse workspace at the target and execute exactly one
   direct oracle HVP on the converged step;
6. derive the residual image as `r_final-g`, then compute exact roots and
   frozen error/model metrics for direct, accumulated and residual lanes;
7. require finite positive direct predicted reduction and no step/target
   mutation;
8. select one route under the precedence below;
9. retain exact static binding, one workspace build/release, one new HVP,
   zero precision audits, zero trial formation/acceptance, zero all-pair calls
   and rollback;
10. run one fresh process from each of two clean Release builds.

## Candidate bounds

A no-HVP image is admissible only if all are true:

```text
relative L2 image error        <= 1e-10
maximum scaled component error <= 1e-10
relative p.Hp error            <= 1e-10
relative predicted error       <= 1e-10
direct/candidate predicted     finite and positive
```

Denominators must use a nonzero scale derived from the direct result,
candidate result and direct HVP absolute-term sum. A zero/invalid denominator
or nonfinite metric makes that lane inadmissible.

## Frozen new work

```text
direct oracle HVPs          1
workspace builds/releases   1 / 1
maximum live workspaces     1
precision audits            0
candidate trials formed     0
candidate acceptances       0
all-pair calls              0
```

Vector folds and model scalars add no HVP credit.

## Routes and precedence

1. `KRYLOV_MODEL_IMAGE_RESIDUAL_CANDIDATE` when residual-derived passes;
2. `KRYLOV_MODEL_IMAGE_ACCUMULATED_CANDIDATE` when residual-derived fails and
   accumulated passes;
3. `DIRECT_MODEL_HVP_REQUIRED` when both no-HVP lanes fail but the direct
   oracle and all controls pass.

Parent/identity/target/prefix, oracle, finite, work/lifecycle, static binding,
rollback, build/process repeat or route-precedence mismatch is hard FAIL.

## Authority boundary

This contract grants one replay-only model-image discriminator. It does not
authorize recurrence grace, production cap changes, trial formation or
acceptance, a full transaction, another candidate nominal substep, timing,
performance claims, public state mutation or runtime/production authority.

## Result

The discriminator passes reproducibly and selects
`KRYLOV_MODEL_IMAGE_RESIDUAL_CANDIDATE`; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r19r4-krylov-model-image-evidence-2026-08-23.md).
Residual-derived and direct model quadratic/predicted scalars are bit-exact;
the image L2 error is `1.24e-15`. Research/freeze a sixth-trial-only guarded
recurrence-grace reclosure next. This result itself grants no grace, trial or
production authority.
