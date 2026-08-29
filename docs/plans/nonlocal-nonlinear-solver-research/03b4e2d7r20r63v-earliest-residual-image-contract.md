# NSR3-B4E2D7R20R63V earliest residual-image contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63V` |
| Parent | R63U semantic `832aa624...7d1de`; two exact final candidates |
| Claim | Earliest stable exact image certificate on two immutable PCG replays |
| Budget | Two states `0..8`; 18 image certificates; full eight updates; no timing |

## Frozen parent roots

```text
R63U semantic   832aa624e33ad947ca288c1507efb6fcf752cb38b6a58f87c3a23966b357d1de
retained cert   38bd22c1b779801d807b525acb8a50a4515b7afb16dc84cf0301619358987adf
exported cert   f1937129df5e6daf0c6b5c2ecd810c8dbf1d52688e7c701ce108f573e18fe29f
final signs     89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094
R63S lanes      2ec70f63...4162 / 5dcc65f7...c255
```

## Replay and certificate

Use the exact R63S starts, factor roots, direct `sigma T(T^T p)` products and
eight PCG updates. At state 0 and after each update, call the unchanged R63U
certificate using exact `H*`, RHS, `Z` and left defect.

A lane passes iff some state in `0..8` passes and every later certificate
passes. Continue after pass. Require complete R63S work counts plus, per lane,
nine certificates, 918 solution conversions, 93,636 residual products,
93,636 image products and 918 sign comparisons. Final certificate roots must
equal R63U and final PCG lane roots must equal R63S.

## Controls and routes

Controls cover pass at state zero, pass after one update, regression after
pass, missing state, recurrence/certificate independence, root mutation and
classifier precedence. R63B--R63U byte regressions must pass.

Routes, in order:

1. `EARLIEST_IMAGE_APPARATUS_REJECTED`.
2. `EARLIEST_IMAGE_CERTIFICATE_REJECTED`.
3. `RETAINED_WIDE_EARLIEST_IMAGE_REJECTED`.
4. `EXPORTED_FACTOR_EARLIEST_IMAGE_REJECTED`.
5. `TWO_LANE_EARLIEST_IMAGE_CANDIDATE`.

## Exclusions

No expected iteration, early stop, R60 signs, stored dense `H/X`, candidate
change, factor/precision/rank change, sparse work, timing, runtime/GPU/
production inference. R64/R65 remain blocked.
