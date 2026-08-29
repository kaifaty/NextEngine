# NSR3-B4C3A2 KKT-scale stage-ledger evidence

Status: `PASS / CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --kkt-scale-stage-ledger-self-test
```

Two complete reports are byte-identical:

```text
status                 PASS
disposition            CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE
raw JSON + LF          aa7b348944b59f9890fcbf94b642ab0b30e7f4076fa5c476931f515f727a7c9b
raw JSON without LF    8ebee39be668d1b99758040b42936ceeea3dfc6bc6d09888a4a99c35612ed0d9
semantic result        b92dc30aeabe970b517d98a40ca1aaf2c3c109118b0dbac9d97cb419cbbfca9c
wall time              97.60 s / 97.28 s
maximum RSS            8,480 KiB / 8,512 KiB
```

The full parent B4C3L report is exact at `ba1684f0...9540`, transitively
preserving the complete historical chain.

The isolated probe passes at:

```text
raw JSON + LF          901adb691139bc815458748c1655034f5215b649e25e8e75ba875154e6ea1948
raw JSON without LF    77e0905c1e7c5e96873a7a73c7195e08de672a38fdcc02bec2429a1d80a4df90
semantic result        5be3b477f7b4ca4e292ae04e7193e2d00ea522fd746401a70175f8f5ea393046
wall time              2.22 s
maximum RSS            6,276 KiB
```

## Identity preservation

Both cases preserve their B4C3A1 canonical and legacy ledger roots exactly,
while producing distinct policy-ledger roots:

| Case | Trajectory | Legacy ledger | KKT policy ledger |
|---|---|---|---|
| P1 `21/42` | `ece58396...939d` | `5feac29a...9eea` | `5e0f3c07...3527` |
| P2 `1/2` | `8138d520...3ddd` | `bb5562f1...b312` | `d30e2edc...0cec` |

The policy root binds representation profile `f57d88c2...79c`, ledger policy
`b1136c2c...e2f`, the complete legacy field set, KKT/strict scales and
residuals, correspondence bounds and policy gates. Changing the policy identity
or omitting its fields changes the root.

## Transaction and physical result

P1 commits only 42 fine frames/entries; P2 commits only two. Coarse roots are
absent from the selected sequences. Repeat, reverse and affine runs reproduce
every canonical frame and policy-ledger field exactly.

| Case | Strict diagnostic | KKT physical | Correspondence utilization |
|---|---:|---:|---:|
| P1 fine | `4.6652e-10` | `4.5711e-10` | `0.6854` |
| P2 fine | `1.2072e-15` | `6.0359e-16` | `0` |

Candidate and legacy stages are physically identical: decoded state, contacts,
publication impulse, pressure/mechanical energy totals and every decomposition
gate match exactly. Embedded gates pass unchanged.

Forced rollback after two private entries and invalid-scale, KKT-overflow,
corrupt-closure, nonfinite-strict and policy-identity negatives all pass.

## Decision

Select `CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE`. This authorizes only
design of a new complete adaptive recovery replay which combines exact
`REJECT_LIMIT` refinement with KKT-scale ledger admission. B4C3TAR remains a
preserved FAIL; fixed-reference, nominal, CUDA, runtime and production work
remain blocked.
