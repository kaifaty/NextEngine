# Nonlocal NPR0 tiny-corpus evidence — 2026-08-20

Status: REPORT_ONLY / PROFILE_RECLOSURE_REMEDIATION_1 / NPR1_BLOCKED

## Disposition

Neither predeclared profile passes TPH-1 mean positive compression at the
frozen 1e-4 limit. No product profile is selected and NPR1/runtime work remains
blocked.

The command itself passes because it completed the bounded comparison and
applied the frozen selection rule. Its semantic disposition is
PROFILE_RECLOSURE_REMEDIATION_1.

## Results

| Gate | Control | Derived | Limit |
| --- | ---: | ---: | ---: |
| TPF-1 position error | 1.11e-15 m | 1.11e-15 m | 1e-12 m |
| TPF-1 velocity error | 3.21e-14 m/s | 3.21e-14 m/s | 1e-12 m/s |
| TPH-1 mean positive compression | 0.3325094461 | 0.0078648680 | 0.0001 |
| TPH-1 maximum positive compression | 0.3589346324 | 0.0157297360 | report only |
| TPH-1 horizontal COM drift | 6.94e-18 m | 6.94e-18 m | 1e-10 m |
| TPH-1 maximum penetration | 0 m | 0 m | 0.0025 m |
| TPH-1 maximum speed | 0.6512911 m/s | 0.0393598 m/s | report only |
| TPR-1 return position / spacing | 6.51e-16 | 1.09e-15 | 1e-9 |
| TPR-1 return velocity / initial speed | 1.38e-14 | 3.35e-14 | 1e-9 |
| TPW-1 face/corner features | exact | exact | exact |
| TPW-1 penetration / impulse closure | 0 / 0 | 0 / 0 | 1e-12 / 1e-12 |

The derived profile reduces the hydro compression by about 42 times and
reduces maximum speed substantially, but still misses the immutable gate by
about 79 times. It cannot be promoted on relative improvement.

The bounded result root is
d63188a4117a4cd496abfadfe7555b1e239b7e2edbfd44478decc1c57272797a.

## First failure and likely causes

The first failure for both profiles is TPH-1 mean positive compression. The
other three case classes pass, which localizes remediation to hydrostatic
incompressibility rather than integration, rigid-mode invariance or static
contact.

Three effects are still confounded:

1. the CPU corpus used four SISSM iterations and may be far from the fixed
   point;
2. the product support ratio is h/dx=2 while the paper resolution study uses
   h/dx=3;
3. the first derived coefficient used the discrete cadence ratio t=25/6.
   Exact algebraic similarity also requires gravity to scale as s/t², but the
   product correctly retains physical gravity 9.81 m/s². The coefficient is
   therefore an algebraic bridge hypothesis, not yet a physical scale law.

## Authorized remediation

One bounded diagnostic may:

- sweep fixed iteration counts at h/dx=2 without changing thresholds;
- compare h/dx=3 at the same accepted product state;
- add a constant-gravity/Froude-derived coefficient hypothesis with explicit
  units/dimensionless groups.

It may not fit a coefficient directly to the observed 0.786% result, relax
the 0.01% gate, start long GPU timing or begin NPR1/runtime integration.

