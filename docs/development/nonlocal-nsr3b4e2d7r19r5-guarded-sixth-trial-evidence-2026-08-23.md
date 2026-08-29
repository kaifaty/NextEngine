# NSR3-B4E2D7R19R5 guarded sixth-trial evidence

Date: `2026-08-23`

Status: `PASS / SIXTH_TRIAL_RESIDUAL_MODEL_ACCEPTANCE_CANDIDATE / SHADOW ONLY / NO STATE`

## Outcome

The exact sixth trial is admissible under the one-shot recurrence guard and
residual-derived model image. HVP 33 converges, `r_final-g` preserves the
direct predicted reduction bit-for-bit, the pairwise pre-cancelled divided
reduction gives acceptance ratio `0.999999308792273`, the binary64-owned
long-double audit resolves positive and the existing radius policy leaves the
radius unchanged.

This closes the previously missing trial without a candidate model HVP. The
trial is shadow-only: it is not appended, committed or used to continue the
transaction.

## Parent and target correspondence

D7R19R4/R3/R2 remain exact:

```text
R4 stdout SHA-256  798e8005aaa503eedb7a90ed9fd3590230cec7bbadf85536b39ac135ac7cb3a0
R4 semantic        99b77c1e0d4750911af11a5cc9933d512dc22ee3041a873eabccc2a8389f2cd0
R3 retained        true
R2 retained        true
```

Target roots:

```text
current     54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a
predicted   36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e
step        74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd
trial       932ce178a6238025c8de6ea907d9966638fa0f5abef7780a15fb9c8171f67ea2
radius      0x3f8999999999999a
```

## Guard result

Every first-32 HVP record is finite, positive-curvature and interior. The last
eight available next-residual ratios strictly decrease.

```text
base recurrence HVPs  32
grace HVPs             1
absolute recurrence    33
ratio after 32         3.027232803424963e-05
eta                    2.51880524249163e-05
ratio / eta            1.2018526690179472
HVP 33 forcing         converged
```

The guard changes neither forcing tolerance nor trust radius and cannot grant
a second extra HVP.

## Model and acceptance

```text
candidate model HVPs     0
direct oracle HVPs       1 (control only)
predicted reduction      0x3bc27dd9b2871ea7
direct predicted         0x3bc27dd9b2871ea7
divided reduction        0x3bc27dd8dc16d400
divided reduction value  7.831493101542443e-21
divided/predicted         0.9999993087922731
step norm                 8.586972198522362e-11
would accept              true
radius owner              NONE
radius after              0x3f8999999999999a
```

The divided computation repeats exactly. Its ratio is far from the `0.1`
acceptance boundary and preserves the radius.

## Precision result

The one binary64-owned long-double audit is finite and resolved positive:

```text
precision root       a58caa2c1cb83c23dbcc15b8d2daf243de7749e6e692a92369141bbfd741f8db
resolved positive    true
resolved negative    false
topology mismatch    false
```

No binary128 audit is needed because this trial is not a candidate-effect
acceptance relative to the selected divided/model policy.

## Work and safety

The shadow uses two workspace builds/releases with at most two live, one
long-double audit, one formed trial and zero state commits/all-pair calls. The
33 recurrence HVPs and one direct oracle HVP belong to parent/control lanes;
candidate model-HVP work is zero. Static binding, finite controls and rollback
pass. No candidate nominal substep or transaction continuation runs.

## Reproducibility

Implementation commit:
`f684ac11` (`research: reclose guarded sixth trial`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r5-a.nAMvs4`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r5-b.ZS7RPP`.

Both binaries are `5,867,808` bytes, have SHA-256
`e73e8c91c468321bbdce94647bcec0bd2b366e8b6b47cefc8d89ffc52ecfb4c8`
and GNU build ID `1afaa6ecb8c4c083a9649dc4502b64cfb5003336`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r5-a.PJhJJ8`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r5-b.hWPhP9`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  2,433
stdout SHA-256        fe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294
semantic result       ae751940bf195fdf70aab9f1a5df6ce986decfc9c4f3c5fbeb26bf95d43e8880
route                 SIXTH_TRIAL_RESIDUAL_MODEL_ACCEPTANCE_CANDIDATE
```

## Decision

Authorize research/freeze of D7R19R6 as one full private first-substep
transaction candidate with a guarded-residual completion policy:

- normal trust solves retain their direct model HVP and historical behavior;
- only a solve that reaches the exact base-32 guard may use one 33rd
  recurrence HVP and residual-derived model image;
- absolute per-trust work remains 33 HVPs and total/outer/trial/precision
  budgets remain bounded;
- the first five completed trials must remain exact to R2, and the sixth must
  reproduce this evidence exactly;
- subsequent work may be classified but no state is publicly committed.

This is the smallest integration experiment that can reveal the next real
transaction boundary without generalizing residual-model reuse to untested
ordinary solves.
