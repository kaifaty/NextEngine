# V1 — Serial structural formulation oracle

## Outcome

Select one CPU formulation, integrator, discretization and complete canonical
continuation state for a stiff slender structure. V1 is an isolated Rust
research tool with no tree asset, PhysX coupling, public contract, GPU,
parallelism, fracture or rendering dependency.

## Preconditions

- V0 is complete and hash-closes every profile, curve and threshold.
- The external PyElastica checkout and analytical corpus are independently
  reproducible from recorded revisions.
- Both candidates consume identical geometry/material/load and publish through
  the same fixed-point comparison boundary.

## Required candidates

1. A shearable/extensible geometrically exact Cosserat/Timoshenko rod.
2. A constrained or implicit discrete-rod/corotational beam baseline that
   removes or treats implicitly the stiff axial/shear modes.

The implementation notes the exact equations, strain convention, rotation
parameterization, mass model, damping, boundary treatment, nonlinear solve,
linear solver, ordering and fixed iteration/substep profile. The name of a
method or a library is not sufficient.

## Execution

- serial safe Rust, `f64` inside one fixed outer step;
- stable semantic element/node order; no hash-map or insertion-order reduction;
- checked finite/bounds validation at every publication;
- one ties-to-even canonical conversion per declared field;
- next step decodes only the accepted state; factorization and residual scratch
  are rebuilt;
- stop at first failure and retain the last complete report, not a patched
  trajectory.

Run static/dynamic cantilever, torsion, axial, buckling, taper and Y-junction
cases at the V0 resolution ladder. Include successful analytical controls and
permutations of input record order.

## Selection rule

A candidate is eligible only if every V0 accuracy, stability, conservation,
root and capacity threshold passes. If both pass, select the smaller complete
canonical state and lower measured serial cost; performance is report-only and
cannot rescue a correctness miss. If neither passes, V1 remains failed and no
runtime tree work begins.

The winner freezes:

- exact formulation/integrator and fixed internal cadence;
- element/node state fields and quantization profile;
- iteration/residual/failure semantics;
- resolution rules for V2's exact tree graph;
- external corpus/profile hashes.

## Exit

`VEGETATION-BEAM-REF-P1 = PASS` requires all curves within predeclared bounds,
same-target root identity across repeats/insertion permutations, no hidden
continuation state and a bounded report identifying the selected candidate.
Windows/Linux equality is not yet required for research activation but is
mandatory at V7.
