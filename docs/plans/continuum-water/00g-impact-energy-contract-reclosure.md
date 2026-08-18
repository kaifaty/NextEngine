# W0G — Impact energy-contract reclosure

Status: `SUCCESSOR_IMPACT_ENERGY_ROOTS_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY`.

## Purpose

W1 falsified one inherited corpus assumption after W0F froze the successor
solver. The static pressure/contact implementation is deterministic, bounded,
non-penetrating and momentum-accounted, but its frictionless non-penetrating
velocity projections are inelastic. Requiring the same two-sided mechanical
energy drift from an equilibrium column and a basin-impact trajectory is
therefore contradictory.

The bounded
[W1 energy discriminator](../../development/continuum-water-w1-successor-energy-discriminator-2026-08-18.md)
separates the issue from formula, contact-order and convergence defects. W0G
recloses only the meaning and rooting of the energy metric. It does not change
geometry, density support, pressure/contact operations, capacities, scenarios,
external curve tolerances or canonical state.

## Energy quantities

For every output frame, retain the W0B quantities

```text
E_n = K_n + U_n
D = max(abs(K_0) + abs(U_0), 1 joule)
signed_balance_n = (E_n - E_0) / D
absolute_drift_n = abs(signed_balance_n)
energy_excess_n = max(signed_balance_n, 0)
energy_deficit_n = max(-signed_balance_n, 0)
```

All four derived values use accepted canonical frames and the existing stable
fold and ties-to-even ppb publication. `K`, `U`, gravity representation and the
denominator are unchanged. A static boundary still performs exactly zero work.
Energy removed by an inelastic pressure/contact projection is reported as a
mechanical-energy deficit, not relabelled as wall work or hidden by an adjusted
balance.

The oracle also records cumulative signed energy deltas for divergence
pressure, gravity velocity update, density pressure, static contact, position
integration and canonical publication. These stage values are diagnostic and
must close to the direct signed balance within `2,000 ppb`; they are not a
second route for accepting a frame.

## Scenario classes and blocking rules

### Reversible/equilibrium class

`CW-HYDRO-001`, `CW-FREEFALL-001`, `CW-STILL-001`, and `CW-ORDER-001` retain
the two-sided rule:

```text
absolute_drift_ppb <= 10_000_000
```

This class catches either energy creation or decay in scenarios selected as
equilibrium, exact analytical control or deterministic-order control.

### Static-impact/dissipative class

`CW-DAMBREAK-001`, `CW-ORIFICE-001`, and `CW-SEALED-001` use:

```text
energy_excess_ppb <= 10_000_000
```

There is no lower mechanical-energy floor in this research profile. Dam-break
front/height and orifice transfer comparisons remain mandatory under their
unchanged RMSE and maximum-error thresholds; missing reference input keeps W1
incomplete. The sealed case remains a containment/capacity stress whose frozen
initial translation is incompatible with a stationary closed box, so its
mechanical deficit is diagnostic rather than a material calibration target.

This one-sided rule is not a general statement that arbitrary dissipation is
acceptable in a production water model. A future viscosity model, elastic
boundary response or dynamic rigid/fluid coupling must name its physical
energy sinks/sources and reclose its own reference and work balance.

## Frozen metric projection

The following LF-terminated block is the complete new W0G metric input. W0F
remains immutable and is an explicit parent of every new root.

```text
IMPACT_ENERGY_CONTRACT_V1_BEGIN
contract.id=static-impact-energy-semantics-v1
parent.execution-profile=617ceec10c0ce2e1c90ec45a3a713696e58313a7432cc9379aadbf4adf48cab3
energy.frame-source=accepted-canonical-state
energy.kinetic=left-fold(0.5*mass*dot(velocity,velocity))
energy.potential=left-fold(mass*gravity-magnitude*y)
energy.denominator=max(abs(K0)+abs(U0),1-joule)
energy.signed-balance=(Kn+Un-K0-U0)/denominator
energy.absolute-drift=abs(signed-balance)
energy.excess=max(signed-balance,0)
energy.deficit=max(-signed-balance,0)
energy.static-boundary-work=zero
energy.stage-fold=divergence-pressure,gravity,density-pressure,static-contact,position-integration,canonical-publication
energy.stage-closure-maximum-ppb=2000
scenario.CW-HYDRO-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000
scenario.CW-FREEFALL-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000
scenario.CW-DAMBREAK-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=front,height
scenario.CW-STILL-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000
scenario.CW-ORIFICE-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=transfer
scenario.CW-SEALED-001.energy-class=static-impact-dissipative;metric=energy-excess;maximum-ppb=10000000;mechanical-deficit=diagnostic;reference=none
scenario.CW-ORDER-001.energy-class=reversible-equilibrium;metric=absolute-drift;maximum-ppb=10000000
IMPACT_ENERGY_CONTRACT_V1_END
```

## Root composition

W0G issues domain-separated roots for:

1. this whole document;
2. the exact marked energy-contract block;
3. a successor corpus root over the W0F corpus root and W0G energy-contract
   root;
4. one scenario root over its W0F scenario root and exact W0G scenario line;
5. a composite execution profile over the W0F execution profile, W0G document,
   W0G energy-contract and W0G corpus roots.

The W0F float profile, execution manifest, fixture and geometry roots are
parents and remain byte-identical. Frames and references produced after W0G
use the W0G composite execution/scenario roots; they cannot collide with W0F
diagnostic trajectories.

## Required checks

W0G may authorize W1 continuation only if all checks pass:

1. W0F root verification and all 64 pre-existing crate tests remain exact.
2. Independent signed/absolute/excess/deficit calculators match positive,
   negative, zero and exact-threshold vectors.
3. Every scenario maps to exactly one frozen energy class; unknown and
   duplicate projections fail closed.
4. Stage-delta closure is checked at every output without changing the
   acceptance formula.
5. The unchanged frozen solver reproduces the recorded hydro, free-fall and
   dam-break frame roots under the W0F parent and produces distinct W0G roots.
6. Dam-break passes the new internal excess rule but remains
   `REFERENCE_PENDING` without the independently generated curve input.
7. Two clean closure runs have identical bounded roots after excluding only
   diagnostic wall-clock time.

## Stop rules

- If a reversible/equilibrium scenario breaches the unchanged two-sided `1%`
  gate, do not move it to the impact class after observing the result.
- If an impact scenario creates more than `1%` mechanical energy, treat the
  first output as a solver discriminator; do not increase the threshold.
- If stage closure exceeds `2,000 ppb`, reject the instrumentation or
  operation accounting before using any energy decision.
- If the external dam-break or orifice curve fails, reject the profile even
  when the one-sided energy gate passes.
- Do not start W2, GPU, runtime integration or dynamic PhysX coupling before
  the complete Linux W1 gate and required external references pass.

Windows repeatability is outside the current W1 execution scope by user
decision. It remains a deferred production-promotion gate and is not waived by
W0G.
