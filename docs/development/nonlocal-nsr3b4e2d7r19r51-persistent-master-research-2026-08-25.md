# NSR3-B4E2D7R19R51 persistent-master research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / ONE-OUTER PERSISTENT MASTER SELECTED`.

## Question

Does R50 stall because Hildreth is ineffective, or because every outer drops
previously discovered halfspaces and solves only the currently positive face?

## Evidence

R50 reduces maximum directed upper by `19.1994x`, proving that its exact Gram
and coordinate updates are effective. At the same time, positive rows migrate
`366 -> 488` while the unique row cache reaches 492/512. The correction remains
deep inside contact/trust geometry. This pattern is consistent with constraint
cycling, not a geometric boundary or lack of local descent.

Hildreth's dual coordinates already support natural release: a coordinate is
clamped at zero when its halfspace is inactive. Dropping the row from the master
is therefore unnecessary. Dykstra's restricted least-squares projection view
likewise motivates retaining accumulated convex constraints while updating
their corrections. Primary sources: [Hildreth](https://doi.org/10.1002/nav.3800040113),
[Dykstra](https://doi.org/10.1080/01621459.1983.10477029).

## Selected discriminator

Freeze one R51 outer from the exact R50 terminal witness:

1. take the stable ascending union of all 492 cached R50 rows and all 488
   terminal directed-positive rows;
2. retain the unchanged 512-row capacity. If the exact union exceeds it, stop
   before adding a partial row;
3. reuse every captured row gradient and Gram column bit-for-bit. Build only
   missing union rows, each with one VJP and one JVP;
4. solve the full union, including currently negative rows, with exactly the
   unchanged eight Hildreth sweeps and fresh zero dual coordinates. Negative
   rows remain in the master but may release through `lambda=0`;
5. project the correction through the unchanged ball-box operator and audit the
   unchanged largest-first dyadic ladder `1..2^-8`;
6. accept only an independent certificate or strict simultaneous decrease in
   `psi`, `h` and maximum directed upper;
7. report how many terminal positive rows lie outside the persistent master.

This is a single-outer discriminator. A second persistent outer would be a new
experiment and is not inferred from the outcome.

Because the parent cache already owns 492 of 512 slots, at most 20 rows can be
new. New operator work is therefore at most 20 row VJPs, 20 Gram JVPs and nine
directed candidate JVPs: 49 pair passes. Parent replay work remains historical
and is not relabelled as new R51 work.

## Classification

- Zero all-row directed positives selects the private next-TRQP compatibility
  candidate.
- Strict progress with new positive rows outside the union selects persistent
  constraint-generation expansion required.
- Strict progress with all positive rows already inside the union selects a
  persistent-master closure candidate.
- Capacity, Gram, projection, globalization and work boundaries remain exact
  separate routes.

Neither Hildreth predicted feasibility nor a closed master proves all-row
compatibility. The R48 directed audit remains authority; no infeasibility claim
is introduced.

## Alternatives not selected

- **Larger cache:** rejected; it would fit capacity to the 492-row observation.
- **Fifth R50 outer:** rejected; it repeats the dropping mechanism under test.
- **Warm-starting multipliers:** postponed. R51 changes the current upper vector,
  so zero duals isolate row persistence without a multiplier-transfer policy.
- **More than eight sweeps:** rejected; the existing exact sweep block is reused.
- **Local sparse row extraction:** required for later performance work, but it
  would mix a data-structure optimization into the mathematical discriminator.

## Recommendation

Freeze and implement one rollback-only persistent-master outer. Do not apply the
witness, exit restoration, update filter/trust state, run a following outer or
admit timing/runtime/production authority.
