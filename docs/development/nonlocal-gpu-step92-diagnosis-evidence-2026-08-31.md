# NCGP5 hydrostatic step-92 diagnosis — 2026-08-31

## Verdict

`SUPPORTED_BOUNDED / NO IMPLEMENTATION REPAIR SELECTED / PERFORMANCE NOT_RUN`.

The frozen NCGP4 witness is reproduced exactly: hydrostatic step 92 has one
maximum CPU/GPU position difference of `5.211209258 mm` while the 4,000-sample
RMSE is `0.238778778 mm`. The outlier is SampleId `21876`, component `z`.

The NCGP5 discriminators do not expose a formula, compensated-state, local
boundary or solver-publication mismatch. On one canonical synchronized
pre-step-92 state, GPU versus independent long-double evaluation has gradient
relative L2 `1.040370151e-5`, HVP relative L2 `1.156370723e-6`, HVP cosine
loss `6.645584148e-13`, and an exact pressure-active signature. One complete
CPU/GPU step from that state has position RMSE `0.000036947 mm` and maximum
error `0.000336541 mm`.

The accumulated trajectory reaches the lower basin plane on adjacent steps.
At step 91 the CPU outlier is already at `y=0.025 m`, while the GPU outlier is
at `y=0.02623346075 m`. At step 92 the GPU route records the actual lower-face
contact mask `0x04`, position `y=0.02500000037 m` and sealed contact impulse
`[-0.0062300246, 0.0145497592, -0.0220035538]`. Error growth from steps
80--92 is smooth: no greater-than-`2x` slope transition exists. This supports
H5E, with a boundary event amplifying ordinary long-trajectory sensitivity;
it does not identify the concrete semantic defect required to authorize a code
repair under NCGP5.

The old frozen `5 mm` maximum gate therefore remains failed. Replacing it with
a product-oriented percentile/contact-aware trajectory criterion is a new
contract decision, not a reinterpretation of NCGP4 or NCGP5.

## Frozen identity

- source commit:
  `c134db6279fbb708e3ee4c82bcde87eafd71301d`
- source tree:
  `2753ad1cf2299b65a402d94b2ca413020a75d33f`
- aggregate NCGP5 contract root (NCGP4 parent plus NCGP5 leaf):
  `782718c186cc96baf7a32c0fec862d667360f272e25a17e5b70202e7afe3cf9f`
- NCGP5 leaf contract SHA-256:
  `792b495b2760fdfbce4790506690913db1e5b47f98e541dd48d6f484a8499d44`
- source root:
  `c2cb2a00b75d98ff0ae6d964bdfd8775aad315e4f105afb1f8b1368c4e55a991`
- input root:
  `f96ad2a8bd7c7fcee2917ff6168c1aac59d7da56d2f5844a3cfcdab396eda80f`
- clean Release binary SHA-256:
  `8d9fbc448fd3178f5d9880e893d7d738c3f807151c2019158157af3c0c08a643`
- raw stdout SHA-256:
  `8a4b0eeedaaca7e3f9daf9455f2b05c4a886bdddc78a470b1970b05cf9c3a40b`
- receipt root:
  `6f442059ccef28d13d3f7b337a447a5547e4ef99965f5a7b0375c44bbeeeabc9`
- result root:
  `a8e7f138089874aca3e413445e7eac06ac6aa32ed2d449d2539386b0bf53e494`
- host: NVIDIA GeForce RTX 3080, SM 8.6, CUDA runtime/driver 13.3;
  allocated device memory `137251397` bytes
- flags: C++ `-O3 -Wall -Wextra -Wpedantic -Werror -ffp-contract=off
  -fno-fast-math`; CUDA `-O3 --fmad=false --prec-div=true --prec-sqrt=true
  --ftz=false`; `SM=86`

The exact command is:

```text
nonlocal-corrected-cuda-step92-diagnosis --diagnose-hydro-step92-outlier
```

The raw JSON remains outside Git at `/tmp/ncgp5-exact-step92.json` for this
host session. Its hash above, all embedded identities and the command are the
durable locator.

## Error distribution and contact history

| Step | Max (mm) | RMSE (mm) | p95 (mm) | p99 (mm) | Max ID | Actual GPU contact |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 80 | 1.138728 | 0.087587 | 0.141001 | 0.388812 | 24137 | none |
| 84 | 2.187369 | 0.124094 | 0.200470 | 0.464889 | 24137 | none |
| 85 | 2.549535 | 0.136442 | 0.215428 | 0.501630 | 21876 | none |
| 90 | 4.616049 | 0.210596 | 0.285639 | 0.719794 | 21876 | none |
| 91 | 4.913653 | 0.223565 | 0.306306 | 0.773196 | 21876 | none |
| 92 | 5.211209 | 0.238779 | 0.332493 | 0.821256 | 21876 | lower y, mask `0x04` |

At step 92 the outlier itself is pressure-inactive on both CPU and GPU. It is
active on both routes through step 86 and inactive on both from step 87.
Its one-sided density difference is `0.170962%`, far below the trajectory
density gates. The GPU outlier row has 96 neighbors and eight active neighbor
centers; its canonical neighbor root is
`49f718acc2c4d93394bc880aa369f6bce575a21ebd0e975df84f091b3d1a772e`.

## Hypothesis update

| ID | Status | Decisive observation |
| --- | --- | --- |
| H5A active-set bifurcation | `NOT_SELECTED` | the outlier CPU/GPU active flags agree on every recorded step and the error has no active-transition slope break |
| H5B boundary semantic mismatch | `NOT_SELECTED_AS_DEFECT` | a real lower-face event is present, but the same-state one-step boundary/solver result remains sub-micrometre; the event timing differs only after accumulated state divergence |
| H5C compensated predictor/operator defect | `FALSIFIED_ON_WITNESS` | same-state gradient, HVP and one-step outputs pass the frozen gates |
| H5D stopping/publication mismatch | `FALSIFIED_ON_WITNESS` | the synchronized complete step is valid on both routes and publishes a maximum difference of only `0.336541 um` |
| H5E long nonlinear trajectory sensitivity | `SUPPORTED_BOUNDED` | smooth tail growth, one contact-timing offset, small RMSE/p99 and no isolated semantic mismatch |

`SUPPORTED_BOUNDED` is not a proof that every basin/contact state is free of a
GPU defect. It states only that the exact NCGP4 witness is better explained by
long-trajectory contact sensitivity than by the frozen alternative defects.

## Apparatus controls

The corrected and ID-permuted GPU routes have exact state identity. Deliberate
sample-record, neighbor-root and work mutations are all rejected. Retained
controls passed on the exact build:

| Control | Raw stdout SHA-256 | Result |
| --- | --- | --- |
| profile | `d26aaae980f98709f01b49dc4a85ce60454fb657fdb4aee1dd24975e6a4cfa5f` | PASS |
| graph | `61bcaec4de24d88b6ca9ec226756169ce756d8c2cae6b24d344a1f6d112c7051` | PASS |
| boundary | `c7e4d3f416188b2fad7a62e841cdade7ad797e69c7d022e8aa4faf44d65fc0c3` | PASS |
| transaction | `dfdd2d17da8ece2d6ad0a8383d06eca429385ed4240bce722e246c6897267b39` | PASS |
| physics | `72cf54d44c21cff11c157175d051ac80a66aa9fb19856df37547dcba7e52e982` | PASS |

Independent review is `NOT_TESTED`: NCGP5 was not given a separate reviewer.
No sanitizer or performance run was required or admitted after the unchanged
NCGP4 trajectory gate remained failed.

## Decision and next boundary

NCGP5 selects no implementation change. Do not tune pressure, surface,
contacts, trust-region tolerances or the 128-HVP ceiling in response to this
witness. Do not claim the neighbor-only `~1.0--1.18 ms` result as full-water
performance.

The smallest successor is a separately frozen product-level admission
decision. It may retain strict same-state/tiny/operator gates while defining
long-horizon game adequacy by distributional trajectory error plus physical
invariants and explicit contact-event treatment. Only after that new contract
passes may 16k/50k correctness and complete-step timing run.
