# NSR3-B2 -- split static-boundary formula contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / TRAJECTORY_EXECUTION_BLOCKED`

Parent selection: `NSR_MULTISTEP_CANDIDATE`, B1R1 semantic SHA-256
`e215b0facc30445541a6f2fa9446fd9d8140bf5f66983180cb435de9863f535e`.

Objective/solver identity is extended only for this report as
`nuv-variational-fcr2 + split-static-boundary-r0`. The nonlinear solver remains
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Ownership split

B2 contains two deliberately separate operations:

1. fixed boundary samples contribute density support to fluid-centered FCR2
   pressure energy and have analytic fluid gradient/HVP plus a virtual
   boundary derivative for reaction accounting;
2. exact analytical swept-sphere contact consumes a tentative fluid segment,
   emits a nonpenetrating state and equal/opposite contact reaction.

Hard contact is not represented as a differentiable Nonlocal energy and does
not inherit the pressure HVP. The 2026 semi-analytical SISPH contact potential
is a separately rooted future candidate and is not implemented in B2.

## Static support formula

Let `F` be dynamic fluid samples and `B` immutable static support samples.
All samples use the B1 mass `m=rho0*dx^3`; only fluid samples own inertia and
only fluid centers own pressure energy:

```text
rho_i(y,q) = m W(0)
           + sum_{j in F, j != i} m W(|y_i-y_j|)
           + sum_{b in visible B} m W(|y_i-q_b|)

s_i = max(rho_i/rho0 - 1, 0)
Phi_support(y;q) = kappa/2 sum_{i in F} s_i^2.
```

`W`, `W'` and `W''` use the exact FCR2 common scale. Boundary positions `q`
are fixed parameters in the solve. No density/pressure center is created for
a boundary sample; no boundary inertia, viscosity or surface term exists.
B2 sets `lambda=mu=gamma=0` to isolate support/contact.

The canonical fluid gradient and fluid-fluid HVP follow the existing FCR2
formula. Fluid-boundary contributions use the same radial derivatives. A
virtual derivative with respect to `q` is evaluated only for reaction and
translation closure; it never makes `q` an optimizer unknown.

## Geometry and support layers

Use exact axis-aligned plane/box integer geometry, particle radius `0.025 m`,
spacing `dx=0.05 m`, horizon `H=0.15 m`, and the side-oriented lattice
complement. Two layers are the candidate. A three-layer complement is a
counterfactual only.

First prove `W(H)=W'(H)=W''(H)=0`. On face, edge and corner fixtures require
two-layer and three-layer density, pressure energy, fluid gradient, fluid HVP
and virtual reaction to agree at relative error `<=2e-12`. Pair/work counts
are reported separately and need not agree because zero shell pairs may be
enumerated.

Near-wall uncompressed fluid density must agree with its corresponding full
infinite-lattice stencil within `1e-12 rho0`. A `0.99` tangential/normal
compression must activate pressure without moving any boundary sample.

## Derivative and reaction oracles

For smooth active-set face and corner fixtures:

- centered directional fluid gradient error `<=1e-7`;
- analytic fluid HVP versus centered gradient difference `<=2e-6`;
- dense Hessian symmetry and analytic-product error `<=2e-12`;
- no support or pressure branch crossing under finite-difference probes;
- simultaneous translation of fluid and virtual boundary coordinates changes
  support energy by at most `1e-12` relative;
- the sum of fluid pressure gradient and virtual boundary gradient has
  normalized norm `<=1e-12`;
- fixed boundary displacement is bit-zero.

The report publishes active fluid centers, visible fluid-boundary pairs,
minimum branch margins, density range, gradient/HVP errors, dense spectrum,
fluid impulse, virtual support reaction and closure residual.

## Hard-contact oracle

Reuse the exact plane/box source independently inside this report. Freeze one
face, one edge and one corner crossing plus grazing and moving-away negatives.
For active contacts require:

- stable sorted feature IDs;
- centre penetration `<=1e-12 m`;
- finite time-of-impact fraction in `[0,1]`;
- velocity reconstructed from accepted position and substep start;
- fluid impulse plus static-boundary reaction norm `<=1e-12 kg m/s`;
- no post-contact clamp, retry or positional repair.

A ghost-support-only high-speed face crossing must still penetrate. This is a
required negative control proving that support cannot be relabelled as
nonpenetration.

## Capacity, repeatability and exit

- At most 512 fluid plus static samples and `80*(F+B)` unique support pairs in
  derivative controls.
- Two complete reports are byte-identical.
- B1R1, B1R and all prior checked raw reports remain byte-identical.
- One derivation/transcription correction is allowed; a second formula
  mismatch stops B2.

PASS selects `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE` and authorizes only B3
smoke-trajectory contract design. It does not authorize a hydrostatic run,
product support profile, moving rigid body, CUDA, runtime schema, save/replay
or production integration. Failure preserves the exact first boundary and
keeps all trajectory work blocked.
