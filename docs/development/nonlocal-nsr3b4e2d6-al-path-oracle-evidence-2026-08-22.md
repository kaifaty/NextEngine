# NSR3-B4E2D6 augmented-Lagrangian path-oracle evidence

Date: `2026-08-22`

Status: `PASS / AL_PATH_VIABLE / NO_TRAJECTORY`

## Reproducibility

Two clean Release builds produce byte-identical 4,584,840-byte executables at
SHA `9ec0309381e8387541c5050fa7d3cd07e00f23fe921abd25f07254e84d88710a`
and Build ID `e402653a7de67c21bebea23d2be10a527c8e9dfb`.

Two fresh processes exit zero, emit empty stderr and byte-identical 2,966-byte
reports at SHA
`7deec6fd7aeb596cb358657f2e900e8b150f9a82285dec76ac018e2d642d0cc8`.
The semantic result is
`6b05ed4cf6da4a6007a1e4557ef559b56ab9bffafeb5730841c15bc34853403f`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d6.7fKI3u`.

## Result

The actual B2 kernel path has eight fluid and 176 support samples. The rest
topology contains 708 pairs; the frozen compressed `q=0.01` state contains 828
pairs and all eight centres are active. Density-path derivative controls pass:

- first derivative relative error `7.205689663043338e-09`;
- second derivative relative error `4.215657603565454e-09`;
- topology is unchanged across the derivative stencil.

Cold AL convergence is monotone:

| Outer | `q` | Primal / scaled dual | Stationarity | Complementarity |
|---:|---:|---:|---:|---:|
| 0 | `1.8648882978595793e-05` | `4.355940436795436e-06` | `3.553589063634566e-11` | `2.326713370527541e-08` |
| 1 | `3.557477612048388e-08` | `8.3092530633877e-09` | `3.1670941910699746e-12` | `4.446830707005771e-11` |
| 2 | `6.781192496418953e-11` | `1.5837553490882783e-11` | `2.963551084221583e-11` | `8.475752779008996e-14` |

The cold state root is `e3975fd4...a0d`. Warm start converges in one outer
update at `q=2.9103830456733704e-13`; its maximum multiplier difference from
the cold result is `8.195694123358521e-11`. The inactive control returns
exactly `q=-0.01` and bit-exact zero multipliers.

Resetting multipliers yields positive `q=1.8648882978595793e-05` and positive
compression `4.355940436795436e-06`. Forced rollback preserves the public
state bit-exact, and a one-ULP multiplier mutation changes its bound result
root. No trajectory or timing measurement runs.

## Decision

The selected route is `AL_PATH_VIABLE`: PHR multiplier updates converge over a
real nonlocal density path, and warm start behaves as expected. This does not
establish convergence in non-symmetric particle modes. The next stage must be
a tiny dense-vector oracle with analytic AL gradient/HVP, full trust-region
inner solves, multiplier rollback and independent KKT residuals.
