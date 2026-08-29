# NSR3-B4C3TAR2 combined adaptive replay evidence

Status: `PASS / CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE`

Date: `2026-08-21`

## Reproducible result

Command:

```text
nonlocal-formula-reclosure --combined-adaptive-replay-self-test
```

Two complete reports are byte-identical:

```text
status                 PASS
disposition            CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE
raw JSON + LF          911f4ee088d3c8a2e9ad8d5e11d2060295207ff57c70a003147dcd80b30d81c6
raw JSON without LF    5862a1c9a56414d1fb4809bcc08e3e5decfe059679defb974e05890909b8dc3d
semantic result        ae52a97a6c0bd6fe5cacd7746d3131a7f45ebc1e9a79e231e665e1e87cb777f7
wall time              107.72 s / 107.50 s
maximum RSS            8,332 KiB / 7,972 KiB
```

The full B4C3A2 parent is exact at raw-without-LF
`8ebee39b...d0d9`; its transitive historical chain is therefore preserved.
The isolated combined probe passes at:

```text
raw JSON + LF          5dfb9bb042ee0ae8534e45211490fbd32b16d49c2e39377d386847f7d4e1bfd3
raw JSON without LF    77bd22b35364227cd70845376d1cbfffed259bfcf411c29eca7525f0a6e09f6a
semantic result        f025b2d692c2e612fc876dc101cdd103fc01b8b8f0f57c70ac07f8903907925c
wall time              11.29 s
maximum RSS            8,088 KiB
```

## Complete controller result

| Case | Macro frames | Accepted / attempted / discarded | Nonlinear / spectral HVP | Recovered frames |
|---|---:|---:|---:|---:|
| P1 supported | 8 | `364 / 563 / 199` | `3810 / 384` | 2 |
| P2 released | 16 | `82 / 124 / 42` | `218 / 96` | 0 |

P1 recovers only the two known exact `KKT_SOLVE:REJECT_LIMIT` failures. The
first commits 64 after failed 16 and passing 32/64 candidates; the second
commits 84 after failed 21 and passing 42/84 candidates. All failed and coarse
candidate frames and ledger entries remain private. Attempted substeps equal
the sum of every attempted level and the controller's executed-work counter.

P2 retains the exact pressure-onset schedule: frames 0--13 are
`INACTIVE_EXACT`, frame 14 is `FORECAST_ACTIVE`, and frame 15 is
`START_ACTIVE`. Post-commit forced rollback, recovery-classifier negatives and
all KKT-ledger policy negatives pass.

## Canonical and ledger identity

| Case | Canonical trajectory | Legacy ledger | KKT-policy ledger |
|---|---|---|---|
| P1 | `01d2bd98...95e6` | `34b97e58...dbb0` | `87140263...bb9` |
| P2 | `1ffcec92...ebf1` | `65240556...a210` | `00c4384e...57c0` |

Committed frame and ledger counts are equal, global steps are contiguous and
only the selected fine level contributes to any root. The policy root binds
representation `f57d88c2...79c` and ledger policy `b1136c2c...e2f` while the
legacy root remains available as an unchanged diagnostic identity.

P1 contains two strict max-scale diagnostic excursions, with maximum
`1.1268128727201902e-9`. The corresponding physical KKT sum-scale residual is
`9.9175114078160991e-10` and passes the unchanged `1e-9` gate. P2 has no strict
excursion; its maxima are `2.8213744026556824e-10` strict and
`1.8970497692918031e-10` KKT-scale.

## Physical bounds

All pre-frozen binary, energy, contact and schedule gates pass without
refitting. P1's maximum position/velocity bound utilizations are `0.015695`
and `0.569576`; publication mechanical and pressure budget utilizations are
`0.0164895` and `0.0164953`. Its maximum density strain is `6.82674e-4`,
penetration is zero and maximum speed is `0.216978 m/s`. P2 preserves zero
contact-time error, terminal contacts and negligible lateral drift.

## Decision

Select `CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE`. This
authorizes only design of B4C3TR complete canonical fixed `48/96/192`
reference lanes. B4C3TC comparison, B4C4 packaging, nominal corpus, CUDA,
runtime/schema and production work remain blocked.
