//! `CONTINUUM-WATER-VOLUME-P1` (ADR-100): the reference basin is an exact,
//! command-driven, save/replay-bound `WaterVolume` whose submersion queries
//! never read presentation state.

use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::identity::{PrincipalRecordV1, PrincipalStatus};
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ContentHash, PersistentId, SchemaId, StateRoot, ToolPrincipalId,
    content_hash_from_bytes,
};
use next_contracts::physics::{
    PhysicalCommandV1, PhysicsWorldCheckpointV1, WATER_VOLUME_CAPABILITY_ID,
    WaterSubmersionClassV1, WaterVolumeCommandV1, WaterVolumeSetV1,
};
use next_reference_game::{
    REFERENCE_WATER_BASIN_ID, REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
    ReferenceGameSession, cooked_project_rpg_snapshot, player_submersion,
    reference_water_basin_definition, water_surface_translation,
};
use next_runtime::{CommandDisposition, RejectionCode, RuntimeState};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

const TOOL_PRINCIPAL_ID: &str = "nextengine.tool.water-volume-check";
const RAISED_LEVEL_MICROMETRES: i64 = 1_500_000;
const LOWERED_LEVEL_MICROMETRES: i64 = 800_000;
const OUT_OF_EXTENT_LEVEL_MICROMETRES: i64 = 3_000_000;
const UNKNOWN_VOLUME_ID: PersistentId = PersistentId::from_bytes([0x7b; 16]);

/// Authored probe points inside and around the reference basin.
const PROBES: [(&str, [i64; 3]); 5] = [
    ("basin_floor_centre", [6_500_000, 0, 2_000_000]),
    ("basin_knee", [6_500_000, 300_000, 2_000_000]),
    (
        "basin_above_initial_surface",
        [6_500_000, 600_000, 2_000_000],
    ),
    ("basin_low_corner", [4_500_000, 0, 1_000_000]),
    ("outside_basin", [2_000_000, 0, 0]),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterProbeResultV1 {
    pub label: String,
    pub point_micrometres: [i64; 3],
    pub in_volume: bool,
    pub depth_micrometres: i64,
    pub class: WaterSubmersionClassV1,
}

/// Player walk into the basin: `-z`, then `+x`, then `+z`, each leg one
/// axial locomotion intent per tick at the production capsule speed.
const WALK_LEGS: [([i16; 2], u64); 3] = [([0, -32767], 5), ([32767, 0], 65), ([0, 32767], 25)];
const EXPECTED_PLAYER_POSE_MICROMETRES: [i64; 3] = [6_500_000, 900_000, 2_000_000];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeCheckReportV1 {
    pub basin_id: PersistentId,
    pub player_pose_micrometres: [i64; 3],
    pub player_class_initial: WaterSubmersionClassV1,
    pub player_class_raised: WaterSubmersionClassV1,
    pub surface_translation_initial_micrometres: [i64; 3],
    pub surface_translation_raised_micrometres: [i64; 3],
    pub initial_level_micrometres: i64,
    pub committed_level_micrometres: i64,
    pub initial_probes: Vec<WaterProbeResultV1>,
    pub raised_probes: Vec<WaterProbeResultV1>,
    pub committed_commands: u64,
    pub rejected_commands: u64,
    pub water_events: u64,
    pub checkpoint_round_trip: bool,
    pub restored_run_identical: bool,
    pub repeated_run_identical: bool,
    pub final_state_root: StateRoot,
    pub final_physics_checkpoint_hash: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterVolumeCheckErrorV1 {
    context: String,
    detail: String,
}

impl WaterVolumeCheckErrorV1 {
    fn new(context: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            detail: detail.into(),
        }
    }

    fn condition(context: impl Into<String>) -> Self {
        Self::new(context, "condition not met")
    }
}

impl Display for WaterVolumeCheckErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for WaterVolumeCheckErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationEvidenceV1 {
    player_pose_micrometres: [i64; 3],
    player_class_initial: WaterSubmersionClassV1,
    player_class_raised: WaterSubmersionClassV1,
    surface_translation_initial_micrometres: [i64; 3],
    surface_translation_raised_micrometres: [i64; 3],
    initial_probes: Vec<WaterProbeResultV1>,
    raised_probes: Vec<WaterProbeResultV1>,
    committed_commands: u64,
    rejected_commands: u64,
    water_events: u64,
    checkpoint_round_trip: bool,
    restored_run_identical: bool,
    final_state_root: StateRoot,
    final_physics_checkpoint_hash: ContentHash,
}

pub fn run_water_volume_check() -> Result<WaterVolumeCheckReportV1, WaterVolumeCheckErrorV1> {
    let scratch = ScratchContext::new(&std::env::temp_dir()).map_err(|error| {
        WaterVolumeCheckErrorV1::new("create scratch context", error.to_string())
    })?;
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.water-volume-check")
            .map_err(|error| {
                WaterVolumeCheckErrorV1::new("activate reference package", error.to_string())
            })?;
    let result = (|| {
        let first = run_generation(&prepared.package)?;
        let repeated = run_generation(&prepared.package)?;
        if repeated != first {
            return Err(WaterVolumeCheckErrorV1::condition(
                "repeated water volume generation is identical",
            ));
        }
        let matrix_digest = evidence_digest(&first);
        Ok(WaterVolumeCheckReportV1 {
            basin_id: REFERENCE_WATER_BASIN_ID,
            player_pose_micrometres: first.player_pose_micrometres,
            player_class_initial: first.player_class_initial,
            player_class_raised: first.player_class_raised,
            surface_translation_initial_micrometres: first.surface_translation_initial_micrometres,
            surface_translation_raised_micrometres: first.surface_translation_raised_micrometres,
            initial_level_micrometres: REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
            committed_level_micrometres: RAISED_LEVEL_MICROMETRES,
            initial_probes: first.initial_probes,
            raised_probes: first.raised_probes,
            committed_commands: first.committed_commands,
            rejected_commands: first.rejected_commands,
            water_events: first.water_events,
            checkpoint_round_trip: first.checkpoint_round_trip,
            restored_run_identical: first.restored_run_identical,
            repeated_run_identical: true,
            final_state_root: first.final_state_root,
            final_physics_checkpoint_hash: first.final_physics_checkpoint_hash,
            matrix_digest,
        })
    })();
    prepared.finish(result, |error| {
        WaterVolumeCheckErrorV1::new("remove reference package", error.to_string())
    })
}

fn run_generation(
    package: &next_project::ActivatedProjectPackage,
) -> Result<GenerationEvidenceV1, WaterVolumeCheckErrorV1> {
    let mut fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            WaterVolumeCheckErrorV1::new("build reference session", error.to_string())
        })?;
    let (principal, stream_id) = register_tool_principal(&mut fixture)?;

    // The basin is part of the production reference scene, not a test-only
    // insertion: the cooked session already carries it.
    let authored = reference_water_basin_definition();
    let scene_basin = fixture
        .bootstrap
        .physics_checkpoint
        .water_volumes
        .definitions
        .get(&REFERENCE_WATER_BASIN_ID)
        .ok_or_else(|| WaterVolumeCheckErrorV1::condition("reference scene declares the basin"))?;
    require(
        scene_basin == &authored,
        "reference scene basin matches the authored definition",
    )?;

    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(&fixture),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("activate runtime", error.to_string()))?;

    let initial_probes = probe_all(
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    );
    expect_probe(
        &initial_probes,
        "basin_floor_centre",
        true,
        500_000,
        WaterSubmersionClassV1::Wading,
    )?;
    expect_probe(
        &initial_probes,
        "basin_knee",
        true,
        200_000,
        WaterSubmersionClassV1::Wading,
    )?;
    expect_probe(
        &initial_probes,
        "basin_above_initial_surface",
        true,
        0,
        WaterSubmersionClassV1::Dry,
    )?;
    expect_probe(
        &initial_probes,
        "basin_low_corner",
        true,
        500_000,
        WaterSubmersionClassV1::Wading,
    )?;
    expect_probe(
        &initial_probes,
        "outside_basin",
        false,
        0,
        WaterSubmersionClassV1::Dry,
    )?;

    let mut committed_commands = 0;
    let mut rejected_commands = 0;
    let mut water_events = 0;
    let mut sequence = 0;

    // The player walks into the basin through the production locomotion
    // path; the foot point then classifies as wading at the initial level.
    let dry_start = player_submersion(
        &fixture,
        runtime.physics_snapshot(),
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("classify player start", error.to_string()))?;
    require(
        dry_start.class == WaterSubmersionClassV1::Dry && dry_start.volume_id.is_none(),
        "player starts dry outside the basin",
    )?;
    let mut movement_sequence = 0;
    for (direction_q15, ticks) in WALK_LEGS {
        for _ in 0..ticks {
            let command = WorldCommand::physical(
                fixture.movement_stream_id,
                fixture.principal.clone(),
                movement_sequence,
                runtime.next_tick(),
                fixture.body_id,
                PhysicalCommandV1::SetCapsuleLocomotionIntent { direction_q15 },
            )
            .map_err(|error| {
                WaterVolumeCheckErrorV1::new("build locomotion command", error.to_string())
            })?;
            movement_sequence += 1;
            let report = runtime.run_tick([command]).map_err(|error| {
                WaterVolumeCheckErrorV1::new("run locomotion tick", error.to_string())
            })?;
            require_disposition(&report, CommandDisposition::Committed, "locomotion commits")?;
        }
    }
    let player_pose_micrometres = runtime
        .physics_snapshot()
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or_else(|| WaterVolumeCheckErrorV1::condition("player body exists"))?
        .pose
        .translation_micrometres;
    if player_pose_micrometres != EXPECTED_PLAYER_POSE_MICROMETRES {
        return Err(WaterVolumeCheckErrorV1::new(
            "player reaches the authored basin pose",
            format!("{player_pose_micrometres:?}"),
        ));
    }
    let player_initial = player_submersion(
        &fixture,
        runtime.physics_snapshot(),
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("classify player", error.to_string()))?;
    require(
        player_initial.class == WaterSubmersionClassV1::Wading
            && player_initial.depth_micrometres == REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
        "player wades at the initial level",
    )?;
    let surface_translation_initial_micrometres = water_surface_translation(
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("surface translation", error.to_string()))?;
    require(
        surface_translation_initial_micrometres
            == [0, REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES, 0],
        "surface follows the initial level",
    )?;

    // Raise the level through the production command path.
    let report = run_water_command(
        &mut runtime,
        stream_id,
        &principal,
        &mut sequence,
        WaterVolumeCommandV1::SetLevel {
            volume_id: REFERENCE_WATER_BASIN_ID,
            expected_record_revision: 0,
            level_micrometres: RAISED_LEVEL_MICROMETRES,
        },
    )?;
    require_disposition(
        &report,
        CommandDisposition::Committed,
        "raise level commits",
    )?;
    committed_commands += 1;
    let events = water_event_count(&report);
    require(events == 1, "raise level emits exactly one water event")?;
    water_events += events;
    let state = runtime
        .physics_checkpoint()
        .water_volumes
        .states
        .get(&REFERENCE_WATER_BASIN_ID)
        .ok_or_else(|| WaterVolumeCheckErrorV1::condition("basin state survives the commit"))?;
    require(
        state.record_revision == 1 && state.level_micrometres == RAISED_LEVEL_MICROMETRES,
        "basin record revision and level follow the committed command",
    )?;

    let player_raised = player_submersion(
        &fixture,
        runtime.physics_snapshot(),
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("classify raised player", error.to_string()))?;
    require(
        player_raised.class == WaterSubmersionClassV1::Swimming
            && player_raised.depth_micrometres == RAISED_LEVEL_MICROMETRES,
        "player swims at the raised level",
    )?;
    let surface_translation_raised_micrometres = water_surface_translation(
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("raised surface", error.to_string()))?;
    require(
        surface_translation_raised_micrometres == [0, RAISED_LEVEL_MICROMETRES, 0],
        "surface follows the raised level",
    )?;
    let raised_probes = probe_all(
        &runtime.physics_checkpoint().water_volumes,
        runtime.next_tick(),
    );
    expect_probe(
        &raised_probes,
        "basin_floor_centre",
        true,
        1_500_000,
        WaterSubmersionClassV1::Swimming,
    )?;
    expect_probe(
        &raised_probes,
        "basin_knee",
        true,
        1_200_000,
        WaterSubmersionClassV1::Swimming,
    )?;
    expect_probe(
        &raised_probes,
        "basin_above_initial_surface",
        true,
        900_000,
        WaterSubmersionClassV1::Wading,
    )?;
    expect_probe(
        &raised_probes,
        "outside_basin",
        false,
        0,
        WaterSubmersionClassV1::Dry,
    )?;

    // Deterministic rejections leave the table untouched and emit nothing.
    // The flow vessels (ADR-103) move every tick; only the basin, the
    // target of every command here, must stay untouched by rejections.
    let basin_state = |runtime: &RuntimeState| {
        runtime
            .physics_checkpoint()
            .water_volumes
            .states
            .get(&REFERENCE_WATER_BASIN_ID)
            .cloned()
    };
    let before_rejections = basin_state(&runtime);
    for (command, code, label) in [
        (
            WaterVolumeCommandV1::SetLevel {
                volume_id: REFERENCE_WATER_BASIN_ID,
                expected_record_revision: 0,
                level_micrometres: LOWERED_LEVEL_MICROMETRES,
            },
            RejectionCode::WaterVolumeRevisionStale,
            "stale revision rejects",
        ),
        (
            WaterVolumeCommandV1::SetLevel {
                volume_id: REFERENCE_WATER_BASIN_ID,
                expected_record_revision: 1,
                level_micrometres: OUT_OF_EXTENT_LEVEL_MICROMETRES,
            },
            RejectionCode::WaterVolumeLevelOutOfExtent,
            "level above the basin rejects",
        ),
        (
            WaterVolumeCommandV1::SetLevel {
                volume_id: UNKNOWN_VOLUME_ID,
                expected_record_revision: 0,
                level_micrometres: LOWERED_LEVEL_MICROMETRES,
            },
            RejectionCode::WaterVolumeUnknown,
            "unknown volume rejects",
        ),
    ] {
        let report =
            run_water_command(&mut runtime, stream_id, &principal, &mut sequence, command)?;
        require_disposition(&report, CommandDisposition::Rejected(code), label)?;
        require(
            water_event_count(&report) == 0,
            "rejection emits no water event",
        )?;
        rejected_commands += 1;
    }
    require(
        basin_state(&runtime) == before_rejections,
        "rejections leave the water table unchanged",
    )?;

    // Save closure: the physics segment round-trips byte-exactly and a
    // restored runtime continues to the same roots as the live one.
    let checkpoint = runtime.world_checkpoint().map_err(|error| {
        WaterVolumeCheckErrorV1::new("build world checkpoint", error.to_string())
    })?;
    let physics_bytes = checkpoint
        .physics_checkpoint
        .canonical_bytes()
        .map_err(|error| {
            WaterVolumeCheckErrorV1::new("encode physics checkpoint", error.to_string())
        })?;
    let decoded = PhysicsWorldCheckpointV1::from_canonical_bytes(
        &physics_bytes,
        CanonicalDecodeLimits::default(),
    )
    .map_err(|error| {
        WaterVolumeCheckErrorV1::new("decode physics checkpoint", error.to_string())
    })?;
    let checkpoint_round_trip = decoded == checkpoint.physics_checkpoint
        && decoded.water_volumes == runtime.physics_checkpoint().water_volumes;
    require(
        checkpoint_round_trip,
        "physics checkpoint with water round-trips",
    )?;

    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint.clone(),
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("restore runtime", error.to_string()))?;
    require(
        restored.physics_checkpoint().water_volumes == runtime.physics_checkpoint().water_volumes,
        "restored runtime carries the same water table",
    )?;
    let lower = WaterVolumeCommandV1::SetLevel {
        volume_id: REFERENCE_WATER_BASIN_ID,
        expected_record_revision: 1,
        level_micrometres: LOWERED_LEVEL_MICROMETRES,
    };
    let live_sequence = sequence;
    let mut restored_sequence = sequence;
    let live_report = run_water_command(
        &mut runtime,
        stream_id,
        &principal,
        &mut sequence,
        lower.clone(),
    )?;
    let restored_report = run_water_command(
        &mut restored,
        stream_id,
        &principal,
        &mut restored_sequence,
        lower,
    )?;
    require_disposition(
        &live_report,
        CommandDisposition::Committed,
        "live lower level commits",
    )?;
    require_disposition(
        &restored_report,
        CommandDisposition::Committed,
        "restored lower level commits",
    )?;
    require(live_sequence + 1 == sequence, "live sequence advances once")?;
    committed_commands += 1;
    water_events += water_event_count(&live_report);
    let live_final = runtime.world_checkpoint().map_err(|error| {
        WaterVolumeCheckErrorV1::new("build final checkpoint", error.to_string())
    })?;
    let restored_final = restored.world_checkpoint().map_err(|error| {
        WaterVolumeCheckErrorV1::new("build restored final checkpoint", error.to_string())
    })?;
    let restored_run_identical = live_final.state_root == restored_final.state_root
        && live_report.events == restored_report.events
        && live_report.physics_checkpoint_hash == restored_report.physics_checkpoint_hash;
    require(
        restored_run_identical,
        "restored and live runs reach the same root",
    )?;

    Ok(GenerationEvidenceV1 {
        player_pose_micrometres,
        player_class_initial: player_initial.class,
        player_class_raised: player_raised.class,
        surface_translation_initial_micrometres,
        surface_translation_raised_micrometres,
        initial_probes,
        raised_probes,
        committed_commands,
        rejected_commands,
        water_events,
        checkpoint_round_trip,
        restored_run_identical,
        final_state_root: live_final.state_root,
        final_physics_checkpoint_hash: live_report.physics_checkpoint_hash,
    })
}

fn register_tool_principal(
    fixture: &mut ReferenceGameSession,
) -> Result<(IssuerPrincipal, CommandStreamId), WaterVolumeCheckErrorV1> {
    let principal =
        IssuerPrincipal::Tool(ToolPrincipalId::new(TOOL_PRINCIPAL_ID).map_err(|error| {
            WaterVolumeCheckErrorV1::new("tool principal id", error.to_string())
        })?);
    let capability = CapabilityId::new(WATER_VOLUME_CAPABILITY_ID)
        .map_err(|error| WaterVolumeCheckErrorV1::new("water capability id", error.to_string()))?;
    fixture
        .bootstrap
        .principal_registry
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes(sha256(
                    b"nextengine.principal.water-volume-check.v1\0",
                )),
                capability_subject_id: SchemaId::new(
                    "nextengine.capability-subject.water-volume-check",
                )
                .map_err(|error| {
                    WaterVolumeCheckErrorV1::new("capability subject id", error.to_string())
                })?,
                status: PrincipalStatus::Active,
            },
        )
        .map_err(|error| {
            WaterVolumeCheckErrorV1::new("register tool principal", error.to_string())
        })?;
    let stream_id = fixture
        .bootstrap
        .stream_registry
        .allocate_stream(principal.clone())
        .map_err(|error| WaterVolumeCheckErrorV1::new("allocate tool stream", error.to_string()))?;
    fixture
        .authority
        .register(principal.clone(), [capability])
        .map_err(|error| {
            WaterVolumeCheckErrorV1::new("grant water capability", error.to_string())
        })?;
    Ok((principal, stream_id))
}

fn run_water_command(
    runtime: &mut RuntimeState,
    stream_id: CommandStreamId,
    principal: &IssuerPrincipal,
    sequence: &mut u64,
    payload: WaterVolumeCommandV1,
) -> Result<next_runtime::TickReport, WaterVolumeCheckErrorV1> {
    let command = WorldCommand::water_volume(
        stream_id,
        principal.clone(),
        *sequence,
        runtime.next_tick(),
        payload,
    )
    .map_err(|error| WaterVolumeCheckErrorV1::new("build water command", error.to_string()))?;
    *sequence += 1;
    runtime
        .run_tick([command])
        .map_err(|error| WaterVolumeCheckErrorV1::new("run water tick", error.to_string()))
}

fn require_disposition(
    report: &next_runtime::TickReport,
    expected: CommandDisposition,
    label: &str,
) -> Result<(), WaterVolumeCheckErrorV1> {
    let actual = report
        .results
        .iter()
        .map(|result| result.disposition.clone())
        .collect::<Vec<_>>();
    if actual != [expected] {
        return Err(WaterVolumeCheckErrorV1::new(label, format!("{actual:?}")));
    }
    Ok(())
}

fn water_event_count(report: &next_runtime::TickReport) -> u64 {
    report
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.payload,
                next_contracts::command::EventPayload::WaterVolume(_)
            )
        })
        .count() as u64
}

fn probe_all(water: &WaterVolumeSetV1, tick: u64) -> Vec<WaterProbeResultV1> {
    PROBES
        .iter()
        .map(|(label, point)| {
            let submersion = water.submersion_at(*point, tick);
            WaterProbeResultV1 {
                label: (*label).to_owned(),
                point_micrometres: *point,
                in_volume: submersion.volume_id.is_some(),
                depth_micrometres: submersion.depth_micrometres,
                class: submersion.class,
            }
        })
        .collect()
}

fn expect_probe(
    probes: &[WaterProbeResultV1],
    label: &str,
    in_volume: bool,
    depth_micrometres: i64,
    class: WaterSubmersionClassV1,
) -> Result<(), WaterVolumeCheckErrorV1> {
    let probe = probes
        .iter()
        .find(|probe| probe.label == label)
        .ok_or_else(|| WaterVolumeCheckErrorV1::condition(format!("probe {label} exists")))?;
    if probe.in_volume != in_volume
        || probe.depth_micrometres != depth_micrometres
        || probe.class != class
    {
        return Err(WaterVolumeCheckErrorV1::new(
            format!("probe {label}"),
            format!(
                "expected in_volume={in_volume} depth={depth_micrometres} class={class:?}, got in_volume={} depth={} class={:?}",
                probe.in_volume, probe.depth_micrometres, probe.class
            ),
        ));
    }
    Ok(())
}

fn evidence_digest(evidence: &GenerationEvidenceV1) -> ContentHash {
    let mut preimage = b"nextengine.water-volume-check.v1\0".to_vec();
    for probes in [&evidence.initial_probes, &evidence.raised_probes] {
        for probe in probes {
            preimage.extend_from_slice(probe.label.as_bytes());
            for axis in probe.point_micrometres {
                preimage.extend_from_slice(&axis.to_le_bytes());
            }
            preimage.push(u8::from(probe.in_volume));
            preimage.extend_from_slice(&probe.depth_micrometres.to_le_bytes());
            preimage.push(probe.class as u8);
        }
    }
    for axis in evidence.player_pose_micrometres {
        preimage.extend_from_slice(&axis.to_le_bytes());
    }
    preimage.push(evidence.player_class_initial as u8);
    preimage.push(evidence.player_class_raised as u8);
    for translation in [
        evidence.surface_translation_initial_micrometres,
        evidence.surface_translation_raised_micrometres,
    ] {
        for axis in translation {
            preimage.extend_from_slice(&axis.to_le_bytes());
        }
    }
    preimage.extend_from_slice(&evidence.committed_commands.to_le_bytes());
    preimage.extend_from_slice(&evidence.rejected_commands.to_le_bytes());
    preimage.extend_from_slice(&evidence.water_events.to_le_bytes());
    preimage.push(u8::from(evidence.checkpoint_round_trip));
    preimage.push(u8::from(evidence.restored_run_identical));
    preimage.extend_from_slice(evidence.final_state_root.as_bytes());
    preimage.extend_from_slice(evidence.final_physics_checkpoint_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn require(condition: bool, context: &str) -> Result<(), WaterVolumeCheckErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(WaterVolumeCheckErrorV1::condition(context))
    }
}
