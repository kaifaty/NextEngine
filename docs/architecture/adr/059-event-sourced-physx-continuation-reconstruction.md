# ADR-059: Event-sourced PhysX continuation reconstruction

| Field | Value |
|---|---|
| ID | ADR-059 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md) |
| Supersedes | Partially supersedes ADR-058's direct fresh-scene continuation-state import assumption and SPEC-26's requirement to serialize portable TGS accumulated-impulse state |
| Superseded by | none |

## Context

The native PhysX TGS articulation path can export root/joint pose and velocity,
but its public API does not expose a complete portable representation of all
hidden manifold, island and solver caches or permit importing those caches into
a fresh scene. Importing the visible state and one preceding effort therefore
does not reproduce exact continuation. Treating raw vendor serialization as
authority would cross the engine-owned boundary, while accepting a tolerance
would weaken canonical replay.

Stage 0 already records the exact canonical reset and every post-safety effort.
Those records are sufficient to reconstruct the same hidden solver history by
executing the same fixed scene from its episode origin. The cost is bounded by
the existing 60-second/3,600-motor-tick Stage 0 episode limit.

## Decision

### Checkpoint closure

A restorable humanoid checkpoint is an atomic closure of:

1. the exact catalog/build/scene/quantization compatibility identity;
2. one canonical episode-origin reset snapshot;
3. an ordered prefix of post-safety efforts for every 240 Hz substep;
4. an exact engine-owned canonical physics witness hash at every 60 Hz motor
   compare point;
5. the final canonical physics witness and complete explicit motor/RNG
   state.

The current Stage 0 prefix contains at most 3,600 motor frames, exactly four
substeps per frame and exactly the declared actuator count per substep. The
fixed-width 23-DoF effort stream is approximately 2.65 MiB at the bound; the
current standalone motor checkpoint envelope is capped at 4 MiB. A project may
select a lower episode bound. Raising either bound requires a new hash-bound
profile and performance/security validation.

`WorldCheckpointV5` remains the final state witness. `SaveManifestV3` publishes
the replay prefix as a Motor-owned exact segment in the same atomic generation;
`ReplayManifestV6` binds the corresponding `MotorTrajectoryManifestV1` and
ordered `MotorStepRecordV1` chain. The aggregate checkpoint is not considered
restorable when that referenced owner-segment closure is absent or corrupt.
This uses the existing current-only alpha contracts and does not change their
wire fields.

### Fresh-scene reconstruction

Restore validates the complete closure before replacing a live world, then:

1. creates a fresh scene from the exact catalog in semantic-ID order;
2. establishes and verifies the canonical reset origin;
3. applies the recorded post-safety efforts directly, without evaluating PD or
   a policy;
4. compares the reconstructed canonical witness at every motor frame and
   stops at the first mismatch;
5. requires the final full witness to match exactly;
6. restores explicit motor/controller/RNG state and atomically publishes the
   reconstructed world only at the canonical boundary.

The prefix is authoritative replay input, not serialized vendor state. Hidden
PhysX caches are reconstructible consequences. Vendor serialization, `Px*`
objects, raw-float witness hashes, allocator state, callback order and opaque
cache bytes remain forbidden in public/save/replay authority. A private
same-build raw snapshot MAY be retained only as a reconstructible diagnostic
cache validated against the canonical witness.

Re-evaluating action to PD/safety effort is a separate parity check. It must
produce the same recorded efforts, but its output cannot replace, repair or
advance authoritative replay history.

### Failure and cost semantics

Missing origin/prefix data, non-contiguous motor ticks, wrong substep or channel
count, capacity overflow, effort conversion failure, witness mismatch or final
continuation mismatch fails closed before target-world publication. The source
save/checkpoint bytes and prior active world remain unchanged.

Restore is O(prefix ticks x substeps x actuators) and retained history is
O(prefix ticks x substeps x actuators). CPU vector environments account this
memory per slot and stream completed trajectories to the configured external
store; no unbounded in-memory history is admitted. The conditional performance
gate records worst-case restore time and peak working set. These costs cannot
be hidden by omitting compare points or falling back to direct state import.

## Alternatives rejected

- Import visible pose/velocity and accept the next-step mismatch: rejected
  because it is not deterministic continuation.
- Serialize private PhysX/TGS caches: rejected because the representation is
  vendor/build-private and not a stable engine-owned contract.
- Recompute PD from actions during restore: rejected because evaluator/control
  re-execution is parity evidence, not authoritative recorded input.
- Store an unbounded event log: rejected because training slots require an
  explicit memory, decode and restore-time ceiling.

## Product checks

- `PHYS-SNAPSHOT-P1` and `MOTOR-STATE` restore random bounded checkpoints into
  independent fresh scenes and compare every motor-frame witness plus at least
  10,000 subsequent physics substeps.
- `persistence-replay` mutates origin, effort, frame order/count and final
  witness independently; every mutation returns a stable fail-closed result
  before publication.
- Restore idempotence creates two fresh scenes from the same checkpoint and
  yields identical continuation under worker/slot permutations.
- `performance` reports maximum-bound encoded bytes, restore duration and peak
  per-slot/aggregate memory. It remains a cutover gate under ADR-058.

## Supersession

ADR-058 remains authoritative for PhysX-only backend selection, fixed scene,
Stage 0 scope and cutover gates. This ADR supersedes only the assumption that a
portable direct import of hidden solver-continuation state is available.
SPEC-26 remains authoritative for public canonical physics facts, but hidden
TGS accumulated impulses are reconstructed from the bounded canonical effort
prefix rather than serialized as `PhysicsSolverContinuationStateV1`.
