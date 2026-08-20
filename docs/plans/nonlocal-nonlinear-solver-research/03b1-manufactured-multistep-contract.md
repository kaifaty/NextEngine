# NSR3-B1 -- manufactured multi-step contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / BOUNDARY_EXECUTION_BLOCKED`

Objective identity: `nuv-variational-fcr2`.

Solver identity: `nuv-newton-krylov-r0` with
`outer-state-hessian-tape-v1`.

## Question and isolation boundary

B0R proves derivatives and one minimized state. B1 asks whether repeated
state transitions preserve the invariants and temporal behavior required of a
continuum integrator. It deliberately contains no wall, ghost, contact,
surface-energy or named product trajectory. A failure therefore belongs to
the material objective, nonlinear solve or time integration rather than an
unverified boundary formula.

## Frozen transition

For old position `x_n`, velocity `v_n`, constant acceleration `g` and step
`dt`, construct

```text
y_star = x_n + dt (v_n + dt g)
y      = arg min E(y; x_n, y_star, dt)
x_n+1  = y
v_n+1  = (y - x_n) / dt.
```

The inertia term is the existing
`m |y-y_star|^2 / (2 dt^2)`. Pressure and viscosity retain the FCR2 formulas,
compression-only active set and common normalized cubic. Each accepted
position becomes the next reference position; no state is reset between
steps. Pair support is rebuilt from that reference at every outer state.

Common physical-profile anchors for B1-FF, B1-RT, B1-CR and B1-RO are:

```text
rho0   = 1000 kg/m^3
dx     = 0.05 m
H      = 0.15 m
m      = rho0 dx^3 = 0.125 kg
dt     = 1/240 s
kappa  = 1226.25 J
lambda = 1.413823172873555e-5 kg m/s
mu     = 0
gamma  = 0.
```

`kappa` is the declared one-metre-head / `1e-3` compression anchor. `lambda`
is the independently derived water-like simple-shear-equivalent anchor;
`mu=0` avoids the already diagnosed artificial rigid-rotation dissipation.
These values are manufactured controls, not a selected production profile.

## Frozen cases

### B1-FF -- exact free flight

One particle at `x0=(0.25,-0.1,0.4) m`, 240 steps, initial velocity
`v0=(1.2,0.7,-0.35) m/s`, and `g=(0,-9.81,0)`. All material terms are zero
because no pair exists. At step `n`, require the semi-implicit closed form

```text
v_n = v_0 + n dt g
x_n = x_0 + n dt v_0 + n(n+1) dt^2 g / 2
```

within `1e-11` relative/absolute mixed error. No nonlinear iteration or HVP
is admitted.

### B1-RT -- rigid translation

A centered `4x4x4` reference lattice at spacing `dx` runs 240 steps at
constant velocity `v=(0.37,-0.21,0.13) m/s` with `g=0`, pressure/viscosity
coefficients present and surface disabled. Require maximum offset and velocity
error, and center-of-mass drift relative to the analytic translation, each
`<=1e-11`. Internal momentum and material-work residuals are `<=1e-12`. No
pressure center may become active.

### B1-GC -- Galilean covariance

Reuse the B0R tetrahedron geometry and velocity field, but explicitly set
`kappa=200`, `lambda=20`, `mu=0`, `gamma=0` and set its synthetic rest density
to the first-particle density divided by `1.1`. This exception exists only to
exercise active pressure and viscosity with four particles; it is not a
physical profile.

Run it for 32 steps twice: once from its declared velocity state and once with
uniform velocity `u=(0.6,-0.3,0.2) m/s` added to every sample. After removing
`n dt u`, maximum position and velocity mismatch must be `<=1e-10`.
Active-set, accept/reject, stop-reason and HVP-count sequences must match
exactly. This is covariance, not bit identity of translated floating-point
coordinates.

### B1-CR -- free compression/relaxation

A `7x7x7` lattice starts at spacing `0.99 dx`, zero velocity and no external
force. Run to `T=0.05 s` at `dt`, `dt/2` and `dt/4`; every run uses at most 32
outer states, 8 rejected trials and 128 HVP calls per step. Require:

- normalized center-of-mass drift `<=1e-11`;
- normalized accumulated internal-momentum residual `<=1e-10`;
- maximum density ratio never above the initial value plus `1e-6`;
- final active pressure-center count below the initial count;
- finite objective, state and velocity at every accepted transition.

Let `e_h` be mass-weighted RMS final-position difference between `dt` and
`dt/2`, and `e_h2` the difference between `dt/2` and `dt/4`. Require
`e_h/e_h2 >= 1.5` and `e_h2 > 1e-14`. The gate demonstrates resolved
first-order-or-better convergence; it does not fit a material coefficient.

### B1-RO -- rigid-rotation objectivity discriminator

Kinematically rotate a centered `4x4x4` reference lattice at spacing `dx`
about the z axis at `omega=2 rad/s` for `T=0.25 s`. Evaluate, but do not
minimize, accumulated viscosity energy at `dt`, `dt/2` and `dt/4`.

With the selected physical control `mu=0`, the finite-step normal increment
is second order, so accumulated artificial energy must decrease quadratically:
both halving ratios lie in `[3.5,4.5]`. A declared diagnostic comparator sets
`mu=lambda`; its tangential penalty must expose the known non-objectivity with
both ratios in `[0.8,1.2]`. The comparator is evidence against selecting
positive `mu`, not an admitted material profile.

## Global gates and publication

- Two complete reports are byte-identical and have a new timing-independent
  semantic root.
- Every internal reduction uses the canonical unique-pair/full-coefficient
  convention; mass remains exactly `N*m`.
- Every accepted nonlinear trial has positive actual and predicted reduction,
  and rejected trials do not mutate the state.
- The largest case is 343 particles with at most `80*N` unique pairs. All
  per-step work counters and capacities are published.
- All historical FCR1/NSR and B0R raw hashes remain byte-identical.
- One implementation/transcription defect may be corrected without changing
  this contract. A failed physical discriminator is preserved as evidence and
  stops B1; thresholds and profiles are not tuned after observing it.

## Exit

PASS selects only `NSR_MULTISTEP_CANDIDATE` and authorizes design of the
NSR3-B2 static-boundary formula contract. It grants no hydrostatic,
dam-break, CUDA, runtime, public-schema, save/replay or production authority.
