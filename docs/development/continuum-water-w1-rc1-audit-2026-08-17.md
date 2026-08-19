# Continuum water W1-RC1 independent hydro audit — 2026-08-17

Status: `REPORT_ONLY / EXACT_MATCH / W0B_REOPEN_REQUIRED`.

## Question

The first W1 nominal interacting scenario failed before step 1. RC1 asks one
bounded question: does `CW-HYDRO-001` fail because the production oracle
implements W0B incorrectly, or does a separate implementation reproduce the
same frozen-profile result?

## Method

Commit `d54e10e55bb20528c4cf485bc3bc6be5a0a6b5c3` adds
`xtask continuum water audit-hydro`. Its independent path:

- generates the exact `20 × 15 × 20` fluid lattice without using the scenario
  generator;
- generates all `2,402` outer-shell samples without using boundary code;
- finds neighbors by complete ascending O(N²) scans rather than the production
  integer grid;
- independently implements the cubic kernel, boundary volumes, density,
  factor, pressure acceleration, matrix action, Jacobi update and exact ppb
  conversion;
- compares all fluid inputs, all boundary positions and volume bits, the
  20-value global residual curve, and full iteration traces for corner, edge,
  face and interior rows.

The independent numeric path does not call production grid, kernel, boundary
or solver helpers. Shared code is limited to report types/formatting and final
comparison.

## Clean-tree provenance

- Tool commit:
  `d54e10e55bb20528c4cf485bc3bc6be5a0a6b5c3`.
- Tool tree state: `CLEAN`.
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM `22.1.6`.
- Target/profile: `x86_64-unknown-linux-gnu`, `water-oracle`.
- Flags: exact W0B `target-cpu`, negative target-feature set and
  `-Cllvm-args=-fp-contract=off`.
- W0B document, float and corpus roots remain
  `d357bca64983fbd2074961a462743a4fb5d3fedc2af09631d32ec178bd299550`,
  `d6152c575fd88bb53d0d63d0e1e8b2e86a82465268fc8d102ebdb1d77c092d63`
  and `cb091e0f3a04f2c052b614aeab72034bddd3c0b32430fba85854ede2d183aa91`.
- Fluid-input root:
  `bd18fe6e6a305ccc875ba12014e90f0cbca74a05068c7bd573ef95d3f3f51dbc`.
- Boundary-position/volume root:
  `301246ba4e6efdf1b2f13e60605a2394aab46b22826f2083f67c6cce1d8df0a7`.
- Raw report SHA-256:
  `c69766b17b490c982c2f8da32a380b6bae4df0be4e0347cb78d9c341130cb951`.

Exact command shape:

```text
CARGO_ENCODED_RUSTFLAGS=<exact W0B flags> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water audit-hydro \
  --output <absolute-path-outside-Git>
```

## Result

The audit projection is `EXACT_MATCH`; `first_mismatch` is absent. Production
and independent traces both terminate with
`WATER_DENSITY_NONCONVERGENCE` at `74,482,699 ppb`.

The complete global density residual curve is:

```text
109612910, 107194012, 104863276, 102407068, 100016772,
 97784848,  95690870,  93711663,  91825658,  90016351,
 88271285,  86581229,  84939533,  83340641,  81780557,
 80257825,  78769447,  77311489,  75882952,  74482699
```

| Role | SampleId | Fluid neighbors | Boundary neighbors | Initial `rho_ratio` bits | Iteration-20 error bits |
| --- | ---: | ---: | ---: | --- | --- |
| corner `x0/y0/z0` | 0 | 10 | 16 | `0x3ffcc7c4fd8b7bfb` | `0x3f97536e54125fc0` |
| edge `x0/y0` | 200 | 15 | 16 | `0x3ffbbf8aca02b916` | `0x3fb3179eaf60af90` |
| face `x0` | 3000 | 22 | 12 | `0x3ff8b6f79bf3d20f` | `0x3fb9af6e51138620` |
| interior | 3010 | 32 | 0 | `0x3fefffc641d85e33` | `0x3fb218498abb5f00` |

The interior starts close to rest density while boundary-adjacent rows start
substantially compressed. Pressure propagation reaches the interior by
iteration 20, but the global residual remains 744 times the accepted
threshold.

## Conclusion and limits

RC1 finds no discrepancy in the tested production input generation, boundary
volumes, neighborhood membership/order or density Jacobi implementation. The
leading W1 implementation-mismatch hypothesis is rejected for this audit
projection. The frozen W0B combination of hydro initialization, boundary
quadrature and convergence policy must be recalibrated before W1 can resume.

This is not an external scientific validation: both implementations transcribe
the same W0B equations, only Linux was run, and four complete row traces stand
in for all 6,000 row traces. Full-corpus and SPlisHSPlasH comparison remain
unrun. `CONTINUUM-WATER-REF-P1` therefore stays `NOT_RUN`.

The next bounded package is
[W0C hydro calibration reclosure](../plans/continuum-water/00c-hydro-calibration-reclosure.md).
The original W0B document and roots remain immutable evidence of the rejected
profile; W0C must select a revised profile before any roots or W1 constants
change.
