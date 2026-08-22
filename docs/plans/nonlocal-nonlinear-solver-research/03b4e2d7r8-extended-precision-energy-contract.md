# NSR3-B4E2D7R8 -- extended-precision energy discriminator contract

Status: `FROZEN / NOT_RUN / PRIVATE_DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r8-extended-precision-energy-discriminator|v1|parent=f29336079dcb1a54db81fe5c8913130c3cbf72c272336c80fcad6d02c68eb47e:a29e3f7921f35065b3bcac1ebbf7f6b1d25422e059b4287d3b7a31cf76865588:3615964074fd477ab384f5710b274d8da4f4d1c77fd3f11ecb08a45eec0039ff|states=eta1e-8:21ad77e22bdba4520ca231bb78d51947a1b67e4263e08dae33af13b0aafabd05;eta1e-9:075442656934aeed156642091e9fc1ed41bc739513b5710990cdfb231d8aabc2;eta1e-10:299e4ce372a8a3d418ac6f354c70c362fd772acdf278464fb603a3881527901c|profile=linux-x86_64;flt-radix2;sizeof-long-double16;ldbl-mant-dig64|formula=independent-long-double-radius,kernel,density,phr,inertia;binary64-inputs-and-kernel-scale-promoted-exact|sum=fixed-order-and-compensated;resolved=agree-sign-and-abs-reduction>=1024-extended-total-ulp|trace=all-three-unique-failures;all-trials;total,phr,inertia;naive,compensated;ulp-ratios;binary64-extended-pair-membership;h-half-margin;h-margin;parent-model,raw,direct|controls=d7r7-complete-bytes;state-roots;common-tight-state;formula-independence;forced-rollback|routes=representation-topology-precision;binary64-energy-evaluation;analytic-gradient-hvp-reclosure;stronger-precision-required|precedence=topology,energy,derivative,stronger|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;trial-acceptance=none;public-commit=none;physics-mutation=none|credit=one-local-numerical-remedy-research-only
```

Identity SHA-256:
`be70da6bc83bc2ca3d0659984452225ee0493a8efd81c59e8aff18a6ec826495`.

## Required command

Add `--nonlocal-al-extended-precision-energy-discriminator`. It must:

1. reproduce the complete D7R7 parent bytes and three exact state roots;
2. fail before evaluation unless the frozen long-double profile is exact;
3. independently reevaluate every D7R7 failed-inner current/trial pair using
   promoted binary64 inputs and long-double arithmetic;
4. emit fixed-order plus compensated total/PHR/inertia reductions, extended
   ULP scales and the frozen resolved-sign test;
5. compare every fluid/boundary pair membership and margins to `h/2` and `h`;
6. preserve every D7R7 work/acceptance fact and never accept a replay trial;
7. retain forced rollback and zero public commits;
8. emit exactly one route under the frozen precedence.

## Routes

1. `REPRESENTATION_TOPOLOGY_PRECISION_RESEARCH`: at least one causative
   binary64/extended pair-membership decision disagrees.
2. `BINARY64_ENERGY_EVALUATION_RESEARCH`: no preceding route; topology agrees
   and at least one model-positive, binary64-negative trial has a resolved
   positive extended reduction.
3. `ANALYTIC_GRADIENT_HVP_RECLOSURE`: no preceding route; topology agrees and
   at least one model-positive trial has a resolved negative extended
   reduction.
4. `STRONGER_PRECISION_REQUIRED`: every parent/profile/formula/control fact is
   exact but the preceding sign tests are unresolved.

Parent, profile, state-root, formula independence, nonfinite extended value,
work/acceptance, rollback or route-precedence mismatch is hard FAIL. Naive and
compensated sign disagreement is unresolved and routes only to stronger
precision. PASS is diagnostic classification only. It grants no long-double
production path, kernel/formula change, trial acceptance, cap/tolerance,
pressure gate, `beta`, solver-family, trajectory, performance, runtime,
GPU/PhysX or production authority.
