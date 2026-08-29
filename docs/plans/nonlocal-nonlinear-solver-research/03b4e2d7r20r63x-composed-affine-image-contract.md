# NSR3-B4E2D7R20R63X composed affine-image contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63X` |
| Parent | R63W semantic `35a74726...b48d2c3`; exact containment with componentwise-image rejection |
| Engineering consumer | Decide whether preserving residual correlation admits a finite certificate |
| Claim class | Correlation-preserving binary128 affine-image enclosure |
| Budget | Exact profile build; two immutable states `0..2`; six finite image certificates; no timing |

## Frozen parent roots

```text
R63W semantic  35a74726a47d377ec8f97aac77dc52b22fb2463de1b33d77e6ea1fa63b48d2c3
R63W stdout    1e85677e07e22d91fc8b3ca7f9a4b7f9bdfbafcb7a404b4797854c1fc531c2fe
R63V semantic  905ce406943303ac51b5f33a1580ddc438ad8e6adb6d38ed2b8eb179954acad8
retained lane  22d3b114dcec5845183fa72982203242f8c608fdeb0b6c1f6cbbf37ccc75d8a0
exported lane  1f6a7041734089ed0a07bfc5fb6ecce51ea742376fa31b2f8efe1353520407d6
final signs    89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094
```

## Exact profile builder

Using exact dyadic `Z`, `b` and common `H_num/d`, build:

```text
g_i       = sum_j Z_ij b_j
M_num_ik = sum_j Z_ij H_num_jk
M_ik      = M_num_ik/d.
```

Emit binary128 `g_center/g_radius`, `M_center/M_radius` and `rho_upper`.
Require exact containment of 102 `g` entries, 10,404 `M` entries and the left
defect. Bind exact builder work, all arrays and roots. The finite certificate
signature contains no dyadic object.

## Finite certificate

At each candidate state, perform exactly 102 compensated Dot2 operations of
length 103 and 10,404 outward matrix-radius terms. No intermediate residual
center/radius is constructed.

Require finite/normal-or-zero data, nonnegative radii, `rho_upper<1`, positive
lower denominator, exact work counts and root closure. Sign classification is
strictly from the outward finite error.

The independent exact R63V image audits all `6*102=612` interval
containments. Exact values/signs cannot repair the finite result. Finite
classification must be reject/reject/pass with state-2 sign root equal to the
frozen R63V root in both lanes.

## Controls and routes

Controls cover identity, high-cancellation affine form, nonzero profile
radii, dropped `g` radius, dropped `M` radius, wrong `Z/H` orientation,
underflow/nonfinite/negative radius, noncontractive `rho`, oracle independence,
root mutation and classifier precedence. R63B--R63W byte regressions pass.

Routes:

1. `AFFINE_IMAGE_APPARATUS_REJECTED`.
2. `AFFINE_IMAGE_CONTAINMENT_REJECTED`.
3. `AFFINE_IMAGE_CERTIFICATE_REJECTED`.
4. `RETAINED_WIDE_AFFINE_IMAGE_REJECTED`.
5. `EXPORTED_FACTOR_AFFINE_IMAGE_REJECTED`.
6. `TWO_LANE_AFFINE_IMAGE_CANDIDATE`.

## Ceiling

Success authorizes only research/freeze of a factorized or sparse
correlation-preserving image enclosure. No state update, production stop,
runtime binary128, exact/dense profile construction in runtime, corpus claim,
timing, GPU or production authority. R64/R65 remain blocked.
