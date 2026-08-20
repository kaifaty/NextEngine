# Nonlocal NPR0 boundary research — 2026-08-20

Status: REPORT_ONLY / SPLIT_BOUNDARY_SELECTED

## Finding

The current Nonlocal source does not provide one source-backed sealed basin
operation. Fixed samples participate in the solver and are pinned, while the
examples use a separate volume boundary. Therefore the production question
cannot be reduced to “add ghost particles.”

The repository already has a stronger product geometry contract than the
upstream example: an exact integer outer box, two-layer REST_VOLUME support,
stable feature IDs and swept particle-radius contact. Retaining that owner
boundary is the smallest architecture that lets Nonlocal compete as the
internal water solver without reopening geometry authority.

## Primary-source comparison

| Candidate | What the source establishes | Gap for this engine |
| --- | --- | --- |
| Fixed ghosts in pinned Nonlocal/PeriDyno | fixed samples can participate in density and pair terms; the viscosity example uses them | no general non-penetration guarantee; examples still use a separate volume boundary |
| PeriDyno VolumeBoundary | practical external containment in examples | not the unified energy, exact product geometry, reaction or canonical operation |
| 2026 semi-analytical boundary energy | virtual support plus polynomial contact potential and reduced-order CCD are GPU-oriented and robust in reported moving-boundary scenes | separate SISPH formulation; current paper is one-way and explicitly leaves feedback forces open |
| Existing SPEC-38 boundary | rooted support, visibility, swept contact, feature order, capacity and reaction semantics already have independent evidence | must be connected to a Nonlocal tentative trajectory and revalidated physically |

Sources:

- [Nonlocal paper](https://peridynamics.com/publications/2026-Liu-NUV.pdf);
- [pinned Nonlocal code](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu);
- [2026 semi-analytical boundary-energy paper](https://peridynamics.com/publications/2026-Liu-SAM.pdf);
- [PeriDyno publication/code index](https://peridynamics.com/publications.html).

## Resource consequence exposed by the product boundary

The exact support count is 24,704; with 48,000 fluid samples the solver
neighbor index domain contains 72,704 participants. The retained P2
optimization uses 16-bit global neighbor IDs only through 65,535, so the
product boundary forces its checked 32-bit fallback.

This is not a failure of the boundary profile, but it invalidates any claim
that the old compact-P2 memory result transfers unchanged. A later measured
optimization may use typed split index spaces such as (fluid, u16) and
(static-boundary, u16) without changing logical pair order. It is not
authorized until the physical profile is selected.

## Recommendation

Use split static support/contact for the first production candidate. Evaluate
semi-analytical contact energy later only as a separately rooted improvement,
especially if operator-split wall behavior fails the physical corpus.

This recommendation keeps the high-value part of Nonlocal—the coupled internal
fluid terms—while avoiding an unsupported claim that its current upstream
boundary is already production complete.

