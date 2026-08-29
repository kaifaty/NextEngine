# NSR3-B1S3 -- embedded spectral error-controller contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent failure: `SPECTRAL_ONLY_ERROR_POLICY_REJECTED`, B1S2 semantic SHA-256
`ce64874a7f891bae7e4f6387d8d444b556a15c26734e6bbc8b7dfd05f9115081`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Controller

Use the B1S2 spectrum and target `0.15` only to choose initial `n`. Measure
finite-amplitude error by deterministic interval step doubling:

```text
run n and 2n
if gate(n,2n):       accept the n state
else run 4n
if gate(2n,4n):      accept the 2n state
else run 8n
if gate(4n,8n):      accept the 4n state
else                 reject the controller.
```

`gate(coarse,fine)` is exactly the B1S/B1S2 accuracy gate:

- position difference `<=0.05 dx`;
- velocity difference `<=0.001 c`;
- relative kinetic difference `<=0.15`;
- pressure-active exit-time difference no greater than one coarse `dt` with
  only the existing binary64 comparison allowance.

No tolerance is estimated from a fitted error coefficient. Maximum refinement
depth is two doublings beyond the initial pair. Accepted state is the declared
coarse state; the fine state is a comparator and must be counted as discarded
work.

## Parent matrix and new holdout

Reproduce all six B1S2 spectra, counts, `n/2n/4n` phase-state hashes and
observables exactly. Apply the controller to those cases.

Add one previously unexecuted holdout:

```text
spacing = 0.97 dx
kappa  = 1226.25
lambda = physical B1 anchor
mu = gamma = 0
T = 3/240 s.
```

Compute its spectrum and initial `n` only after this contract is frozen. Run at
least three levels so its final two position/velocity self-convergence ratios
can be checked in `[1.5,2.5]`; run `8n` only if required by the controller.

## State and work gates

Every executed level retains all B1S2 finite-state, density, final-active,
COM, momentum, per-step work and pair/tape capacity gates. Accepted-versus-
comparator differences must be finite and nonzero.

Publish separately for every case:

- spectral HVP calls;
- initial, accepted and comparator substeps per frame;
- total executed substeps and nonlinear HVP calls;
- substeps belonging to the accepted state;
- comparator/discarded substeps and HVP calls;
- refinement depth and accepted/comparator phase-state hashes.

Accepted substeps are capped at 192 per frame. Validation may execute at most
`8n<=768` substeps per frame. Spectrum remains 48 HVP calls for active cases.

## Degenerate controls

Inherit the B1S2 `kappa=0` and inactive controls: one substep, zero spectral
HVP, no comparator run, exact one-frame free-flight/translation gates.

## Decision and exit

Two reports must be byte-identical and all prior raw hashes unchanged. PASS
selects only `EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0` for report-only CPU
research and authorizes B1R multi-step selection design.

PASS still grants no B2, boundary, hydrostatic, CUDA or runtime authority. The
controller's discarded work is part of the result; it cannot be omitted from
later performance reasoning.

