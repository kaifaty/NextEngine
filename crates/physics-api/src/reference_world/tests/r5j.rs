use std::collections::BTreeMap;

use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::physics::{
    ContactPhaseV1, PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsPoseV1, PhysicsShapeIdV1,
    PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1, PhysicsWorldCheckpointV1,
};

use super::fixture::{capsule_state, reconstruct, shape, step, world_with_wall};
use crate::{ReferencePhysicsError, ReferencePhysicsWorld};

const ATTACHED_LOCAL_CENTRE: [i64; 3] = [0, 400_000, 400_000];
const ATTACHED_HALF_EXTENTS: [i64; 3] = [200_000; 3];

#[test]
fn attached_load_blocks_before_capsule_and_restores_exact_contact_continuity() {
    let source = world_with_wall(30, 60, [0, 900_000, 0], 1, 900_000, 100_000);
    let capsule_body_id = avatar_body_id(&source);
    let capsule_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 0,
    };
    let attached_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 1,
    };
    let wall_shape_id = PhysicsShapeIdV1 {
        body_id: PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([3; 16]),
            body_slot: 0,
        },
        shape_slot: 0,
    };
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    add_attached_box(
        bodies.get_mut(&capsule_body_id).expect("avatar descriptor"),
        attached_shape_id,
        &material_id(&source),
    );
    let mut direct = rebuild(&source, bodies).expect("compound avatar activates");

    let mut phases = Vec::new();
    for tick in 0..3 {
        phases.extend(
            step(&mut direct, tick, Some([0, 32_767]))
                .contact_batch
                .events
                .into_iter()
                .filter(|event| {
                    [event.participant_low, event.participant_high].contains(&wall_shape_id)
                })
                .map(|event| {
                    assert!(
                        [event.participant_low, event.participant_high]
                            .contains(&attached_shape_id)
                    );
                    assert!(
                        ![event.participant_low, event.participant_high,]
                            .contains(&capsule_shape_id)
                    );
                    assert_eq!(event.participant_low, attached_shape_id);
                    assert_eq!(event.feature_low, 6, "attached box must report its +Z face");
                    assert_eq!(event.feature_high, 5, "wall must report its -Z face");
                    event.phase
                }),
        );
    }

    let centre = capsule_state(&direct).pose.translation_micrometres;
    assert_eq!(centre[2], 200_000);
    let capsule_front = centre[2] + 300_000;
    let wall_minimum = 800_000;
    assert!(capsule_front < wall_minimum);
    assert_eq!(
        centre[2] + ATTACHED_LOCAL_CENTRE[2] + ATTACHED_HALF_EXTENTS[2],
        wall_minimum
    );
    assert!(phases.contains(&ContactPhaseV1::Begin));
    assert!(phases.contains(&ContactPhaseV1::Persist));
    assert!(
        direct
            .snapshot()
            .sorted_contact_continuity_states
            .values()
            .any(|state| {
                [state.participant_low, state.participant_high].contains(&attached_shape_id)
                    && [state.participant_low, state.participant_high].contains(&wall_shape_id)
            })
    );

    let checkpoint = direct.checkpoint().clone();
    let mut restored = reconstruct(checkpoint, &direct).expect("compound checkpoint restores");
    let expected = step(&mut direct, 3, Some([0, -32_767]));
    let actual = step(&mut restored, 3, Some([0, -32_767]));
    assert_eq!(actual, expected);
    assert_eq!(restored.checkpoint(), direct.checkpoint());
    assert!(actual.contact_batch.events.iter().any(|event| {
        event.phase == ContactPhaseV1::End
            && [event.participant_low, event.participant_high].contains(&attached_shape_id)
            && [event.participant_low, event.participant_high].contains(&wall_shape_id)
    }));
}

#[test]
fn bounded_profile_rejects_a_second_attached_box() {
    let source = world_with_wall(30, 60, [0, 900_000, 0], 1, 900_000, 100_000);
    let capsule_body_id = avatar_body_id(&source);
    let mut bodies = source.checkpoint().catalog.bodies.clone();
    let descriptor = bodies.get_mut(&capsule_body_id).expect("avatar descriptor");
    for shape_slot in [1, 2] {
        add_attached_box(
            descriptor,
            PhysicsShapeIdV1 {
                body_id: capsule_body_id,
                shape_slot,
            },
            &material_id(&source),
        );
    }

    assert_eq!(
        rebuild(&source, bodies),
        Err(ReferencePhysicsError::UnsupportedProfile)
    );
}

fn add_attached_box(
    descriptor: &mut PhysicsBodyDescriptorV1,
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
) {
    let mut attached = shape(
        shape_id,
        PhysicsGeometryV1::Box {
            half_extents_micrometres: ATTACHED_HALF_EXTENTS,
        },
        material_id,
        1,
        PhysicsContactReportingV1::BeginPersistEnd,
    );
    attached.local_pose = PhysicsPoseV1 {
        translation_micrometres: ATTACHED_LOCAL_CENTRE,
        ..PhysicsPoseV1::default()
    };
    descriptor.shapes.insert(shape_id, attached);
}

fn avatar_body_id(world: &ReferencePhysicsWorld) -> PhysicsBodyIdV1 {
    *world
        .checkpoint()
        .catalog
        .avatar_bindings
        .values()
        .next()
        .expect("avatar binding")
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
