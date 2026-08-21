# NSR3-B4DR1C1 -- dam manifest-identity reclosure contract

Status: `PASS / R1C_REJECTED / TRAJECTORY_IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c1-manifest-identity-reclosure|v1|parent=a061f43ea3bc60fc3ff3af03aa242c298ad059e094e2215bab71642902f199d0|correction=CW-DAM-001->CW-DAMBREAK-001|manifest=441:4c126bd7af7cac871c72d2ab4bfd47c902c105650005f35cdaefae4a3e10cc05|fluid=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76|boundary=16384:1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d|hydro=unchanged|orifice=unchanged|execution=manifest-only-repeat|trajectory=conditional|credit=none
```

Identity SHA-256:
`865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927`.

## Correction boundary

The parent R1C manifest-only implementation failed before Simulation creation
because its dam scenario block used `CW-DAM-001` while its frozen fluid and
boundary roots were generated with the normative corpus ID
`CW-DAMBREAK-001`. The parent identity is rejected and preserved as negative
evidence.

R1C1 changes exactly:

- dam `scenario_id` from `CW-DAM-001` to `CW-DAMBREAK-001`;
- the dam manifest block length from 436 to 441 bytes;
- the dam scenario-manifest root from
  `45b586513a4b6ade4635c7af4ed4bdc05c9f2e8b2825a81cde6136e38692ea28`
  to
  `4c126bd7af7cac871c72d2ab4bfd47c902c105650005f35cdaefae4a3e10cc05`;
- the report/implementation contract identity from the rejected parent to
  `865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927`.

The dam fluid root remains
`9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76`;
the dam boundary count/root remain `16,384` and
`1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d`.
Hydro, orifice, upstream patch, solver profile, serialization, validators,
mutation gates and all exclusions are inherited unchanged from R1C.

## Corrected exact dam scenario block

The block includes its final LF:

```text
B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-DAMBREAK-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
```

Its root is:

```text
SHA256("nextengine.nonlocal.nsr3b4dr1c-scenario.v1\0" || exact_block_bytes)
= 4c126bd7af7cac871c72d2ab4bfd47c902c105650005f35cdaefae4a3e10cc05
```

## Exit

Implement the correction and run `--r1c-manifest-preflight` twice in fresh
processes. Both reports must be byte-identical and all three scenario/fluid/
boundary roots must pass. Run the forced manifest mismatch separately and
require nonzero exit with `simulation_created=false` and
`trajectory_started=false`.

Only dated R1C1 PASS evidence restores the parent's conditional trajectory
authority. It does not itself execute a solver or authorize R1D, R1E, B4E,
runtime integration, CUDA or production claims.
