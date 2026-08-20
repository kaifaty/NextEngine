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

1. **NSR3-B0 dimensional/profile derivation.** Derive units and nondimensional
   groups for inertia, compression penalty, bulk/shear viscosity and surface
   energy. Freeze water, viscous and surface controls without visual tuning.
2. **NSR3-B1 multi-step manufactured controls.** Free flight, rigid
   translation, uniform compression/relaxation and rotating material controls;
   exact repeat, mass/momentum and step-doubling evidence.
3. **NSR3-B2 static boundary formula.** Re-derive wall energy/contact and its
   gradient/HVP under the corrected objective; pass pair/edge/corner finite
   differences before trajectories.
4. **NSR3-B3 smoke trajectories.** Small hydro, still tank, dam-break and
   sealed/orifice analogues; exact repeat and capacity before nominal runs.
5. **NSR4 nominal physical corpus.** Reuse the attested DFSPH hydro, dam-break
   and orifice aggregate curves plus analytical free-flight and Poiseuille
   controls. Compare aggregate observables, never particle identity.

## Fixed guardrails

- no coefficient may be inherited merely because it passed the stopped v4
  lineage;
- no coefficient is selected from a visual result or best-of-sweep outcome;
- surface and viscosity profiles are separate from water and cannot donate
  success to it;
- canonical publication/root design precedes any long run;
- Windows, moving rigid bodies, PhysX reactions, public schemas, save/replay,
  GPU authority, ML and adaptive split/merge remain out of scope.

