# NSR3-B4DR1C4 trajectory evidence -- 2026-08-21

Status: `FAIL / HYDRO_DAM_PAIRS_PASS / ORIFICE_DOMAIN_EXTENT_REJECTED`

## Result

The cap-300 profile closes pressure convergence for both Hydro and Dam. Each
scenario passes twice with byte-identical reports and complete payloads.
The first Orifice process then fails at contact projection because the adapter
projects `x_max = boundary_nx * spacing = 1.0`, while the frozen Orifice domain
and R1B geometry require `x_max = 2.0`.

The Orifice DFSPH step itself converges: pressure takes 205 iterations at
residual bits `0x3fb9835174ca584f`, divergence converges in one iteration with
zero residual, and timestep bits are exact. No Orifice payload is published
and the second process does not run.

This is an adapter geometry-ownership error. `boundary_nx=20` intentionally
describes the one-metre source-side Akinci support only; it cannot also own the
two-metre analytical domain extent.

## Build identity

| Input | Attested value |
|---|---|
| implementation commit | `534bb4f9d8472a2aa83a98fa90526704f74a8a53` |
| R1C4 identity | `7490aa5390296c5f7fc29a56f7039458ced1969ddee555bfe0ea07f50d49be41` |
| executable | 1,568,680 bytes; `166d28939a8f68fccc053d7ce63dc69a83bef3382e561fbb6f8bbdbc01261eeb` |
| ELF build ID | `d3c96e2f3dc6b308f60aaf979609a51c66c435d1` |
| focused tests | `11/11 PASS` |

R1B and R1C1 manifest reports remain exact. The R1C4 relative-output test
rejects before Simulation creation.

## Passing pairs

| Scenario | Payload bytes | Payload SHA-256 | Report SHA-256 | Max P/V iterations | Contact hits | Wall per process |
|---|---:|---|---|---|---:|---:|
| Hydro | 7,804,828 | `c5246c2b19fb474980e101ff8df0a135027052557c006e3fe0df1df64745cc6b` | `ef47484aa6e1acd693688c9998323cfeb42bc8670611289370d38058bfd8817f` | `220 / 12` | 19,056 | 2.42 / 2.49 s |
| Dam | 7,804,825 | `bba813b30c1fd1e36ccdde2f42c2a965f98dd8217d1f06b7e8476d6c04f15111` | `ea6deba5a32af9cf2c10cb00369cf816db5e6e334056b186a042e0d6d57fe791` | `203 / 9` | 18,537 | 1.89 / 1.89 s |

Both same-scenario report and payload comparisons are byte-exact. They remain
R1C4 evidence only and cannot be copied into a new profile whose manifest
identity changes.

## Orifice negative evidence

The exact 507-byte report root is
`73b880fc4bf7cbd30447808f80d68bd278d9bf34617cb812011311695d923100`:

```text
status=FAIL
reason=orifice contact requires two metre x extent
simulation_created=true
trajectory_started=true
failure_phase=contact_projection
failure_step=1
pressure_iterations=205
pressure_error_bits=0x3fb9835174ca584f
pressure_converged=true
divergence_iterations=1
divergence_error_bits=0x0000000000000000
divergence_converged=true
time_step_bits=0x3f71111111111111
```

Stderr is empty, the external output directory has zero entries, wall time is
0.91 seconds and maximum resident memory is 12,764 KiB.

## Decision

R1C4 fails overall and R1D remains blocked. Reclose the scenario model so
analytical domain extent and source-side boundary lattice extent are separate
frozen fields. Orifice uses domain `x_max=2.0` with `boundary_nx=20`; Hydro and
Dam retain `1.0/20` and `4.0/80`. Give the corrected global profile a new
identity and rerun all pairs because the payload manifest changes.
