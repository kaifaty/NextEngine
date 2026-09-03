# ADR-104: Water V1 authority is the exact table and flow network; particle water is presentation only

| Field | Value |
|---|---|
| ID | ADR-104 |
| Status | Proposed |
| Version | 0.2 |
| Proposal date | 2026-09-02 |
| Last verified | 2026-09-02 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-076](076-continuum-material-physics-track.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md), [ADR-101](101-presentation-only-dynamic-surface-ring.md), [ADR-102](102-presentation-particle-surface-pass.md), [ADR-103](103-authoritative-water-flow-network.md) |
| Supersedes | ADR-076 water clauses and the water promotion ladder of its "Product impact and promotion" section (including the Windows/Linux root requirement retired by ADR-090): "CPU `f64` DFSPH is the sole canonical candidate for water V1", the canonical particle water boundary, exact active-sample persistence and checkpoint epochs for water, and the particle reaction batch as the coupling path; the SPEC-38 1.x promotion ladder that placed `CONTINUUM-WATER-REF-P1`, `CONTINUUM-COUPLING-P1`, `CONTINUUM-PERSISTENCE-P1` and `CONTINUUM-MIRROR-P1` before water promotion |
| Superseded by | none |

## Context

ADR-076 (2026-08) selected CPU `f64` DFSPH as the canonical water candidate
and defined a promotion ladder through a particle reference, one-pass
particle-rigid coupling, exact particle persistence and an optional GPU
mirror. ADR-100 then made the exact `WaterVolume` table the only
authoritative water and demoted particle dynamics to presentation, and
ADR-103 added the exact flow network that carries water mechanics
(gates, pipes, pumps, communicating vessels). Both are implemented
(`CONTINUUM-WATER-VOLUME-P1`, `CONTINUUM-WATER-FLOW-P1` pass) and
neither reads a particle.

The particle lanes meanwhile did their job as research: the CPU reference
proved the fidelity ceiling and its cost (`270 ms` per 48k step), and the
Nonlocal GPU candidate now drives the live presentation and produced the
calibration the network uses (`Cd 0.40..0.44` for a flush wall opening,
`0.13` for a long lined duct, exit speed `0.5..0.55` of free fall). The
product wants water that scales by orders of magnitude and supports
mechanics of the Timberborn class; that is a property of the exact
network plus presentation tiers, not of any particle authority.

Left as written, SPEC-38 1.x and ADR-076 still make heavy particle
simulation the definition of done for water: the promotion checks, the
`48,000`-sample coupling scenario and the particle persistence clauses
would keep water Proposed forever and would send the next implementer
down a ladder the product no longer needs.

## Decision

### The authority ladder for water V1 is closed by ADR-100 and ADR-103

1. **Authoritative water** is the exact integer pair in the physics
   checkpoint: `WaterVolumeSetV1` (levels, submersion) and
   `WaterFlowNetworkV1` (cells, edges, one exact step per tick). Gameplay,
   saves, replay and roots read nothing else. This needs no floating-point
   execution profile; ADR-081 guardrails for `f64` authority do not apply
   to it.
2. **Particle water is presentation only, permanently for V1.** The CPU
   DFSPH lane, the Nonlocal GPU lane and any later solver render the free
   surface, jets, spray and foam through the ADR-101 ring and the ADR-102
   particle pass. No particle state is canonical, saved, replayed, queried
   by gameplay or reduced into a rigid reaction. A later authoritative
   particle solver is a new ADR with its own evidence, not a continuation
   of ADR-076.
3. **Rigid coupling for water V1 reads exact levels.** Buoyancy and drag
   on PhysX bodies, when a consumer needs them, are computed from the
   exact cell level and the body's exact submerged geometry and delivered
   through the one-pass reaction batch defined by ADR-076/081 (one
   composite `PhysicalStep`, one exchange tuple, PhysX the sole rigid
   writer), whose first implementation is this buoyancy consumer under
   ADR-105 and plan `continuum-water/08`. The batch source is the network or
   a plain `WaterVolume` level, never a particle set. The plan fixes the
   drag law, the exact submerged geometry of the reference box, the
   substep at which the batch applies (the same tick as the flow step;
   no delayed reaction) and the batch record.
4. **Research lanes stay research.** `CONTINUUM-WATER-REF-P1`,
   `CONTINUUM-MIRROR-P1`, `CONTINUUM-PARTICLE-PERSISTENCE-R1` and
   `CONTINUUM-PARTICLE-COUPLING-R1` (the former particle forms of
   `CONTINUUM-PERSISTENCE-P1` and `CONTINUUM-COUPLING-P1`) remain research
   reports and
   calibration sources with their frozen plans and evidence; they are not
   product promotion gates and are not scheduled. Their numbers
   (discharge coefficients, exit speeds, boundary support findings) enter
   the product only as profile constants of the network and the
   presentation solver.
5. **Terrain is a separate lane.** The MPM dry-sand lane and everything
   after it keep their own ladder under ADR-076/SPEC-38; they are neither
   a prerequisite nor a consequence of water promotion.

### Product checks that promote water

| ID | Role |
|---|---|
| `CONTINUUM-WATER-VOLUME-P1` | exact table, submersion, level command (PASS, R8c) |
| `CONTINUUM-WATER-FLOW-P1` | exact network, conservation, gate response, roots (PASS, R8d) |
| `CONTINUUM-WATER-PRESENT-P1` | presentation surface in the game root with identical roots with and without the solver (planned) |
| `CONTINUUM-WATER-BUOYANCY-P1` | exact-level buoyancy/drag batch on one reference body through the one-pass coupling path; identical roots on `game` and `headless`; the batch never reads presentation (planned, first coupling consumer) |
| `RENDER-DYNSURF-P1`, `RENDER-PARTICLE-SURFACE-P1` | presentation paths (ADR-101/102) |
| conditional `performance` | the presentation solver's own budget row; the network step is reported per edge count (`183 us` at `64` cells / `256` edges today, no frozen bound yet) |

Water is promoted (ADR-100, ADR-103 and this ADR Accepted; SPEC-38
Accepted for water, with any still-open terrain clauses moved to their own
Proposed SPEC) when the four `CONTINUUM-WATER-*` checks pass on the
reference host with pinned roots and the routing/traceability/roadmap
updates land.

## Consequences

- SPEC-38 becomes 2.0: the water authority ladder is the exact table and
  network; the particle profile, particle coupling and particle
  persistence sections become the research appendix; the check table
  splits into product checks and research reports; the `48,000`-sample
  coupling scenario becomes a research scenario.
- ADR-076 is superseded in its water clauses; its terrain clauses, the
  one-pass composite step and the rigid-writer authority stay.
- Traceability, routing and the roadmap name the four water checks as the
  definition of done and drop the particle ladder from the water row.
- Scale comes from tiers, not particles: unlimited exact bodies, a
  presentation height field or lattice network for large surfaces, and a
  fixed particle budget spent where water is active.

## Considered alternatives

- Keep the ADR-076 ladder and add the network beside it: rejected; two
  authorities for one substance, and the product would still be gated on
  research fidelity.
- Promote the Nonlocal GPU solver to authority: rejected by ADR-100
  (device-bound `f32`, vendor toolchain) and unnecessary for mechanics.
- Delete the research lanes: rejected; they are the calibration oracle and
  the only fidelity reference the presentation tiers are measured against.
