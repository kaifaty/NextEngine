# NSR3-B2 split static-boundary evidence -- 2026-08-21

Status: `PASS / SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE / REPORT_ONLY`

This report executes the frozen
[B2 contract](../plans/nonlocal-nonlinear-solver-research/03b2-split-static-boundary-contract.md)
over the selected `nuv-variational-fcr2` objective. It does not execute a
boundary trajectory and grants no runtime, CUDA, moving-solid or production
authority.

## Result

The split construction passes:

```text
fixed lattice samples -> fluid-centred density and pressure energy
                     -> virtual derivative for support reaction

tentative segment     -> independent analytical swept-sphere contact
                     -> accepted state and contact reaction
```

Boundary samples are immutable parameters rather than optimizer unknowns.
They do not own density/pressure centres, inertia, viscosity or surface
energy. Hard contact is not represented as a pressure energy and does not
inherit the smooth pressure HVP.

The result selects `SPLIT_STATIC_BOUNDARY_FORMULA_CANDIDATE` and authorizes
only B3 smoke-trajectory contract design.

## Kernel and layer result

The independently transcribed normalized cubic gives:

| Quantity | Result |
|---|---:|
| lattice scale | `7.9856680787724716` |
| `W(H)` | `0` |
| `W'(H)` | `-0` |
| `W''(H)` | `0` |

The two-layer candidate and three-layer counterfactual agree exactly after
the frozen `0.99` inward compression:

| Fixture | 2-layer samples/pairs | 3-layer samples/pairs | density / energy / gradient / HVP / reaction error |
|---|---:|---:|---:|
| corner `2x2x2` | `184 / 828` | `196 / 828` | `0 / 0 / 0 / 0 / 0` |
| face slab `5x5x2` | `454 / 4561` | `479 / 4561` | `0 / 0 / 0 / 0 / 0` |

Rest-lattice density error is at most `2.78e-15 rho0`. The third shell changes
enumerated storage but contributes no visible pair: its nearest axial samples
are exactly at `H`, where density, gradient and curvature all vanish. Two
layers therefore remain the selected bounded representation for this exact
kernel/horizon/spacing identity.

This conclusion is not valid after changing `H/dx`, the kernel, support
visibility or the maximum admissible displacement. Any such change must
reopen layer correspondence.

## Smooth derivative and reaction controls

| Control | corner | face slab | Limit |
|---|---:|---:|---:|
| active fluid centres | `8 / 8` | `42 / 50` | `> 0` |
| density ratio range | `1.002362..1.002362` | `0.997663..1.016241` | reported |
| minimum pressure-branch margin | `2.36e-3` | `2.34e-3` | probes stable |
| directional gradient error | `2.75e-10` | `6.19e-10` | `<= 1e-7` |
| analytic HVP vs gradient FD | `7.98e-10` | `3.00e-9` | `<= 2e-6` |
| dense symmetry error | `4.21e-15` | `2.47e-15` | `<= 2e-12` |
| dense product error | `8.53e-16` | `6.76e-16` | `<= 2e-12` |
| translation energy error | `1.17e-13` | `1.68e-14` | `<= 1e-12` |
| reaction closure | `3.12e-17` | `1.11e-16` | `<= 1e-12` |
| fixed boundary displacement | `0` | `0` | bit-zero |

Both controls remain below 512 samples and far below `80*(F+B)` unique
support pairs.

The dense pressure Hessian is symmetric but indefinite:

| Fixture | eigenvalue range |
|---|---:|
| corner | `[-395.195, 38139.536]` |
| face slab | `[-3347.258, 129149.142]` |

This is expected from the finite-compression radial curvature term and is
material evidence: support does not make the pressure objective convex. B3
must retain the selected safeguarded trust-region path and publish negative-
curvature events; an SPD-only boundary solver is not authorized.

## Hard contact and required negative

The independent unit-box sweep passes the frozen cases:

| Case | sorted feature IDs | penetration |
|---|---|---:|
| face | `[2]` | `0 m` |
| edge | `[0,2]` | `0 m` |
| corner | `[0,2,4]` | `0 m` |
| grazing | `[]` | `0 m` |
| moving away | `[]` | `0 m` |

Each active case has finite time of impact and exact equal/opposite impulse
closure. Accepted velocity is reconstructed from the accepted position and
substep start as required by SPEC-38's current static-contact convention.

The ghost-only negative starts at `y=0.075 m` with `vy=-120 m/s`; after one
`1/240 s` segment its tentative centre is `y=-0.425 m`. Density support alone
therefore penetrates the `y=0.025 m` particle boundary. This independently
preserves the earlier conclusion that ghost support cannot be relabelled as
nonpenetration.

## Repeatability and regression

Two complete B2 executions are byte-identical:

```text
raw report SHA-256:
d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9

semantic result SHA-256:
80a01b2ed0cf844841da322233b121b33e273c71eace8682795d0cad4e1dfb80
```

The selected B1R1 control remains byte-identical after B2:

```text
raw:      af34c3d8e142610bb11d26592a8b9f673af6ff941f89c7b8631f178cd70f0af1
semantic: e215b0facc30445541a6f2fa9446fd9d8140bf5f66983180cb435de9863f535e
status:   PASS
```

## Interpretation and next gate

B2 closes formula ownership, derivative correctness, reaction accounting and
the static contact split. It does **not** show that their sequential
composition is stable over time. The smallest next experiment is B3:

1. freeze one tiny gravity-loaded face/corner smoke trajectory;
2. run fine-state-owned adaptive Nonlocal substeps;
3. apply hard contact once at the accepted substep boundary;
4. rebuild support and spectrum after contact;
5. require nonpenetration, bounded support reaction/contact impulse,
   conservation accounting, convergence and an independent fine reference.

Hydrostatic, dam-break, product scale, CUDA and performance work remain
blocked until B3 passes.
