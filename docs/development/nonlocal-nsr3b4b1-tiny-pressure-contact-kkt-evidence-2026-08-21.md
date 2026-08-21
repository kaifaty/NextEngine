# NSR3-B4B1 tiny pressure contact-KKT evidence -- 2026-08-21

Status: `FAIL / P1_FRAME0_KINETIC_REFERENCE / P2_NOT_EXECUTED`

## Reproduction

```text
nonlocal-formula-reclosure --tiny-pressure-contact-kkt-self-test
```

Three reports are byte-identical:

```text
raw JSON plus LF  949b590057827de329b60b9b7a9979672b800f73ad0fa1a43b7a330cd6b2c791
JSON without LF   2b40f5662f9aa38f2bc6bae35d4e9d2e21c9b0f967af40c1831562d7fa4c98c9
semantic result   11302033bacf1a3656c3b584f9db68f573e088b786c56ceea10b32dfee509a2e
```

The first failure is `P1_SUPPORTED_COLUMN:FRAME_COMPARISON`; P2 is not
executed under the frozen stop rule.

## What the KKT repair closes

The adaptive P1 path and all fixed `48/96/192` references complete without a
KKT, objective, penetration, complementarity or ledger failure:

| Observable | Result / range | Gate |
|---|---:|---:|
| adaptive accepted / executed substeps | `286 / 429` | charged |
| adaptive nonlinear HVPs | `3010` | charged |
| maximum positive density strain | `6.6203247843787949e-4` | `<=1e-3` |
| maximum speed | `0.21603929971557492 m/s` | `<=0.01c` |
| maximum complete ledger | `4.3548532471313074e-10` | `<=1e-9` |
| vertical COM change | `8.8929734415468809e-4 m` | `<=0.05dx` |
| energy creation | `0 J` | `<=0.044145 J` |

Fixed convergence passes with position ratio `1.782088942890091` and
velocity ratio `1.8369858344027901`. Every later candidate frame passes its
fixed-192 aggregate comparison. This preserves the one-step KKT conclusion:
the new blocker is not contact stationarity.

## Exact controller miss

At frame zero the current state has zero active pressure centres, so the old
pressure-only spectral policy selects one initial substep. The embedded
`1/2` pair reports:

```text
position error = 3.6249035539735351e-4 dx
velocity error = 1.998092633246416e-5 c
kinetic error  = 0.085760941514086919
```

and passes the `0.15` kinetic gate. The committed two-substep state versus
fixed-192 instead has:

```text
position error = 4.1089266598801327e-4 dx
velocity error = 5.1496342561234217e-5 c
kinetic error  = 0.24455261228910977
```

Only the inherited kinetic gate fails. Its absolute difference is
`3.4301954292963893e-4 J`; the energy is not at a binary64 floor.

The discrepancy is non-asymptotic at contact onset: the initial state is
pressure-inactive, but clamping the frame predictor makes the bottom layer
contact-active and creates compression. The start-state pressure Hessian
therefore supplies no stiffness signal for an event that occurs inside the
candidate segment.

## Decision

- Preserve B4B1 FAIL and the `0.15` kinetic gate.
- Do not accept the otherwise physical adaptive trajectory or execute P2.
- Research a feasible-predictor contact-onset forecast before changing the
  controller. It must retain `n=1` for detached P2 free flight and charge any
  extra spectrum/HVP work.
- Do not use a hard minimum substep count or fixed-192 oracle inside the
  adaptive decision.
