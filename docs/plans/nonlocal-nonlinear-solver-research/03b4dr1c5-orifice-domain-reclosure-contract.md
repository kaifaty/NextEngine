# NSR3-B4DR1C5 -- Orifice analytical-domain reclosure contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / NEW_ROOT_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c5-orifice-domain-reclosure|v1|parent=7490aa5390296c5f7fc29a56f7039458ced1969ddee555bfe0ea07f50d49be41|failed_report=507:73b880fc4bf7cbd30447808f80d68bd278d9bf34617cb812011311695d923100|correction=orifice-domain-x-max:1->2;boundary-source-nx:20-unchanged|geometry=hydro:1,dam:4,orifice:2|pressure-max=300|all-else=r1c4|runs=2-fresh-byte-exact-per-scenario|order=hydro,dam,orifice;stop-first-failure|credit=new-root-only
```

Identity SHA-256:
`3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f`.

## Correction boundary

The scenario descriptor gains an explicit analytical domain x extent. Values
are exactly:

| Scenario | `domain_x_max` | `boundary_nx` | Boundary meaning |
|---|---:|---:|---|
| `CW-HYDRO-001` | 1.0 | 20 | full-domain two-layer complement |
| `CW-DAMBREAK-001` | 4.0 | 80 | full-domain two-layer complement |
| `CW-ORIFICE-001` | 2.0 | 20 | source-chamber support only |

Boundary generation, counts and roots do not change. The manifest-only gate
must assert all three pairs before Simulation creation. The contact projection
uses `domain_x_max`; it must never derive domain extent from boundary lattice
width.

R1C5 changes trajectory schema/contract and the prepended payload profile
identity. Pressure cap remains 300. Every other R1C4 solver, contact,
serialization and publication rule is unchanged.

## Execution and exit

Add a distinct `--r1c5-trajectory <scenario> <absolute-empty-directory>` mode
and a no-Simulation negative output test. Run paired Hydro, Dam and Orifice in
that order, stopping at the first failure/mismatch. All six reports and
same-scenario payloads must be byte-identical.

Only complete R1C5 PASS authorizes R1D under the corrected new-root profile.
It grants no R1E, B4E, runtime or production authority.
