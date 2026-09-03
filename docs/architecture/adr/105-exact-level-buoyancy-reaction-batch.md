# ADR-105: Exact-level buoyancy reaction batch (first one-pass coupling consumer)

| Field | Value |
|---|---|
| ID | ADR-105 |
| Status | Proposed |
| Version | 0.1 |
| Proposal date | 2026-09-03 |
| Last verified | 2026-09-03 |
| Normative dependencies | [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-076](076-continuum-material-physics-track.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md), [ADR-103](103-authoritative-water-flow-network.md), [ADR-104](104-water-v1-authority-is-the-exact-table-and-flow-network.md) |
| Supersedes | none; narrows the ADR-058 clause "PhysX владеет только private live solver objects" for water by admitting one engine-owned, exact, replayed impulse batch into the canonical physics step input, as ADR-076 required of "a later Accepted ADR narrowing ADR-058" |
| Superseded by | none |

## Context

ADR-104 names `CONTINUUM-WATER-BUOYANCY-P1` as the coupling check that
promotes water, and requires that its reaction source is the exact water
level, never a particle set. ADR-076/081 define the one-pass composite
step (freeze, solve, one reaction batch, PhysX applies and integrates
once, publish or nothing) and the exact exchange tuple every
cross-owner record binds. No reaction record exists yet in the contracts;
the physics step input carries only accepted locomotion intents.

## Decision

### One exact impulse batch per tick, inside the physics owner

1. **Source.** At the start of every gameplay tick the physics owner
   computes, from the committed water table and network levels and the
   committed canonical body poses of the previous tick, a
   `WaterBuoyancyBatchV1`: for every dynamic body whose canonical
   axis-aligned bounds intersect a water volume horizontally and lie
   below its effective level, the displaced volume is the exact integer
   volume of the bounds clipped by the level plane (cubic millimetres),
   the buoyancy impulse is `rho_water * g * V * dt` upward at the centroid
   of the clipped bounds, and the drag impulse is
   `-k_damp * rho_water * V * v * dt` with `v` the committed linear
   velocity. `rho_water`, `g`, `k_damp` and the bounds rule are permille
   profile constants of the batch profile, not code constants. Bodies
   outside every volume receive no record. Records sort by body id, one
   per body.
2. **Delivery.** The batch rides the canonical step input:
   `PhysicsStepInputV3` adds `external_impulses` (body id, linear impulse
   in micronewton-seconds, application point in micrometres, exchange
   tuple), validated like the intents. The reference world and the PhysX
   backend apply each impulse exactly once at the first substep of the
   tick; PhysX remains the sole writer of poses and velocities. Because
   the batch is part of the step input, it is replayed and hashed with
   the step and needs no floating-point profile (integers only).
3. **One pass, no delay.** The batch is computed from the frozen
   previous-tick state and applied in the same tick's rigid step; there
   is no reaction on the next substep, no iteration and no
   arrival-order result. The water side never reads the rigid outcome of
   the tick it feeds. A batch that fails validation (overflow, unknown
   body, capacity) rejects the whole uncommitted step (invariant fault).
4. **Exchange tuple.** Each record binds the ADR-081 tuple: namespace,
   source owner (water), destination owner (physics), world id, expected
   source revision/root (the water flow record revision and the physics
   checkpoint hash), expected destination revision/root (world revision
   and snapshot hash), tick, substep `0`, edge profile
   `nextengine.water-buoyancy.v1`, body id, operation slot `0`.
5. **Bounds.** At most `64` records per tick (the water-volume bound);
   a body larger than a cell is clipped to the cell; no coupling to
   articulated bodies, characters or sensors in this increment.

### What this does not decide

Particle coupling stays research (ADR-104); wake, splash and wetness
presentation of a floating body is an ADR-102 increment; buoyancy for the
player capsule is a later consumer.

## Product check

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `CONTINUUM-WATER-BUOYANCY-P1` | One `0.5 m` dynamic cube of `50 kg` in the reference basin at level `0.5 m`; step to rest; raise the level by command; save, restore, continue; repeat. | Equilibrium immersion `0.20 +- 0.05 m` within the frozen settling time; the cube follows the raised level; identical roots on `game` and `headless`, live and restored; a body outside every volume gets no record; the batch reads no presentation state; the step input with the batch round-trips byte-exactly. | Absent network and table: no batch, PhysX unchanged. |

## Consequences

- `PhysicsStepInputV2` becomes `V3` (schema bump, pinned roots refresh).
- The reference scene gains one floating crate in the basin.
- SPEC-26 gains the batch record and the step-input field; SPEC-38's
  coupling section names this ADR as the water V1 coupling path.

## Considered alternatives

- Apply buoyancy as a PhysX force field outside the step input: rejected;
  it would not be replayed or hashed and would make PhysX read water.
- Particle-derived pressure forces: rejected by ADR-104.
- Exact mesh clipping of arbitrary hulls: deferred; the bounds rule is
  exact and sufficient for the first consumer.
