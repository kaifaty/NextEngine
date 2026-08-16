# Package 11T — Serial MLS-MPM dry-sand reference

## Outcome

Implement a serial safe-Rust APIC/MLS-MPM reference for the exact 10T
Drucker-Prager profile and prove the reference corpus before parallelism,
PhysX contact or wet material.

## Material state and step

Use material-specific SoA keyed by stable `SampleId` for mass, canonical
position/velocity, affine velocity matrix, deformation gradient and exact
plastic/internal fields from the profile. Private `f64` values exist within
one substep; every future-affecting field is converted once to its declared
fixed-point descriptor at publication, and the next substep starts from it.

Each fixed substep executes:

1. sort samples by stable ID and build fixed-stencil contributions;
2. compute P2G contributions into private fragments ordered by
   `(grid node key, SampleId, contribution field)`;
3. merge checked mass/momentum/stress contributions in that exact order;
4. apply gravity and analytical grid boundary/contact update;
5. execute G2P/APIC transfer in stable stencil order;
6. update deformation gradient;
7. perform the fixed eight-sweep cyclic-Jacobi SVD and canonicalize singular
   value/vector order and signs;
8. apply the exact Drucker-Prager/plastic projection;
9. quantize and validate the complete sample generation;
10. discard the background grid and all solver scratch.

Unordered atomics, hash-map grid reduction, backend SVD defaults, adaptive
grid, particle reseeding and float state across the boundary are forbidden.

## Evidence

- exact mass and bounded external-force-aware momentum/energy balance;
- identical same-target roots on repeat and sample insertion permutations;
- column collapse/angle-of-repose curve and final geometry thresholds;
- direct-shear stress/displacement curves at all frozen loads;
- plate load/sinkage/unloading curves;
- fixed-point/SVD sign/order golden tests and near-degenerate spectra;
- `N-1/N/N+1` sample, grid-node, stencil, iteration and report capacities;
- nonfinite, invalid determinant, constitutive projection and boundary faults;
- performance at the selected patch size is report-only.

The serial tool stops at the first fault and emits the last accepted root plus
first stable diagnostic. It never clamps an undeclared constitutive failure or
continues an invalid trajectory.

## Exit

The reference part of `CONTINUUM-TERRAIN-P1` passes every 10T curve and exact
same-target requirement. A failure returns to profile/equation diagnosis; it
cannot be hidden by parallel, PhysX or wet-material work.

## Non-goals

Parallel performance, PhysX wheel contact, persistent patch, saturation,
water flux, public schemas, full vehicle and generic soil.
