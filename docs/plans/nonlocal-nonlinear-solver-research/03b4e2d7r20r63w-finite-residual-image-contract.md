# NSR3-B4E2D7R20R63W finite residual-image contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63W` |
| Parent | R63V semantic `905ce406...acad8`; both first pass at state 2 |
| Engineering consumer | Decide whether exact residual-image proof admits a bounded finite checker before sparse work |
| Claim class | Binary128 outward residual/image enclosure with exact correspondence oracle |
| Budget | Two immutable states `0..2`; six finite plus six exact certificates; no timing |

## Frozen parent roots

```text
R63V semantic    905ce406943303ac51b5f33a1580ddc438ad8e6adb6d38ed2b8eb179954acad8
R63V stdout      62e1c37617017c169bdd0ab05793c91f542cc0582eec610cf17fd763d1202e44
retained lane    22d3b114dcec5845183fa72982203242f8c608fdeb0b6c1f6cbbf37ccc75d8a0
exported lane    1f6a7041734089ed0a07bfc5fb6ecce51ea742376fa31b2f8efe1353520407d6
state-2 retained 98cb0fea923560f6128dbe8315d05f73848fc030741f97335daadaedc62d9626
state-2 exported bbd5aa70796f443f363991ea686ed5797b8be29e6ec315ee8d15dbafd0cd87d4
final signs      89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094
```

## Finite profile and algorithm

The exact-only builder converts every common `H*` entry to a nearest
binary128 center and a nonnegative upward binary128 radius containing the
conversion error. It converts the exact R63R left defect norm to `rho_up` and
requires exact containment plus `0<=rho<=rho_up<1`.

The certificate function signature contains no dyadic/rational object. It
uses R38 `Dot2` under the frozen binary128 profile:

1. 102 residual dots of length 103;
2. 10,404 outward matrix-radius propagation terms;
3. 102 image dots of length 102;
4. 10,404 outward residual-radius propagation terms;
5. one outward image infinity norm and error division;
6. 102 strict sign comparisons.

All values, bounds and intermediate sums must be finite and normal-or-zero.
Every radius is nonnegative. The lower denominator is one round toward
`-infinity` from `1-rho_up` and must remain positive.

## Exact correspondence

For both lanes at states `0,1,2`, the independent R63U certificate supplies
exact residual and image vectors only to a containment auditor. Require
`612/612` residual and image entry containments across all six certificates.

Finite classification must be exactly:

```text
state 0  reject with at least one unresolved sign
state 1  reject with at least one unresolved sign
state 2  24 positive / 78 negative / 0 unresolved
```

Both finite state-2 sign roots must equal the independent R63V sign root.
Containment or sign mismatch never widens a radius after observation and
never imports an exact sign into the finite result.

## Controls and routes

Controls cover identity/pass, nonzero interval propagation, cancellation,
dropped matrix radius, dropped residual radius, underflow/nonfinite data,
negative radius, noncontractive `rho_up`, wrong image orientation, finite
result independence from oracle mutation, root mutation and classifier
precedence. R63B--R63V byte regressions must pass.

Routes, in order:

1. `FINITE_IMAGE_APPARATUS_REJECTED`.
2. `FINITE_IMAGE_CONTAINMENT_REJECTED`.
3. `FINITE_IMAGE_CERTIFICATE_REJECTED`.
4. `RETAINED_WIDE_FINITE_IMAGE_REJECTED`.
5. `EXPORTED_FACTOR_FINITE_IMAGE_REJECTED`.
6. `TWO_LANE_FINITE_IMAGE_CANDIDATE`.

## Stop and ceiling

Success selects a finite offline checker and permits a separate sparse/direct
realization contract. Failure localizes center/radius, residual propagation,
image propagation or sign-margin insufficiency before any precision change.

No candidate update, expected-state repair, exact arithmetic inside the
finite function, runtime binary128 selection, sparse work, corpus claim,
timing, GPU or production inference is admitted. R64/R65 remain blocked.
