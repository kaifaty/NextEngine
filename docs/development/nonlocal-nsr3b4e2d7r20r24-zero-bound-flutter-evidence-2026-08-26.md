# NSR3-B4E2D7R20R24 zero-bound mask-flutter evidence

Status: `PASS / ZERO_BOUND_ROUNDING_FLUTTER`.

Implementation `74dc6dca` preserves R23 semantic exactly and emits:

```text
9f3c90728389047d56f777750f9941a33ced533ef97975405f534e69e6d6d67b
```

The complete step-32 mask run is:

```text
predicted-new [0..5]
current-old   [6]
predicted-new [7..64]
```

There are 64 new-face samples, one old-face sample, zero other-mask/ball
samples and two mask transitions. The actual preprojection scalar changes sign
twice. Its independent affine value is nonnegative and monotone across all 65
points and has zero sign transitions. Maximum actual/affine difference is
`7.8361392161690741e-36`.

This proves that R23's later event is a one-sample zero-bound rounding return,
not a second geometric breakpoint. The final all-new suffix begins at power 7,
but that observed suffix is not a permitted policy constant. A componentwise
forward error bound for the complete `alpha -> lambda -> A^T lambda -> z`
evaluation must own a stable-side predicate.

No new ladder point, mask override or solver state was introduced.

