mod fixture;
mod r5b;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use next_contracts::ids::{CommandStreamId, PersistentId};
use next_contracts::physics::{
    ClosedPhysicsContactBatchV1, ContactPhaseV1, PHYSICS_QUERY_SCHEMA_VERSION,
    PHYSICS_STEP_INPUT_SCHEMA_VERSION, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsContractError, PhysicsGeometryV1, PhysicsMotionKindV1,
    PhysicsQueryCardinalityV1, PhysicsQueryFilterV1, PhysicsQueryGeometryV1, PhysicsQueryIdV1,
    PhysicsQueryRequestV1, PhysicsQueryResultPayloadV1, PhysicsShapeIdV1,
    PhysicsSnapshotSelectorV1, PhysicsStepInputV2, PhysicsWorldCatalogProfilesV1,
    PhysicsWorldCatalogV1, PhysicsWorldCheckpointV1, derive_physics_contact_id,
};

use super::query::contact_normal_and_feature;
use super::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepRequest,
    GroundedCapsuleSweepResult, GroundedCapsuleWorld, ReferencePhysicsError, ReferencePhysicsWorld,
    reference_grounded_capsule_sweep,
};
use crate::{
    PhysicsBackendError, PhysicsSceneQueryError, PhysicsWorldBackend, execute_scene_query,
};
use fixture::*;

#[derive(Debug)]
struct CountingStagingQuery {
    fast_forks: Arc<AtomicUsize>,
    recreates: Arc<AtomicUsize>,
    supports_fast_fork: bool,
    reject_fast_fork: bool,
}

impl GroundedCapsuleQuery for CountingStagingQuery {
    fn backend_kind(&self) -> crate::PhysicsBackendKind {
        crate::PhysicsBackendKind::Reference
    }

    fn try_fork_for_staging(&self) -> Result<Option<Self>, ReferencePhysicsError> {
        if self.reject_fast_fork {
            return Err(ReferencePhysicsError::BackendFailure);
        }
        if !self.supports_fast_fork {
            return Ok(None);
        }
        self.fast_forks.fetch_add(1, Ordering::Relaxed);
        Ok(Some(Self {
            fast_forks: Arc::clone(&self.fast_forks),
            recreates: Arc::clone(&self.recreates),
            supports_fast_fork: true,
            reject_fast_fork: false,
        }))
    }

    fn recreate(&self) -> Result<Self, ReferencePhysicsError> {
        self.recreates.fetch_add(1, Ordering::Relaxed);
        Ok(Self {
            fast_forks: Arc::clone(&self.fast_forks),
            recreates: Arc::clone(&self.recreates),
            supports_fast_fork: self.supports_fast_fork,
            reject_fast_fork: self.reject_fast_fork,
        })
    }

    fn sweep_axis(
        &mut self,
        request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
        reference_grounded_capsule_sweep(request)
    }
}

#[test]
fn staging_fork_matches_checkpoint_reconstruction_without_mutating_source() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let before = source.checkpoint().clone();
    let input = step_input(&source, 0, Some([0, 32_767]));

    let mut fast = PhysicsWorldBackend::fork_for_staging(&source).expect("fast staging fork");
    let mut reconstructed = PhysicsWorldBackend::fork_from_checkpoint(&source, before.clone())
        .expect("checkpoint reconstruction");

    assert_eq!(source.checkpoint(), &before);
    let fast_result = fast.step(&input).expect("fast staged step");
    let reconstructed_result = reconstructed
        .step(&input)
        .expect("reconstructed staged step");

    assert_eq!(fast_result, reconstructed_result);
    assert_eq!(fast.checkpoint(), reconstructed.checkpoint());
    assert_eq!(
        fast.checkpoint().checkpoint_hash().expect("fast hash"),
        reconstructed
            .checkpoint()
            .checkpoint_hash()
            .expect("reconstructed hash")
    );
    assert_eq!(source.checkpoint(), &before);
}

#[test]
fn staging_fork_uses_fast_copy_or_checkpoint_fallback_explicitly() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let fast_forks = Arc::new(AtomicUsize::new(0));
    let recreates = Arc::new(AtomicUsize::new(0));
    let fast_world = GroundedCapsuleWorld::with_query(
        source.checkpoint().clone(),
        *source.tick_rate_profile(),
        source.numeric_profile().clone(),
        source.quantization_profile().clone(),
        CountingStagingQuery {
            fast_forks: Arc::clone(&fast_forks),
            recreates: Arc::clone(&recreates),
            supports_fast_fork: true,
            reject_fast_fork: false,
        },
    )
    .expect("fast world");
    let fast_before = fast_world.checkpoint().clone();
    let fast_staging =
        PhysicsWorldBackend::fork_for_staging(&fast_world).expect("fast staging world");
    assert_eq!(fast_forks.load(Ordering::Relaxed), 1);
    assert_eq!(recreates.load(Ordering::Relaxed), 0);
    assert_eq!(fast_staging.checkpoint(), &fast_before);
    assert_eq!(fast_world.checkpoint(), &fast_before);

    let fallback_world = GroundedCapsuleWorld::with_query(
        source.checkpoint().clone(),
        *source.tick_rate_profile(),
        source.numeric_profile().clone(),
        source.quantization_profile().clone(),
        CountingStagingQuery {
            fast_forks: Arc::clone(&fast_forks),
            recreates: Arc::clone(&recreates),
            supports_fast_fork: false,
            reject_fast_fork: false,
        },
    )
    .expect("fallback world");
    let fallback_before = fallback_world.checkpoint().clone();
    let fallback_staging =
        PhysicsWorldBackend::fork_for_staging(&fallback_world).expect("fallback staging world");
    assert_eq!(fast_forks.load(Ordering::Relaxed), 1);
    assert_eq!(recreates.load(Ordering::Relaxed), 1);
    assert_eq!(fallback_staging.checkpoint(), &fallback_before);
    assert_eq!(fallback_world.checkpoint(), &fallback_before);
}

#[test]
fn staging_fork_failure_leaves_source_generation_unchanged() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let mut candidate = GroundedCapsuleWorld::with_query(
        source.checkpoint().clone(),
        *source.tick_rate_profile(),
        source.numeric_profile().clone(),
        source.quantization_profile().clone(),
        CountingStagingQuery {
            fast_forks: Arc::new(AtomicUsize::new(0)),
            recreates: Arc::new(AtomicUsize::new(0)),
            supports_fast_fork: true,
            reject_fast_fork: true,
        },
    )
    .expect("candidate world");
    candidate.set_checkpoint_revision(41);
    let before_failure = candidate.checkpoint().clone();

    let error = PhysicsWorldBackend::fork_for_staging(&candidate)
        .expect_err("fast-fork error must be visible");
    assert_eq!(
        error,
        PhysicsBackendError::World(ReferencePhysicsError::BackendFailure)
    );
    assert_eq!(candidate.checkpoint(), &before_failure);
}

#[test]
fn activation_accepts_exact_touching_and_rejects_penetration() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let capsule_body_id = *source
        .checkpoint()
        .catalog
        .avatar_bindings
        .values()
        .next()
        .expect("capsule binding");

    let mut touching = source.checkpoint().clone();
    touching
        .snapshot
        .sorted_body_states
        .get_mut(&capsule_body_id)
        .expect("capsule state")
        .pose
        .translation_micrometres[2] = 300_000;
    reconstruct(touching, &source).expect("exact touching activates");

    let mut penetrating = source.checkpoint().clone();
    penetrating
        .snapshot
        .sorted_body_states
        .get_mut(&capsule_body_id)
        .expect("capsule state")
        .pose
        .translation_micrometres[2] = 300_001;
    assert_eq!(
        reconstruct(penetrating, &source),
        Err(ReferencePhysicsError::SnapshotPenetrating)
    );
}

#[test]
fn unsupported_static_shape_is_rejected_before_world_activation() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let source_catalog = &source.checkpoint().catalog;
    let mut bodies = source_catalog.bodies.clone();
    let static_body = bodies
        .values_mut()
        .find(|body| body.motion_kind == PhysicsMotionKindV1::Static)
        .expect("static body");
    static_body
        .shapes
        .values_mut()
        .next()
        .expect("static shape")
        .geometry = PhysicsGeometryV1::Sphere {
        radius_micrometres: 100_000,
    };
    let catalog = PhysicsWorldCatalogV1::new(
        source_catalog.world_descriptor.world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: source_catalog.coordinate_profile.clone(),
            limits: source_catalog.limits_profile.clone(),
            solver: source_catalog.solver_profile.clone(),
            tick_rate_hash: source
                .tick_rate_profile()
                .profile_hash()
                .expect("tick hash"),
            authoritative_numeric_hash: source
                .numeric_profile()
                .profile_hash()
                .expect("numeric hash"),
            quantization_hash: source
                .quantization_profile()
                .profile_hash()
                .expect("quantization hash"),
        },
        source_catalog.materials.clone(),
        bodies,
        source_catalog.avatar_bindings.clone(),
    )
    .expect("valid generic sphere catalog");
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        source.tick_rate_profile(),
        source.numeric_profile(),
        source.quantization_profile(),
    )
    .expect("valid generic sphere checkpoint");
    let checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint");
    assert_eq!(
        ReferencePhysicsWorld::new(
            checkpoint,
            source.tick_rate_profile().to_owned(),
            source.numeric_profile().to_owned(),
            source.quantization_profile().to_owned(),
        ),
        Err(ReferencePhysicsError::UnsupportedProfile)
    );
}

#[test]
fn backend_failure_leaves_the_previous_checkpoint_untouched() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let mut failing = GroundedCapsuleWorld::with_query(
        source.checkpoint().clone(),
        source.tick_rate_profile().to_owned(),
        source.numeric_profile().to_owned(),
        source.quantization_profile().to_owned(),
        FailingQuery,
    )
    .expect("failing backend world activates");
    let before = failing.checkpoint().clone();
    let snapshot = failing.snapshot();
    let input = PhysicsStepInputV2 {
        schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
        world_id: snapshot.world_id,
        expected_world_revision: snapshot.world_revision,
        expected_snapshot_hash: snapshot.snapshot_hash().expect("snapshot hash"),
        expected_catalog_hash: failing
            .checkpoint()
            .catalog
            .catalog_hash()
            .expect("catalog hash"),
        gameplay_tick: 0,
        first_physics_tick: 1,
        physics_substeps: failing
            .tick_rate_profile()
            .physics_substeps_per_gameplay_tick,
        accepted_intents: Vec::new(),
    };
    assert_eq!(
        failing.step(&input),
        Err(ReferencePhysicsError::BackendFailure)
    );
    assert_eq!(failing.checkpoint(), &before);
}

#[test]
fn checkpoint_closure_rejects_missing_body_static_drift_and_solver_mismatch() {
    let mut world = world(30, 60, [0, 900_000, 0], 1);
    let _ = step(&mut world, 0, None);
    let checkpoint = world.checkpoint().clone();
    let capsule_body_id = *checkpoint
        .catalog
        .avatar_bindings
        .values()
        .next()
        .expect("capsule binding");
    let static_body_id = checkpoint
        .catalog
        .bodies
        .keys()
        .copied()
        .find(|body_id| *body_id != capsule_body_id)
        .expect("static body");

    let mut missing_body = checkpoint.clone();
    missing_body
        .snapshot
        .sorted_body_states
        .remove(&static_body_id);
    assert_eq!(
        missing_body.validate(),
        Err(PhysicsContractError::ReferenceInvalid)
    );

    let mut static_drift = checkpoint.clone();
    static_drift
        .snapshot
        .sorted_body_states
        .get_mut(&static_body_id)
        .expect("static state")
        .linear_velocity_micrometres_per_second[0] = 1;
    assert_eq!(
        static_drift.validate(),
        Err(PhysicsContractError::ReferenceInvalid)
    );

    let mut missing_solver_state = checkpoint;
    missing_solver_state
        .snapshot
        .sorted_solver_continuation_states
        .clear();
    assert_eq!(
        missing_solver_state.validate(),
        Err(PhysicsContractError::ReferenceInvalid)
    );
}

#[test]
fn activation_rejects_continuity_outside_the_capsule_box_graph() {
    let mut source = world(30, 60, [0, 900_000, 0], 1);
    let _ = step(&mut source, 0, None);
    let mut checkpoint = source.checkpoint().clone();
    let static_shapes = checkpoint
        .catalog
        .bodies
        .values()
        .filter(|body| body.motion_kind == PhysicsMotionKindV1::Static)
        .flat_map(|body| body.shapes.keys().copied())
        .collect::<Vec<_>>();
    let (_, mut contact) = checkpoint
        .snapshot
        .sorted_contact_continuity_states
        .pop_first()
        .expect("floor contact");
    contact.participant_low = static_shapes[0];
    contact.participant_high = static_shapes[1];
    contact.feature_low = 1;
    contact.feature_high = 1;
    contact.contact_id = derive_physics_contact_id(
        contact.participant_low,
        contact.participant_high,
        contact.feature_low,
        contact.feature_high,
    );
    checkpoint
        .snapshot
        .sorted_contact_continuity_states
        .insert(contact.contact_id, contact.clone());
    checkpoint
        .snapshot
        .sorted_solver_continuation_states
        .clear();
    checkpoint
        .snapshot
        .sorted_solver_continuation_states
        .insert(contact.contact_id, [0; 3]);
    checkpoint
        .validate()
        .expect("the generic checkpoint closure remains structurally valid");

    assert_eq!(
        reconstruct(checkpoint, &source),
        Err(ReferencePhysicsError::SnapshotMismatch)
    );
}

#[test]
fn checkpoint_and_contact_batch_reject_stale_ticks_and_invalid_features() {
    let mut world = world(30, 60, [0, 900_000, 0], 1);
    let step_result = step(&mut world, 0, None);
    let checkpoint = world.checkpoint().clone();

    let mut stale_tick = checkpoint.clone();
    stale_tick
        .snapshot
        .sorted_contact_continuity_states
        .values_mut()
        .next()
        .expect("floor contact")
        .last_seen_physics_tick -= 1;
    assert_eq!(
        stale_tick.validate(),
        Err(PhysicsContractError::ContactIdentityMismatch)
    );

    let mut invalid_feature = checkpoint.clone();
    let (_, mut contact) = invalid_feature
        .snapshot
        .sorted_contact_continuity_states
        .pop_first()
        .expect("floor contact");
    contact.feature_low = 0;
    contact.contact_id = derive_physics_contact_id(
        contact.participant_low,
        contact.participant_high,
        contact.feature_low,
        contact.feature_high,
    );
    invalid_feature
        .snapshot
        .sorted_contact_continuity_states
        .insert(contact.contact_id, contact.clone());
    invalid_feature
        .snapshot
        .sorted_solver_continuation_states
        .clear();
    invalid_feature
        .snapshot
        .sorted_solver_continuation_states
        .insert(contact.contact_id, [0; 3]);
    assert_eq!(
        invalid_feature.validate(),
        Err(PhysicsContractError::ReferenceInvalid)
    );

    let event = step_result
        .contact_batch
        .events
        .first()
        .expect("floor begin")
        .clone();
    let mut wrong_tick = event.clone();
    wrong_tick.physics_tick += 1;
    assert_eq!(
        ClosedPhysicsContactBatchV1::new(
            step_result.contact_batch.gameplay_tick,
            step_result.contact_batch.first_physics_tick,
            step_result.contact_batch.substep_count,
            vec![wrong_tick],
            step_result.contact_batch.source_snapshot_hash,
        ),
        Err(PhysicsContractError::NonCanonicalOrder)
    );

    let mut wrong_feature = event;
    wrong_feature.feature_low = 0;
    wrong_feature.contact_id = derive_physics_contact_id(
        wrong_feature.participant_low,
        wrong_feature.participant_high,
        wrong_feature.feature_low,
        wrong_feature.feature_high,
    );
    let batch = ClosedPhysicsContactBatchV1::new(
        step_result.contact_batch.gameplay_tick,
        step_result.contact_batch.first_physics_tick,
        step_result.contact_batch.substep_count,
        vec![wrong_feature],
        step_result.contact_batch.source_snapshot_hash,
    )
    .expect("identity-valid batch");
    assert_eq!(
        batch.validate_against_catalog(&checkpoint.catalog),
        Err(PhysicsContractError::ReferenceInvalid)
    );
}

#[test]
fn contact_reporting_policy_does_not_change_continuity() {
    let mut disabled = world_with_wall_reporting(
        30,
        60,
        [0, 900_000, 0],
        1,
        700_000,
        100_000,
        PhysicsContactReportingV1::Disabled,
    );
    for tick in 0..4 {
        let result = step(&mut disabled, tick, Some([0, 32_767]));
        assert!(result.contact_batch.events.is_empty());
    }
    assert!(
        !disabled
            .snapshot()
            .sorted_contact_continuity_states
            .is_empty()
    );

    let mut begin_end = world_with_wall_reporting(
        30,
        60,
        [0, 900_000, 0],
        1,
        700_000,
        100_000,
        PhysicsContactReportingV1::BeginEnd,
    );
    let mut phases = Vec::new();
    for tick in 0..4 {
        phases.extend(
            step(&mut begin_end, tick, Some([0, 32_767]))
                .contact_batch
                .events
                .into_iter()
                .map(|event| event.phase),
        );
    }
    phases.extend(
        step(&mut begin_end, 4, Some([0, -32_767]))
            .contact_batch
            .events
            .into_iter()
            .map(|event| event.phase),
    );
    assert!(phases.contains(&ContactPhaseV1::Begin));
    assert!(phases.contains(&ContactPhaseV1::End));
    assert!(!phases.contains(&ContactPhaseV1::Persist));
}

#[test]
fn landing_begins_then_persists_with_zero_vertical_velocity() {
    let mut world = world(30, 60, [0, 1_200_000, 0], 1);
    let mut phases = Vec::new();
    for tick in 0..12 {
        phases.extend(
            step(&mut world, tick, None)
                .contact_batch
                .events
                .into_iter()
                .map(|event| event.phase),
        );
    }
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[1],
        900_000
    );
    assert_eq!(
        capsule_state(&world).linear_velocity_micrometres_per_second[1],
        0
    );
    assert_eq!(
        phases
            .iter()
            .filter(|phase| **phase == ContactPhaseV1::Begin)
            .count(),
        1
    );
    assert!(phases.contains(&ContactPhaseV1::Persist));
}

#[test]
fn wall_stops_capsule_and_departure_emits_one_end() {
    let mut world = world(30, 60, [0, 900_000, 0], 1);
    let mut wall_begin = 0;
    for tick in 0..5 {
        wall_begin += step(&mut world, tick, Some([0, 32_767]))
            .contact_batch
            .events
            .iter()
            .filter(|event| {
                event.phase == ContactPhaseV1::Begin && event.normal_low_to_high_q1_30[2] != 0
            })
            .count();
    }
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[2],
        300_000
    );
    assert_eq!(wall_begin, 1);
    let departure = step(&mut world, 5, Some([0, -32_767]));
    assert_eq!(
        departure
            .contact_batch
            .events
            .iter()
            .filter(|event| {
                event.phase == ContactPhaseV1::End && event.normal_low_to_high_q1_30[2] != 0
            })
            .count(),
        1
    );
}

#[test]
fn axial_sweep_cannot_tunnel_through_one_micrometre_wall() {
    let mut world = world_with_wall(20, 60, [0, 900_000, 0], 1, 350_001, 1);
    let result = step(&mut world, 0, Some([0, 32_767]));
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[2],
        50_000
    );
    assert!(result.contact_batch.events.iter().any(|event| {
        event.phase == ContactPhaseV1::Begin && event.normal_low_to_high_q1_30[2] != 0
    }));
}

#[test]
fn restored_contact_continuity_resumes_with_persist() {
    let mut original = world(30, 60, [0, 900_000, 0], 1);
    let first = step(&mut original, 0, None);
    assert!(
        first
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Begin)
    );
    let checkpoint = original.checkpoint().clone();
    let mut restored = ReferencePhysicsWorld::new(
        checkpoint,
        original.tick_rate_profile().to_owned(),
        original.numeric_profile().to_owned(),
        original.quantization_profile().to_owned(),
    )
    .expect("restored world");
    let continued = step(&mut restored, 1, None);
    assert!(
        continued
            .contact_batch
            .events
            .iter()
            .all(|event| event.phase != ContactPhaseV1::Begin)
    );
    assert!(
        continued
            .contact_batch
            .events
            .iter()
            .any(|event| event.phase == ContactPhaseV1::Persist)
    );
}

#[test]
fn checkpoint_after_quantized_corner_sweep_reconstructs_exactly() {
    let source = world(30, 60, [0, 900_000, 0], 1);
    let source_catalog = &source.checkpoint().catalog;
    let mut bodies = source_catalog.bodies.clone();
    let obstacle_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([3; 16]),
        body_slot: 0,
    };
    let obstacle = bodies
        .get_mut(&obstacle_body_id)
        .expect("reference obstacle body");
    obstacle.initial_pose.translation_micrometres = [700_000, 900_000, 200_000];
    obstacle
        .shapes
        .values_mut()
        .next()
        .expect("reference obstacle shape")
        .geometry = PhysicsGeometryV1::Box {
        half_extents_micrometres: [100_000, 900_000, 100_000],
    };
    let catalog = PhysicsWorldCatalogV1::new(
        source_catalog.world_descriptor.world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: source_catalog.coordinate_profile.clone(),
            limits: source_catalog.limits_profile.clone(),
            solver: source_catalog.solver_profile.clone(),
            tick_rate_hash: source
                .tick_rate_profile()
                .profile_hash()
                .expect("tick hash"),
            authoritative_numeric_hash: source
                .numeric_profile()
                .profile_hash()
                .expect("numeric hash"),
            quantization_hash: source
                .quantization_profile()
                .profile_hash()
                .expect("quantization hash"),
        },
        source_catalog.materials.clone(),
        bodies,
        source_catalog.avatar_bindings.clone(),
    )
    .expect("corner-obstacle catalog");
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        source.tick_rate_profile(),
        source.numeric_profile(),
        source.quantization_profile(),
    )
    .expect("corner-obstacle snapshot");
    let checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint");
    let mut original = ReferencePhysicsWorld::new(
        checkpoint,
        source.tick_rate_profile().to_owned(),
        source.numeric_profile().to_owned(),
        source.quantization_profile().to_owned(),
    )
    .expect("corner-obstacle world");

    for tick in 0..4 {
        let _ = step(&mut original, tick, Some([32_767, 0]));
    }
    assert_eq!(
        original.snapshot().sorted_contact_continuity_states.len(),
        2,
        "the capsule should retain floor and corner-obstacle continuity"
    );

    let checkpoint = original.checkpoint().clone();
    let restored = reconstruct(checkpoint.clone(), &original)
        .expect("a checkpoint produced by a committed step must reconstruct");
    assert_eq!(restored.checkpoint(), &checkpoint);
}

#[test]
fn filtered_wall_does_not_collide_or_report_contact() {
    let mut world = world(30, 60, [0, 900_000, 0], 0);
    let mut wall_events = 0;
    for tick in 0..6 {
        wall_events += step(&mut world, tick, Some([0, 32_767]))
            .contact_batch
            .events
            .iter()
            .filter(|event| event.normal_low_to_high_q1_30[2] != 0)
            .count();
    }
    assert_eq!(
        capsule_state(&world).pose.translation_micrometres[2],
        600_000
    );
    assert_eq!(wall_events, 0);
}

#[test]
fn primitive_box_edge_and_vertex_features_have_canonical_ids() {
    let body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x44; 16]),
        body_slot: 0,
    };
    let shape = GroundedCapsuleStaticBox {
        shape_id: PhysicsShapeIdV1 {
            body_id,
            shape_slot: 0,
        },
        minimum: [0, 0, 0],
        maximum: [10, 10, 10],
        contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
        collision_layer: 0,
        collision_mask: 1,
    };
    let (edge_normal, edge_feature) =
        contact_normal_and_feature([13, 5, 14], 1, &shape).expect("edge feature");
    assert_eq!(edge_feature, 14);
    assert!(edge_normal[0] > 0 && edge_normal[1] == 0 && edge_normal[2] > 0);

    let (vertex_normal, vertex_feature) =
        contact_normal_and_feature([13, 20, 14], 1, &shape).expect("vertex feature");
    assert_eq!(vertex_feature, 26);
    assert!(vertex_normal.into_iter().all(|component| component > 0));
}

#[test]
fn supported_cadences_move_exactly_three_metres_per_second_without_wall() {
    for gameplay_hz in [20, 30, 60] {
        for physics_hz in [60, 120, 240] {
            let mut world = world(gameplay_hz, physics_hz, [0, 900_000, 0], 0);
            for tick in 0..u64::from(gameplay_hz) {
                let _ = step(&mut world, tick, Some([32_767, 0]));
            }
            assert_eq!(
                capsule_state(&world).pose.translation_micrometres[0],
                3_000_000
            );
        }
    }
}

fn closest_point_request(
    world: &ReferencePhysicsWorld,
    point_micrometres: [i64; 3],
    cardinality: PhysicsQueryCardinalityV1,
    maximum_published_hits: u32,
) -> PhysicsQueryRequestV1 {
    let snapshot = world.snapshot();
    PhysicsQueryRequestV1 {
        schema_version: PHYSICS_QUERY_SCHEMA_VERSION,
        query_id: PhysicsQueryIdV1 {
            physics_tick: snapshot.physics_tick,
            query_slot: 0,
            issuer_stream_id: CommandStreamId::from_bytes([0x51; 16]),
        },
        world_id: snapshot.world_id,
        snapshot_selector: PhysicsSnapshotSelectorV1 {
            physics_tick: snapshot.physics_tick,
            completed_substep: 0,
            physics_snapshot_hash: snapshot.snapshot_hash().expect("snapshot hash"),
        },
        geometry: PhysicsQueryGeometryV1::ClosestPoint {
            point_micrometres,
            maximum_distance_micrometres: 5_000_000,
        },
        filter: PhysicsQueryFilterV1 {
            query_collision_layer: 0,
            query_collision_mask: 1,
            include_solid: true,
            include_sensor: false,
            include_query_only: false,
            excluded_bodies: Vec::new(),
            excluded_shapes: Vec::new(),
        },
        cardinality,
        maximum_published_hits,
    }
}

#[test]
fn closest_point_query_is_exact_for_box_and_capsule_and_excludes_actor() {
    let world = world(30, 60, [0, 900_000, 0], 1);
    let mut actor_query = closest_point_request(
        &world,
        [0, 900_000, 0],
        PhysicsQueryCardinalityV1::Closest,
        1,
    );
    let result =
        execute_scene_query(world.checkpoint(), &actor_query).expect("closest actor query");
    let PhysicsQueryResultPayloadV1::Closest {
        hit: Some(actor_hit),
    } = result.payload
    else {
        panic!("closest actor hit");
    };
    assert_eq!(
        actor_hit.shape_id.body_id.subject_id,
        PersistentId::from_bytes([1; 16])
    );
    assert_eq!(actor_hit.distance_micrometres, 0);
    assert_eq!(actor_hit.feature_id, 1);

    actor_query
        .filter
        .excluded_bodies
        .push(actor_hit.shape_id.body_id);
    let result =
        execute_scene_query(world.checkpoint(), &actor_query).expect("actor exclusion query");
    let PhysicsQueryResultPayloadV1::Closest { hit: Some(box_hit) } = result.payload else {
        panic!("closest box hit");
    };
    assert_eq!(
        box_hit.shape_id.body_id.subject_id,
        PersistentId::from_bytes([3; 16])
    );
    assert_eq!(box_hit.distance_micrometres, 600_000);
    assert_eq!(box_hit.feature_id, 5);
    assert_eq!(box_hit.outward_normal_q1_30, [0, 0, -(1 << 30)]);
}

#[test]
fn closest_point_query_publishes_canonical_prefix_after_complete_scan() {
    let world = world(30, 60, [0, 900_000, 0], 1);
    let request = closest_point_request(
        &world,
        [0, 900_000, 350_000],
        PhysicsQueryCardinalityV1::All,
        2,
    );
    let first = execute_scene_query(world.checkpoint(), &request).expect("all query");
    let second = execute_scene_query(world.checkpoint(), &request).expect("repeat query");
    assert_eq!(first, second);
    let PhysicsQueryResultPayloadV1::All {
        eligible_hit_count,
        truncated,
        hits,
    } = first.payload
    else {
        panic!("all result");
    };
    assert_eq!(eligible_hit_count, 3);
    assert!(truncated);
    assert_eq!(hits.len(), 2);
    assert_eq!(
        hits[0].cmp_canonical_for_kind(&hits[1], request.geometry.kind()),
        std::cmp::Ordering::Less
    );
    assert_eq!(
        hits[0].shape_id.body_id.subject_id,
        PersistentId::from_bytes([1; 16])
    );
    assert_eq!(
        hits[1].shape_id.body_id.subject_id,
        PersistentId::from_bytes([3; 16])
    );
}

#[test]
fn scene_query_rejects_wrong_snapshot_and_unsupported_kind_without_partial_result() {
    let world = world(30, 60, [0, 900_000, 0], 1);
    let mut stale = closest_point_request(
        &world,
        [0, 900_000, 0],
        PhysicsQueryCardinalityV1::Closest,
        1,
    );
    stale.snapshot_selector.physics_snapshot_hash =
        next_contracts::ids::content_hash_from_bytes([0; 32]);
    assert_eq!(
        execute_scene_query(world.checkpoint(), &stale),
        Err(PhysicsSceneQueryError::SnapshotMismatch)
    );

    let mut ray = closest_point_request(
        &world,
        [0, 900_000, 0],
        PhysicsQueryCardinalityV1::Closest,
        1,
    );
    ray.geometry = PhysicsQueryGeometryV1::RayCast {
        origin_micrometres: [0, 900_000, 0],
        unit_direction_q1_30: [0, 0, 1 << 30],
        maximum_distance_micrometres: 5_000_000,
    };
    assert_eq!(
        execute_scene_query(world.checkpoint(), &ray),
        Err(PhysicsSceneQueryError::UnsupportedQueryKind(
            next_contracts::physics::PhysicsQueryKindV1::RayCast
        ))
    );
}

#[test]
fn scene_query_applies_bilateral_layer_filter_before_geometry() {
    let world = world(30, 60, [0, 900_000, 0], 1);
    let mut request =
        closest_point_request(&world, [0, 900_000, 0], PhysicsQueryCardinalityV1::Any, 0);
    request.filter.query_collision_mask = 0;
    let result = execute_scene_query(world.checkpoint(), &request).expect("filtered query");
    assert_eq!(
        result.payload,
        PhysicsQueryResultPayloadV1::Any {
            eligible_hit: false
        }
    );
}
