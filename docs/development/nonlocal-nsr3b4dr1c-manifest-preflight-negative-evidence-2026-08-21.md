# NSR3-B4DR1C manifest-preflight negative evidence -- 2026-08-21

Status: `FAIL_CLOSED / CONTRACT_IDENTITY_REJECTED / R1C1_RECLOSURE_REQUIRED`

## Result

The first implementation of the committed R1C manifest-only gate stopped
before creating a SPlisHSPlasH Simulation, model, boundary or time step. Hydro
scenario, fluid and boundary identities passed. The dam fluid projection then
failed exactly at:

```text
reason=CW-DAM-001:FLUID_ROOT
simulation_created=false
trajectory_started=false
```

CTest was `5/6`; the R1B positive/negative tests and the forced R1C manifest-
mismatch rejection passed. The R1C positive manifest test failed. No solver
step, output payload or physical trajectory exists.

The candidate executable SHA-256 was
`131757cd41b934a03f0eaee569631b6986dbe768a2cfb71e5155ddb245807b70`.
Its 238-byte LF report SHA-256 was
`adf6faf668448cdfccb785a8fe9bd9fef5eefd4b3f7b0173b99d91d6363bca20`.
This binary came from the uncommitted manifest implementation and receives no
implementation identity or trajectory authority.

## Root cause

The R1C contract wrote the shortened scenario ID `CW-DAM-001`, but retained
fluid and boundary roots computed with the normative historical ID
`CW-DAMBREAK-001` from the continuum-water corpus contracts. Independent
generation proves:

| Projection | Shortened ID | Normative ID |
|---|---|---|
| fluid root | `c7f446268f4141db8547b86da44b85fb180727cb6e8211145df42d05c139d53e` | `9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76` |
| boundary root | `3d36cd25d5a83134fd37de8c2c2f2ceafb464df87b0c5ea3038f2e8e2c747a9c` | `1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d` |

The failure is therefore a contract-internal identity inconsistency, not a
fluid formula, boundary construction or DFSPH convergence failure.

## Decision

Reject R1C identity
`a061f43ea3bc60fc3ff3af03aa242c298ad059e094e2215bab71642902f199d0`.
Do not edit it into an apparent PASS. Freeze a child R1C1 reclosure which
changes only the dam scenario ID and derived scenario-manifest root. Rebuild
and repeat the manifest-only gate twice before considering trajectory mode.
