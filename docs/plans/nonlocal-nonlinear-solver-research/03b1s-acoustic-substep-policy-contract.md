# NSR3-B1S -- acoustic substep-policy contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent disposition: `TEMPORAL_STIFFNESS_CONFIRMED`, B1D1 semantic SHA-256
`64592cd38c3371decd14baee6517e85bd0407e9654255e2eaab03fba8fd770c1`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Candidate policy

For frame interval `Delta=1/240 s`, particle spacing `dx`, particle mass `m`
and compression coefficient `kappa`, define

```text
c       = sqrt(kappa / m)
C_frame = Delta c / dx
n       = max(1, ceil(C_frame / 0.25))
dt      = Delta / n.
```

This is `acoustic-courant-substeps-r0` with target `C<=0.25`. The formula is
fixed before the matrix runs; no observed trajectory may change `n`.

The target is deliberately below the B1D1 onset near Courant one and inside
the two-refinement asymptotic region. It is a correctness discriminator, not
yet a performance-feasible production choice.

## Frozen matrix

Reuse the centered `7x7x7` free relaxation with zero velocity/force,
FCR2 kernel, physical `lambda`, `mu=0`, `gamma=0` and terminal time
`T=3 Delta=0.0125 s`.

Cross these initial spacing and stiffness factors:

```text
spacing / dx = {0.99, 0.98}
kappa / 1226.25 = {0.25, 1, 4}.
```

The exact policy substep counts per frame are respectively `17`, `34` and
`67`, independent of compression amplitude. For every one of the six cases
run three predeclared levels: `n`, `2n` and `4n` substeps per frame. The last
two are reference refinements, not candidate costs.

## Required observables and gates

For each level publish the same state, density, active-exit, conservation,
capacity and nonlinear-work observables as B1D1. For adjacent levels publish
mass-weighted final position/velocity differences and the self-convergence
ratios.

Every matrix case must satisfy:

- both position and velocity ratios are in `[1.5,2.5]`;
- policy-versus-`2n` position difference is `<=0.05 dx`;
- policy-versus-`2n` velocity difference is `<=0.001 c`;
- relative policy-versus-`2n` kinetic-energy difference is `<=0.15`;
- pressure-active exit-time difference is at most one policy `dt`;
- all differences are finite and both reference differences are nonzero;
- density never exceeds the case's initial maximum plus `1e-6`, final active
  count is zero, normalized COM drift is `<=1e-11`, and accumulated normalized
  internal momentum residual is `<=1e-10`;
- every step stays within 32 outer trials, 8 rejects, 128 HVP calls,
  `80*N` pairs, 160 neighbors and the frozen tape cap.

The candidate policy count must be `<=96` per frame. Reference levels may use
up to 268 substeps per frame and cannot contribute a performance claim.

## Degenerate controls

With `kappa=0`, the formula must return one substep. Re-run one B1 free-flight
frame and one rigid-translation frame: both must satisfy their analytic
`1e-11` gates with zero pressure work and zero HVP calls. These controls prove
that the acoustic policy does not invent substeps when no acoustic stiffness
exists; they do not cover viscosity-, surface- or collision-driven limits.

## Repeatability and decision

Two reports must be byte-identical. B1D1, B1D, B1, B0R and all frozen FCR1/NSR
reports remain byte-identical.

PASS selects only `ACOUSTIC_SUBSTEP_POLICY_R0` for report-only CPU research and
authorizes a new B1R multi-step selection contract. Failure rejects this target
and requires a policy/timestep reclosure; matrix thresholds and target Courant
cannot be tuned after execution.

## Exit boundary

Even on PASS, B2 static boundaries, hydrostatics, CUDA, runtime, public-schema,
save/replay and production work remain blocked. The report must explicitly
publish base/high-stiffness counts `34/67` so their performance cost cannot be
hidden by the correctness result.

