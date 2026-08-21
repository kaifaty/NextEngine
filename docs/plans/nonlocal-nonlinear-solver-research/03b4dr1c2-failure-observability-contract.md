# NSR3-B4DR1C2 -- pressure-failure observability contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c2-failure-observability|v1|parent=865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927|failed_implementation=235e826afbb2dcbbcf49b29dc2a56d0599c285ca|failed_report=225:1d3f7c4ed3bd582b657c4180947506d31fe3982fe6427f7a445b3ecd013bc67e|scope=step,phase,pressure-iterations,pressure-error-bits,pressure-converged,divergence-iterations,divergence-error-bits,divergence-converged,dt-bits|physics=unchanged|runs=hydro-one|next-scenario=forbidden|credit=diagnostic-only
```

Identity SHA-256:
`cf4e7dced6d2a597ea3ee8267daaab587c4fadb97fe20daa58643ab2412012cc`.

## Authority

The first R1C Hydro process failed with report root
`1d3f7c4ed3bd582b657c4180947506d31fe3982fe6427f7a445b3ecd013bc67e`.
This contract authorizes only adapter failure-observability changes and one
new Hydro diagnostic process. It authorizes no second scenario, retry under
the old identity, solver/profile change, R1D, R1E, B4E, runtime or production
work.

## Required implementation

The new diagnostic mode must execute the same manifest preflight, output-path
validation, Simulation construction, solver setup, boundary setup, upstream
step and post-step contact path as implementation commit
`235e826afbb2dcbbcf49b29dc2a56d0599c285ca`. The only semantic changes are:

1. populate a failure context immediately after every upstream step;
2. update its phase before convergence, contact, serialization and publication;
3. on failure, append canonical fields when solver diagnostics exist:

```text
failure_phase=<token>
failure_step=<u32>
pressure_iterations=<u32>
pressure_error_bits=0x<16 lowercase hex>
pressure_converged=<true|false>
divergence_iterations=<u32>
divergence_error_bits=0x<16 lowercase hex>
divergence_converged=<true|false>
time_step_bits=0x<16 lowercase hex>
```

Raw bits are authoritative; no locale-sensitive decimal is added. Failures
before the first upstream step retain the old compact lifecycle report. The
new report schema and contract identity must distinguish diagnostic output
from R1C output.

No upstream file, formula, build flag, parameter, iteration cap, tolerance,
warm-start state, sample, boundary, contact calculation, successful payload
byte or publication rule may change.

## Execution and exit

Rebuild and pass the existing focused tests plus a no-Simulation negative
diagnostic CLI test. Then run only `CW-HYDRO-001` once in a fresh process and
fresh external output directory.

Preserve its report, stderr, output-entry count and timing. A failure with the
required fields closes R1C2 and authorizes a new research/design decision
based on the observed values. A pass is an unexpected diagnostic discrepancy
and must be investigated without granting R1C credit. Dam, Orifice and R1D
remain blocked in either case.
