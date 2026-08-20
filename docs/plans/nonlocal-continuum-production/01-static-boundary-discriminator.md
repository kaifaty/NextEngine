# NPR0-D — static boundary discriminator

Status: SPLIT_STATIC_BOUNDARY_SELECTED / TINY_CORPUS_NEXT / REPORT_ONLY

## Decision

The fixed basin boundary is two separately testable operations:

1. the existing SPEC-38 two-layer REST_VOLUME lattice complement supplies
   missing density/constitutive support;
2. the existing stable-feature swept-sphere operation supplies
   non-penetration and reaction accounting.

Fixed ghost samples are not contact constraints. The Nonlocal solver may
consume them as fixed neighbors, but it cannot call the basin sealed until
the swept contact operation also passes.

This selection is intentionally narrower than importing a new triangle/SDF
boundary-energy model. It reuses the rooted product geometry and keeps the
boundary owner independent of the internal water solver family.

## Source boundary

The pinned Nonlocal implementation pins fixed samples during the local update
and includes them in neighborhood terms. Its public examples still attach a
separate volume-boundary operation; the viscosity example additionally merges
fixed ghost particles. The source therefore supports fixed ghosts as a
source-faithful support experiment, not as proof of sealed contact:

- [pinned solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu);
- [surface-tension example](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/examples/Cuda/UnifiedFluid/GL_SurfaceTension/main.cu);
- [viscosity/ghost example](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/examples/Cuda/UnifiedFluid/Qt_Viscosity/main.cu).

The 2026
[semi-analytical boundary-energy paper](https://peridynamics.com/publications/2026-Liu-SAM.pdf)
is a credible later research direction. It combines virtual boundary support,
a nonlocal contact potential and reduced-order CCD in a SISPH solver. The
paper also identifies the current method as one-way boundary treatment without
feedback forces on moving/deformable solids. It is not the pinned Nonlocal
equation identity and cannot replace the selected product contact or NPR4
reaction derivation by citation alone.

## Frozen static-support profiles

Both profiles use the exact product extent
(-2,0,-1)..(2,1,1) m, fluid lattice 80×15×40, two exterior layers and
24,704 fixed support samples.

| Identity | Coefficients | Purpose |
| --- | --- | --- |
| nuv-basin-48k-static-support-control.v3 | kappa=1, lambda=1.5 | unchanged source-coefficient control |
| nuv-basin-48k-static-support-derived.v3 | kappa=576, lambda=360 | dimensionally derived s=10, t=25/6 hypothesis |

The fluid capacity remains 50,000 and the static support capacity remains
32,768. The actual solver index space is 48,000 + 24,704 = 72,704.
Consequently the current compact CSR cannot encode every participant with a
16-bit global index. The mandatory behavior is an explicit checked 32-bit
fallback; truncation, a hidden reduction of boundary support or charging
support against fluid capacity is forbidden.

The v3 GPU preflight runs density/constitutive support only. Its declared
contact identity is
analytical_swept_sphere_external_not_in_gpu_preflight.

## Tiny negative discriminator

The independent CPU fixture contains one fluid sample and the exact
two-layer complement of a 0.1 m cube: 208 fixed support samples and 209
total solver samples. The fluid starts at 120 m/s toward the bottom face.

The command passes only if:

- every ghost remains fixed;
- the ghost-only Nonlocal solve is finite but penetrates the bottom wall;
- the separately evaluated swept-sphere counterfactual stops at one particle
  radius;
- fluid impulse plus boundary reaction closes exactly.

The expected semantic result is SPLIT_BOUNDARY_REQUIRED. A result in which
ghosts happen to stop this one sample would not prove sealing and would require
a wider velocity/edge/corner discriminator.

## Candidate schedule for the tiny physical corpus

    prior fluid state + rooted static geometry
      -> rebuild fluid/fixed-support neighbors
      -> Nonlocal tentative position solve
      -> derive tentative trajectory/velocity
      -> generate swept-sphere constraints from the same rooted geometry
      -> stable feature/sample active-set projection
      -> accepted position/velocity + per-feature reaction
      -> complete validation or no result

The contact step is a named operator boundary. It does not preserve a claim
that boundary contact is part of the single Nonlocal energy minimization.
There is no post-publication clamp, retry, teleport or delayed reaction.

## Next gate

The control and derived profiles enter the same independent binary64 tiny
corpus: exact free fall, hydrostatic rest, reversible perturbation and
bottom/face/corner wall contact. Metrics and limits are declared before the
run. Only one result may advance:

- one profile becomes NONLOCAL_PRODUCT_PROFILE_CANDIDATE; or
- both fail and NPR0 records a bounded remediation/stop decision.

The moving PhysX crate, triangle/SDF geometry, semi-analytical contact energy,
two-way rigid feedback and runtime schemas remain outside NPR0-D.

