# NSR3-B4BF contact-onset forecast research -- 2026-08-21

Status: `COMPLETE / FEASIBLE_PREDICTOR_SPECTRUM_SELECTED / IMPLEMENTATION_PENDING`

## Question

How can the embedded controller detect contact-created pressure stiffness
before executing a macro frame, without fitting a minimum substep count to the
B4B1 failure?

## Competing policies

### Start-state pressure spectrum

This is the inherited controller. It is correct for already-active smooth
pressure states but sees zero curvature at P1 frame zero. B4B1 proves that a
passing `1/2` pair can then underestimate fixed-reference kinetic error.

Disposition: retain for event-free active frames, reject as the sole contact
onset forecast.

### Two consecutive embedded passes

Requiring `(n,2n)` and `(2n,4n)` to pass is a generic safeguard. It does not
identify the missing physical event and can still accept a non-asymptotic
sequence. It also doubles work on every frame, including detached free flight.

Disposition: diagnostic in the level curve, not selected as the primary
policy.

### Hard minimum or fixed-reference lookup

A minimum such as 21/48 substeps would reproduce this fixture but has no state
derivation. Consulting fixed-192 would make the oracle part of the adaptive
algorithm.

Disposition: rejected.

### Feasible macro-predictor spectrum

Construct the ordinary macro displacement
`Delta*=Hf(v+Hf*g)`, project it to the exact static box and evaluate pressure
activity at that feasible position. If it remains inactive, keep the exact
one-step path. If contact projection creates active pressure, run the same
48-HVP pressure spectrum there and use the unchanged target `0.15`:

```text
n_forecast = ceil(Hf*sqrt(lambda_max/M)/0.15).
```

This is a conservative stiffness forecast, not a committed physical state:
it mutates no trajectory and owns no contact impulse. All 48 HVPs are charged.
For detached P2 the macro predictor neither contacts the wall nor activates
pressure, so its exact one-step path remains available.

Disposition: selected for a bounded discriminator.

## Risks

- A full-frame clamped predictor can overcompress and overestimate work.
- Pressure can activate without endpoint contact during high-speed pass-through;
  general geometry still needs CCD and is out of scope.
- The unconstrained pressure Hessian is only a conservative estimate on the
  KKT free subspace.
- A passing forecast pair must still be checked against fixed-192 in the
  discriminator; no assumption of asymptotic order is made.

## Decision

Freeze and execute
[B4BF](../plans/nonlocal-nonlinear-solver-research/03b4bf-contact-forecast-controller-contract.md).
It publishes the first-frame level curve, tests the derived forecast pair and
retains detached P2 as a zero-work negative. PASS may authorize one separately
frozen B4B2 controller retry only.
