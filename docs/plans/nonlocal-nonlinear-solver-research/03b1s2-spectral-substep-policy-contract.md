# NSR3-B1S2 -- spectral substep-policy contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent premise: `SPECTRAL_POLICY_PREMISE_VALID`, B1S1 semantic SHA-256
`5836107d2e9b90d3638ed9775bf966e0c0b2c88bcdf7ca90e99cd02b311651c0`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Candidate policy

At the first state of the three-frame B1S relaxation, compute the B1S1
pressure-only `omega_max` with exactly 48 deterministic Lanczos HVP calls and
set

```text
n  = max(1, ceil(Delta omega_max / 0.15))
dt = Delta / n,
```

for `Delta=1/240 s`. Hold `n` fixed through the three-frame manufactured run.
This is `pressure-spectrum-substeps-r0`.

The target `0.15` is frozen from the previous negative evidence: under the
observed first-order regime, the B1S worst velocity error requires at most
`0.1525` and its worst position error at most `0.1606`; `0.15` is the lower
rounded bound. It cannot be changed after B1S2 execution.

## Exact spectrum and counts

Reproduce the B1S1 eigenvalues/residuals before trajectories. The exact
candidate substep counts for spacing `0.99/0.98 dx` and stiffness factors
`0.25/1/4` are:

```text
0.99 dx: 20 / 39 / 78
0.98 dx: 22 / 43 / 86.
```

Every realized `dt*omega_max` must be `<=0.15`; candidate count is capped at
96. Publish the 48-HVP estimate separately from nonlinear-solver work.

## Trajectory matrix and gates

Reuse B1S exactly: the same six initial states, zero velocity/force,
`T=3 Delta`, physical normal viscosity, `mu=gamma=0`. For each candidate `n`,
run `n`, `2n` and `4n` levels. Apply every B1S state, conservation, capacity,
work and accuracy gate unchanged:

- position/velocity ratios in `[1.5,2.5]`;
- policy-versus-`2n` errors `<=0.05 dx` and `<=0.001 c`, where
  `c=sqrt(kappa/m)` remains only the reporting scale;
- relative kinetic error `<=0.15`;
- pressure-exit error no greater than one candidate `dt`;
- finite nonzero adjacent differences, final inactive pressure, no density
  overshoot, normalized COM `<=1e-11`, accumulated momentum `<=1e-10`;
- per step `32/8/128` outer/reject/HVP caps and frozen pair/tape capacities.

Reference levels may exceed the candidate substep cap and have no performance
authority.

## Degenerate controls

For `kappa=0` and for the inactive `1.00 dx` state, detect zero pressure
activity before Lanczos, return `omega_max=0`, `n=1`, and perform zero spectral
HVP calls. Repeat the B1S one-frame analytic free-flight and rigid-translation
gates.

## Repeatability and decision

Two complete reports are byte-identical. B1S1, B1S, B1D1, B1D, B1, B0R and
all frozen FCR1/NSR reports remain byte-identical.

PASS selects only `PRESSURE_SPECTRUM_SUBSTEPS_R0` for report-only CPU research
and authorizes B1R multi-step selection design. Failure rejects the spectral
target/policy and stops substep selection until reclosure.

## Exit boundary

PASS does not authorize B2, hydrostatics, CUDA or runtime integration. The
report must expose base counts `39/43`, high-stiffness counts `78/86`, and the
48-HVP estimate so production cost cannot be inferred from correctness alone.

