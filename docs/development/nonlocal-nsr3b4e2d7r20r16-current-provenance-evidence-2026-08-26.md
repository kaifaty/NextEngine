# NSR3-B4E2D7R20R16 current-provenance evidence

Status: `PASS / DUAL_REFINEMENT_RESOLVES_ALL`.

Implementation `e7194f05` preserves all R13–R15 roots and emits semantic:

```text
e4c5e8d1fa173359010c46c8b44d2921fc462b507657c3a53955c3e04d611f7f
```

Both collisions have direct, bit-corresponding current-solve provenance:

| case | current dimension | current old → refined | candidate refined | dual-refined ordering |
|---|---:|---:|---:|---|
| corner | 66 | `9.325e-4 → 9.720e-25` | `2.242e-23` | strict, bounds about `1.68e-22` |
| shear | 62 | `5.198e-8 → 3.580e-25` | `6.210e-22` | strict, bounds `2.72e-20` / `5.80e-22` |

The current inverse audits certify with `rho` `6.29e-25` and `1.48e-24`.
All central vectors, supports, case roots and solver decisions remain unchanged.

This supports P1 and falsifies P2/P3/P4 on both finite counterexamples. The
root cause is now bounded precisely: a rigorous ratio ordering sometimes needs
verified enclosures for both directly solved directions, while the existing
path refines only passive sign ambiguity and discards current-solve provenance.

The counterfactual still updates no state. A separately frozen candidate replay
is required before the method can be considered repaired.

