# R5a physical-animation owner — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / CONTRACT_FREEZE` |
| Updated | 2026-08-16 |
| Task key | `r5a-physical-animation-owner` |
| Scope | One production capsule-driven humanoid animation owner shared by the reference player and NPC, with exact neutral clip sampling, identity retarget, presentation-only foot IK, bind-pose fallback and a separate save/replay segment |
| Definition of done | The bounded player/NPC consumer passes focused, `play`, `persistence-replay`, `content-package` and workspace checks without claiming the remaining R5 physics, root-motion, skinning, Stage 0 or learned-policy scope |
| Authority | Working context only; Accepted SPEC/ADR, `docs/roadmap.md`, checked-in schemas and exact ProductCheck results outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R5 starts with a bounded production animation owner over committed capsule poses; the stopped R141 training lineage remains out of scope.
- **Why:** Neutral skeleton and clip content plus capsule locomotion already exist, but no runtime graph, retarget/IK projection or animation save/replay owner consumes them.
- **Next action:** Freeze the canonical profile/snapshot, implement the deterministic owner in `next_motor`, then connect it to the reference composition root before changing persisted formats.
- **Current blocker:** None for R5a. PhysX-only cutover, slopes/stairs/push/fall/recovery, dynamic gameplay bodies, root-motion admission, real skinning and R5 performance evidence remain later work.
- **Do not retry:** R141, R142, learned control, optimizer/training, vendor/model state as runtime authority, direct animation writes to physics pose or a speculative generic animation framework.
- **Reconsider when:** The production player/NPC consumer proves that one additional graph state, retarget field or owner fact is required for exact continuation or visible fallback behavior.

## Product decisions

### D-001 — Capsule displacement selects animation; animation never owns pose

- **Observation:** Physics already commits the player capsule and static NPC bodies, while accepted architecture assigns pose/contact authority to Physics.
- **Decision:** The graph reads consecutive committed physics projections. Actual planar displacement selects `Idle` or `Locomotion`; the animation result is a read-only presentation projection and cannot teleport or mutate a body.
- **Rejected alternatives:** Driving the capsule from sampled root transforms, using renderer time to select state, or storing a vendor graph cursor.
- **Consequences:** The same recorded physics continuation produces the same graph state and clip phase. Root motion is explicitly deferred until a typed intent passes ordinary physical validation.

### D-002 — One exact archetype, two production instances

- **Observation:** R5 requires one humanoid physical archetype used by player and NPC rather than parallel character stacks.
- **Decision:** Reference alpha binds one neutral skeleton, an idle clip and a locomotion clip to one engine-owned profile, then instantiates it for the player and reference NPC body.
- **Rejected alternatives:** A player-only demo, separate NPC graph semantics, or test-only animation fixtures.
- **Consequences:** Player movement exercises the locomotion transition; the NPC exercises the same profile's idle path and both are present in the owner snapshot.

### D-003 — Persist only future-affecting graph state

- **Observation:** Clip samples, retargeted poses and IK output are reconstructible, but graph state and phase affect future frames.
- **Decision:** A separate Physical Embodiment segment stores profile revision, next simulation tick and sorted per-subject graph records. Joint poses, IK caches and renderer data stay derived.
- **Rejected alternatives:** Presentation-cache serialization, embedding animation state in RPG/Physics snapshots, or omitting graph continuation from save/replay.
- **Consequences:** Current-only replay advances to V10 and compares ten owner descriptors for reference alpha; earlier alpha replay remains typed unsupported under ADR-046.

### D-004 — Deterministic fallback is part of the production projection

- **Observation:** Animation/retarget/IK availability must never block gameplay or become hidden authority.
- **Decision:** Exact fixed-point sampling and identity retarget are the primary route. Any presentation sampling or IK failure selects the declared bind-pose/no-IK projection for that frame while retaining the same authoritative owner snapshot.
- **Rejected alternatives:** Partial joint output, last-frame cache reuse without identity proof, or gameplay failure on an optional presentation path.
- **Consequences:** Tests must prove primary/fallback gameplay-root parity and no mutation of Physics/RPG/ledger state.

## Required reading before semantic edits

1. `AGENTS.md`, architecture README, SPEC-00/SPEC-01/glossary and `docs/architecture/agent-routing.md`.
2. SPEC-04/05/12/14/15/26/27/28/30/35 and ADR-013/027/028/030/032/035/046/058/059/062-071.
3. `docs/roadmap.md` R5, the completed R4d task state, current animation content, physics checkpoint, presentation extraction and persistence/replay consumers.

## Sequential implementation order

1. **In progress:** freeze the minimal profile, graph record, snapshot and fallback projection contracts.
2. **Pending:** implement exact sampling, identity retarget, basic foot IK and owner publication in `next_motor` with negative/canonical tests.
3. **Pending:** instantiate the same profile for player/NPC and consume the derived root projection in reference presentation.
4. **Pending:** add the tenth owner segment to live state, save/load and Replay V10; prove corrupt/stale/non-canonical rejection and exact continuation.
5. **Pending:** run mapped checks, record honest conditional `NotRun` results and promote only the bounded R5a decision.

## Explicitly deferred from R5a

- PhysX-only production cutover and removal of the reference backend policy.
- Slopes, stairs, pushing dynamic bodies, sensors, carrying, melee contacts, fall/recovery and the full physical fixture matrix.
- Root-motion command admission, layered/blended graphs, non-identity retarget profiles, GPU skinning and full-body physical IK.
- BodySchema V2 production projection and the procedural 23-DoF safety/recovery controller.
- Fresh `r5-physics-16.v1` baseline/hard gate, paired Windows/Linux evidence, learned policy training/export and Stage 0/R5 completion claims.
