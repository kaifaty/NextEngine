# NSR3-B4C3A1 balanced publication ledger evidence -- 2026-08-21

Status: `PASS / CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE / B4C3T_DESIGN_AUTHORIZED`

## Reproduction

```text
nonlocal-formula-reclosure --canonical-stage-ledger-self-test
```

Two reports are byte-identical:

```text
raw JSON plus LF  bfce9a1135a7aac89e4a4906d7226119f273a0e55bdf7e03259aed99e075eb68
JSON without LF   107e56e5be49ebb157b0662767c934ced6addf6f3a7054d696772d6699ee1e90
semantic result   5b50800cc5427e812cd5637b2952e6f4657803e430203214bf95de3667620367
```

Whole-chain wall times were `66.36/65.99 s`; maximum resident sets were
`8,424/8,420 KiB`. Each run includes B4C3Q and its complete parent chain. This
is harness cost, not a production performance result.

## Momentum result

The data confirms the architectural distinction frozen by the contract.

| Case/level | Max raw published residual | Max compensated residual | Cumulative publication impulse `(x,y,z) kg m/s` |
|---|---:|---:|---|
| P1 coarse 21 | `5.76728e-6` | `5.41391e-13` | `(7.15e-10, -1.3914e-7, -2.67084e-7)` |
| P1 fine 42 | `9.98133e-6` | `4.66520e-10` | `(-9.50136e-8, -8.13424e-8, -9.65832e-8)` |
| P2 coarse 1 | `0` | `0` | `(0,0,0)` |
| P2 fine 2 | `9.06104e-7` | `1.20717e-15` | `(0,-1.25e-7,0)` |

Thus applying the old `1e-9` residual directly to the decoded state would
reject an explicitly modeled representation impulse. Subtracting the recorded
`I_q` reproduces the solver ledger and retains the original KKT gate. Maximum
impulse-report closure is `3.46e-17 kg m/s` on P1 and `9.75e-17 kg m/s` on P2;
compensated vector closure is at most `9.47e-22 kg m/s`.

Fine ledger roots are:

```text
P1  5feac29a07dbd03dc0aa4467056fed84bcec698de3c720ad16de854aeb559eea
P2  bb5562f1dc268a7dcc885092eaa45d1db2b21a76dc0088f9de79116d6fc7b312
```

## Energy result

Committed fine publication deltas accumulate in absolute value as follows:

| Case | Kinetic | Pressure | Gravitational | Mechanical |
|---|---:|---:|---:|---:|
| P1 | `2.49086e-8 J` | `6.54759e-5 J` | `1.36924e-5 J` | `6.73231e-5 J` |
| P2 | `3.83372e-9 J` | `0 J` | `7.48523e-7 J` | `7.49802e-7 J` |

Every per-entry kinetic inequality, aggregate gravitational bound and mechanical
decomposition passes. Maximum decomposition error is `8.01e-16 J`. Pressure
dominates P1's one-frame representation-energy perturbation; this is recorded
as long-horizon B4C3T design evidence, not retroactively bounded by a fitted
threshold.

## Atomicity and physical correspondence

P1 commits exactly 42 frames plus 42 ledger entries; P2 commits `2+2`. No
coarse root or coarse ledger total is committed. Repeated, reverse and affine
publication runs reproduce frames and every ledger scalar exactly. The selected
trajectory roots remain:

```text
P1  ece583962cb07f7a15bb1b84a83719895ec5af2c2735b0904ea76f833180939d
P2  8138d5202b4a754c43b8ee7306200b29d6d1c75b959717f9a0f3c54c1f543ddd
```

Binary/nearest physical differences and exact contacts remain within B4C3Q
bounds. A forced failure retains two private frames and two private ledger
entries but commits `0+0`; pre-transaction state/root remains exact. All B4C3Q
typed failures and its B4C3A/B4C2T parent hashes pass transitively.

## Decision

Select `CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE`. This authorizes only B4C3T
full-controller and long-horizon physical-bound design. No complete canonical
trajectory, nominal, CUDA, runtime, schema or production authority is granted.
