# NSR3-B1R -- transactional multi-step controller contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent selection: `EMBEDDED_SPECTRAL_ERROR_CONTROLLER_R0`, B1S3 semantic
SHA-256 `fc3e0cff38f748f5cfd09803f2e0b2df2dd65069c91ef33938dd01245ab3b5c1`.

Objective/solver identity remains `nuv-variational-fcr2` /
`nuv-newton-krylov-r0 + outer-state-hessian-tape-v1`.

## Research question

B1S3 proves bounded interval error from a common initial state. B1R asks
whether accepting the coarse member of each passing pair and composing those
states over the original `0.05 s` B1 horizon preserves global accuracy and
invariants. It also measures how often spectrum/comparator work recurs.

This is a reclosure under a new integration policy. The original B1 failure
and hashes remain negative evidence and cannot be relabelled.

## Transactional macro-frame algorithm

Use 12 macro frames of `1/240 s`. At the start of each frame, snapshot
position and velocity. No candidate may observe another candidate's state.

For a pressure-active snapshot:

1. estimate the current pressure-tangent `omega_max` with the frozen 48-call,
   fully reorthogonalized Lanczos operator;
2. choose `n=ceil(frame_time*omega_max/0.15)`;
3. run `n` and `2n` from the same snapshot;
4. accept the coarse state if the unchanged B1S3 gate passes, otherwise run
   `4n` from the snapshot and test `2n/4n`, then optionally `8n` and test
   `4n/8n`;
5. commit only the coarse state of the first passing pair. All other states
   and work are discarded.

The gate remains position `<=0.05dx`, velocity `<=0.001c`, relative kinetic
difference `<=0.15`, and pressure-exit-time difference no greater than one
coarse step with the existing binary64 allowance.

An inactive snapshot may take one macro step with zero spectrum/comparator
work only when its uniform-acceleration free-flight prediction is also
pressure-inactive and every supported pair has non-negative relative radial
velocity **and zero relative velocity under the selected viscosity model**
(within the existing binary64 exact/finite comparison). Otherwise it enters
the embedded path with initial `n=1`. This extra condition is required because
pressure inactivity alone does not disable bulk viscosity. The fast path must
end pressure-inactive and retain the exact free-flight solution.

Spectrum is recomputed only at an active macro-frame boundary. It is not
silently reused across accepted state changes.

## Cases and independent reference

Run the normalized `7x7x7` relaxation at base `kappa` from spacings
`0.99dx`, `0.98dx` and `0.97dx`, zero velocity, zero force, and
`T=0.05 s`. The last amplitude was the B1S3 holdout and is now a composition
stress case.

For each case independently run fixed `96`, `192` and `384` substeps per
macro frame from the original initial state. These references do not consume
controller states or counts. Require both final position and velocity ratios

```text
error(96,192) / error(192,384)
```

in `[1.5,2.5]`, with finite nonzero denominator.

Compare the composed controller state with the fixed-384 state. Require:

- mass-weighted RMS position difference `<=0.05dx`;
- mass-weighted RMS velocity difference `<=0.001c`;
- relative kinetic-energy difference `<=0.15`;
- normalized center-of-mass drift `<=1e-11`;
- accumulated internal-momentum residual `<=1e-10`;
- no density above the initial maximum plus `1e-6`;
- final pressure-active count below its initial count.

## Transaction, work and capacity gates

- Rejected/coarser and comparator candidates never mutate the committed state.
- Publish per frame: active/fast-path status, spectrum and operator calls,
  initial/accepted/comparator counts, refinement depth, accepted/comparator
  hashes, nonlinear work and discarded work.
- Publish total accepted, controller-executed and discarded substeps/HVP,
  active spectrum calls, fast-path frames and maximum per-step work.
- Accepted count is at most 192 per frame; the optional comparator is at most
  384; all candidates preserve the existing `32/8/128` per-step
  outer/reject/HVP caps and `80*N` pair capacity.
- Controller-executed speculative work is at most 768 substeps in any frame.
- All computed states and report scalars are finite; accepted trials have
  positive actual/predicted reduction; rejected trials are immutable.

## Degenerate and lineage controls

Compose B1 free flight and rigid translation for the same 12 macro frames.
They must use the inactive fast path, zero spectral/comparator/HVP work, and
match their semi-implicit analytic states under the existing `1e-11` gates.

Two reports must be byte-identical. B1S3 and every older checked raw report
must remain byte-identical.

## Exit

PASS selects `NSR_MULTISTEP_CANDIDATE` and authorizes design only of the
NSR3-B2 static-boundary formula contract. FAIL preserves the first exact
transaction, accuracy, convergence or work failure and keeps B2 blocked.

Neither result grants hydrostatic, dam-break, CUDA, runtime, public-schema,
save/replay or production authority. The measured speculative-work factor is
an input to later performance research, not an optional accounting detail.
