# NSR3-B4E2D5 pressure-state formulation evidence

Date: `2026-08-22`

Status: `PASS / AUGMENTED_LAGRANGIAN_SELECTED / NO_TRAJECTORY`

## Reproducibility

Two clean Release builds produce byte-identical 4,553,488-byte executables at
SHA `1e7cd1258b155d76cc15d8053cfbdf8cc7045454342ffcece2260f22ef33214a`
and Build ID `b48cbde1ae2a26e3d58238d5dbac139e76fb219d`.

Two fresh processes exit zero, emit empty stderr and byte-identical 1,617-byte
reports at SHA
`f21020e2497befb6b843eff930b1874978346510371405c238c0a27851783ed8`.
The semantic result is
`1e7f856f35c4f684fae49487cd7419325ffee4ae500974e152346c006348cee8`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d5.J73C1M`.

## Closed controls

- `K_eff = 9,810,000 Pa` and the B4E2D4 converged strain maps to
  `11,516.192195951897 Pa` or `1.1739237712489192 m` head.
- A penalty-only local repair needs at least `kappa=1439.524024493987 J`
  and raises the acoustic scale by `1.0834776284025984x`.
- Ten- and hundred-metre head anchors raise that scale to
  `3.1622776601683795x` and `10x` before impact pressure.
- PHR with zero multiplier matches the unilateral quadratic penalty exactly.
- Active derivative and curvature relative errors are
  `1.164319518587524e-12` and `5.560804506864448e-13`; the inactive branch is
  exactly zero.
- `lambda_star=1.4395240244939873 J` maps to the same pressure at zero strain,
  remains unchanged by the multiplier update and has exact complementarity.
- The zero-multiplier mutation produces exactly zero pressure at zero strain.

No particles, neighborhood, trajectory or timing measurement run.

## Decision

Select an explicit augmented-Lagrangian pressure state as the next solver
lineage. It resolves the structural inability of a finite penalty to support
pressure at zero compression while retaining the existing inner variational
machinery. A semismooth primal-dual solve remains a predeclared fallback if
outer multiplier convergence is poor.

Next freeze a tiny dense oracle with explicit multiplier update, primal/dual/
complementarity residuals, warm start, rollback and a zero-pressure inactive
control. Do not touch nominal Dam or persistent runtime state yet.
