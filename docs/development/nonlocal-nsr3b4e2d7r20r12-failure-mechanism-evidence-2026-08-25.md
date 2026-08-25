# NSR3-B4E2D7R20R12 failure-mechanism evidence

Status: `PASS / V3_FAILURE_MECHANISM_IDENTIFIED`.

Implementation `83c693b4` reproduces both R11 failed case roots exactly and
emits semantic result:

```text
99b0b72c964044c6b17e31439fcad4b6001db467d1e8d6aeb27fe972d4647f57
```

Both cases stop at `BOUNDARY_RATIO_UNRESOLVED`:

| case | failing outer step | natural face | ambiguous rows | entered / removed / transitions | direction enclosure |
|---|---:|---:|---:|---:|---:|
| corner | 7 | 67 | 0 | 85 / 18 / 103 | `9.325e-4` |
| shear | 11 | 67 | 0 | 85 / 22 / 107 | `5.198e-8` |

All 103/107 principal systems before the stop are solved. Neither path invokes
the verified inverse. This falsifies natural-face ambiguity, passive-sign
ambiguity, principal factor rejection and the transition cap as the first
failure. It supports the active-set boundary-ratio mechanism on both frozen
counterexamples.

The result does not yet distinguish a nonpositive denominator, an out-of-range
step, or overlapping certified ratio intervals. It therefore identifies the
failing predicate family, not its mathematical cause and not a repair.

No successful case was solved, no algorithm changed, and timing/runtime/
production authority remains false.

