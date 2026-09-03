//! WR1 (plan `continuum-water/10`): exact vertical free-body dynamics for
//! dynamic boxes in the bounded capsule profile.

use std::time::Instant;

use next_contracts::ids::PersistentId;
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsMotionKindV1, PhysicsParticipationV1};

use super::fixture::{capsule_state, reconstruct, step, step_input, world};
use super::r5b::{box_body, material_id, rebuild};
use crate::reference_world::world::MAX_DYNAMIC_BOXES;
use crate::{ReferencePhysicsError, ReferencePhysicsWorld};

const BOX_HALF_EXTENTS: [i64; 3] = [200_000, 300_000, 200_000];
/// Fixture gravity (`-9.792 m/s^2`) and physics rate (`60 Hz`).
const GRAVITY_MICROMETRES_PER_SECOND_SQUARED: i64 = -9_792_000;
const PHYSICS_HZ: i64 = 60;

fn world_with_boxes(
    capsule_centre: [i64; 3],
    boxes: &[(u8, [i64; 3])],
) -> Result<ReferencePhysicsWorld, ReferencePhysicsError> {
    let source = world(30, 60, capsule_centre, 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    for (byte, translation) in boxes {
        let (body_id, descriptor) = box_body(
            *byte,
            PhysicsMotionKindV1::Dynamic,
            *translation,
            BOX_HALF_EXTENTS,
            PhysicsParticipationV1::Solid,
            &material_id,
        );
        bodies.insert(body_id, descriptor);
    }
    rebuild(&source, bodies)
}

fn box_state(
    world: &ReferencePhysicsWorld,
    byte: u8,
) -> &next_contracts::physics::PhysicsBodyStateV2 {
    world
        .snapshot()
        .sorted_body_states
        .iter()
        .find(|(id, _)| id.subject_id.as_bytes()[0] == byte)
        .map(|(_, state)| state)
        .expect("box state")
}

/// Substeps until a box released `drop` above its support touches it under
/// the integer recurrence `v += g / hz; y += v / hz` from rest.
fn recurrence_landing_substep(drop_micrometres: i64) -> u64 {
    let mut velocity = 0_i64;
    let mut height = drop_micrometres;
    let mut substeps = 0_u64;
    while height > 0 {
        velocity += GRAVITY_MICROMETRES_PER_SECOND_SQUARED / PHYSICS_HZ;
        height += velocity / PHYSICS_HZ;
        substeps += 1;
    }
    substeps
}

// G1: a box released 1 m above the floor lands on it exactly and at rest,
// within one substep of the analytic fall time.
#[test]
fn released_box_lands_exactly_on_the_floor() {
    let mut world = world_with_boxes([0, 900_000, 0], &[(0xa0, [2_000_000, 1_300_000, 0])])
        .expect("free-body world activates");
    let initial_revision = box_state(&world, 0xa0).body_revision;
    let mut landing_tick = None;
    for tick in 0..60 {
        step(&mut world, tick, None);
        let state = box_state(&world, 0xa0);
        if landing_tick.is_none() && state.pose.translation_micrometres[1] == 300_000 {
            landing_tick = Some(tick);
        }
    }
    let state = box_state(&world, 0xa0);
    assert_eq!(state.pose.translation_micrometres, [2_000_000, 300_000, 0]);
    assert_eq!(state.linear_velocity_micrometres_per_second, [0; 3]);
    assert!(state.body_revision > initial_revision);
    let landing_tick = landing_tick.expect("the box lands");
    let recurrence = recurrence_landing_substep(1_000_000);
    // Two substeps per gameplay tick: the landing substep lies in the tick.
    assert!(recurrence.div_ceil(2) == landing_tick + 1);
    let analytic = (2.0_f64 * 1.0 / 9.792).sqrt() * PHYSICS_HZ as f64;
    assert!((recurrence as f64 - analytic.ceil()).abs() <= 1.0);
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres,
        [0, 900_000, 0]
    );
}

// G2: a resting box keeps its state; restore mid-fall continues identically.
#[test]
fn resting_box_is_unchanged_and_mid_fall_restore_is_exact() {
    let mut world = world_with_boxes(
        [0, 900_000, 0],
        &[
            (0xa0, [2_000_000, 300_000, 0]),
            (0xa1, [3_000_000, 2_300_000, 0]),
        ],
    )
    .expect("free-body world activates");
    let resting_before = box_state(&world, 0xa0).clone();
    for tick in 0..5 {
        step(&mut world, tick, None);
    }
    let mut restored = reconstruct(world.checkpoint().clone(), &world).expect("restore mid-fall");
    for tick in 5..600 {
        let expected = step(&mut world, tick, None);
        let actual = step(&mut restored, tick, None);
        assert_eq!(actual, expected);
    }
    assert_eq!(box_state(&world, 0xa0), &resting_before);
    assert_eq!(restored.checkpoint(), world.checkpoint());
    let fallen = box_state(&world, 0xa1);
    assert_eq!(fallen.pose.translation_micrometres, [3_000_000, 300_000, 0]);
    assert_eq!(fallen.linear_velocity_micrometres_per_second, [0; 3]);
}

// G3: a box released above another lands on its top face.
#[test]
fn released_box_stacks_on_a_resting_box() {
    let mut world = world_with_boxes(
        [0, 900_000, 0],
        &[
            (0xa0, [2_000_000, 300_000, 0]),
            (0xa1, [2_000_000, 1_500_000, 0]),
        ],
    )
    .expect("free-body world activates");
    let lower_before = box_state(&world, 0xa0).clone();
    for tick in 0..60 {
        step(&mut world, tick, None);
    }
    assert_eq!(box_state(&world, 0xa0), &lower_before);
    let upper = box_state(&world, 0xa1);
    assert_eq!(upper.pose.translation_micrometres, [2_000_000, 900_000, 0]);
    assert_eq!(upper.linear_velocity_micrometres_per_second, [0; 3]);
}

// G4: the capsule pushes box A into resting box B; A stops on B's face and
// B does not move.
#[test]
fn pushed_box_stops_at_a_resting_box_without_chain_push() {
    let mut world = world_with_boxes(
        [0, 900_000, 0],
        &[
            (0xa0, [600_000, 300_000, 0]),
            (0xa1, [1_500_000, 300_000, 0]),
        ],
    )
    .expect("free-body world activates");
    let blocker_before = box_state(&world, 0xa1).clone();
    for tick in 0..12 {
        step(&mut world, tick, Some([32_767, 0]));
    }
    let pushed = box_state(&world, 0xa0);
    assert_eq!(pushed.pose.translation_micrometres, [1_100_000, 300_000, 0]);
    assert_eq!(pushed.linear_velocity_micrometres_per_second, [0; 3]);
    assert_eq!(box_state(&world, 0xa1), &blocker_before);
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[0],
        600_000
    );
}

// G5: a box released above the capsule rests on the capsule's axis-aligned
// top; the capsule is unchanged and the checkpoint restores.
#[test]
fn released_box_rests_on_the_capsule_bounds() {
    let mut world = world_with_boxes([0, 900_000, 0], &[(0xa0, [0, 2_800_000, 0])])
        .expect("free-body world activates");
    for tick in 0..60 {
        step(&mut world, tick, None);
    }
    let state = box_state(&world, 0xa0);
    assert_eq!(state.pose.translation_micrometres, [0, 2_100_000, 0]);
    assert_eq!(state.linear_velocity_micrometres_per_second, [0; 3]);
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres,
        [0, 900_000, 0]
    );
    let restored =
        reconstruct(world.checkpoint().clone(), &world).expect("restore with a box on the capsule");
    assert_eq!(restored.checkpoint(), world.checkpoint());
}

// Profile bound: sixteen boxes activate, seventeen reject; two boxes that
// penetrate each other reject.
#[test]
fn dynamic_box_bound_and_pair_penetration_are_enforced() {
    let boxes = |count: usize| {
        (0..count)
            .map(|index| {
                (
                    0xb0 + index as u8,
                    [2_000_000 + 500_000 * index as i64, 300_000, 0],
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(MAX_DYNAMIC_BOXES, 16);
    assert!(world_with_boxes([0, 900_000, 0], &boxes(16)).is_ok());
    assert_eq!(
        world_with_boxes([0, 900_000, 0], &boxes(17)).err(),
        Some(ReferencePhysicsError::UnsupportedProfile)
    );
    assert_eq!(
        world_with_boxes(
            [0, 900_000, 0],
            &[
                (0xa0, [2_000_000, 300_000, 0]),
                (0xa1, [2_100_000, 300_000, 0]),
            ],
        )
        .err(),
        Some(ReferencePhysicsError::SnapshotPenetrating)
    );
}

// G7 apparatus: one gameplay tick with sixteen resting boxes against the
// capsule-only baseline. The frozen bound (`200 us`, release) is recorded
// in the plan evidence from a release run of this test; a debug run only
// reports.
#[test]
fn sixteen_resting_boxes_tick_cost() {
    let scene = |count: usize| {
        (0..count)
            .map(|index| {
                (
                    0xb0 + index as u8,
                    [2_000_000 + 500_000 * index as i64, 300_000, 0],
                )
            })
            .collect::<Vec<_>>()
    };
    let mut readings = Vec::new();
    for count in [0_usize, 1, 4, 8, 16] {
        let mut world = world_with_boxes([0, 900_000, 0], &scene(count)).expect("boxes activate");
        let mut samples = Vec::with_capacity(60);
        for tick in 0..60 {
            // The step input (with its own snapshot and catalog hashes) is
            // built outside the timed region: the gate measures the step.
            let input = step_input(&world, tick, None);
            let started = Instant::now();
            world.step(&input).expect("physics step");
            samples.push(started.elapsed().as_micros());
        }
        for (byte, _) in scene(count) {
            assert_eq!(
                box_state(&world, byte).linear_velocity_micrometres_per_second,
                [0; 3]
            );
        }
        let first = samples[0];
        let steady = &samples[1..];
        let steady_maximum = steady.iter().copied().max().unwrap_or(0);
        let mean = samples.iter().sum::<u128>() / samples.len() as u128;
        readings.push(format!(
            "{count} boxes: first tick {first} us, steady maximum {steady_maximum} us, mean {mean} us"
        ));
    }
    eprintln!(
        "WR1 G7 (per gameplay tick, debug build: {}): {}",
        cfg!(debug_assertions),
        readings.join("; ")
    );
}

// ADR-105: an external impulse changes a box's vertical velocity by J / m
// once at the first substep; an impulse on an unknown body rejects.
#[test]
fn external_impulse_launches_a_box_and_unknown_bodies_reject() {
    use next_contracts::physics::{
        ExternalImpulseV1, WaterExchangeContextV1, WaterExchangeTupleV1,
    };

    let mut world = world_with_boxes([0, 900_000, 0], &[(0xa0, [2_000_000, 300_000, 0])])
        .expect("free-body world activates");
    let body_id = world
        .snapshot()
        .sorted_body_states
        .keys()
        .copied()
        .find(|id| id.subject_id.as_bytes()[0] == 0xa0)
        .expect("box body id");
    let mut input = step_input(&world, 0, None);
    let context = WaterExchangeContextV1 {
        world_id: input.world_id,
        source_revision: 0,
        source_root: input.expected_snapshot_hash,
        destination_revision: input.expected_world_revision,
        destination_root: input.expected_snapshot_hash,
    };
    // 20 kg box, 40 N s upward: 2 m/s.
    input.external_impulses = vec![ExternalImpulseV1 {
        body_id,
        impulse_micronewton_seconds: [0, 40_000_000, 0],
        application_point_micrometres: [2_000_000, 300_000, 0],
        exchange: WaterExchangeTupleV1::water_buoyancy(&context, 0, body_id).expect("tuple"),
    }];
    let result = world.step(&input).expect("impulse step");
    assert!(result.after_snapshot_hash != result.before_snapshot_hash);
    let state = box_state(&world, 0xa0);
    // Two substeps at 60 Hz: v = 2 m/s - 2 * 0.1632, y rises by
    // floor((2 - 0.1632)/60) + floor((2 - 0.3264)/60) = 30_613 + 27_893 um.
    assert_eq!(state.linear_velocity_micrometres_per_second[1], 1_673_600);
    assert_eq!(state.pose.translation_micrometres[1], 300_000 + 58_506);

    let mut unknown = step_input(&world, 1, None);
    let unknown_body = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0xee; 16]),
        body_slot: 0,
    };
    let context = WaterExchangeContextV1 {
        world_id: unknown.world_id,
        source_revision: 0,
        source_root: unknown.expected_snapshot_hash,
        destination_revision: unknown.expected_world_revision,
        destination_root: unknown.expected_snapshot_hash,
    };
    unknown.external_impulses = vec![ExternalImpulseV1 {
        body_id: unknown_body,
        impulse_micronewton_seconds: [0, 1_000, 0],
        application_point_micrometres: [0; 3],
        exchange: WaterExchangeTupleV1::water_buoyancy(&context, 1, unknown_body).expect("tuple"),
    }];
    assert_eq!(
        world.step(&unknown).err(),
        Some(ReferencePhysicsError::StepInputMismatch)
    );
}
