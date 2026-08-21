# NSR3-B4DR1C3 -- one-step pressure-cap sweep contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c3-pressure-cap-sweep|v1|parent=cf4e7dced6d2a597ea3ee8267daaab587c4fadb97fe20daa58643ab2412012cc|baseline=step1,cap100,error:0x3fea7a64ac09a4ac,threshold:0x3fb999999999999a|caps=25,50,75,100,125,150,200,300|scenario=CW-HYDRO-001|steps=1|order=ascending-stop-first-converged|profile=all-else-unchanged|output=report-only|credit=diagnostic-only
```

Identity SHA-256:
`926e594fedec03e9c3b08aa76fc57a22988049e97f60c5e47113387aae879678`.

## Authority and invariants

This contract authorizes a report-only research mode and the fixed ascending
sweep. It grants no R1C retry, later scenario, payload, R1D, R1E, B4E, runtime
or production authority.

The implementation must reuse R1C2's manifest, initialization, upstream
patch, build closure and first-step diagnostic capture. It may change exactly
the pressure `MAX_ITERATIONS` setter to one allowed cap and terminate after
the first upstream Hydro step. Pressure minimum/tolerance, divergence profile,
mass, volume, density, kernel, gravity, timestep, boundary and all other state
remain exact.

The CLI is:

```text
--r1c3-pressure-cap <25|50|75|100|125|150|200|300> \
  <absolute-empty-output-directory>
```

Invalid cap and output-path controls reject before Simulation creation. No
mode writes a payload. Every process report includes cap, observed iteration,
pressure/divergence error bits and convergence booleans, and timestep bits.
Convergence is recorded rather than treated as process failure; a finite
non-converged point is a valid sweep result. Any other validator failure exits
nonzero.

## Execution and exit

Run fresh one-thread processes in ascending cap order. Preserve report,
stderr, timing and zero output-entry evidence for every point. Stop after the
first `pressure_converged=true`; do not execute larger caps. If no point
converges by 300, preserve that boundary.

R1C3 PASS means only that the sweep is complete and deterministic enough to
select the next remediation design. It cannot itself modify or pass R1C.
