# ADR-116: Explicit per-iteration force scheduling

| Field | Value |
|---|---|
| ID | ADR-116 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Normative dependencies | [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-115](115-full-principal-inertia-body-successor.md) |
| Supersedes | The implicit frame-start external-force assumption of the ADR-058/071 native scene only for explicitly selected CompiledBodySchemaV4. Existing compiled V1–V3 bodies, environments, checkpoints and runtime routes remain unchanged. |
| Superseded by | none |

## Evidence and decision

The [bounded native experiment](../../development/r8b-upright-reference-research-2026-09-05.md)
isolates a substantial mean pose/velocity discrepancy in V6 nominal standing.
Changing only TGS external-force scheduling nearly removes the root's mean
discrepancy. The improved final frame is not stable-posture acceptance:
oscillation remains. Do not repair this by inventing measured velocities,
disabling gravity, moving anatomical mass or silently changing old scenes.

Add `nextengine.physics.humanoid-per-iteration-external-forces.v1`.
It selects `eENABLE_EXTERNAL_FORCES_EVERY_ITERATION_TGS` before scene creation.
Gravity, timestep, iteration counts, materials, body geometry/inertia, explicit
PD and safety remain those of the compiled input. There is no runtime toggle,
ambient environment-variable override or automatic selection by BodySchema ID.

`CompiledBodySchemaV4` wraps the complete immutable V3 compilation. Its outer
hash binds the domain `nextengine.compiled-body-schema.v4`, legacy complete
descriptor hash, new profile ID, force-schedule extension version 1 and the
closed schedule tag 1. Consumers must use the outer hash, not relabel the base
as if it selected the new dynamics. The body's own hash does not change when
only solver force scheduling changes. Its `create_world` selects that exact
profile through the normal native adapter and material/scene validation.

## Additive native boundary

Keep ABI 4 and every existing input/output layout and entry point unchanged.
Add the statically linked, versioned entry point
`ne_physx_world_configure_scene_with_force_schedule_v1(world, input, schedule)`.
The input is the existing 36-byte scene record; schedule is a separate u32:
0 = frame-start legacy, 1 = every solver position iteration. Other tags fail
before scene construction. The old scene entry delegates to tag 0.

This is an additive extension, not reinterpretation of an ABI-4 field. A
missing extension fails static linking; there is no dynamic capability
fallback that silently changes the selected profile. Rust passes a closed
enum as a u32 and never constructs an enum from unchecked native bytes.
Ownership, synchronous lifetime, material-before-scene and configure-once
rules remain within the existing reviewed FFI crate. Mock mode tests lifecycle
only and does not assert physical equivalence.

## Verification and remaining scope

Require unchanged legacy descriptor/body fixtures and exact legacy trajectory,
new outer hash, native distinction between schedules, exact same-effort fresh
world reconstruction, invalid-tag rejection without scene mutation and native
material/configure-once coverage. Compare the new profiled 30-second result
with all 7,200 steps of the earlier one-flag experiment, including contacts and
efforts. Run native motor/FFI tests, host-check and applicable play/replay/content
checks. No performance claim or default runtime hot-path change is made.

The standing probe can explicitly select the profile. This decision does not
select a new training environment, transfer old weights, admit an Isaac mirror,
or declare posture, foot mechanics or learned locomotion complete. A later
environment must bind the new compiled identity and its reset/replay inputs.
Continuation reconstructs the same selected scene and effort history; direct
pose/velocity import alone remains insufficient under ADR-059.

Rollback is explicit selection of the preserved old compiler/profile, never
changing the new profile's meaning or hiding it under old hashes.
