# NSR3-B1D -- temporal-stiffness diagnostic contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Parent negative result:
`NSR3B1_COMPRESSION_RELAXATION` with semantic SHA-256
`0625dba917f0105b5629d7a2d5e6c8475e9aaa4a4b6b44b201c88122cd00187f`.

Objective identity: `nuv-variational-fcr2`.

Solver identity: `nuv-newton-krylov-r0` with
`outer-state-hessian-tape-v1` and the selected numerical-floor stop for the
main ladder.

## Question

B1 established stable nonlinear solves but did not establish temporal
convergence. B1D distinguishes two hypotheses without changing B1:

1. its three steps were all above the asymptotic acoustic-resolution regime;
2. the FCR2 state transition does not converge under refinement.

B1D is a diagnostic. A valid bounded answer passes this stage regardless of
which hypothesis survives; it does not select a production time step or
retroactively pass B1.

## Frozen fixture and ladder

Reuse B1-CR exactly: centered `7x7x7` lattice, initial spacing `0.99 dx`, zero
velocity, no external force, no surface energy, `mu=0`, the FCR2 normalized
cubic and the B1 physical anchors. Hold terminal time at `T=0.05 s`.

Use exactly this continuation ladder:

| Level | `dt` | Steps | Acoustic Courant |
|---|---:|---:|---:|
| D0 | `1/960 s` | 48 | `2.063446752402397` |
| D1 | `1/1920 s` | 96 | `1.031723376201199` |
| D2 | `1/3840 s` | 192 | `0.5158616881005993` |
| D3 | `1/7680 s` | 384 | `0.2579308440502997` |
| D4 | `1/15360 s` | 768 | `0.1289654220251498` |

The Courant values use the already derived
`K_eff=9.81 MPa`, `c=99.04544411531506 m/s` and `dx=0.05 m`. D0 must exactly
overlap the frozen B1 fine run, including final phase-state SHA-256
`7f667eb41a86e1c840630ff8d81da5a258a759a6106442c2407f28ac5069028e`
and its work/capacity counters.

## Required observables

For every level publish:

- final phase-state hash, RMS radius about COM, RMS relative speed and kinetic
  energy;
- first time after which the pressure-active count remains zero;
- maximum density, normalized COM drift and accumulated normalized internal
  momentum residual;
- maximum and total outer/reject/HVP work, pair/tape capacity and maximum
  final scaled nonlinear residual.

For adjacent levels define mass-weighted final-position differences `e_x[i]`
and final-velocity differences `e_v[i]`. For every three consecutive levels
publish self-convergence ratios

```text
q_x[i] = e_x[i] / e_x[i+1]
q_v[i] = e_v[i] / e_v[i+1].
```

Also publish adjacent absolute changes in RMS radius, RMS speed, kinetic
energy and pressure-active exit time. No particle permutation or best-match
alignment is admitted; material identity is fixed.

To separate nonlinear stopping error from temporal error, repeat D3 and D4
with scaled-displacement tolerance `1e-10` and the numerical-floor stop
disabled; every other choice remains fixed. Publish normal-versus-strict final
position differences `s_x` and velocity differences `s_v` at both levels.
This sensitivity pair is an oracle check, not a candidate solver change.

## Validity gates

- Every run completes with finite state, positive accepted reductions,
  immutable rejected states and no unrecognized stop.
- Per step: at most 32 outer trials, 8 rejects and 128 HVP calls.
- At most `80*N` unique pairs, 160 neighbors and the frozen linear tape cap.
- Normalized COM drift is `<=1e-11`; accumulated normalized internal momentum
  residual is `<=1e-10`; density never exceeds the B1 initial ratio plus
  `1e-6` and final pressure-active count is zero.
- All adjacent position and velocity differences are finite and nonzero. At
  D3 and D4, `s_x` and `s_v` must each be no more than `0.1` times the
  corresponding D3--D4 temporal difference; apparent convergence at the
  nonlinear stopping floor is invalid.
- Two reports are byte-identical. B0R, B1 and all FCR1/NSR historical reports
  remain byte-identical.

## Frozen decision rule

After the validity gates pass, select exactly one disposition:

- `ASYMPTOTIC_REGIME_OBSERVED` when the final two `q_x` and final two `q_v`
  are each in `[1.5,2.5]`, their corresponding adjacent errors decrease
  strictly, and the last two adjacent changes of RMS radius, RMS speed and
  kinetic energy also decrease strictly;
- `NO_ASYMPTOTIC_REGIME_AT_C0P129` otherwise.

Pressure-active exit time is reported as a nonsmooth event diagnostic but is
not allowed to veto otherwise consistent state convergence.

## Exit

`ASYMPTOTIC_REGIME_OBSERVED` authorizes design of a separate B1S substep
selection contract based on acoustic Courant. It does not authorize B2
directly. `NO_ASYMPTOTIC_REGIME_AT_C0P129` stops coefficient/substep work and
requires a time-integration or material-transition reclosure. Neither result
grants boundary, hydrostatic, CUDA, runtime, public-schema, save/replay or
production authority.
