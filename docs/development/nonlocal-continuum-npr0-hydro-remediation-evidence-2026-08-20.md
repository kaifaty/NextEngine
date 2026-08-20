# Nonlocal NPR0 hydro-remediation evidence — 2026-08-20

Status: `REPORT_ONLY / H3_SUPPORT_REMEDIATION_CANDIDATE / NPR1_BLOCKED`

## Disposition

The frozen NPR0-R1 matrix selects `H3_SUPPORT_REMEDIATION_CANDIDATE`.
The physical-head coefficient at `h=2dx` misses the unchanged compression
gate at every allowed iteration count. The same coefficient at `h=3dx` first
passes the formal TPH-1 gate at 16 iterations.

This does not select a product profile. The product contract still uses
`h=2dx` and a 32,768 static-boundary capacity. Exact three-layer support for
the full basin requires 38,856 boundary samples, so the candidate needs an
explicit support/capacity redesign and a complete NPR0-E rerun before NPR1.

## Frozen results

Mean positive compression, unchanged limit `0.0001`:

| Candidate | 4 iterations | 8 | 16 | 32 | 50 | First pass |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| algebraic-cadence-h2, kappa=576 | 0.0078648680 | 0.0045580578 | 0.0035213663 | 0.0034258783 | 0.0034250970 | none |
| hydro-head-h2, kappa=9196.875 | 0.0043250212 | 0.0022052205 | 0.0011522615 | 0.0005905445 | 0.0003956033 | none |
| hydro-head-h3, kappa=9196.875 | 0.0029638603 | 0.0004730405 | 0 | 0 | 0 | 16 |

At the selected h3/16 point:

- maximum positive compression is zero;
- maximum speed is 0.0381594637 m/s;
- final centre-of-mass Y is 0.0485953996 m from an initial 0.05 m;
- horizontal COM drift is 6.94e-18 m;
- penetration and fixed-support displacement are zero;
- 8 fluid plus 624 rooted support samples execute, all finite.

The zero positive-compression result proves only the frozen one-sided metric.
It does not prove long-horizon hydrostatic rest or bound negative density
error. Those are deliberately NPR1 concerns and cannot be inferred from this
bounded discriminator.

## Capacity consequence

The exact full-basin h3 complement is:

    (80 + 6) * (20 + 6) * (40 + 6) - 80 * 20 * 40
    = 38,856 static boundary samples

This is 6,088 samples above the current static-boundary capacity. Together
with 48,000 fluid samples it forms 86,856 solver participants. The h3 result
therefore cannot inherit the v3 GPU preflight or the old fixed-work timing.

## Reproducibility

- command: `nonlocal-feasibility --cpu-hydro-remediation`
- bounded result root:
  `f2e8181640049176176535a4f356fdc0422b81ad5a769db49593533cac810b70`
- raw JSON SHA-256:
  `b9854da97276cbd2dea4b42971a20d3733bc55e6c8902e2df5b93bac5e6a1c14`
- diagnostic wall time: 9.26 seconds; report-only and not a gate

## Next admitted step

Freeze one v4 support-profile discriminator with kappa=9196.875,
lambda=360, `h=3dx`, three rooted layers and 16 fixed iterations. It must:

1. declare the larger capacity rather than truncate support;
2. pass profile audit and exact full-basin P1/P2 preflights;
3. rerun all four NPR0-E cases with h3-consistent support;
4. retain the split swept contact and every existing NPR0-E gate;
5. stop without entering NPR1 if any case or capacity check fails.

Semi-analytical or compressed boundary representations remain a later
research alternative. They change the boundary formulation and cannot borrow
the particle-complement result root.
