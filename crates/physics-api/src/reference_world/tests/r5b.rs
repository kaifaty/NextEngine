use std::collections::BTreeMap;

use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::physics::{
    ContactPhaseV1, PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsMotionKindV1, PhysicsParticipationV1,
    PhysicsShapeIdV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1,
};

use super::fixture::{body, capsule_state, reconstruct, shape, step, world};
use crate::{ReferencePhysicsError, ReferencePhysicsWorld};

#[test]
fn capsule_climbs_quantized_slope_and_stairs_but_tall_riser_blocks() {
    let source = world(30, 60, [0, 900_000, 0], 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    for (index, top) in [
        50_000, 100_000, 150_000, 200_000, 400_000, 600_000, 1_000_000,
    ]
    .into_iter()
    .enumerate()
    {
        let minimum_x = 400_000 + i64::try_from(index).expect("bounded index") * 400_000;
        let centre = [minimum_x + 200_000, top / 2, 0];
        let (body_id, descriptor) = box_body(
            0x80 + u8::try_from(index).expect("bounded index"),
            PhysicsMotionKindV1::Static,
            centre,
            [200_000, top / 2, 1_000_000],
            PhysicsParticipationV1::Solid,
            &material_id,
        );
        bodies.insert(body_id, descriptor);
    }
    let mut world = rebuild(&source, bodies).expect("R5b traversal world activates");
    let mut maximum_height = i64::MIN;
    for tick in 0..40 {
        step(&mut world, tick, Some([32_767, 0]));
        maximum_height = maximum_height.max(capsule_state(&world).pose.translation_micrometres[1]);
    }
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[0],
        2_500_000
    );
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[1],
        1_500_000
    );
    assert_eq!(maximum_height, 1_500_000);
}

#[test]
fn dynamic_box_push_is_bounded_and_checkpoint_reconstructs_exactly() {
    let source = world(30, 60, [0, 900_000, 0], 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    let (dynamic_body_id, dynamic) = box_body(
        0x90,
        PhysicsMotionKindV1::Dynamic,
        [600_000, 300_000, 0],
        [200_000, 300_000, 200_000],
        PhysicsParticipationV1::Solid,
        &material_id,
    );
    let dynamic_shape_id = *dynamic.shapes.keys().next().expect("dynamic shape");
    bodies.insert(dynamic_body_id, dynamic);
    let (blocker_body_id, blocker) = box_body(
        0x91,
        PhysicsMotionKindV1::Static,
        [1_350_000, 900_000, 0],
        [50_000, 900_000, 500_000],
        PhysicsParticipationV1::Solid,
        &material_id,
    );
    bodies.insert(blocker_body_id, blocker);
    let mut original = rebuild(&source, bodies).expect("dynamic push world activates");
    let mut dynamic_phases = Vec::new();
    for tick in 0..12 {
        dynamic_phases.extend(
            step(&mut original, tick, Some([32_767, 0]))
                .contact_batch
                .events
                .into_iter()
                .filter(|event| {
                    event.participant_low == dynamic_shape_id
                        || event.participant_high == dynamic_shape_id
                })
                .map(|event| event.phase),
        );
    }
    let dynamic_state = &original.snapshot().sorted_body_states[&dynamic_body_id];
    assert_eq!(dynamic_state.pose.translation_micrometres[0], 1_100_000);
    assert_eq!(dynamic_state.linear_velocity_micrometres_per_second, [0; 3]);
    assert_eq!(
        capsule_state(&original).pose.translation_micrometres[0],
        600_000
    );
    assert!(dynamic_phases.contains(&ContactPhaseV1::Begin));
    assert!(dynamic_phases.contains(&ContactPhaseV1::Persist));

    let checkpoint = original.checkpoint().clone();
    let mut restored = reconstruct(checkpoint.clone(), &original).expect("restore push checkpoint");
    let expected = step(&mut original, 12, Some([-32_767, 0]));
    let actual = step(&mut restored, 12, Some([-32_767, 0]));
    assert_eq!(actual, expected);
    assert_eq!(restored.checkpoint(), original.checkpoint());
}

#[test]
fn sensor_emits_begin_persist_end_without_clipping_capsule() {
    let source = world(30, 60, [0, 900_000, 0], 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    let (sensor_body_id, sensor) = box_body(
        0xa0,
        PhysicsMotionKindV1::Static,
        [500_000, 900_000, 0],
        [100_000, 900_000, 500_000],
        PhysicsParticipationV1::Sensor,
        &material_id,
    );
    let sensor_shape_id = *sensor.shapes.keys().next().expect("sensor shape");
    bodies.insert(sensor_body_id, sensor);
    let mut world = rebuild(&source, bodies).expect("sensor world activates");
    let mut phases = Vec::new();
    for tick in 0..12 {
        phases.extend(
            step(&mut world, tick, Some([32_767, 0]))
                .contact_batch
                .events
                .into_iter()
                .filter(|event| {
                    event.participant_low == sensor_shape_id
                        || event.participant_high == sensor_shape_id
                })
                .map(|event| event.phase),
        );
    }
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[0],
        1_200_000
    );
    let begin_count = phases
        .iter()
        .filter(|phase| **phase == ContactPhaseV1::Begin)
        .count();
    let end_count = phases
        .iter()
        .filter(|phase| **phase == ContactPhaseV1::End)
        .count();
    assert!(begin_count > 0);
    assert!(phases.contains(&ContactPhaseV1::Persist));
    assert_eq!(end_count, begin_count);
}

#[test]
fn capsule_falls_to_lower_support_and_resumes_locomotion() {
    let source = world(30, 60, [0, 900_000, 0], 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    let floor_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([2; 16]),
        body_slot: 0,
    };
    let floor = bodies.get_mut(&floor_id).expect("fixture floor");
    floor
        .shapes
        .values_mut()
        .next()
        .expect("floor shape")
        .geometry = PhysicsGeometryV1::Box {
        half_extents_micrometres: [500_000, 100_000, 10_000_000],
    };
    let (lower_id, lower) = box_body(
        0xb0,
        PhysicsMotionKindV1::Static,
        [2_500_000, -1_100_000, 0],
        [2_000_000, 100_000, 10_000_000],
        PhysicsParticipationV1::Solid,
        &material_id,
    );
    bodies.insert(lower_id, lower);
    let mut world = rebuild(&source, bodies).expect("fall course activates");
    let mut minimum_height = i64::MAX;
    let mut landed_x = None;
    for tick in 0..30 {
        step(&mut world, tick, Some([32_767, 0]));
        let state = capsule_state(&world);
        minimum_height = minimum_height.min(state.pose.translation_micrometres[1]);
        if state.pose.translation_micrometres[1] == -100_000
            && state.linear_velocity_micrometres_per_second[1] == 0
        {
            landed_x.get_or_insert(state.pose.translation_micrometres[0]);
        }
    }
    let landed_x = landed_x.expect("capsule lands on lower support");
    let final_state = capsule_state(&world);
    assert_eq!(minimum_height, -100_000);
    assert_eq!(final_state.pose.translation_micrometres[1], -100_000);
    assert_eq!(final_state.linear_velocity_micrometres_per_second[1], 0);
    assert!(final_state.pose.translation_micrometres[0] > landed_x);
}

#[test]
fn second_dynamic_box_is_rejected_by_bounded_profile() {
    let source = world(30, 60, [0, 900_000, 0], 0);
    let material_id = material_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    for (byte, x) in [(0xc0, 600_000), (0xc1, 1_200_000)] {
        let (body_id, descriptor) = box_body(
            byte,
            PhysicsMotionKindV1::Dynamic,
            [x, 300_000, 0],
            [200_000, 300_000, 200_000],
            PhysicsParticipationV1::Solid,
            &material_id,
        );
        bodies.insert(body_id, descriptor);
    }
    assert_eq!(
        rebuild(&source, bodies),
        Err(ReferencePhysicsError::UnsupportedProfile)
    );
}

fn material_id(world: &ReferencePhysicsWorld) -> SchemaId {
    world
        .checkpoint()
        .catalog
        .materials
        .keys()
        .next()
        .expect("fixture material")
        .clone()
}

fn box_body(
    identity_byte: u8,
    motion_kind: PhysicsMotionKindV1,
    translation: [i64; 3],
    half_extents: [i64; 3],
    participation: PhysicsParticipationV1,
    material_id: &SchemaId,
) -> (PhysicsBodyIdV1, PhysicsBodyDescriptorV1) {
    let body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([identity_byte; 16]),
        body_slot: 0,
    };
    let shape_id = PhysicsShapeIdV1 {
        body_id,
        shape_slot: 0,
    };
    let mut shape = shape(
        shape_id,
        PhysicsGeometryV1::Box {
            half_extents_micrometres: half_extents,
        },
        material_id,
        1,
        PhysicsContactReportingV1::BeginPersistEnd,
    );
    shape.participation = participation;
    (body_id, body(body_id, motion_kind, translation, shape))
}

fn rebuild(
    source: &ReferencePhysicsWorld,
    bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV1>,
) -> Result<ReferencePhysicsWorld, ReferencePhysicsError> {
    let source_catalog = &source.checkpoint().catalog;
    let catalog = PhysicsWorldCatalogV1::new(
        source_catalog.world_descriptor.world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: source_catalog.coordinate_profile.clone(),
            limits: source_catalog.limits_profile.clone(),
            solver: source_catalog.solver_profile.clone(),
            tick_rate_hash: source.tick_rate_profile().profile_hash()?,
            authoritative_numeric_hash: source.numeric_profile().profile_hash()?,
            quantization_hash: source.quantization_profile().profile_hash()?,
        },
        source_catalog.materials.clone(),
        bodies,
        source_catalog.avatar_bindings.clone(),
    )?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        source.tick_rate_profile(),
        source.numeric_profile(),
        source.quantization_profile(),
    )?;
    let checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot)?;
    ReferencePhysicsWorld::new(
        checkpoint,
        *source.tick_rate_profile(),
        source.numeric_profile().clone(),
        source.quantization_profile().clone(),
    )
}
