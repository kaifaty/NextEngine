# NSR3-B4E2D7R20R63Y portable expansion-affine contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63Y` |
| Parent | R63X semantic `904614d7...95a65`; dense binary128 affine-image pass |
| Engineering consumer | Select the minimum portable expansion width for the offline certificate |
| Claim class | Strict-binary64 onefold/twofold affine-image enclosure |
| Budget | Two widths; two immutable lanes; states `0..2`; no timing |

## Frozen parent roots

```text
R63X semantic   904614d78c1948f4af836167936bdc528a0c69a0ded7e85cfe9ae264be395a65
R63X stdout     9e218dbd248fc079fd058a44753d5d09b4dbaa8ed5ac9695eca2be68ccf25bf7
R63X profile    0505cdb98db31a7f087cbc23290b5e0adc964dce05f4e73a4429083d860e97d0
retained lane   a52826e657f81c401306fe19451d9dccd99fd4a8d66b1c300bfb9e88bed61cb9
exported lane   907c3780ed8e41ae64a74fc60df856a1368348ee96dccb6fe12bb97f73c3a1d1
final signs     89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094
```

## Profile projection

Rebuild exact R63X `g=Zb` and `M=ZH*` once. For widths `K=1,2`, repeatedly
round the exact remainder to binary64, subtract that component exactly and
emit an outward binary64 radius for the final remainder. Do the same for each
binary128 candidate component. Require exact containment, finite/normal-or-
zero components, nonnegative radii, strict `rho_upper<1`, exact work counts and
root closure.

The finite certificate signature contains only binary64 components/radii,
`rho_upper`, width and roots. It contains no binary128, dyadic, rational or
exact-oracle value.

## Finite certificate

For every row and width `K`, execute one `Dot2Err` over
`K + 102*K*K` products. Add outward vector, matrix, candidate-projection and
cross-radius terms. Reconstruct a contained interval for every candidate
component and derive the final sign only from that interval plus
`image_inf/(1-rho_upper)`.

Execute all twelve lane/state certificates:

```text
widths 1,2 x lanes retained,exported x states 0,1,2
```

There is no early exit after a pass. Exact R63V audits all `12*102=1,224`
image containments but is unavailable to finite classification. Each passing
state-2 sign root must equal the frozen root.

## Controls

Controls cover onefold/twofold exact reconstruction, nonzero second component,
high cancellation, coefficient and candidate radii, dropped radii,
`TwoSum`/`TwoProductFMA`, subnormal/overflow/nonfinite rejection, noncontractive
`rho`, orientation, oracle independence, root mutation and classifier
precedence. R63B--R63X byte regressions pass.

## Routes

1. `PORTABLE_EXPANSION_APPARATUS_REJECTED`.
2. `PORTABLE_EXPANSION_CONTAINMENT_REJECTED`.
3. `ONEFOLD_AFFINE_IMAGE_CANDIDATE` when both onefold lanes pass.
4. `RETAINED_TWOFOLD_AFFINE_IMAGE_REJECTED`.
5. `EXPORTED_TWOFOLD_AFFINE_IMAGE_REJECTED`.
6. `TWOFOLD_AFFINE_IMAGE_CANDIDATE` when both twofold lanes pass.

The onefold result is observational; twofold still executes when onefold
passes. No decimal tolerance, adaptive width or post-run route change exists.

## Ceiling

Success authorizes research/freeze of a factorized sparse candidate producer
and an offline producer/verifier artifact boundary using the selected width.
It does not authorize runtime state update, a state-2 stop rule, dense profile
construction in runtime, threefold arithmetic, corpus claims, timing, GPU or
production authority. Failure localizes the first expansion-width boundary;
it does not authorize weaker containment or sign thresholds.
