use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{
    CommandId, CommandLedgerHash, ContentHash, StateRoot, content_hash_from_bytes,
};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::{
    AcceptedLocomotionIntentV2, ContactPhaseV1, PHYSICS_STEP_INPUT_SCHEMA_VERSION,
    PhysicsGeometryV1, PhysicsShapeIdV1, PhysicsStepInputV2, PhysicsStepResultV1,
};
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1};
use next_physics_api::ReferencePhysicsWorld;
use next_reference_game::{ReferenceGameSession, ReferenceRunOutcomeV2};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalGameplayConformanceReportV1 {
    pub trip_contact_events: u64,
    pub trip_final_pose_micrometres: [i64; 3],
    pub carry_contact_events: u64,
    pub carry_final_pose_micrometres: [i64; 3],
    pub carried_load_pose_micrometres: [i64; 3],
    pub capsule_clearance_micrometres: i64,
    pub restored_contact_ticks: u64,
    pub melee_contact_events: u64,
    pub melee_npc_health: i32,
    pub repeated_run_identical: bool,
    pub final_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_physics_checkpoint_hash: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalGameplayConformanceErrorV1 {
    context: String,
    detail: String,
}

impl PhysicalGameplayConformanceErrorV1 {
    fn new(context: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            detail: detail.into(),
        }
    }

    fn condition(context: impl Into<String>) -> Self {
        Self::new(context, "acceptance condition was false")
    }
}

impl Display for PhysicalGameplayConformanceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for PhysicalGameplayConformanceErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationEvidenceV1 {
    trip_contact_events: u64,
    trip_final_pose_micrometres: [i64; 3],
    carry_contact_events: u64,
    carry_final_pose_micrometres: [i64; 3],
    carried_load_pose_micrometres: [i64; 3],
    capsule_clearance_micrometres: i64,
    restored_contact_ticks: u64,
    melee_contact_events: u64,
    melee_npc_health: i32,
    final_state_root: StateRoot,
    final_command_ledger_hash: CommandLedgerHash,
    final_physics_checkpoint_hash: ContentHash,
    matrix_digest: ContentHash,
}

pub fn run_physical_gameplay_conformance_check()
-> Result<PhysicalGameplayConformanceReportV1, PhysicalGameplayConformanceErrorV1> {
    let scratch = ScratchContext::new(&std::env::temp_dir()).map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new("create scratch context", error.to_string())
    })?;
    let prepared = prepare_fixture_project_package_with_scratch(
        &scratch,
        "nextengine.physical-gameplay-conformance",
    )
    .map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new("activate reference package", error.to_string())
    })?;
    let result = (|| {
        let first = run_generation(&prepared.package)?;
        let repeated = run_generation(&prepared.package)?;
        if repeated != first {
            return Err(PhysicalGameplayConformanceErrorV1::condition(
                "repeated physical gameplay generation is identical",
            ));
        }
        Ok(PhysicalGameplayConformanceReportV1 {
            trip_contact_events: first.trip_contact_events,
            trip_final_pose_micrometres: first.trip_final_pose_micrometres,
            carry_contact_events: first.carry_contact_events,
            carry_final_pose_micrometres: first.carry_final_pose_micrometres,
            carried_load_pose_micrometres: first.carried_load_pose_micrometres,
            capsule_clearance_micrometres: first.capsule_clearance_micrometres,
            restored_contact_ticks: first.restored_contact_ticks,
            melee_contact_events: first.melee_contact_events,
            melee_npc_health: first.melee_npc_health,
            repeated_run_identical: true,
            final_state_root: first.final_state_root,
            final_command_ledger_hash: first.final_command_ledger_hash,
            final_physics_checkpoint_hash: first.final_physics_checkpoint_hash,
            matrix_digest: first.matrix_digest,
        })
    })();
    prepared.finish(result, |error| {
        PhysicalGameplayConformanceErrorV1::new("remove reference package", error.to_string())
    })
}

fn run_generation(
    package: &next_project::ActivatedProjectPackage,
) -> Result<GenerationEvidenceV1, PhysicalGameplayConformanceErrorV1> {
    let session = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            PhysicalGameplayConformanceErrorV1::new(
                "build physical gameplay session",
                error.to_string(),
            )
        })?;
    let (trip_contact_events, trip_final_pose_micrometres) = run_trip_lane(&session)?;
    let carry = run_carry_lane(&session)?;
    let melee =
        next_reference_game::run_reference_game(package.clone(), true).map_err(|error| {
            PhysicalGameplayConformanceErrorV1::new("run contact-driven melee", error.to_string())
        })?;
    let melee_contact_events = melee_contact_events(&melee, &session)?;
    let melee_npc_health = npc_health(&melee, &session)?;
    require(
        melee_contact_events > 0,
        "melee has committed player/NPC physical contact",
    )?;
    require(
        melee_npc_health == 0,
        "contact-driven melee reaches the declared NPC health outcome",
    )?;
    let checkpoint = melee.runtime.world_checkpoint().map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new("build final melee checkpoint", error.to_string())
    })?;
    let final_command_ledger_hash =
        checkpoint
            .runtime_snapshot
            .command_ledger_hash()
            .map_err(|error| {
                PhysicalGameplayConformanceErrorV1::new(
                    "hash final command ledger",
                    error.to_string(),
                )
            })?;
    let mut evidence = GenerationEvidenceV1 {
        trip_contact_events,
        trip_final_pose_micrometres,
        carry_contact_events: carry.contact_events,
        carry_final_pose_micrometres: carry.final_pose_micrometres,
        carried_load_pose_micrometres: carry.load_pose_micrometres,
        capsule_clearance_micrometres: carry.capsule_clearance_micrometres,
        restored_contact_ticks: carry.restored_contact_ticks,
        melee_contact_events,
        melee_npc_health,
        final_state_root: checkpoint.state_root,
        final_command_ledger_hash,
        final_physics_checkpoint_hash: carry.final_checkpoint_hash,
        matrix_digest: ContentHash::default(),
    };
    evidence.matrix_digest = evidence_digest(&evidence);
    Ok(evidence)
}

fn run_trip_lane(
    session: &ReferenceGameSession,
) -> Result<(u64, [i64; 3]), PhysicalGameplayConformanceErrorV1> {
    let mut world = reference_world(session)?;
    let mut gameplay_tick = 0;
    for _ in 0..60 {
        step_world(
            &mut world,
            gameplay_tick,
            session.physics_body_id,
            [0, -32_767],
        )?;
        gameplay_tick += 1;
    }
    let mut contacts = 0;
    for _ in 0..12 {
        let result = step_world(
            &mut world,
            gameplay_tick,
            session.physics_body_id,
            [32_767, 0],
        )?;
        gameplay_tick += 1;
        contacts += contact_event_count(
            &result,
            session.r5b_course.trip_shape_id,
            primary_capsule_shape_id(session),
        )?;
    }
    let pose = world.snapshot().sorted_body_states[&session.physics_body_id]
        .pose
        .translation_micrometres;
    require(contacts > 0, "low riser emits an avatar trip contact")?;
    require(
        pose[0] > 1_100_000,
        "avatar traverses beyond the contacted low riser",
    )?;
    Ok((contacts, pose))
}

struct CarryEvidenceV1 {
    contact_events: u64,
    final_pose_micrometres: [i64; 3],
    load_pose_micrometres: [i64; 3],
    capsule_clearance_micrometres: i64,
    restored_contact_ticks: u64,
    final_checkpoint_hash: ContentHash,
}

fn run_carry_lane(
    session: &ReferenceGameSession,
) -> Result<CarryEvidenceV1, PhysicalGameplayConformanceErrorV1> {
    let mut world = reference_world(session)?;
    let mut gameplay_tick = 0;
    for _ in 0..70 {
        step_world(
            &mut world,
            gameplay_tick,
            session.physics_body_id,
            [0, -32_767],
        )?;
        gameplay_tick += 1;
    }
    let mut contacts = 0;
    for _ in 0..80 {
        let result = step_world(
            &mut world,
            gameplay_tick,
            session.physics_body_id,
            [32_767, 0],
        )?;
        gameplay_tick += 1;
        contacts += contact_event_count(
            &result,
            session.carried_load_shape_id,
            session.r5b_course.carry_blocker_shape_id,
        )?;
    }
    let pose = world.snapshot().sorted_body_states[&session.physics_body_id]
        .pose
        .translation_micrometres;
    let load_pose = checked_add_pose(pose, session.carried_load_local_translation_micrometres)?;
    let capsule_clearance = capsule_clearance_to_blocker(session, pose)?;
    require(contacts > 0, "carried load emits a clearance contact")?;
    require(
        pose == [7_200_000, 900_000, -7_000_000],
        "carried load stops the production avatar at the declared boundary",
    )?;
    require(
        capsule_clearance > 0,
        "primary capsule remains geometrically clear of the carry blocker",
    )?;

    let checkpoint = world.checkpoint().clone();
    let mut restored = ReferencePhysicsWorld::new(
        checkpoint,
        *world.tick_rate_profile(),
        world.numeric_profile().clone(),
        world.quantization_profile().clone(),
    )
    .map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new(
            "restore carried-load checkpoint",
            error.to_string(),
        )
    })?;
    const RESTORED_CONTACT_TICKS: u64 = 4;
    for _ in 0..RESTORED_CONTACT_TICKS {
        let expected = step_world(
            &mut world,
            gameplay_tick,
            session.physics_body_id,
            [32_767, 0],
        )?;
        let actual = step_world(
            &mut restored,
            gameplay_tick,
            session.physics_body_id,
            [32_767, 0],
        )?;
        require(
            actual == expected && restored.checkpoint() == world.checkpoint(),
            "restored carried-load contact continues exactly",
        )?;
        gameplay_tick += 1;
    }
    let final_checkpoint_hash = world.checkpoint().checkpoint_hash().map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new("hash carried-load checkpoint", error.to_string())
    })?;
    Ok(CarryEvidenceV1 {
        contact_events: contacts,
        final_pose_micrometres: pose,
        load_pose_micrometres: load_pose,
        capsule_clearance_micrometres: capsule_clearance,
        restored_contact_ticks: RESTORED_CONTACT_TICKS,
        final_checkpoint_hash,
    })
}

fn reference_world(
    session: &ReferenceGameSession,
) -> Result<ReferencePhysicsWorld, PhysicalGameplayConformanceErrorV1> {
    ReferencePhysicsWorld::new(
        session.bootstrap.physics_checkpoint.clone(),
        session.bootstrap.tick_rate_profile,
        session.bootstrap.authoritative_numeric_profile.clone(),
        session.bootstrap.physics_quantization_profile.clone(),
    )
    .map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new(
            "activate reference physics world",
            error.to_string(),
        )
    })
}

fn step_world(
    world: &mut ReferencePhysicsWorld,
    gameplay_tick: u64,
    body_id: next_contracts::physics::PhysicsBodyIdV1,
    direction_q15: [i16; 2],
) -> Result<PhysicsStepResultV1, PhysicalGameplayConformanceErrorV1> {
    let mut command_id = [0; 16];
    command_id[..8].copy_from_slice(&gameplay_tick.to_le_bytes());
    let input = PhysicsStepInputV2 {
        schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
        world_id: world.snapshot().world_id,
        expected_world_revision: world.snapshot().world_revision,
        expected_snapshot_hash: world.snapshot_hash().map_err(|error| {
            PhysicalGameplayConformanceErrorV1::new(
                "hash physics input snapshot",
                error.to_string(),
            )
        })?,
        expected_catalog_hash: world.catalog_hash().map_err(|error| {
            PhysicalGameplayConformanceErrorV1::new("hash physics input catalog", error.to_string())
        })?,
        gameplay_tick,
        first_physics_tick: world
            .snapshot()
            .physics_tick
            .checked_add(1)
            .ok_or_else(|| {
                PhysicalGameplayConformanceErrorV1::condition("physics tick remains bounded")
            })?,
        physics_substeps: world.tick_rate_profile().physics_substeps_per_gameplay_tick,
        accepted_intents: vec![AcceptedLocomotionIntentV2 {
            causal_command_id: CommandId::from_bytes(command_id),
            controlled_target_id: body_id.subject_id,
            body_id,
            target_gameplay_tick: gameplay_tick,
            direction_q15,
        }],
    };
    world.step(&input).map_err(|error| {
        PhysicalGameplayConformanceErrorV1::new("step reference physics", error.to_string())
    })
}

fn contact_event_count(
    result: &PhysicsStepResultV1,
    first: PhysicsShapeIdV1,
    second: PhysicsShapeIdV1,
) -> Result<u64, PhysicalGameplayConformanceErrorV1> {
    u64::try_from(
        result
            .contact_batch
            .events
            .iter()
            .filter(|event| {
                matches!(event.phase, ContactPhaseV1::Begin | ContactPhaseV1::Persist)
                    && [event.participant_low, event.participant_high].contains(&first)
                    && [event.participant_low, event.participant_high].contains(&second)
            })
            .count(),
    )
    .map_err(|_| PhysicalGameplayConformanceErrorV1::condition("contact count remains bounded"))
}

fn primary_capsule_shape_id(session: &ReferenceGameSession) -> PhysicsShapeIdV1 {
    PhysicsShapeIdV1 {
        body_id: session.physics_body_id,
        shape_slot: 0,
    }
}

fn capsule_clearance_to_blocker(
    session: &ReferenceGameSession,
    pose: [i64; 3],
) -> Result<i64, PhysicalGameplayConformanceErrorV1> {
    let catalog = &session.bootstrap.physics_checkpoint.catalog;
    let avatar = &catalog.bodies[&session.physics_body_id];
    let blocker = &catalog.bodies[&session.r5b_course.static_body_id];
    let capsule = &avatar.shapes[&primary_capsule_shape_id(session)];
    let blocker_shape = &blocker.shapes[&session.r5b_course.carry_blocker_shape_id];
    let PhysicsGeometryV1::Capsule {
        radius_micrometres, ..
    } = capsule.geometry
    else {
        return Err(PhysicalGameplayConformanceErrorV1::condition(
            "primary avatar geometry is a capsule",
        ));
    };
    let PhysicsGeometryV1::Box {
        half_extents_micrometres,
    } = blocker_shape.geometry
    else {
        return Err(PhysicalGameplayConformanceErrorV1::condition(
            "carry blocker geometry is a box",
        ));
    };
    let blocker_minimum_z = blocker.initial_pose.translation_micrometres[2]
        .checked_add(blocker_shape.local_pose.translation_micrometres[2])
        .and_then(|centre| centre.checked_sub(half_extents_micrometres[2]))
        .ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition(
                "carry blocker coordinates remain bounded",
            )
        })?;
    blocker_minimum_z
        .checked_sub(pose[2].checked_add(radius_micrometres).ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition("capsule coordinates remain bounded")
        })?)
        .ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition("capsule clearance remains bounded")
        })
}

fn checked_add_pose(
    pose: [i64; 3],
    local: [i64; 3],
) -> Result<[i64; 3], PhysicalGameplayConformanceErrorV1> {
    Ok([
        pose[0].checked_add(local[0]).ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition("carried load x remains bounded")
        })?,
        pose[1].checked_add(local[1]).ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition("carried load y remains bounded")
        })?,
        pose[2].checked_add(local[2]).ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition("carried load z remains bounded")
        })?,
    ])
}

fn melee_contact_events(
    outcome: &ReferenceRunOutcomeV2,
    session: &ReferenceGameSession,
) -> Result<u64, PhysicalGameplayConformanceErrorV1> {
    u64::try_from(
        outcome
            .tick_reports
            .iter()
            .flat_map(|report| &report.contact_batch.events)
            .filter(|event| {
                matches!(event.phase, ContactPhaseV1::Begin | ContactPhaseV1::Persist)
                    && [
                        event.participant_low.body_id.subject_id,
                        event.participant_high.body_id.subject_id,
                    ]
                    .contains(&session.body_id)
                    && [
                        event.participant_low.body_id.subject_id,
                        event.participant_high.body_id.subject_id,
                    ]
                    .contains(&session.npc_character_id)
            })
            .count(),
    )
    .map_err(|_| {
        PhysicalGameplayConformanceErrorV1::condition("melee contact count remains bounded")
    })
}

fn npc_health(
    outcome: &ReferenceRunOutcomeV2,
    session: &ReferenceGameSession,
) -> Result<i32, PhysicalGameplayConformanceErrorV1> {
    let rpg = outcome.runtime.rpg_snapshot();
    let Some(RpgAggregatePayloadV1::Character(character)) = next_reference_game::aggregate_payload(
        &rpg,
        RpgAggregateKindV1::Character,
        session.npc_character_id,
    ) else {
        return Err(PhysicalGameplayConformanceErrorV1::condition(
            "melee target character aggregate exists",
        ));
    };
    character
        .resources
        .iter()
        .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
        .map(|resource| resource.current_value)
        .ok_or_else(|| {
            PhysicalGameplayConformanceErrorV1::condition(
                "melee target character health resource exists",
            )
        })
}

fn evidence_digest(evidence: &GenerationEvidenceV1) -> ContentHash {
    let mut preimage = b"nextengine.physical-gameplay-conformance.v1\0".to_vec();
    for value in [
        evidence.trip_contact_events,
        evidence.carry_contact_events,
        evidence.restored_contact_ticks,
        evidence.melee_contact_events,
    ] {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for pose in [
        evidence.trip_final_pose_micrometres,
        evidence.carry_final_pose_micrometres,
        evidence.carried_load_pose_micrometres,
    ] {
        for coordinate in pose {
            preimage.extend_from_slice(&coordinate.to_le_bytes());
        }
    }
    preimage.extend_from_slice(&evidence.capsule_clearance_micrometres.to_le_bytes());
    preimage.extend_from_slice(&evidence.melee_npc_health.to_le_bytes());
    preimage.extend_from_slice(evidence.final_state_root.as_bytes());
    preimage.extend_from_slice(evidence.final_command_ledger_hash.as_bytes());
    preimage.extend_from_slice(evidence.final_physics_checkpoint_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn require(
    condition: bool,
    context: &'static str,
) -> Result<(), PhysicalGameplayConformanceErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(PhysicalGameplayConformanceErrorV1::condition(context))
    }
}
