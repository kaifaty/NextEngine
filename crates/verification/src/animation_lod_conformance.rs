use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::WorldCommand;
use next_contracts::ids::{
    CommandLedgerHash, ContentHash, SchemaId, StateRoot, content_hash_from_bytes,
};
use next_contracts::physical_animation::{ROOT_MOTION_MOVE_PERFORMED_PHASE_ID, RootMotionIntentV1};
use next_contracts::presentation::{
    CharacterDeformationLodV1, CharacterSkinningPresentationRecordV1,
    QuantizedPresentationTransformV1, ScenePresentationFlagsV1,
};
use next_contracts::project::domain_hash;
use next_motor::{
    PhysicalAnimationLodDecisionV1, PhysicalAnimationLodLevelV1, PhysicalAnimationLodProfileV1,
    PhysicalAnimationLodProjectionV1, PhysicalAnimationLodPublicationModeV1,
    PhysicalAnimationLodRequestV1, PhysicalAnimationLodResourcesV1, PhysicalAnimationOwnerV1,
};
use next_presentation::{PresentationBindingV1, PresentationExtractorV1};
use next_reference_game::{
    ReferenceGameSession, cooked_project_rpg_snapshot, reference_b0_presentation_profile_hash,
    reference_character_skinning_record_from_lod_projection, reference_physical_animation_owner,
};
use next_render::{RenderTargetV1, build_b0_frame_plan};
use next_runtime::RuntimeState;

use crate::build_neutral_player_fixture;

pub const ANIMATION_LOD_CONFORMANCE_CYCLES: u64 = 10_000;
const MATRIX_BLOCK_CYCLES: u64 = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimationLodConformanceReportV1 {
    pub cycles: u64,
    pub full_pose_requests: u64,
    pub reduced_pose_requests: u64,
    pub held_pose_requests: u64,
    pub intent_only_requests: u64,
    pub culled_pose_requests: u64,
    pub sampled_pose_projections: u64,
    pub held_pose_projections: u64,
    pub bind_pose_projections: u64,
    pub no_pose_projections: u64,
    pub complete_snapshot_publications: u64,
    pub rejected_snapshot_publications: u64,
    pub due_intent_evaluations: u64,
    pub resource_fallbacks: u64,
    pub authoritative_isolation_checks: u64,
    pub renderer_frame_plans: u64,
    pub lod_profile_revision: ContentHash,
    pub final_state_root: StateRoot,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_physics_checkpoint_hash: ContentHash,
    pub final_animation_snapshot_hash: ContentHash,
    pub final_presentation_snapshot_hash: ContentHash,
    pub final_frame_plan_hash: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimationLodConformanceErrorV1 {
    context: String,
    detail: String,
}

impl AnimationLodConformanceErrorV1 {
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

impl Display for AnimationLodConformanceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for AnimationLodConformanceErrorV1 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum MatrixCase {
    FullSample = 1,
    ReducedCadenceHold = 2,
    ReducedSample = 3,
    RequestedHold = 4,
    IntentOnly = 5,
    RequestedCull = 6,
    ClipFaultHold = 7,
    CadenceBindFallback = 8,
    ResourceCullFallback = 9,
    PublicationFault = 10,
}

impl MatrixCase {
    const fn request(self) -> PhysicalAnimationLodRequestV1 {
        match self {
            Self::FullSample | Self::PublicationFault => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::FullPose,
                resources: PhysicalAnimationLodResourcesV1::FULL,
            },
            Self::ReducedCadenceHold | Self::ReducedSample => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::ReducedPose,
                resources: PhysicalAnimationLodResourcesV1::FULL,
            },
            Self::RequestedHold => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::HeldPresentationPose,
                resources: PhysicalAnimationLodResourcesV1::FULL,
            },
            Self::IntentOnly => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::IntentOnly,
                resources: PhysicalAnimationLodResourcesV1::FULL,
            },
            Self::RequestedCull => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::CulledPresentation,
                resources: PhysicalAnimationLodResourcesV1::FULL,
            },
            Self::ClipFaultHold | Self::CadenceBindFallback => PhysicalAnimationLodRequestV1 {
                requested_lod: if matches!(self, Self::CadenceBindFallback) {
                    PhysicalAnimationLodLevelV1::ReducedPose
                } else {
                    PhysicalAnimationLodLevelV1::FullPose
                },
                resources: PhysicalAnimationLodResourcesV1::CLIP_UNAVAILABLE,
            },
            Self::ResourceCullFallback => PhysicalAnimationLodRequestV1 {
                requested_lod: PhysicalAnimationLodLevelV1::FullPose,
                resources: PhysicalAnimationLodResourcesV1::NO_POSE,
            },
        }
    }

    const fn expected_decision(self) -> PhysicalAnimationLodDecisionV1 {
        match self {
            Self::FullSample | Self::ReducedSample | Self::PublicationFault => {
                PhysicalAnimationLodDecisionV1::Sampled
            }
            Self::ReducedCadenceHold => PhysicalAnimationLodDecisionV1::ReducedCadenceHeld,
            Self::RequestedHold => PhysicalAnimationLodDecisionV1::RequestedHeld,
            Self::IntentOnly => PhysicalAnimationLodDecisionV1::IntentOnly,
            Self::RequestedCull => PhysicalAnimationLodDecisionV1::RequestedCull,
            Self::ClipFaultHold => PhysicalAnimationLodDecisionV1::ClipUnavailableHeld,
            Self::CadenceBindFallback => PhysicalAnimationLodDecisionV1::ReducedCadenceBind,
            Self::ResourceCullFallback => PhysicalAnimationLodDecisionV1::ClipUnavailableCull,
        }
    }

    const fn uses_previous_pose(self) -> bool {
        matches!(
            self,
            Self::ReducedCadenceHold
                | Self::ReducedSample
                | Self::RequestedHold
                | Self::ClipFaultHold
        )
    }

    const fn is_resource_fallback(self) -> bool {
        matches!(
            self,
            Self::ClipFaultHold | Self::CadenceBindFallback | Self::ResourceCullFallback
        )
    }

    const fn rejects_publication(self) -> bool {
        matches!(self, Self::PublicationFault)
    }
}

#[derive(Default)]
struct MatrixCounters {
    requests: [u64; 5],
    sampled: u64,
    held: u64,
    bind: u64,
    no_pose: u64,
    published: u64,
    rejected: u64,
    due_intent: u64,
    resource_fallbacks: u64,
    isolation: u64,
    frame_plans: u64,
}

struct MatrixGeneration {
    runtime: RuntimeState,
    animation: PhysicalAnimationOwnerV1,
    presentation: PresentationExtractorV1,
    last_published_pose: Option<PhysicalAnimationLodProjectionV1>,
}

pub fn run_animation_lod_conformance_check()
-> Result<AnimationLodConformanceReportV1, AnimationLodConformanceErrorV1> {
    run_animation_lod_conformance_cycles(ANIMATION_LOD_CONFORMANCE_CYCLES)
}

fn run_animation_lod_conformance_cycles(
    cycles: u64,
) -> Result<AnimationLodConformanceReportV1, AnimationLodConformanceErrorV1> {
    if cycles == 0 || !cycles.is_multiple_of(MATRIX_BLOCK_CYCLES) {
        return Err(AnimationLodConformanceErrorV1::condition(
            "animation LOD matrix cycle count is a positive multiple of ten",
        ));
    }
    let fixture =
        build_neutral_player_fixture("nextengine.animation-lod-conformance").map_err(|error| {
            AnimationLodConformanceErrorV1::new("activate fixture", error.to_string())
        })?;
    let phase = SchemaId::new(ROOT_MOTION_MOVE_PERFORMED_PHASE_ID).map_err(|error| {
        AnimationLodConformanceErrorV1::new("root-motion phase", error.to_string())
    })?;
    let lod_profile = PhysicalAnimationLodProfileV1::REFERENCE_R5;
    let mut generation = None;
    let mut counters = MatrixCounters::default();
    let mut transcript = b"nextengine.anim-lod-p1.matrix.v1\0".to_vec();
    let mut final_command_ledger_hash = CommandLedgerHash::default();
    let mut final_physics_checkpoint_hash = ContentHash::default();
    let mut final_animation_snapshot_hash = ContentHash::default();
    let mut final_presentation_snapshot_hash = ContentHash::default();
    let mut final_frame_plan_hash = ContentHash::default();

    for cycle in 0..cycles {
        if cycle.is_multiple_of(MATRIX_BLOCK_CYCLES) {
            generation = Some(activate_generation(&fixture)?);
        }
        let generation = generation
            .as_mut()
            .expect("every matrix block activates a generation");
        let matrix_case = matrix_case(cycle);
        let request = matrix_case.request();
        increment_request_counter(&mut counters, request.requested_lod);
        counters.resource_fallbacks += u64::from(matrix_case.is_resource_fallback());

        let runtime_before = generation.runtime.snapshot();
        let rpg_before = generation.runtime.rpg_snapshot();
        let physics_before = generation.runtime.physics_snapshot().clone();
        let animation_before = generation.animation.snapshot().clone();
        let intent_sequence = cycle % MATRIX_BLOCK_CYCLES + 1;
        let intent_before = generation
            .animation
            .root_motion_intent(
                fixture.body_id,
                intent_sequence,
                phase.clone(),
                &physics_before,
            )
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    format!("evaluate due intent before cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        let previous = matrix_case
            .uses_previous_pose()
            .then_some(generation.last_published_pose.as_ref())
            .flatten();
        let projection = generation
            .animation
            .project_pose_lod(
                fixture.body_id,
                &physics_before,
                Some(0),
                lod_profile,
                request,
                previous,
            )
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    format!("project animation LOD cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        if projection.decision() != matrix_case.expected_decision()
            || projection.logical_animation_tick() != generation.runtime.next_tick()
            || projection.source_physics_tick() != physics_before.physics_tick
            || projection.lod_profile_revision() != lod_profile.revision()
        {
            return Err(AnimationLodConformanceErrorV1::condition(format!(
                "cycle {cycle} selects the declared LOD decision"
            )));
        }
        increment_projection_counter(&mut counters, projection.publication_mode());

        let publication = publish_projection(
            &fixture,
            &mut generation.presentation,
            &physics_before,
            &projection,
            matrix_case.rejects_publication(),
        )?;
        if publication.published {
            counters.published += 1;
            if projection.pose().is_some() {
                generation.last_published_pose = Some(projection.clone());
            }
        } else {
            counters.rejected += 1;
        }
        let render = verify_renderer_repetition(
            cycle,
            generation.presentation.accepted_snapshot().ok_or_else(|| {
                AnimationLodConformanceErrorV1::condition(
                    "the matrix has an accepted snapshot before rendering",
                )
            })?,
            &fixture,
            projection.pose().is_some() && publication.published,
        )?;
        counters.frame_plans += render.frame_plan_count;

        let intent_after = generation
            .animation
            .root_motion_intent(
                fixture.body_id,
                intent_sequence,
                phase.clone(),
                generation.runtime.physics_snapshot(),
            )
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    format!("evaluate due intent after cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        if intent_after != intent_before
            || generation.runtime.snapshot() != runtime_before
            || generation.runtime.rpg_snapshot() != rpg_before
            || generation.runtime.physics_snapshot() != &physics_before
            || generation.animation.snapshot() != &animation_before
        {
            return Err(AnimationLodConformanceErrorV1::condition(format!(
                "cycle {cycle} preserves every authoritative root and due intent"
            )));
        }
        counters.due_intent += 1;
        counters.isolation += 1;

        let report = generation
            .runtime
            .run_tick(Vec::<WorldCommand>::new())
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    format!("advance authoritative tick after cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        if !report.results.is_empty() || !report.physics_step_input.accepted_intents.is_empty() {
            return Err(AnimationLodConformanceErrorV1::condition(format!(
                "cycle {cycle} injects no presentation command or physical intent"
            )));
        }
        generation
            .animation
            .advance(
                &physics_before,
                generation.runtime.physics_snapshot(),
                generation.runtime.next_tick(),
            )
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    format!("advance physical animation after cycle {cycle}"),
                    error.to_string(),
                )
            })?;
        final_command_ledger_hash = report.snapshot.command_ledger_hash().map_err(|error| {
            AnimationLodConformanceErrorV1::new("command ledger hash", error.to_string())
        })?;
        final_physics_checkpoint_hash = report.physics_checkpoint_hash;
        final_animation_snapshot_hash = content_hash_from_bytes(sha256(
            &generation
                .animation
                .snapshot()
                .canonical_bytes()
                .map_err(|error| {
                    AnimationLodConformanceErrorV1::new(
                        "animation snapshot bytes",
                        error.to_string(),
                    )
                })?,
        ));
        final_presentation_snapshot_hash = publication.snapshot_hash;
        final_frame_plan_hash = render.frame_plan_hash;
        append_transcript_record(
            &mut transcript,
            cycle,
            matrix_case,
            request,
            &projection,
            &intent_before,
            publication,
            render,
            report.physics_checkpoint_hash,
            final_animation_snapshot_hash,
            final_command_ledger_hash,
        )?;
    }

    verify_matrix_counts(cycles, &counters)?;
    let generation = generation.expect("positive cycle count activates a generation");
    let final_state_root = generation
        .runtime
        .world_checkpoint()
        .map_err(|error| {
            AnimationLodConformanceErrorV1::new("final world checkpoint", error.to_string())
        })?
        .state_root;
    Ok(AnimationLodConformanceReportV1 {
        cycles,
        full_pose_requests: counters.requests[0],
        reduced_pose_requests: counters.requests[1],
        held_pose_requests: counters.requests[2],
        intent_only_requests: counters.requests[3],
        culled_pose_requests: counters.requests[4],
        sampled_pose_projections: counters.sampled,
        held_pose_projections: counters.held,
        bind_pose_projections: counters.bind,
        no_pose_projections: counters.no_pose,
        complete_snapshot_publications: counters.published,
        rejected_snapshot_publications: counters.rejected,
        due_intent_evaluations: counters.due_intent,
        resource_fallbacks: counters.resource_fallbacks,
        authoritative_isolation_checks: counters.isolation,
        renderer_frame_plans: counters.frame_plans,
        lod_profile_revision: lod_profile.revision(),
        final_state_root,
        final_command_ledger_hash,
        final_physics_checkpoint_hash,
        final_animation_snapshot_hash,
        final_presentation_snapshot_hash,
        final_frame_plan_hash,
        matrix_digest: content_hash_from_bytes(sha256(&transcript)),
    })
}

fn activate_generation(
    fixture: &ReferenceGameSession,
) -> Result<MatrixGeneration, AnimationLodConformanceErrorV1> {
    let runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(fixture),
    )
    .map_err(|error| {
        AnimationLodConformanceErrorV1::new("activate Runtime generation", error.to_string())
    })?;
    let animation = reference_physical_animation_owner(fixture, runtime.physics_snapshot())
        .map_err(|error| {
            AnimationLodConformanceErrorV1::new(
                "activate physical-animation generation",
                error.to_string(),
            )
        })?;
    let snapshot_epoch = domain_hash(
        "nextengine.animation-lod-conformance-epoch.v1",
        fixture
            .activated_project
            .project_lock
            .project_lock_sha256
            .as_bytes(),
    );
    let presentation = PresentationExtractorV1::new_with_snapshot_epoch_and_batch_limits(
        snapshot_epoch,
        reference_b0_presentation_profile_hash(),
        8,
        1,
    )
    .map_err(|error| {
        AnimationLodConformanceErrorV1::new("activate presentation generation", error.to_string())
    })?;
    Ok(MatrixGeneration {
        runtime,
        animation,
        presentation,
        last_published_pose: None,
    })
}

fn matrix_case(cycle: u64) -> MatrixCase {
    match cycle % MATRIX_BLOCK_CYCLES {
        0 => MatrixCase::FullSample,
        1 => MatrixCase::ReducedCadenceHold,
        2 => MatrixCase::ReducedSample,
        3 => MatrixCase::RequestedHold,
        4 => MatrixCase::IntentOnly,
        5 => MatrixCase::RequestedCull,
        6 => MatrixCase::ClipFaultHold,
        7 => MatrixCase::CadenceBindFallback,
        8 => MatrixCase::ResourceCullFallback,
        9 => MatrixCase::PublicationFault,
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy)]
struct PublicationObservation {
    published: bool,
    snapshot_sequence: u64,
    snapshot_hash: ContentHash,
}

fn publish_projection(
    fixture: &ReferenceGameSession,
    extractor: &mut PresentationExtractorV1,
    physics: &next_contracts::physics::PhysicsCanonicalSnapshotV2,
    projection: &PhysicalAnimationLodProjectionV1,
    inject_fault: bool,
) -> Result<PublicationObservation, AnimationLodConformanceErrorV1> {
    let mut records = reference_character_skinning_record_from_lod_projection(
        fixture,
        extractor.snapshot_epoch(),
        projection,
    )
    .map_err(|error| {
        AnimationLodConformanceErrorV1::new("compose character skinning record", error.to_string())
    })?
    .into_iter()
    .collect::<Vec<_>>();
    let bindings = records
        .first()
        .map(|record| presentation_binding(fixture, record))
        .transpose()?
        .into_iter()
        .collect::<Vec<_>>();
    let prior = extractor
        .accepted_snapshot()
        .map(|snapshot| (snapshot.snapshot_sequence, snapshot.canonical_hash));
    if inject_fault {
        let record = records.first().ok_or_else(|| {
            AnimationLodConformanceErrorV1::condition(
                "publication fault starts from a complete pose record",
            )
        })?;
        records = vec![
            CharacterSkinningPresentationRecordV1::new(
                record.object_key,
                record.mesh_revision,
                record.skinning_profile_revision,
                record.source_skeleton_revision,
                record.source_body_schema_revision,
                record.source_animation_profile_hash,
                record.projection_mode,
                CharacterDeformationLodV1::Culled,
                record.ordered_local_joint_poses.clone(),
            )
            .map_err(|error| {
                AnimationLodConformanceErrorV1::new(
                    "construct valid faulted skinning record",
                    error.to_string(),
                )
            })?,
        ];
    }
    let result = extractor.extract_with_character_skinning(
        projection.logical_animation_tick(),
        fixture.activated_project.project_lock.project_lock_sha256,
        fixture
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        physics,
        &bindings,
        &[],
        Vec::new(),
        records,
    );
    if inject_fault {
        if result.is_ok() {
            return Err(AnimationLodConformanceErrorV1::condition(
                "faulted scene/skinning closure rejects atomically",
            ));
        }
        let accepted = extractor.accepted_snapshot().ok_or_else(|| {
            AnimationLodConformanceErrorV1::condition(
                "publication fault retains the prior complete snapshot",
            )
        })?;
        if prior != Some((accepted.snapshot_sequence, accepted.canonical_hash)) {
            return Err(AnimationLodConformanceErrorV1::condition(
                "publication fault retains exact prior sequence and hash",
            ));
        }
        accepted.validate().map_err(|error| {
            AnimationLodConformanceErrorV1::new(
                "validate retained presentation snapshot",
                error.to_string(),
            )
        })?;
        return Ok(PublicationObservation {
            published: false,
            snapshot_sequence: accepted.snapshot_sequence,
            snapshot_hash: accepted.canonical_hash,
        });
    }
    let accepted = result.map_err(|error| {
        AnimationLodConformanceErrorV1::new("publish complete LOD snapshot", error.to_string())
    })?;
    accepted.validate().map_err(|error| {
        AnimationLodConformanceErrorV1::new("validate complete LOD snapshot", error.to_string())
    })?;
    Ok(PublicationObservation {
        published: true,
        snapshot_sequence: accepted.snapshot_sequence,
        snapshot_hash: accepted.canonical_hash,
    })
}

fn presentation_binding(
    fixture: &ReferenceGameSession,
    record: &CharacterSkinningPresentationRecordV1,
) -> Result<PresentationBindingV1, AnimationLodConformanceErrorV1> {
    let catalog = &fixture.activated_project.render_content_catalog;
    let mesh = catalog.mesh(record.mesh_revision).ok_or_else(|| {
        AnimationLodConformanceErrorV1::condition("LOD skinning mesh exists in exact catalog")
    })?;
    let material_revision = catalog
        .materials()
        .first()
        .and_then(|material| material.asset_revision().ok())
        .ok_or_else(|| {
            AnimationLodConformanceErrorV1::condition("LOD scene material exists in exact catalog")
        })?;
    Ok(PresentationBindingV1 {
        persistent_id: record.object_key.persistent_id,
        presentation_role: record.object_key.presentation_role,
        incarnation: record.object_key.incarnation,
        presentation_layer: 1,
        mesh_revision: record.mesh_revision,
        material_revision,
        instance_ordinal: 0,
        local_bounds: mesh.bounds(),
        feature_flags: ScenePresentationFlagsV1::SKINNED,
        physics_body_id: Some(fixture.physics_body_id),
        fallback_transform: QuantizedPresentationTransformV1::default(),
        visible: true,
    })
}

#[derive(Clone, Copy)]
struct RenderObservation {
    cadence_hz: u32,
    target_revision: u64,
    frame_plan_count: u64,
    frame_plan_hash: ContentHash,
}

fn verify_renderer_repetition(
    cycle: u64,
    snapshot: &next_contracts::presentation::PresentationSnapshotV3,
    fixture: &ReferenceGameSession,
    new_pose_published: bool,
) -> Result<RenderObservation, AnimationLodConformanceErrorV1> {
    let cadence_hz: u32 = match cycle % 3 {
        0 => 30,
        1 => 60,
        _ => 144,
    };
    let target = RenderTargetV1 {
        extent: [1_920, 1_080],
        target_revision: cycle % 3 + 1,
    };
    let repeats = u64::from(cadence_hz.div_ceil(30));
    let first = build_b0_frame_plan(
        snapshot,
        &fixture.activated_project.render_content_catalog,
        target,
    )
    .map_err(|error| {
        AnimationLodConformanceErrorV1::new("build B0 animation LOD frame", error.to_string())
    })?;
    let expected_streams = usize::from(new_pose_published);
    if first.skinned_vertex_streams.len() != expected_streams {
        return Err(AnimationLodConformanceErrorV1::condition(format!(
            "cycle {cycle} renders the declared complete pose count"
        )));
    }
    for _ in 1..repeats {
        let repeated = build_b0_frame_plan(
            snapshot,
            &fixture.activated_project.render_content_catalog,
            target,
        )
        .map_err(|error| {
            AnimationLodConformanceErrorV1::new(
                "repeat newest complete B0 animation LOD frame",
                error.to_string(),
            )
        })?;
        if repeated != first {
            return Err(AnimationLodConformanceErrorV1::condition(format!(
                "cycle {cycle} repeats one exact complete frame at {cadence_hz} Hz"
            )));
        }
    }
    Ok(RenderObservation {
        cadence_hz,
        target_revision: target.target_revision,
        frame_plan_count: repeats,
        frame_plan_hash: first.frame_plan_hash,
    })
}

fn increment_request_counter(counters: &mut MatrixCounters, level: PhysicalAnimationLodLevelV1) {
    let index = match level {
        PhysicalAnimationLodLevelV1::FullPose => 0,
        PhysicalAnimationLodLevelV1::ReducedPose => 1,
        PhysicalAnimationLodLevelV1::HeldPresentationPose => 2,
        PhysicalAnimationLodLevelV1::IntentOnly => 3,
        PhysicalAnimationLodLevelV1::CulledPresentation => 4,
    };
    counters.requests[index] += 1;
}

fn increment_projection_counter(
    counters: &mut MatrixCounters,
    mode: PhysicalAnimationLodPublicationModeV1,
) {
    match mode {
        PhysicalAnimationLodPublicationModeV1::Sampled => counters.sampled += 1,
        PhysicalAnimationLodPublicationModeV1::HeldPresentationPose => counters.held += 1,
        PhysicalAnimationLodPublicationModeV1::BindPoseFallback => counters.bind += 1,
        PhysicalAnimationLodPublicationModeV1::NoPose => counters.no_pose += 1,
    }
}

fn verify_matrix_counts(
    cycles: u64,
    counters: &MatrixCounters,
) -> Result<(), AnimationLodConformanceErrorV1> {
    let blocks = cycles / MATRIX_BLOCK_CYCLES;
    if counters.requests != [blocks * 4, blocks * 3, blocks, blocks, blocks]
        || counters.sampled != blocks * 3
        || counters.held != blocks * 3
        || counters.bind != blocks
        || counters.no_pose != blocks * 3
        || counters.published != blocks * 9
        || counters.rejected != blocks
        || counters.due_intent != cycles
        || counters.resource_fallbacks != blocks * 3
        || counters.isolation != cycles
        || counters.frame_plans != expected_frame_plan_count(cycles)
    {
        return Err(AnimationLodConformanceErrorV1::condition(
            "animation LOD matrix has the exact declared category counts",
        ));
    }
    Ok(())
}

fn expected_frame_plan_count(cycles: u64) -> u64 {
    (0..cycles)
        .map(|cycle| match cycle % 3 {
            0 => 1,
            1 => 2,
            _ => 5,
        })
        .sum()
}

#[allow(
    clippy::too_many_arguments,
    reason = "the digest binds every immutable per-cycle conformance observation"
)]
fn append_transcript_record(
    transcript: &mut Vec<u8>,
    cycle: u64,
    matrix_case: MatrixCase,
    request: PhysicalAnimationLodRequestV1,
    projection: &PhysicalAnimationLodProjectionV1,
    intent: &RootMotionIntentV1,
    publication: PublicationObservation,
    render: RenderObservation,
    physics_checkpoint_hash: ContentHash,
    animation_snapshot_hash: ContentHash,
    ledger_hash: CommandLedgerHash,
) -> Result<(), AnimationLodConformanceErrorV1> {
    transcript.extend_from_slice(&cycle.to_le_bytes());
    transcript.push(matrix_case as u8);
    transcript.push(request.requested_lod as u8);
    transcript.push(u8::from(request.resources.clip_sampling_available));
    transcript.push(u8::from(request.resources.foot_ik_available));
    transcript.push(u8::from(request.resources.bind_pose_available));
    transcript.push(projection.decision() as u8);
    transcript.push(projection.publication_mode() as u8);
    transcript.extend_from_slice(
        &projection
            .pose_source_animation_tick()
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    let intent_bytes = intent.canonical_payload_bytes().map_err(|error| {
        AnimationLodConformanceErrorV1::new("canonical due intent", error.to_string())
    })?;
    let decoded = RootMotionIntentV1::from_canonical_payload_bytes(
        &intent_bytes,
        CanonicalDecodeLimits::default(),
    )
    .map_err(|error| {
        AnimationLodConformanceErrorV1::new("decode canonical due intent", error.to_string())
    })?;
    if &decoded != intent {
        return Err(AnimationLodConformanceErrorV1::condition(
            "due intent canonical round trip is exact",
        ));
    }
    transcript.extend_from_slice(&sha256(&intent_bytes));
    transcript.push(u8::from(publication.published));
    transcript.extend_from_slice(&publication.snapshot_sequence.to_le_bytes());
    transcript.extend_from_slice(publication.snapshot_hash.as_bytes());
    transcript.extend_from_slice(&render.cadence_hz.to_le_bytes());
    transcript.extend_from_slice(&render.target_revision.to_le_bytes());
    transcript.extend_from_slice(&render.frame_plan_count.to_le_bytes());
    transcript.extend_from_slice(render.frame_plan_hash.as_bytes());
    transcript.extend_from_slice(physics_checkpoint_hash.as_bytes());
    transcript.extend_from_slice(animation_snapshot_hash.as_bytes());
    transcript.extend_from_slice(ledger_hash.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_matrix_has_the_declared_ten_thousand_cycle_partition() {
        let mut counts = [0_u64; 10];
        for cycle in 0..ANIMATION_LOD_CONFORMANCE_CYCLES {
            counts[usize::from(matrix_case(cycle) as u8 - 1)] += 1;
        }
        assert_eq!(counts, [1_000; 10]);
        assert_eq!(
            expected_frame_plan_count(ANIMATION_LOD_CONFORMANCE_CYCLES),
            26_665
        );
    }

    #[test]
    fn production_matrix_smoke_is_exactly_reproducible() {
        let first = run_animation_lod_conformance_cycles(10).expect("first LOD matrix block");
        let second = run_animation_lod_conformance_cycles(10).expect("second LOD matrix block");
        assert_eq!(first, second);
        assert_eq!(first.cycles, 10);
        assert_eq!(first.complete_snapshot_publications, 9);
        assert_eq!(first.rejected_snapshot_publications, 1);
        assert_eq!(first.due_intent_evaluations, 10);
        assert_eq!(first.resource_fallbacks, 3);
        assert_eq!(first.renderer_frame_plans, 25);
    }
}
