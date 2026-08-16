# Package 12T — Exclusive wheel/terrain coupling

## Outcome

Prove the prescribed single-wheel consumer with one unambiguous contact owner:
MLS-MPM owns the deformable wheel/terrain pair and emits the sole reaction
batch; PhysX integrates the wheel/body but does not also solve ground contact
for that pair.

## Pair authority

Activation identifies exact wheel shapes/material revisions and the bounded
deformable patch. The matching PhysX ground collision pair is disabled before
the first substep. Failure to resolve the complete exclusion or coupling
mapping rejects activation; running both contacts and subtracting/merging
their results is forbidden.

Each substep freezes the canonical wheel/body projection, executes the MPM
grid contact, reduces stable grid/sample reactions and publishes one private
batch keyed by world/tick/substep/patch. Each body record contains complete
`PhysicsBodyIdV1`, expected revisions/roots, fixed-point linear/angular
impulse and frozen centre of mass. PhysX validates/applies it and integrates
once; material and rigid state commit together or neither commits.

## Contact patch projection

An implementation-local diagnostic projection may contain canonical wheel and
patch identity, normal/shear impulse, sinkage, displaced volume and a bounded
surface sample. It is read-only evidence. Raw MPM samples/grid nodes, PhysX
types and contact solver handles do not cross the boundary.

## Corpus

- frozen slip-ratio/load matrix from Package 10T;
- zero-slip rolling and locked-wheel limits;
- prescribed acceleration/deceleration and repeated passes;
- berm/displaced-volume and unloading/recovery measurements;
- offset centre of mass and angular reaction sign;
- exact pair exclusion verification;
- missing/stale wheel, mapping/revision mismatch, duplicate/conflicting batch,
  capacity, nonfinite, constitutive and PhysX rejection.

For every case, rigid reaction and terrain momentum accounting include external
drive/load work and must meet the frozen 10T thresholds. Contact output and
roots are exact across insertion/reduction permutations.

## Exit

`CONTINUUM-TERRAIN-P1` coupling evidence requires reference curves, exclusive
pair ownership and all-or-nothing composite failure. It proves one prescribed
wheel only and adds no generic vehicle or terrain query API.

## Non-goals

Tire deformation, suspension/vehicle controller, multiple wheels, full
vehicle, wet sand, free water, persistence, sleep and public contracts.
