# Nonlocal NSR3-B2 boundary-formula research -- 2026-08-21

Status: `RESEARCH_COMPLETE / SPLIT_BOUNDARY_FORMULA_RECOMMENDED / REPORT_ONLY`

## Question

Can static wall handling be treated as an inherited part of the selected
Nonlocal objective, or must density support, nonpenetration and reaction be
specified as separate operations before physical trajectories?

## Primary-source result

The selected Nonlocal paper does not define fluid-solid collision as part of
its validated objective. It lists fluid-solid collision handling as future
work. The pinned PeriDyno examples use fixed particles in solver data and a
separate volume-boundary mechanism, which is practical evidence but not a
closed variational wall/reaction formula.

The 2026 semi-analytical boundary paper does define two distinct energy
components for SISPH:

- virtual boundary particles correct truncated bulk/density support;
- virtual contact particles add a polynomial signed-distance potential;
- reduced-order CCD supplies the initial iterate;
- a separate line-search filter and adaptive contact parameters stabilize the
  substitution solver.

That is a different solver/model identity, not a drop-in term for the selected
Nonlocal Newton-Krylov objective. More importantly for the engine, the paper
states that its current framework is one-way and does not model feedback
forces on moving or deformable solids.

Primary sources:

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://peridynamics.com/publications/2026-Liu-NUV.pdf);
- [A Semi-Analytical Energy Model for Particle-Based Fluid Simulation Involving Complex Moving Boundaries](https://peridynamics.com/publications/2026-Liu-SAM.pdf);
- [pinned PeriDyno Nonlocal solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu).

## Recommended B2 identity

Use `split-static-boundary-r0` for the first report-only correctness
candidate:

```text
fixed lattice samples
  -> density support inside the FCR2 pressure energy
  -> analytic fluid gradient/HVP
  -> virtual boundary derivative for support reaction

tentative fine-owned trajectory
  -> exact analytical swept-sphere face/edge/corner contact
  -> corrected fluid state + equal/opposite contact reaction
```

Static samples are immutable parameters, not nonlinear unknowns. They
contribute to density of fluid-centered pressure terms; no pressure energy is
centered on boundary samples. Only fluid samples receive inertia. Water B2
keeps viscosity and surface terms disabled so that wall support/contact is
isolated.

The support reaction is not guessed from a later clamp. It is the virtual
derivative of the support-dependent energy with respect to fixed boundary
positions. At a stationary material solve, that derivative must correspond to
the fluid material impulse and close equal/opposite momentum. Hard-contact
impulse is then reported separately and added only at the reaction boundary.

## Why contact stays outside the B2 Hessian

The retained product geometry already supplies stable feature IDs, openings,
visibility and swept particle-radius contact. That operation is nonsmooth at
feature/activation changes and is not part of the published Nonlocal energy.
Pretending it has inherited gradient/HVP semantics would create false
mathematical evidence.

B2 therefore proves gradient/HVP only for density-support pressure energy.
It proves nonpenetration, feature order and impulse closure with a separate
contact oracle. B3 will be the first test of their ordered composition inside
the fine-owned time controller.

The semi-analytical contact energy remains a legitimate later `boundary-r1`
research candidate if split composition fails. It would require a new formula
identity, contact-parameter derivation, globalized solve and a two-way reaction
derivation; none can inherit B2 success.

## Support-layer consequence

FCR2 uses `H=3dx`, while SPEC-38 names two static lattice layers. For the
selected cubic, `W(H)=W'(H)=W''(H)=0`; therefore samples in a third axis shell
at exactly `H` should contribute no density, gradient or curvature. B2 must
prove two-layer/three-layer correspondence on face, edge and corner controls
rather than conservatively importing the earlier 38,856-sample three-layer
capacity result.

If correspondence passes, the existing 24,704-sample product support count
remains mathematically sufficient. If it fails because geometry or floating
construction places third-shell samples inside support, B2 stops with a
profile/capacity reclosure requirement; it cannot silently raise the SPEC-38
capacity.

## Decision

Freeze a bounded B2 split-boundary discriminator. It must establish support
density, gradient, HVP, virtual support reaction, two-versus-three-layer
correspondence, ghost-only penetration as a negative control, and independent
face/edge/corner hard-contact closure. Only PASS may authorize B3 smoke
trajectory design. No boundary execution, product profile, CUDA, runtime or
production authority follows from the research decision itself.
