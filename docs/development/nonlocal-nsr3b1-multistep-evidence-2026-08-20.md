# Nonlocal NSR3-B1 multi-step evidence -- 2026-08-20

Status: `FAIL / TEMPORAL_CONVERGENCE_NOT_DEMONSTRATED / REPORT_ONLY`

## Outcome

The normalized FCR2 objective and A2 Newton--Krylov path pass four of the five
frozen boundary-free controls. Free flight, rigid translation, Galilean
covariance and the viscosity-objectivity discriminator behave as predicted.
The free compression/relaxation case is stable and conservative but fails its
predeclared step-doubling gate, so B1 does not select a multi-step candidate
and NSR3-B2 boundary work remains blocked.

## Passed controls

| Control | Key result | Work | Result |
|---|---:|---:|---|
| B1-FF | position `5.45e-14`, velocity `1.66e-13` | 240 steps, 0 outer, 0 HVP | PASS |
| B1-RT | position `4.28e-15`, velocity `9.88e-15`, COM `1.03e-15` | 240 steps, 0 outer, 0 HVP | PASS |
| B1-GC | position `2.90e-16`, velocity `1.01e-14`, signatures exact | each run 247 outer, 2 reject, 772 HVP total | PASS |
| B1-RO | selected halving `3.99997 / 3.99999`; positive-`mu` comparator `0.999990 / 0.999998` | 60/120/240 imposed intervals | PASS |

The Galilean pair has identical per-step active-set, accept/reject,
stop-reason and HVP signatures. Every accepted trial has positive actual and
predicted reduction, and coordinate hashes prove that rejected trials do not
mutate their current state.

The rotation result is also a model discriminator: `mu=0` removes the
first-order tangential penalty and leaves a quadratically vanishing finite-step
normal error. Setting `mu=lambda` produces resolution-independent artificial
rigid-rotation dissipation, as predicted by B0.

## Failed compression/relaxation control

All three `7x7x7` runs complete successfully and remain inside their frozen
work/capacity bounds:

| `dt` | Steps | Max outer / reject / HVP per step | Final active | Maximum density ratio |
|---:|---:|---:|---:|---:|
| `1/240` | 12 | `10 / 0 / 28` | 0 | `1.0308294231753403` |
| `1/480` | 24 | `7 / 0 / 19` | 0 | `1.0308294231753403` |
| `1/960` | 48 | `10 / 0 / 22` | 0 | `1.0308294231753403` |

Normalized COM drift is at most `7.55e-16`; accumulated normalized internal
momentum residual is at most `2.32e-15`. Pressure activity falls from 81
centers to zero, and density never overshoots its initial value. The failure
is specifically temporal correspondence:

```text
e(dt, dt/2)     = 0.006031468864392272 m
e(dt/2, dt/4)   = 0.006803611148522976 m
ratio           = 0.8865099331406775
required ratio >= 1.5
```

## Temporal-stiffness diagnosis

The B0 coefficient mapping gives

```text
K_eff = kappa rho0 / m = 9.81 MPa
c      = sqrt(K_eff/rho0) = 99.04544411531506 m/s.
```

At `dx=0.05 m`, the three tested acoustic Courant numbers are therefore
`8.2538`, `4.1269` and `2.0634`. Even the finest B1 step is above one. The
observed non-monotone step differences are consistent with all three runs
remaining outside the asymptotic time-resolution regime of the pressure-wave
transient. This is an engineering inference from the nondimensional scale and
the isolated failure, not yet proof of its cause.

The failure is not reclassified as a solver success: implicit stability and
nonlinear convergence do not establish trajectory accuracy. B1 thresholds and
profiles remain frozen. A separate B1D acoustic-Courant ladder must determine
whether convergence appears below a reproducible nondimensional threshold or
whether the state transition/formula itself is inconsistent.

## Repeatability and lineage

- B1 semantic result SHA-256:
  `0625dba917f0105b5629d7a2d5e6c8475e9aaa4a4b6b44b201c88122cd00187f`;
- two byte-identical raw B1 reports:
  `c0f4a8362028323ea3281851c5a06a6cc96eb282e4bcec6e9460efa5920d917b`;
- B0R remains byte-identical at
  `177fd9f5a53c594921a9c7751403c7ce6182045d6c828cead3da1e4390b0946b`;
- all eight frozen FCR1/NSR raw hashes remain unchanged.

## Decision

Preserve `NSR3B1_COMPRESSION_RELAXATION` as a negative result. Do not tune the
step-doubling threshold, coefficient anchor or original three time steps.
Design a separate bounded B1D temporal-stiffness diagnostic. Static boundaries,
hydrostatics, CUDA and production claims remain blocked.

