# 05 — MLS-MPM dry deformable terrain

## Outcome

Create a separate dry sand/soil laboratory using APIC/MLS-MPM and prove a
bounded character/vehicle contact-patch result. Do not retrofit DFSPH water
particles with solid fields.

## State and step

Material samples use stable IDs and SoA fields for mass, position, velocity,
affine velocity, deformation gradient and plastic/internal variables. The
background grid is cleared/rebuilt every substep and is never persisted.
Particle-to-grid, grid update/contact and grid-to-particle transfers have fixed
stencil and canonical reduction order.

Start with one Drucker-Prager sand profile: density, elastic parameters,
friction angle, cohesion/dilation policy, hardening and bounded singular-value
projection. Unknown/nonphysical parameter combinations reject before world
creation. MCC and μ(I) remain comparative spikes.

## Contact patch

Expose only an implementation-local patch result keyed by stable body/shape:
normal/shear impulse, sinkage, displaced volume and bounded surface sample.
Raw particles/grid nodes and PhysX types never cross the boundary. PhysX owns
wheel/foot/body integration; the terrain lane owns material deformation.

## Scenario corpus

- column collapse and angle of repose;
- direct shear box under several normal loads;
- plate/foot sinkage and unloading;
- wheel/tire patch at fixed slip ratios and loads;
- repeated passes, berm formation and recovery;
- boundary/capacity/nonfinite/constitutive-projection failures.

## Exit

`CONTINUUM-TERRAIN-P1` requires mass/momentum conservation, repeatability,
bounded energy behavior, curve agreement targets and a stable contact patch.
No wetness, erosion, adaptivity, production persistence or full vehicle model
is part of this package.
