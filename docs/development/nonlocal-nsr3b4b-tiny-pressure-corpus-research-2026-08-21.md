# NSR3-B4B tiny pressure-corpus research -- 2026-08-21

Status: `COMPLETE / TWO_PHASE_PRESSURE_CORPUS_SELECTED / EXECUTION_PENDING`

## Question

Which tiny trajectories can distinguish pressure support, free-surface
transport and impact coupling before scalable neighborhoods, external DFSPH
references or long equilibrium horizons exist?

## Why this is not yet a hydrostatic-equilibrium test

An initially uniform lattice under gravity is not the discrete equilibrium of
the selected compressible penalty. It must build a small depth-dependent
compression after execution starts. With no physical viscosity selected, a
short trajectory may also retain acoustic motion; calling its final frame
"hydrostatic" would overstate the evidence.

For the B0 water anchor, the effective continuum bulk modulus is

```text
K = kappa / Vp = kappa*rho0/m = 9.81 MPa.
```

A `0.15 m` water head implies a small-strain continuum estimate near
`rho0*g*H/K = 1.5e-4`. This is a scale check, not an exact particle oracle:
the unilateral density branch and free-surface kernel deficiency make the
discrete profile different near the surface.

B4B therefore tests **supported-column startup**, not settled equilibrium.
Long-window equilibrium remains a later canonical-corpus question.

## Selected independent phases

### Supported column

A `4 x 3 x 4` lattice fills the bottom and horizontal cross-section of a
`4 x 6 x 4` box. It begins exactly on the lower centre-clearance plane. The
bottom layer has complete pressure support while the top is a true free
surface. Gravity must activate compression/support without lateral drift,
penetration or positive energy creation.

### Released block and impact

A boundary-free `3 x 3 x 3` material block begins `10 mm` above the lower
clearance plane in a `0.4 m` cube. It is at exact relative spacing and is far
enough from side/top support that its pre-impact phase has zero pressure. Its
continuous gravity impact estimate is about `45 ms`; a `16/240 s` horizon
contains both free flight and post-impact pressure/contact response.

This fixture falsifies accidental wall support, premature pressure, tunnelling
and loss of the B3R state/ledger repair in one bounded trajectory. The exact
pre-impact semi-implicit recurrence and a separate fixed refinement ladder
provide references without importing particle identity from DFSPH.

## Observable policy

- Exact invariants: count/mass, lateral symmetry, support/contact reactions,
  no penetration and pre-impact rigid free flight.
- Continuum-scale bounds: positive compression below the already derived
  one-metre-head `1e-3` budget and no positive mechanical-energy creation
  above `1%`.
- Numerical accuracy: the existing fine-owned embedded controller versus a
  fixed `48/96/192` ladder, with state and aggregate differences normalized
  by `dx` and `c=sqrt(kappa/m)`.
- Diagnostic only: pressure profile shape, energy deficit, repeated contact,
  work and timings. No visual judgement or best-of-level selection.

The two cases deliberately use no external curve. B4B can reject the pressure
candidate but cannot grant nominal water validity.

## Decision

Freeze and execute
[B4B](../plans/nonlocal-nonlinear-solver-research/03b4b-tiny-pressure-corpus-contract.md)
with `lambda=mu=gamma=0` and the unchanged B3R solver. Preserve the first
failure; do not change a threshold after observing either trajectory.

