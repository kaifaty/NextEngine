# NSR3-B4E2D7R20R15 affine-enclosure evidence

Status: `PASS / AFFINE_SHADOW_RESOLVES_SUBSET`.

Implementation `ed832851` preserves every parent case, step and inverse root.
Semantic result:

```text
d9c42d3b26bf6240dc7f6db15759f211ce0f1df446e12b8c666a45cdcdda1240
```

The affine shadow is finite, nonnegative and never exceeds the old enclosure
through 102/106 shadow updates. However, at both final collisions it equals the
old current error bit-for-bit:

| case | old current error | affine-shadow error | refined ratio |
|---|---:|---:|---|
| corner | `9.325e-4` | `9.325e-4` | ambiguous |
| shear | `5.198e-8` | `5.198e-8` | strict |

Therefore A1 is refuted on the frozen traces. The correlation-aware recurrence
is analytically tighter for interpolation, but accumulated interpolation error
is not the live corner cause. Equality at the failure strongly indicates that
the current enclosure was most recently reset from a complete passive solve;
that provenance must be observed before concluding.

The result avoids an unnecessary solver change. No shadow value was consumed,
and no solver/runtime/production authority is granted.

