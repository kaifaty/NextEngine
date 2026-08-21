# NSR3-B4DR1C2 failure-observability evidence -- 2026-08-21

Status: `PASS / PRESSURE_CAP_CONFIRMED / R1C3_DESIGN_AUTHORIZED`

## Result

The one authorized diagnostic Hydro process reproduces the parent failure and
closes the missing observability. Pressure fails on step 1 after exactly 100
iterations. Divergence has already converged in one iteration with a zero
residual, and the time-step bits remain exact. No contact, payload publication
or later scenario receives credit.

The pressure residual is finite and equals `0.82744058228594985`. The upstream
stopping formula for the frozen `0.01%` setting and density `1000` permits
`0.1`, so the residual is `8.2744058228594977x` the threshold. This rules out
NaN, changed timestep, divergence failure and a post-contact rejection. It
does not yet prove whether pressure will cross the threshold shortly above
100 iterations or stagnate.

## Build and process identity

| Input | Attested value |
|---|---|
| implementation commit | `52e00ddbd98694bc22f3514ceb3047cfe9554a5a` |
| R1C2 identity | `cf4e7dced6d2a597ea3ee8267daaab587c4fadb97fe20daa58643ab2412012cc` |
| executable | 1,563,896 bytes; `3baecbb8f9e36c58de4226eac304becc8ce1f71e8630ac6949c2ff700043fad4` |
| ELF build ID | `234b62f1847a86968d5c06dca65e67b808a4d2f3` |
| report | 497 bytes; `e804d864eb8d347aecdc3c43e4c00277219139edd25def86cdf460e480a495a7` |
| stderr | 0 bytes; empty SHA-256 `e3b0c442...b855` |
| focused tests | `8/8 PASS` |

R1B and R1C1 manifest reports remain byte-identical to their frozen roots.
The diagnostic relative-output test exits before Simulation creation. The
upstream source/diff/static-library closure is unchanged from R1C.

The process exited `1` after 0.47 seconds wall time and used 12,408 KiB maximum
resident memory. Its external output directory had zero entries.

## Canonical diagnostic

```text
failure_phase=solver_validation
failure_step=1
pressure_iterations=100
pressure_error_bits=0x3fea7a64ac09a4ac
pressure_converged=false
divergence_iterations=1
divergence_error_bits=0x0000000000000000
divergence_converged=true
time_step_bits=0x3f71111111111111
```

## Decision

R1C2 passes as an observability gate only. R1C remains failed and R1D remains
blocked. Freeze a one-step, report-only pressure-cap sweep with all other
profile bytes unchanged. Run fixed caps in ascending order and stop at the
first converged result. Do not yet change mass/volume calibration, tolerance,
warm-start policy, timestep, boundary support or production roadmap.

