# NSR3-B4E2D7R19R47 v1 filter-compatibility diagnostic

Date: `2026-08-25`

Status: `INVALID CONTRACT / FILTER_LINEAR_METRIC_REJECTED / NO SCIENTIFIC
CREDIT`.

## Boundary exposed

The first R47 implementation preserved the exact R46 parent and closed every
reported geometry, filter, work, route and rollback control. It stopped at the
linear metric gate before compatibility classification:

```text
direct captured linear psi  6.8540207066231842e-16
source-(source-linear)      differs by about 2e-31
route                       FILTER_LINEAR_METRIC_REJECTED
semantic                    57c865eb9925c1a1a3b604882623f89968c3a95d64a4b8c95e07cd8e07bd5e5b
```

The v1 contract required these two binary64 values to be bit-exact. That is an
invalid arithmetic requirement: `predicted=fl(source-linear)` loses low bits,
so `fl(source-predicted)` is not the inverse operation. The direct linear
metric was computed from the actual linearized constraint vector and remains
the authoritative quantity.

## Reclosure

R47 v2 changes no physical formula, source/trial state, filter rule,
exact-zero compatibility requirement or expected route. It replaces only the
invalid inverse-subtraction equality with the standard two-subtraction
forward bound

```text
abs(reconstructed - direct)
    <= gamma(4) * (abs(source) + abs(predicted) + abs(direct)).
```

The bound is derived from operation count, not fitted to the nominal
difference. Direct `h=sqrt(2*direct_psi)` remains bit-exact. The v1 diagnostic
was an incremental harness run, not a clean evidence run, and receives no
compatibility or production authority.
