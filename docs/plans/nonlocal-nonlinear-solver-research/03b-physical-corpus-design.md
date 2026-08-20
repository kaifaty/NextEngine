# NSR3-B -- physical corpus design boundary

Status: `DESIGN / EXECUTION_BLOCKED_BY_PROFILE_AND_BOUNDARY_CONTRACTS`

Identity: `nuv-newton-krylov-r0`

## Why the old corpus cannot be inherited directly

The DFSPH aggregate reference files and scenario geometry remain valid
independent evidence, but the stopped Nonlocal v4 coefficients, fixed SISSM
iterations, roots and boundary implementation belong to a different formula
and solver identity. NSR2-C2 proves a nonlinear solve on synthetic lattices;
it does not select water coefficients or a boundary model.

## Ordered physical roadmap

1. **NSR3-B0 dimensional/profile derivation.** Execute the
   [frozen eligibility discriminator](03b0-dimensional-profile-contract.md).
   Derive units and nondimensional groups for inertia, compression penalty,
   bulk/shear viscosity and surface energy. The current raw cubic must not be
   hidden by coefficient tuning; a failed normalization requires a new
   formula identity before profiles can be frozen.
2. **NSR3-B1 multi-step manufactured controls.** Execute the
   [frozen boundary-free contract](03b1-manufactured-multistep-contract.md):
   free flight, rigid translation, Galilean covariance, uniform
   compression/relaxation and rotating-material objectivity; exact repeat,
   mass/momentum and step-doubling evidence. The frozen execution fails only
   compression step doubling; preserve the
   [negative evidence](../../development/nonlocal-nsr3b1-multistep-evidence-2026-08-20.md)
   and diagnose temporal stiffness before changing this roadmap.
3. **NSR3-B2 static boundary formula.** Re-derive wall energy/contact and its
   gradient/HVP under the corrected objective; pass pair/edge/corner finite
   differences before trajectories.
4. **NSR3-B3 boundary composition.** Face/corner impact with exact support and
   contact reactions. B3R now passes after displacement-owned inertia and
   bounded residual globalization; it is not a hydro/dam-break smoke.
5. **NSR3-B4A closed-box eligibility.** Prove box-owned support, free-surface
   separation, all six analytical faces and the nominal all-pairs cost
   boundary. B4A passes and selects a joint cell neighborhood before nominal
   execution.
6. **NSR3-B4B tiny pressure-water corpus.** Hydrostatic support and
   release/impact only, with `lambda=mu=gamma=0`, aggregate observables and an
   independent fixed refinement ladder.
7. **NSR4 scalable pressure corpus.** Joint fluid/support neighborhood,
   canonical accepted-step publication and restored attested DFSPH hydro /
   dam-break references precede nominal runs.
8. **Separate B4O/B4V/B4S lineages.** Internal aperture, viscous wall /
   Poiseuille and macroscopic surface calibration have distinct formulas and
   gates; none may donate success to pressure-water.

## Fixed guardrails

- no coefficient may be inherited merely because it passed the stopped v4
  lineage;
- the density kernel and every one of its derivatives must share one explicit
  normalization factor;
- no coefficient is selected from a visual result or best-of-sweep outcome;
- surface and viscosity profiles are separate from water and cannot donate
  success to it;
- canonical publication/root design precedes any long run;
- Windows, moving rigid bodies, PhysX reactions, public schemas, save/replay,
  GPU authority, ML and adaptive split/merge remain out of scope.
