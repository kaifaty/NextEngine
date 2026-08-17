# Package 10T — Dry-sand product and evidence contract

## Outcome

Freeze one calibrated Drucker-Prager sand profile and one prescribed
single-wheel consumer before any MLS-MPM code. This package is the terrain
equivalent of water W0A/W0B and is a hard evidence blocker, not a parameter
TODO.

## Selected scope

The first material is dry cohesionless sand. `Soil`, clay, snow, cohesive mud,
rate-dependent slurry and generic user-authored constitutive laws are not V1
claims. The first consumer is an instrumented prescribed single-wheel rig over
one bounded deformable patch; a complete vehicle is excluded.

## Profile closure required before Package 11T

One immutable `DrySandReferenceProfileV1` must contain exact numeric values
and bounds for:

- MKS/right-handed coordinates, gravity, particle spacing/mass/density and
  background grid spacing/extent;
- fixed timestep, maximum samples/nodes, stencil width and substep horizon;
- APIC/MLS transfer formulas and affine-state representation;
- Young's modulus, Poisson ratio, friction angle, cohesion fixed to dry-sand
  zero unless the reference material requires a named nonzero value, dilation
  policy, hardening and plastic-volume bounds;
- deformation-gradient/affine/plastic fixed-point descriptors;
- fixed eight-sweep 3×3 cyclic-Jacobi SVD, descending singular-value order,
  canonical vector signs and bounded singular-value projection;
- grid/body boundary/contact policy, friction mapping and reaction units;
- canonical P2G/grid/G2P ordering, rounding and checked-overflow behavior;
- every capacity and stable diagnostic code.

The values must be calibrated against one named published/experimental sand
family. Source curves and extraction scripts are hash-identified. If one
profile cannot fit the frozen angle-of-repose, direct-shear and plate-sinkage
families within their predeclared bounds, Package 11T does not start and the
profile reports `TERRAIN_PROFILE_UNCLOSED`.

## Product and reference fixtures

Freeze exact dimensions, loads, speeds, horizons and output sampling for:

1. column collapse and final angle of repose;
2. direct shear box at no fewer than three normal loads;
3. plate sinkage, hold and unloading;
4. one prescribed wheel radius/width/load at fixed slip ratios;
5. repeated wheel passes with berm/displaced-volume measurement.

The wheel fixture declares a kinematic speed/slip program and a dynamic load
application independent of renderer or vehicle controller. Curves include
longitudinal/normal force, sinkage, displaced volume and surface profile. Exact
numeric curve thresholds are written into this profile before code; a visual
track or plausible rut is never a pass condition.

## State and authority classification

Future-affecting material state is sample identity, mass, position, velocity,
affine velocity matrix, deformation gradient and declared plastic/internal
variables. Every field receives an exact fixed-point descriptor and canonical
order in the profile. The MLS-MPM background grid, stencil contributions,
stress scratch, SVD scratch, surface mesh and contact patch visualization are
reconstructed caches.

## Exit and non-goals

Package 10T exits only with a complete hash-bound profile and curve corpus.
Missing constants/thresholds keep it `BLOCKED_PROFILE_UNCLOSED`; an
implementer cannot choose them from a tutorial default.

Non-goals: MLS-MPM implementation, PhysX coupling, persistence, wetness,
drainage, free water, sleep conversion, cross-region transfer and full vehicle.
