use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::{CommandPayload, WorldCommand};
use next_contracts::ids::{CommandLedgerHash, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::physical_animation::{ROOT_MOTION_MOVE_PERFORMED_PHASE_ID, RootMotionIntentV1};
use next_contracts::physics::{
    PhysicsBodyStateV2, PhysicsCanonicalSnapshotV2, PhysicsPoseV1, PhysicsWorldCatalogProfilesV1,
    PhysicsWorldCatalogV1, PhysicsWorldCheckpointV1,
};
use next_motor::{
    PhysicalAnimationOwnerV1, PhysicalAnimationPresentationAvailabilityV1,
    PhysicalAnimationProjectionModeV1,
};
use next_reference_game::{
    ReferenceGameSession, cooked_project_rpg_snapshot, reference_physical_animation_owner,
    restore_reference_physical_animation_owner,
};
use next_runtime::{
    CommandDisposition, RejectionCode, RuntimeReplayDriver, RuntimeState, TransactionStage,
};

use crate::build_neutral_player_fixture;

pub const ROOT_MOTION_CONFORMANCE_CYCLES: u64 = 10_000;
const MATRIX_BLOCK_CYCLES: u64 = 10;
const REQUESTED_FORWARD_Q15: [i16; 2] = [0, 32_767];
const CLIPPING_FIXTURE_START_Z_MICROMETRES: i64 = 50_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootMotionConformanceReportV1 {
    pub cycles: u64,
    pub accepted_cycles: u64,
    pub rejected_cycles: u64,
    pub retried_cycles: u64,
    pub save_load_cycles: u64,
    pub lod_cycles: u64,
    pub canonical_proposal_round_trips: u64,
    pub motor_safety_decisions: u64,
    pub replayed_cycles: u64,
    pub full_motion_outcomes: u64,
    pub clipped_motion_outcomes: u64,
    pub blocked_motion_outcomes: u64,
    pub fault_no_mutation_outcomes: u64,
    pub lod_full_projection_probes: u64,
    pub lod_fallback_projection_probes: u64,
    pub final_pose: PhysicsPoseV1,
    pub final_physics_checkpoint_hash: ContentHash,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootMotionConformanceErrorV1 {
    context: String,
    detail: String,
}

impl RootMotionConformanceErrorV1 {
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

impl Display for RootMotionConformanceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for RootMotionConformanceErrorV1 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MatrixCase {
    Accepted,
    Rejected(RootMotionFault),
    Retry,
    SaveLoad,
    Lod,
}

impl MatrixCase {
    const fn tag(self) -> u8 {
        match self {
            Self::Accepted => 1,
            Self::Rejected(_) => 2,
            Self::Retry => 3,
            Self::SaveLoad => 4,
            Self::Lod => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum RootMotionFault {
    BodyRevision = 1,
    SourceClip = 2,
    Interval = 3,
    Translation = 4,
    AnimationTick = 5,
    LocomotionProfile = 6,
}

impl RootMotionFault {
    const ALL: [Self; 6] = [
        Self::BodyRevision,
        Self::SourceClip,
        Self::Interval,
        Self::Translation,
        Self::AnimationTick,
        Self::LocomotionProfile,
    ];

    fn for_rejection(rejection_ordinal: u64) -> Self {
        let index = usize::try_from(rejection_ordinal % Self::ALL.len() as u64)
            .expect("fault ordinal is bounded by the fixed variant count");
        Self::ALL[index]
    }
}

#[derive(Default)]
struct MatrixCounters {
    accepted: u64,
    rejected: u64,
    retried: u64,
    save_load: u64,
    lod: u64,
    canonical: u64,
    motor: u64,
    replayed: u64,
    full: u64,
    clipped: u64,
    blocked: u64,
    fault_no_mutation: u64,
    lod_full: u64,
    lod_fallback: u64,
}

struct RetryCandidate {
    command: WorldCommand,
    finalized_receipt_count: u64,
}

pub fn run_root_motion_conformance_check()
-> Result<RootMotionConformanceReportV1, RootMotionConformanceErrorV1> {
    run_root_motion_conformance_cycles(ROOT_MOTION_CONFORMANCE_CYCLES)
}

fn run_root_motion_conformance_cycles(
    cycles: u64,
) -> Result<RootMotionConformanceReportV1, RootMotionConformanceErrorV1> {
    if cycles == 0 || !cycles.is_multiple_of(MATRIX_BLOCK_CYCLES) {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion matrix cycle count is a positive multiple of ten",
        ));
    }

    let mut fixture =
        build_neutral_player_fixture("nextengine.root-motion-conformance").map_err(|error| {
            RootMotionConformanceErrorV1::new("activate fixture", error.to_string())
        })?;
    configure_clipping_fixture(&mut fixture)?;
    let phase = SchemaId::new(ROOT_MOTION_MOVE_PERFORMED_PHASE_ID).map_err(|error| {
        RootMotionConformanceErrorV1::new("root-motion phase", error.to_string())
    })?;
    let mut runtime_generation = None;
    let mut animation_generation = None;
    let mut replay_generation = None;
    let mut next_sequence = 0_u64;
    let mut retry_candidate: Option<RetryCandidate> = None;
    let mut counters = MatrixCounters::default();
    let mut fault_coverage = [false; RootMotionFault::ALL.len()];
    let mut transcript = b"nextengine.anim-root-motion-p1.matrix.v1\0".to_vec();
    let mut final_pose = PhysicsPoseV1::default();
    let mut final_physics_checkpoint_hash = ContentHash::default();
    let mut final_command_ledger_hash = CommandLedgerHash::default();

    for cycle in 0..cycles {
        if cycle.is_multiple_of(MATRIX_BLOCK_CYCLES) {
            let (runtime, animation, replay) = activate_matrix_generation(&fixture)?;
            runtime_generation = Some(runtime);
            animation_generation = Some(animation);
            replay_generation = Some(replay);
            next_sequence = 0;
            retry_candidate = None;
        }
        let runtime = runtime_generation
            .as_mut()
            .expect("every fixed matrix block activates a Runtime generation");
        let animation = animation_generation
            .as_mut()
            .expect("every fixed matrix block activates an animation generation");
        let replay = replay_generation
            .as_mut()
            .expect("every fixed matrix block activates a replay generation");
        let matrix_case = matrix_case(cycle);
        if matrix_case == MatrixCase::SaveLoad {
            restore_direct_generation(&fixture, runtime, animation)?;
            counters.save_load += 1;
        }
        if matrix_case == MatrixCase::Lod {
            verify_lod_isolation(&fixture, animation, runtime.physics_snapshot())?;
            counters.lod += 1;
            counters.lod_full += 1;
            counters.lod_fallback += 1;
        }

        let (command, motor_decision_hash) = match matrix_case {
            MatrixCase::Retry => {
                counters.retried += 1;
                let candidate = retry_candidate.as_ref().ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition(
                        "retry cycle follows its declared rejected command",
                    )
                })?;
                (candidate.command.clone(), None)
            }
            MatrixCase::Accepted | MatrixCase::SaveLoad | MatrixCase::Lod => {
                counters.accepted += u64::from(matrix_case == MatrixCase::Accepted);
                let (command, decision_hash) = fresh_command(
                    &fixture,
                    animation,
                    runtime,
                    next_sequence,
                    phase.clone(),
                    None,
                )?;
                next_sequence = next_sequence.checked_add(1).ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition("root-motion sequence remains bounded")
                })?;
                counters.motor += 1;
                (command, Some(decision_hash))
            }
            MatrixCase::Rejected(fault) => {
                counters.rejected += 1;
                fault_coverage[usize::from(fault as u8 - 1)] = true;
                let (command, decision_hash) = fresh_command(
                    &fixture,
                    animation,
                    runtime,
                    next_sequence,
                    phase.clone(),
                    Some(fault),
                )?;
                next_sequence = next_sequence.checked_add(1).ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition("root-motion sequence remains bounded")
                })?;
                counters.motor += 1;
                (command, Some(decision_hash))
            }
        };

        verify_canonical_command(&command)?;
        counters.canonical += 1;
        let command_id = command.compute_command_id().map_err(|error| {
            RootMotionConformanceErrorV1::new("compute command identity", error.to_string())
        })?;
        let before_physics = runtime.physics_snapshot().clone();
        let before_body = controlled_body(&fixture, &before_physics)?.clone();
        let report = runtime.run_tick([command.clone()]).map_err(|error| {
            RootMotionConformanceErrorV1::new("execute root-motion tick", error.to_string())
        })?;
        verify_recorded_command(&report, &command)?;
        let result = report
            .results
            .iter()
            .find(|result| result.command_id == command_id)
            .ok_or_else(|| {
                RootMotionConformanceErrorV1::condition("command result receipt is present")
            })?;
        let after_body = controlled_body(&fixture, &report.physics_snapshot)?;

        match matrix_case {
            MatrixCase::Accepted | MatrixCase::SaveLoad | MatrixCase::Lod => {
                if result.disposition != CommandDisposition::Committed {
                    return Err(RootMotionConformanceErrorV1::condition(format!(
                        "cycle {cycle} commits an admitted root-motion command"
                    )));
                }
                verify_committed_physics_outcome(
                    cycle,
                    command_id,
                    &command,
                    &before_body,
                    after_body,
                    &report,
                    &mut counters,
                )?;
            }
            MatrixCase::Rejected(_) | MatrixCase::Retry => {
                if result.disposition
                    != CommandDisposition::Rejected(RejectionCode::RootMotionIntentRejected)
                    || !report.physics_step_input.accepted_intents.is_empty()
                    || after_body != &before_body
                {
                    return Err(RootMotionConformanceErrorV1::condition(format!(
                        "cycle {cycle} rejects atomically without body mutation"
                    )));
                }
                counters.fault_no_mutation += 1;
            }
        }

        if matrix_case == MatrixCase::Retry {
            let candidate = retry_candidate.take().ok_or_else(|| {
                RootMotionConformanceErrorV1::condition("retry candidate remains available")
            })?;
            let receipt_count = finalized_receipt_count(&fixture, &report)?;
            if receipt_count != candidate.finalized_receipt_count
                || !report.stage_trace.iter().any(|entry| {
                    entry.stage == TransactionStage::IngressCommit && entry.deduplicated == 1
                })
            {
                return Err(RootMotionConformanceErrorV1::condition(format!(
                    "cycle {cycle} is an exact idempotent receipt retry"
                )));
            }
        } else if matches!(matrix_case, MatrixCase::Rejected(_)) && cycle % 10 == 4 {
            retry_candidate = Some(RetryCandidate {
                command: command.clone(),
                finalized_receipt_count: finalized_receipt_count(&fixture, &report)?,
            });
        }

        let replay_report = replay
            .replay_tick(
                report.closed_ingress_batch.clone(),
                vec![command.clone()],
                &report.command_batches[0],
                &report.physics_step_input,
                &report.contact_batch,
                &report.command_batches[1],
            )
            .map_err(|error| {
                RootMotionConformanceErrorV1::new(
                    format!("replay root-motion cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        if replay_report != report {
            return Err(RootMotionConformanceErrorV1::condition(format!(
                "cycle {cycle} replay report is exact"
            )));
        }
        counters.replayed += 1;

        animation
            .advance(
                &before_physics,
                runtime.physics_snapshot(),
                runtime.next_tick(),
            )
            .map_err(|error| {
                RootMotionConformanceErrorV1::new(
                    format!("advance physical animation after cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        append_transcript_record(
            &mut transcript,
            cycle,
            matrix_case,
            &command,
            &result.disposition,
            after_body,
            report.physics_checkpoint_hash,
            motor_decision_hash,
        );
        final_pose = after_body.pose;
        final_physics_checkpoint_hash = report.physics_checkpoint_hash;
        final_command_ledger_hash = report.snapshot.command_ledger_hash().map_err(|error| {
            RootMotionConformanceErrorV1::new("final command ledger hash", error.to_string())
        })?;
    }

    verify_matrix_counts(cycles, &counters)?;
    if cycles >= 30 && fault_coverage.iter().any(|covered| !covered) {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion matrix covers every declared rejection fault",
        ));
    }

    Ok(RootMotionConformanceReportV1 {
        cycles,
        accepted_cycles: counters.accepted,
        rejected_cycles: counters.rejected,
        retried_cycles: counters.retried,
        save_load_cycles: counters.save_load,
        lod_cycles: counters.lod,
        canonical_proposal_round_trips: counters.canonical,
        motor_safety_decisions: counters.motor,
        replayed_cycles: counters.replayed,
        full_motion_outcomes: counters.full,
        clipped_motion_outcomes: counters.clipped,
        blocked_motion_outcomes: counters.blocked,
        fault_no_mutation_outcomes: counters.fault_no_mutation,
        lod_full_projection_probes: counters.lod_full,
        lod_fallback_projection_probes: counters.lod_fallback,
        final_pose,
        final_physics_checkpoint_hash,
        final_command_ledger_hash,
        matrix_digest: content_hash_from_bytes(sha256(&transcript)),
    })
}

fn configure_clipping_fixture(
    fixture: &mut ReferenceGameSession,
) -> Result<(), RootMotionConformanceErrorV1> {
    let base = &fixture.bootstrap.physics_checkpoint.catalog;
    let mut bodies = base.bodies.clone();
    let body = bodies.get_mut(&fixture.physics_body_id).ok_or_else(|| {
        RootMotionConformanceErrorV1::condition("clipping fixture controlled body exists")
    })?;
    body.initial_pose.translation_micrometres[2] = CLIPPING_FIXTURE_START_Z_MICROMETRES;
    let catalog = PhysicsWorldCatalogV1::new(
        base.world_descriptor.world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: base.coordinate_profile.clone(),
            limits: base.limits_profile.clone(),
            solver: base.solver_profile.clone(),
            tick_rate_hash: base.world_descriptor.tick_rate_profile_hash,
            authoritative_numeric_hash: base.world_descriptor.authoritative_numeric_profile_hash,
            quantization_hash: base.world_descriptor.physics_quantization_profile_hash,
        },
        base.materials.clone(),
        bodies,
        base.avatar_bindings.clone(),
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("rebuild clipping fixture catalog", error.to_string())
    })?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        &fixture.bootstrap.tick_rate_profile,
        &fixture.bootstrap.authoritative_numeric_profile,
        &fixture.bootstrap.physics_quantization_profile,
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("rebuild clipping fixture snapshot", error.to_string())
    })?;
    fixture.bootstrap.physics_checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot)
        .map_err(|error| {
            RootMotionConformanceErrorV1::new(
                "validate clipping fixture checkpoint",
                error.to_string(),
            )
        })?;
    Ok(())
}

fn activate_matrix_generation(
    fixture: &ReferenceGameSession,
) -> Result<
    (RuntimeState, PhysicalAnimationOwnerV1, RuntimeReplayDriver),
    RootMotionConformanceErrorV1,
> {
    let runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(fixture),
    )
    .map_err(|error| RootMotionConformanceErrorV1::new("activate runtime", error.to_string()))?;
    let animation = reference_physical_animation_owner(fixture, runtime.physics_snapshot())
        .map_err(|error| {
            RootMotionConformanceErrorV1::new("activate physical animation", error.to_string())
        })?;
    let initial_checkpoint = runtime.world_checkpoint().map_err(|error| {
        RootMotionConformanceErrorV1::new("create replay checkpoint", error.to_string())
    })?;
    let replay = RuntimeReplayDriver::new_with_definitions(
        initial_checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("activate replay driver", error.to_string())
    })?;
    Ok((runtime, animation, replay))
}

fn matrix_case(cycle: u64) -> MatrixCase {
    match cycle % MATRIX_BLOCK_CYCLES {
        0..=3 => MatrixCase::Accepted,
        4 => MatrixCase::Rejected(RootMotionFault::for_rejection((cycle / 10) * 2)),
        5 => MatrixCase::Retry,
        6 => MatrixCase::Rejected(RootMotionFault::for_rejection((cycle / 10) * 2 + 1)),
        7 => MatrixCase::SaveLoad,
        8 | 9 => MatrixCase::Lod,
        _ => unreachable!(),
    }
}

fn fresh_command(
    fixture: &ReferenceGameSession,
    animation: &PhysicalAnimationOwnerV1,
    runtime: &RuntimeState,
    sequence: u64,
    phase: SchemaId,
    fault: Option<RootMotionFault>,
) -> Result<(WorldCommand, ContentHash), RootMotionConformanceErrorV1> {
    let decision = fixture
        .procedural_motor
        .evaluate(runtime.physics_snapshot(), REQUESTED_FORWARD_Q15)
        .map_err(|error| {
            RootMotionConformanceErrorV1::new("evaluate procedural motor", error.to_string())
        })?;
    if decision.applied_direction_q15 != REQUESTED_FORWARD_Q15
        || decision.clamp_mask != 0
        || decision.controller_profile_hash != fixture.procedural_motor.controller_profile_hash()
        || decision.body_projection_root != fixture.procedural_motor.body_projection_root()
        || decision.actuator_safety_root != fixture.procedural_motor.actuator_safety_root()
    {
        return Err(RootMotionConformanceErrorV1::condition(
            "procedural motor admits the exact project-bound forward route",
        ));
    }

    let mut intent = animation
        .root_motion_intent(fixture.body_id, sequence, phase, runtime.physics_snapshot())
        .map_err(|error| {
            RootMotionConformanceErrorV1::new("sample root-motion proposal", error.to_string())
        })?;
    if intent.expected_body_revision != decision.source_body_revision
        || intent.locomotion_profile_hash != fixture.procedural_motor.locomotion_profile_hash()
    {
        return Err(RootMotionConformanceErrorV1::condition(
            "animation proposal closes over the motor decision and locomotion profile",
        ));
    }
    if let Some(fault) = fault {
        apply_fault(&mut intent, fault)?;
    }
    let command = WorldCommand::root_motion(
        fixture.movement_stream_id,
        fixture.principal.clone(),
        sequence,
        runtime.next_tick(),
        fixture.body_id,
        intent,
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("construct root-motion command", error.to_string())
    })?;
    Ok((command, decision.decision_hash))
}

fn apply_fault(
    intent: &mut RootMotionIntentV1,
    fault: RootMotionFault,
) -> Result<(), RootMotionConformanceErrorV1> {
    match fault {
        RootMotionFault::BodyRevision => {
            intent.expected_body_revision = intent
                .expected_body_revision
                .checked_add(1)
                .ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition("fault body revision remains bounded")
                })?;
        }
        RootMotionFault::SourceClip => {
            intent.source_clip_hash = ContentHash::from_bytes([0xa5; 32]);
        }
        RootMotionFault::Interval => {
            intent.interval_us = intent.interval_us.checked_add(2).ok_or_else(|| {
                RootMotionConformanceErrorV1::condition("fault interval remains bounded")
            })?;
        }
        RootMotionFault::Translation => {
            intent.quantized_local_translation[2] = intent.quantized_local_translation[2]
                .checked_add(1)
                .ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition("fault translation remains bounded")
                })?;
        }
        RootMotionFault::AnimationTick => {
            intent.source_animation_tick =
                intent.source_animation_tick.checked_add(1).ok_or_else(|| {
                    RootMotionConformanceErrorV1::condition("fault animation tick remains bounded")
                })?;
            intent.expected_intent_state_revision = intent.source_animation_tick;
        }
        RootMotionFault::LocomotionProfile => {
            intent.locomotion_profile_hash = ContentHash::from_bytes([0xa6; 32]);
        }
    }
    intent.validate().map_err(|error| {
        RootMotionConformanceErrorV1::new(
            "fault remains a canonical untrusted proposal",
            error.to_string(),
        )
    })
}

fn verify_canonical_command(command: &WorldCommand) -> Result<(), RootMotionConformanceErrorV1> {
    let CommandPayload::RootMotion(intent) = &command.payload else {
        return Err(RootMotionConformanceErrorV1::condition(
            "matrix command carries a root-motion proposal",
        ));
    };
    let intent_bytes = intent.canonical_payload_bytes().map_err(|error| {
        RootMotionConformanceErrorV1::new("encode root-motion proposal", error.to_string())
    })?;
    let decoded_intent = RootMotionIntentV1::from_canonical_payload_bytes(
        &intent_bytes,
        CanonicalDecodeLimits::default(),
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("decode root-motion proposal", error.to_string())
    })?;
    if decoded_intent != *intent {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion proposal canonical round trip is exact",
        ));
    }
    let command_bytes = command.canonical_bytes().map_err(|error| {
        RootMotionConformanceErrorV1::new("encode root-motion command", error.to_string())
    })?;
    let decoded_command =
        WorldCommand::from_canonical_bytes(&command_bytes, CanonicalDecodeLimits::default())
            .map_err(|error| {
                RootMotionConformanceErrorV1::new("decode root-motion command", error.to_string())
            })?;
    if decoded_command != *command {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion command canonical round trip is exact",
        ));
    }
    Ok(())
}

fn verify_recorded_command(
    report: &next_runtime::TickReport,
    command: &WorldCommand,
) -> Result<(), RootMotionConformanceErrorV1> {
    if report.command_batches.len() != 2
        || !report.command_batches[0]
            .body
            .envelopes
            .iter()
            .any(|recorded| recorded == command)
    {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion command is present in the closed ingress batch",
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the conformance assertion names every immutable command/physics input explicitly"
)]
fn verify_committed_physics_outcome(
    cycle: u64,
    command_id: next_contracts::ids::CommandId,
    command: &WorldCommand,
    before: &PhysicsBodyStateV2,
    after: &PhysicsBodyStateV2,
    report: &next_runtime::TickReport,
    counters: &mut MatrixCounters,
) -> Result<(), RootMotionConformanceErrorV1> {
    let accepted = report
        .physics_step_input
        .accepted_intents
        .iter()
        .find(|intent| intent.causal_command_id == command_id)
        .ok_or_else(|| {
            RootMotionConformanceErrorV1::condition(format!(
                "cycle {cycle} lowers the proposal into the physical step"
            ))
        })?;
    if accepted.direction_q15 != REQUESTED_FORWARD_Q15
        || report.physics_step_input.accepted_intents.len() != 1
        || after.pose.rotation_q1_30 != before.pose.rotation_q1_30
        || after.pose.translation_micrometres[0] != before.pose.translation_micrometres[0]
        || after.pose.translation_micrometres[1] != before.pose.translation_micrometres[1]
    {
        return Err(RootMotionConformanceErrorV1::condition(format!(
            "cycle {cycle} has one bounded forward Physics outcome"
        )));
    }
    let CommandPayload::RootMotion(intent) = &command.payload else {
        return Err(RootMotionConformanceErrorV1::condition(
            "committed command carries root motion",
        ));
    };
    let displacement = after.pose.translation_micrometres[2]
        .checked_sub(before.pose.translation_micrometres[2])
        .ok_or_else(|| {
            RootMotionConformanceErrorV1::condition("physics displacement remains representable")
        })?;
    let requested = intent.quantized_local_translation[2];
    if displacement < 0 || displacement > requested {
        return Err(RootMotionConformanceErrorV1::condition(format!(
            "cycle {cycle} applies or clips the requested root delta"
        )));
    }
    match displacement {
        value if value == requested => counters.full += 1,
        0 => counters.blocked += 1,
        _ => counters.clipped += 1,
    }
    Ok(())
}

fn restore_direct_generation(
    fixture: &ReferenceGameSession,
    runtime: &mut RuntimeState,
    animation: &mut PhysicalAnimationOwnerV1,
) -> Result<(), RootMotionConformanceErrorV1> {
    let checkpoint = runtime.world_checkpoint().map_err(|error| {
        RootMotionConformanceErrorV1::new("create root-motion checkpoint", error.to_string())
    })?;
    let expected_checkpoint = checkpoint.clone();
    let animation_snapshot = animation.snapshot().clone();
    let restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new("restore root-motion checkpoint", error.to_string())
    })?;
    let restored_checkpoint = restored.world_checkpoint().map_err(|error| {
        RootMotionConformanceErrorV1::new(
            "rebuild restored root-motion checkpoint",
            error.to_string(),
        )
    })?;
    if restored_checkpoint != expected_checkpoint {
        return Err(RootMotionConformanceErrorV1::condition(
            "checkpoint/restore preserves the complete Runtime generation",
        ));
    }
    let restored_animation = restore_reference_physical_animation_owner(
        fixture,
        animation_snapshot.clone(),
        restored.physics_snapshot(),
        restored.next_tick(),
    )
    .map_err(|error| {
        RootMotionConformanceErrorV1::new(
            "restore physical-animation checkpoint",
            error.to_string(),
        )
    })?;
    if restored_animation.snapshot() != &animation_snapshot {
        return Err(RootMotionConformanceErrorV1::condition(
            "checkpoint/restore preserves the animation intent cursor",
        ));
    }
    *runtime = restored;
    *animation = restored_animation;
    Ok(())
}

fn verify_lod_isolation(
    fixture: &ReferenceGameSession,
    animation: &PhysicalAnimationOwnerV1,
    physics: &next_contracts::physics::PhysicsCanonicalSnapshotV2,
) -> Result<(), RootMotionConformanceErrorV1> {
    let animation_before = animation.snapshot().clone();
    let body_before = controlled_body(fixture, physics)?.clone();
    let full = animation
        .pose(
            fixture.body_id,
            physics,
            Some(0),
            PhysicalAnimationPresentationAvailabilityV1::FULL,
        )
        .map_err(|error| {
            RootMotionConformanceErrorV1::new("project full animation LOD", error.to_string())
        })?;
    let fallback = animation
        .pose(
            fixture.body_id,
            physics,
            Some(0),
            PhysicalAnimationPresentationAvailabilityV1::BIND_POSE_FALLBACK,
        )
        .map_err(|error| {
            RootMotionConformanceErrorV1::new("project fallback animation LOD", error.to_string())
        })?;
    let body_after = controlled_body(fixture, physics)?;
    if full.projection_mode == PhysicalAnimationProjectionModeV1::BindPoseFallback
        || fallback.projection_mode != PhysicalAnimationProjectionModeV1::BindPoseFallback
        || full.skeleton_root_pose != fallback.skeleton_root_pose
        || full.skeleton_root_pose.rotation_q1_30 != body_before.pose.rotation_q1_30
        || animation.snapshot() != &animation_before
        || body_after != &body_before
    {
        return Err(RootMotionConformanceErrorV1::condition(
            "animation LOD projection is reconstructible and has no pose authority",
        ));
    }
    Ok(())
}

fn controlled_body<'a>(
    fixture: &ReferenceGameSession,
    physics: &'a next_contracts::physics::PhysicsCanonicalSnapshotV2,
) -> Result<&'a PhysicsBodyStateV2, RootMotionConformanceErrorV1> {
    physics
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| RootMotionConformanceErrorV1::condition("controlled Physics body exists"))
}

fn finalized_receipt_count(
    fixture: &ReferenceGameSession,
    report: &next_runtime::TickReport,
) -> Result<u64, RootMotionConformanceErrorV1> {
    report
        .snapshot
        .command_ledger
        .streams
        .get(&fixture.movement_stream_id)
        .map(|stream| stream.finalized_receipt_count)
        .ok_or_else(|| {
            RootMotionConformanceErrorV1::condition("movement command stream receipt exists")
        })
}

fn verify_matrix_counts(
    cycles: u64,
    counters: &MatrixCounters,
) -> Result<(), RootMotionConformanceErrorV1> {
    let blocks = cycles / MATRIX_BLOCK_CYCLES;
    if counters.accepted != blocks * 4
        || counters.rejected != blocks * 2
        || counters.retried != blocks
        || counters.save_load != blocks
        || counters.lod != blocks * 2
        || counters.canonical != cycles
        || counters.motor != cycles - counters.retried
        || counters.replayed != cycles
        || counters.fault_no_mutation != counters.rejected + counters.retried
        || counters.lod_full != counters.lod
        || counters.lod_fallback != counters.lod
        || counters.full + counters.clipped + counters.blocked
            != counters.accepted + counters.save_load + counters.lod
        || counters.full == 0
        || counters.clipped == 0
        || counters.blocked == 0
    {
        return Err(RootMotionConformanceErrorV1::condition(
            "root-motion matrix has the exact declared category and outcome counts",
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the digest binds the complete per-cycle immutable conformance observation"
)]
fn append_transcript_record(
    transcript: &mut Vec<u8>,
    cycle: u64,
    matrix_case: MatrixCase,
    command: &WorldCommand,
    disposition: &CommandDisposition,
    body: &PhysicsBodyStateV2,
    physics_checkpoint_hash: ContentHash,
    motor_decision_hash: Option<ContentHash>,
) {
    transcript.extend_from_slice(&cycle.to_le_bytes());
    transcript.push(matrix_case.tag());
    if let MatrixCase::Rejected(fault) = matrix_case {
        transcript.push(fault as u8);
    } else {
        transcript.push(0);
    }
    transcript.extend_from_slice(
        command
            .claimed_command_id
            .expect("canonical matrix command has an identity")
            .as_bytes(),
    );
    transcript.extend_from_slice(disposition_code(disposition).as_bytes());
    transcript.push(0);
    for value in body.pose.translation_micrometres {
        transcript.extend_from_slice(&value.to_le_bytes());
    }
    transcript.extend_from_slice(&body.body_revision.to_le_bytes());
    transcript.extend_from_slice(physics_checkpoint_hash.as_bytes());
    transcript.extend_from_slice(motor_decision_hash.unwrap_or_default().as_bytes());
}

fn disposition_code(disposition: &CommandDisposition) -> &'static str {
    match disposition {
        CommandDisposition::Reserved => "RESERVED",
        CommandDisposition::Committed => "COMMITTED",
        CommandDisposition::Deduplicated => "DEDUPLICATED",
        CommandDisposition::Rejected(code) => code.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_matrix_has_the_declared_ten_thousand_cycle_partition() {
        let counts = (0..ROOT_MOTION_CONFORMANCE_CYCLES).fold([0_u64; 5], |mut counts, cycle| {
            let index = match matrix_case(cycle) {
                MatrixCase::Accepted => 0,
                MatrixCase::Rejected(_) => 1,
                MatrixCase::Retry => 2,
                MatrixCase::SaveLoad => 3,
                MatrixCase::Lod => 4,
            };
            counts[index] += 1;
            counts
        });
        assert_eq!(counts, [4_000, 2_000, 1_000, 1_000, 2_000]);
    }

    #[test]
    fn production_matrix_smoke_crosses_every_boundary_and_fault_kind() {
        let report = run_root_motion_conformance_cycles(30).expect("root-motion matrix smoke");
        assert_eq!(report.cycles, 30);
        assert_eq!(report.accepted_cycles, 12);
        assert_eq!(report.rejected_cycles, 6);
        assert_eq!(report.retried_cycles, 3);
        assert_eq!(report.save_load_cycles, 3);
        assert_eq!(report.lod_cycles, 6);
        assert_eq!(report.canonical_proposal_round_trips, 30);
        assert_eq!(report.motor_safety_decisions, 27);
        assert_eq!(report.replayed_cycles, 30);
        assert_eq!(report.fault_no_mutation_outcomes, 9);
    }
}
