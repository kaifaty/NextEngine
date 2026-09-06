//! `CONTINUUM-WATER-FLOW-P1` (ADR-103): the reference vessels drain through
//! an exact, command-driven, save-bound flow network that conserves volume
//! to the cubic millimetre and never reads presentation state.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Instant;

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::{EventPayload, IssuerPrincipal, WorldCommand};
use next_contracts::identity::{PrincipalRecordV1, PrincipalStatus};
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ContentHash, PersistentId, SchemaId, StateRoot, ToolPrincipalId,
    content_hash_from_bytes,
};
use next_contracts::physics::{
    PhysicsWorldCheckpointV1, WATER_FLOW_CAPABILITY_ID, WaterFlowCommandV1, WaterFlowEdgeKindV1,
    WaterFlowEdgeV1, WaterFlowNetworkV1, WaterVolumeDefinitionV1, WaterVolumeSetV1,
    cell_area_square_millimetres, isqrt_i128,
};
use next_reference_game::{
    REFERENCE_WATER_FLOW_GATE_ID, REFERENCE_WATER_FLOW_SINK_ID, REFERENCE_WATER_FLOW_SOURCE_ID,
    REFERENCE_WATER_FLOW_TICKS_PER_SECOND, REFERENCE_WATER_VESSEL_A_ID,
    REFERENCE_WATER_VESSEL_B_ID, ReferenceGameSession, cooked_project_rpg_snapshot,
    reference_water_flow,
};
use next_runtime::{CommandDisposition, RejectionCode, RuntimeState};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

const TOOL_PRINCIPAL_ID: &str = "nextengine.tool.water-flow-check";
const UNKNOWN_EDGE_ID: PersistentId = PersistentId::from_bytes([0x83; 16]);
/// Plan `continuum-water/07`: the run is long enough for twice the analytic
/// drain time of vessel A (`~27 s` plus the `2 s` gate closure); the save
/// point is the middle of the run.
const RUN_TICKS: u64 = 1_800;
const SAVE_TICK: u64 = 900;
const GATE_CLOSE_TICK: u64 = 300;
const GATE_REOPEN_TICK: u64 = 360;
/// G2 tolerance: vessel A within one millimetre of its floor.
const DRAIN_TOLERANCE_MICROMETRES: i64 = 1_000;
/// G6 synthetic network: the record bounds of ADR-103.
const COST_CELLS: usize = 64;
const COST_STEPS: u32 = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterFlowCheckReportV1 {
    pub vessel_a_id: PersistentId,
    pub vessel_b_id: PersistentId,
    pub gate_id: PersistentId,
    pub run_ticks: u64,
    pub analytic_drain_ticks: u64,
    pub drained_by_tick: Option<u64>,
    pub level_a_initial_micrometres: i64,
    pub level_a_final_micrometres: i64,
    pub level_b_final_micrometres: i64,
    pub total_volume_initial_cubic_millimetres: i128,
    pub total_volume_final_cubic_millimetres: i128,
    pub source_volume_cubic_millimetres: i128,
    pub sink_volume_cubic_millimetres: i128,
    pub conservation_exact: bool,
    pub gate_closed_flux_zero: bool,
    pub gate_reopened_flux_positive: bool,
    pub committed_commands: u64,
    pub rejected_commands: u64,
    pub flow_events: u64,
    pub checkpoint_round_trip: bool,
    pub restored_run_identical: bool,
    pub repeated_run_identical: bool,
    pub step_cost_cells: usize,
    pub step_cost_edges: usize,
    pub step_cost_max_microseconds: u128,
    /// Plan 07 revision 2 apparatus: the mean over the samples next to the
    /// gated maximum.
    pub step_cost_mean_microseconds: u128,
    pub step_cost_debug_build: bool,
    pub final_state_root: StateRoot,
    pub final_physics_checkpoint_hash: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterFlowCheckErrorV1 {
    context: String,
    detail: String,
}

impl WaterFlowCheckErrorV1 {
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

impl Display for WaterFlowCheckErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for WaterFlowCheckErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationEvidenceV1 {
    analytic_drain_ticks: u64,
    drained_by_tick: Option<u64>,
    level_a_initial_micrometres: i64,
    level_a_final_micrometres: i64,
    level_b_final_micrometres: i64,
    total_volume_initial_cubic_millimetres: i128,
    total_volume_final_cubic_millimetres: i128,
    source_volume_cubic_millimetres: i128,
    sink_volume_cubic_millimetres: i128,
    conservation_exact: bool,
    gate_closed_flux_zero: bool,
    gate_reopened_flux_positive: bool,
    committed_commands: u64,
    rejected_commands: u64,
    flow_events: u64,
    checkpoint_round_trip: bool,
    restored_run_identical: bool,
    final_state_root: StateRoot,
    final_physics_checkpoint_hash: ContentHash,
}

pub fn run_water_flow_check() -> Result<WaterFlowCheckReportV1, WaterFlowCheckErrorV1> {
    let scratch = ScratchContext::new(&std::env::temp_dir())
        .map_err(|error| WaterFlowCheckErrorV1::new("create scratch context", error.to_string()))?;
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.water-flow-check")
            .map_err(|error| {
                WaterFlowCheckErrorV1::new("activate reference package", error.to_string())
            })?;
    let result = (|| {
        let first = run_generation(&prepared.package)?;
        let repeated = run_generation(&prepared.package)?;
        if repeated != first {
            return Err(WaterFlowCheckErrorV1::condition(
                "repeated water flow generation is identical",
            ));
        }
        let (step_cost_edges, step_cost_max_microseconds, step_cost_mean_microseconds) =
            step_cost()?;
        let matrix_digest = evidence_digest(&first);
        Ok(WaterFlowCheckReportV1 {
            vessel_a_id: REFERENCE_WATER_VESSEL_A_ID,
            vessel_b_id: REFERENCE_WATER_VESSEL_B_ID,
            gate_id: REFERENCE_WATER_FLOW_GATE_ID,
            run_ticks: RUN_TICKS,
            analytic_drain_ticks: first.analytic_drain_ticks,
            drained_by_tick: first.drained_by_tick,
            level_a_initial_micrometres: first.level_a_initial_micrometres,
            level_a_final_micrometres: first.level_a_final_micrometres,
            level_b_final_micrometres: first.level_b_final_micrometres,
            total_volume_initial_cubic_millimetres: first.total_volume_initial_cubic_millimetres,
            total_volume_final_cubic_millimetres: first.total_volume_final_cubic_millimetres,
            source_volume_cubic_millimetres: first.source_volume_cubic_millimetres,
            sink_volume_cubic_millimetres: first.sink_volume_cubic_millimetres,
            conservation_exact: first.conservation_exact,
            gate_closed_flux_zero: first.gate_closed_flux_zero,
            gate_reopened_flux_positive: first.gate_reopened_flux_positive,
            committed_commands: first.committed_commands,
            rejected_commands: first.rejected_commands,
            flow_events: first.flow_events,
            checkpoint_round_trip: first.checkpoint_round_trip,
            restored_run_identical: first.restored_run_identical,
            repeated_run_identical: true,
            step_cost_cells: COST_CELLS,
            step_cost_edges,
            step_cost_max_microseconds,
            step_cost_mean_microseconds,
            step_cost_debug_build: cfg!(debug_assertions),
            final_state_root: first.final_state_root,
            final_physics_checkpoint_hash: first.final_physics_checkpoint_hash,
            matrix_digest,
        })
    })();
    prepared.finish(result, |error| {
        WaterFlowCheckErrorV1::new("remove reference package", error.to_string())
    })
}

fn run_generation(
    package: &next_project::ActivatedProjectPackage,
) -> Result<GenerationEvidenceV1, WaterFlowCheckErrorV1> {
    let mut fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            WaterFlowCheckErrorV1::new("build reference session", error.to_string())
        })?;
    let (principal, stream_id) = register_tool_principal(&mut fixture)?;

    // The vessels and the network are part of the production reference
    // scene, not a test-only insertion.
    let scene = &fixture.bootstrap.physics_checkpoint;
    let authored = reference_water_flow(&scene.water_volumes)
        .map_err(|error| WaterFlowCheckErrorV1::new("authored network", error.to_string()))?;
    require(
        scene.water_flow == authored,
        "reference scene network matches the authored edges",
    )?;
    let vessel_a = scene
        .water_volumes
        .definitions
        .get(&REFERENCE_WATER_VESSEL_A_ID)
        .cloned()
        .ok_or_else(|| WaterFlowCheckErrorV1::condition("reference scene declares vessel A"))?;
    let analytic_drain_ticks = analytic_drain_ticks(&vessel_a, &authored)?;

    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(&fixture),
    )
    .map_err(|error| WaterFlowCheckErrorV1::new("activate runtime", error.to_string()))?;

    let level_a_initial_micrometres = level_of(&runtime, REFERENCE_WATER_VESSEL_A_ID)?;
    let total_volume_initial_cubic_millimetres =
        runtime.physics_checkpoint().water_flow.total_volume();
    let mut sequence = 0_u64;
    let mut committed_commands = 0_u64;
    let mut rejected_commands = 0_u64;
    let mut flow_events = 0_u64;
    let mut source_volume: i128 = 0;
    let mut sink_volume: i128 = 0;
    let mut conservation_exact = true;
    let mut gate_closed_flux_zero = false;
    let mut gate_reopened_flux_positive = false;
    let mut drained_by_tick: Option<u64> = None;
    let mut restored: Option<RuntimeState> = None;
    let mut checkpoint_round_trip = false;
    let mut restored_run_identical = true;

    for tick in 0..RUN_TICKS {
        let report = if tick == GATE_CLOSE_TICK {
            let report = run_flow_command(
                &mut runtime,
                stream_id,
                &principal,
                &mut sequence,
                WaterFlowCommandV1::SetGate {
                    edge_id: REFERENCE_WATER_FLOW_GATE_ID,
                    expected_record_revision: 0,
                    opening_permille: 0,
                },
            )?;
            require_disposition(&report, CommandDisposition::Committed, "gate close commits")?;
            committed_commands += 1;
            flow_events += flow_event_count(&report);
            report
        } else if tick == GATE_REOPEN_TICK {
            let report = run_flow_command(
                &mut runtime,
                stream_id,
                &principal,
                &mut sequence,
                WaterFlowCommandV1::SetGate {
                    edge_id: REFERENCE_WATER_FLOW_GATE_ID,
                    expected_record_revision: 1,
                    opening_permille: 1000,
                },
            )?;
            require_disposition(
                &report,
                CommandDisposition::Committed,
                "gate reopen commits",
            )?;
            committed_commands += 1;
            flow_events += flow_event_count(&report);
            report
        } else {
            runtime
                .run_tick(Vec::<WorldCommand>::new())
                .map_err(|error| WaterFlowCheckErrorV1::new("run flow tick", error.to_string()))?
        };
        let network = &runtime.physics_checkpoint().water_flow;
        let gate_flux = network
            .edge_flux(REFERENCE_WATER_FLOW_GATE_ID)
            .ok_or_else(|| WaterFlowCheckErrorV1::condition("gate edge has a flux"))?;
        let source_flux = network
            .edge_flux(REFERENCE_WATER_FLOW_SOURCE_ID)
            .ok_or_else(|| WaterFlowCheckErrorV1::condition("source edge has a flux"))?;
        let sink_flux = network
            .edge_flux(REFERENCE_WATER_FLOW_SINK_ID)
            .ok_or_else(|| WaterFlowCheckErrorV1::condition("sink edge has a flux"))?;
        source_volume += i128::from(source_flux);
        sink_volume += i128::from(-sink_flux);
        // G1: exact conservation at every tick.
        if network.total_volume()
            != total_volume_initial_cubic_millimetres + source_volume - sink_volume
        {
            conservation_exact = false;
        }
        // G3: the closed gate carries nothing from the next tick on, and the
        // reopened gate carries water again.
        if tick == GATE_CLOSE_TICK + 1 {
            gate_closed_flux_zero = gate_flux == 0;
        }
        if tick == GATE_REOPEN_TICK + 1 {
            gate_reopened_flux_positive = gate_flux > 0;
        }
        // G2: first tick with vessel A at its floor.
        let level_a = level_of(&runtime, REFERENCE_WATER_VESSEL_A_ID)?;
        if drained_by_tick.is_none()
            && level_a - vessel_a.minimum_micrometres[1] <= DRAIN_TOLERANCE_MICROMETRES
        {
            drained_by_tick = Some(tick);
        }
        if let Some(restored) = restored.as_mut() {
            let restored_report =
                restored
                    .run_tick(Vec::<WorldCommand>::new())
                    .map_err(|error| {
                        WaterFlowCheckErrorV1::new("run restored tick", error.to_string())
                    })?;
            if restored_report.physics_checkpoint_hash != report.physics_checkpoint_hash
                || restored_report.events != report.events
            {
                restored_run_identical = false;
            }
        }
        if tick == SAVE_TICK {
            // G4: the physics segment round-trips byte-exactly and a
            // restored runtime continues to the same roots as the live one.
            let checkpoint = runtime.world_checkpoint().map_err(|error| {
                WaterFlowCheckErrorV1::new("build world checkpoint", error.to_string())
            })?;
            let physics_bytes =
                checkpoint
                    .physics_checkpoint
                    .canonical_bytes()
                    .map_err(|error| {
                        WaterFlowCheckErrorV1::new("encode physics checkpoint", error.to_string())
                    })?;
            let decoded = PhysicsWorldCheckpointV1::from_canonical_bytes(
                &physics_bytes,
                CanonicalDecodeLimits::default(),
            )
            .map_err(|error| {
                WaterFlowCheckErrorV1::new("decode physics checkpoint", error.to_string())
            })?;
            checkpoint_round_trip = decoded == checkpoint.physics_checkpoint
                && decoded.water_flow == runtime.physics_checkpoint().water_flow;
            require(
                checkpoint_round_trip,
                "physics checkpoint with the network round-trips",
            )?;
            let candidate = RuntimeState::restore_world_checkpoint_with_definitions(
                checkpoint,
                fixture.authority.clone(),
                fixture.activated_project.rpg_definitions.clone(),
            )
            .map_err(|error| WaterFlowCheckErrorV1::new("restore runtime", error.to_string()))?;
            require(
                candidate.physics_checkpoint().water_flow
                    == runtime.physics_checkpoint().water_flow,
                "restored runtime carries the same network",
            )?;
            restored = Some(candidate);
        }
    }
    require(
        conservation_exact,
        "volume is conserved exactly at every tick",
    )?;
    require(gate_closed_flux_zero, "closed gate stops the exchange")?;
    require(
        gate_reopened_flux_positive,
        "reopened gate resumes the exchange",
    )?;
    require(
        drained_by_tick.is_some_and(|tick| tick <= 2 * analytic_drain_ticks),
        "vessel A drains within twice the analytic time",
    )?;

    // G5: deterministic rejections leave the network untouched, emit no
    // event and count once each.
    let before_rejections = runtime.physics_checkpoint().water_flow.clone();
    for (command, code, label) in [
        (
            WaterFlowCommandV1::SetGate {
                edge_id: UNKNOWN_EDGE_ID,
                expected_record_revision: 0,
                opening_permille: 500,
            },
            RejectionCode::WaterFlowEdgeUnknown,
            "unknown edge rejects",
        ),
        (
            WaterFlowCommandV1::SetPump {
                edge_id: REFERENCE_WATER_FLOW_GATE_ID,
                expected_record_revision: 2,
                enabled: false,
            },
            RejectionCode::WaterFlowEdgeKindMismatch,
            "pump command on a gate rejects",
        ),
        (
            WaterFlowCommandV1::SetGate {
                edge_id: REFERENCE_WATER_FLOW_GATE_ID,
                expected_record_revision: 0,
                opening_permille: 500,
            },
            RejectionCode::WaterFlowRevisionStale,
            "stale revision rejects",
        ),
        (
            WaterFlowCommandV1::SetGate {
                edge_id: REFERENCE_WATER_FLOW_GATE_ID,
                expected_record_revision: 2,
                opening_permille: 1001,
            },
            RejectionCode::WaterFlowOpeningOutOfRange,
            "opening above 1000 rejects",
        ),
    ] {
        let mut restored_sequence = sequence;
        let report = run_flow_command(
            &mut runtime,
            stream_id,
            &principal,
            &mut sequence,
            command.clone(),
        )?;
        require_disposition(&report, CommandDisposition::Rejected(code), label)?;
        require(
            flow_event_count(&report) == 0,
            "rejection emits no flow event",
        )?;
        rejected_commands += 1;
        if let Some(restored) = restored.as_mut() {
            // The restored runtime receives the same rejected command so
            // both ledgers stay identical.
            let restored_report = run_flow_command(
                restored,
                stream_id,
                &principal,
                &mut restored_sequence,
                command,
            )?;
            require_disposition(
                &restored_report,
                CommandDisposition::Rejected(code),
                "restored runtime rejects identically",
            )?;
            if restored_report.physics_checkpoint_hash != report.physics_checkpoint_hash
                || restored_report.events != report.events
            {
                restored_run_identical = false;
            }
        }
    }
    // The rejection ticks still stepped the network; compare the edge states
    // that commands could have touched.
    require(
        runtime
            .physics_checkpoint()
            .water_flow
            .edge_states
            .values()
            .zip(before_rejections.edge_states.values())
            .all(|(after, before)| {
                after.record_revision == before.record_revision
                    && after.opening_permille == before.opening_permille
                    && after.enabled == before.enabled
                    && after.rate_cubic_millimetres_per_second
                        == before.rate_cubic_millimetres_per_second
            }),
        "rejections leave the edge states unchanged",
    )?;
    let restored =
        restored.ok_or_else(|| WaterFlowCheckErrorV1::condition("a restored runtime exists"))?;
    let live_final = runtime
        .world_checkpoint()
        .map_err(|error| WaterFlowCheckErrorV1::new("build final checkpoint", error.to_string()))?;
    let restored_final = restored.world_checkpoint().map_err(|error| {
        WaterFlowCheckErrorV1::new("build restored final checkpoint", error.to_string())
    })?;
    restored_run_identical =
        restored_run_identical && live_final.state_root == restored_final.state_root;
    require(
        restored_run_identical,
        "restored and live runs reach the same roots",
    )?;
    let final_physics_checkpoint_hash =
        runtime
            .physics_checkpoint()
            .checkpoint_hash()
            .map_err(|error| {
                WaterFlowCheckErrorV1::new("hash final physics checkpoint", error.to_string())
            })?;

    Ok(GenerationEvidenceV1 {
        analytic_drain_ticks,
        drained_by_tick,
        level_a_initial_micrometres,
        level_a_final_micrometres: level_of(&runtime, REFERENCE_WATER_VESSEL_A_ID)?,
        level_b_final_micrometres: level_of(&runtime, REFERENCE_WATER_VESSEL_B_ID)?,
        total_volume_initial_cubic_millimetres,
        total_volume_final_cubic_millimetres: runtime
            .physics_checkpoint()
            .water_flow
            .total_volume(),
        source_volume_cubic_millimetres: source_volume,
        sink_volume_cubic_millimetres: sink_volume,
        conservation_exact,
        gate_closed_flux_zero,
        gate_reopened_flux_positive,
        committed_commands,
        rejected_commands,
        flow_events,
        checkpoint_round_trip,
        restored_run_identical,
        final_state_root: live_final.state_root,
        final_physics_checkpoint_hash,
    })
}

/// Torricelli emptying time of vessel A through the gate down to its floor,
/// with the head measured to the invert (B stays below it), plus the gate
/// closure, in ticks: `t = 2 A_a (sqrt(h0) - sqrt(h1)) / (c_d A sqrt(2 g))`.
fn analytic_drain_ticks(
    vessel_a: &WaterVolumeDefinitionV1,
    network: &WaterFlowNetworkV1,
) -> Result<u64, WaterFlowCheckErrorV1> {
    let gate = network
        .edges
        .get(&REFERENCE_WATER_FLOW_GATE_ID)
        .ok_or_else(|| WaterFlowCheckErrorV1::condition("gate edge exists"))?;
    let WaterFlowEdgeKindV1::Gate {
        invert_micrometres,
        area_square_millimetres,
        coefficient_permille,
        ..
    } = gate.kind
    else {
        return Err(WaterFlowCheckErrorV1::condition("gate edge is a gate"));
    };
    let plan_area = i128::from(cell_area_square_millimetres(
        vessel_a.minimum_micrometres,
        vessel_a.maximum_micrometres,
    ));
    let head_initial = i128::from(vessel_a.initial_level_micrometres - invert_micrometres).max(0);
    let head_floor = i128::from(vessel_a.minimum_micrometres[1] - invert_micrometres).max(0);
    // Micrometre roots: sqrt(h) in sqrt(um); the ratio keeps the units.
    let root_difference = isqrt_i128(head_initial) - isqrt_i128(head_floor);
    let gravity =
        i128::from(next_contracts::physics::WATER_FLOW_GRAVITY_MICROMETRES_PER_SECOND_SQUARED);
    // seconds = 2 * A_a[mm^2] * (sqrt(h0) - sqrt(h1))[sqrt(um)] * 1000
    //           / (c_d * A[mm^2] * sqrt(2 g)[sqrt(um)/s])
    let numerator = 2 * plan_area * root_difference * 1000;
    let denominator = i128::from(coefficient_permille)
        * i128::from(area_square_millimetres)
        * isqrt_i128(2 * gravity);
    if denominator <= 0 {
        return Err(WaterFlowCheckErrorV1::condition(
            "gate law is non-degenerate",
        ));
    }
    let seconds = numerator / denominator;
    let ticks = seconds * i128::from(REFERENCE_WATER_FLOW_TICKS_PER_SECOND)
        + i128::from(GATE_REOPEN_TICK - GATE_CLOSE_TICK);
    u64::try_from(ticks).map_err(|_| WaterFlowCheckErrorV1::condition("drain time is bounded"))
}

fn level_of(runtime: &RuntimeState, cell: PersistentId) -> Result<i64, WaterFlowCheckErrorV1> {
    runtime
        .physics_checkpoint()
        .water_volumes
        .states
        .get(&cell)
        .map(|state| state.level_micrometres)
        .ok_or_else(|| WaterFlowCheckErrorV1::condition("vessel exists in the water table"))
}

/// G6: one step of a network at the record bounds (64 cells, 256 edges).
fn step_cost() -> Result<(usize, u128, u128), WaterFlowCheckErrorV1> {
    let mut definitions = Vec::with_capacity(COST_CELLS);
    for index in 0..COST_CELLS {
        let x = i64::try_from(index).unwrap_or(0) * 3_000_000;
        definitions.push(WaterVolumeDefinitionV1 {
            volume_id: PersistentId::from_bytes([
                0x90 + u8::try_from(index / 16).unwrap_or(0),
                u8::try_from(index % 16).unwrap_or(0),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]),
            minimum_micrometres: [x, 0, 0],
            maximum_micrometres: [x + 2_000_000, 2_000_000, 2_000_000],
            initial_level_micrometres: if index % 2 == 0 { 1_500_000 } else { 200_000 },
            swimming_depth_micrometres: 1_200_000,
            level_ramp: None,
            profile_revision: 1,
        });
    }
    let volumes = WaterVolumeSetV1::from_definitions(definitions.clone())
        .map_err(|error| WaterFlowCheckErrorV1::new("cost volumes", error.to_string()))?;
    let cell = |index: usize| definitions[index].volume_id;
    let mut edges = Vec::new();
    let mut edge_index = 0_u8;
    let mut next_id = || {
        let id =
            PersistentId::from_bytes([0xa0, edge_index, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        edge_index = edge_index.wrapping_add(1);
        id
    };
    for index in 0..COST_CELLS - 1 {
        edges.push(WaterFlowEdgeV1 {
            edge_id: next_id(),
            cell_a: cell(index),
            cell_b: Some(cell(index + 1)),
            kind: WaterFlowEdgeKindV1::Pipe {
                invert_micrometres: 0,
                area_square_millimetres: 20_000,
                coefficient_permille: 400,
            },
        });
        edges.push(WaterFlowEdgeV1 {
            edge_id: next_id(),
            cell_a: cell(index),
            cell_b: Some(cell(index + 1)),
            kind: WaterFlowEdgeKindV1::Open {
                sill_micrometres: 1_000_000,
                width_millimetres: 500,
                coefficient_permille: 385,
            },
        });
    }
    for index in 0..COST_CELLS {
        edges.push(WaterFlowEdgeV1 {
            edge_id: next_id(),
            cell_a: cell(index),
            cell_b: None,
            kind: WaterFlowEdgeKindV1::Source {
                rate_cubic_millimetres_per_second: 100_000,
            },
        });
        edges.push(WaterFlowEdgeV1 {
            edge_id: next_id(),
            cell_a: cell(index),
            cell_b: None,
            kind: WaterFlowEdgeKindV1::Sink {
                rate_cubic_millimetres_per_second: 100_000,
            },
        });
    }
    edges.push(WaterFlowEdgeV1 {
        edge_id: next_id(),
        cell_a: cell(0),
        cell_b: Some(cell(COST_CELLS - 1)),
        kind: WaterFlowEdgeKindV1::Pump {
            rate_cubic_millimetres_per_second: 200_000,
            maximum_head_micrometres: 3_000_000,
            initially_enabled: true,
        },
    });
    edges.push(WaterFlowEdgeV1 {
        edge_id: next_id(),
        cell_a: cell(COST_CELLS - 1),
        cell_b: Some(cell(0)),
        kind: WaterFlowEdgeKindV1::Gate {
            invert_micrometres: 0,
            area_square_millimetres: 10_000,
            coefficient_permille: 400,
            initial_opening_permille: 500,
        },
    });
    let edge_count = edges.len();
    let mut network = WaterFlowNetworkV1::from_edges(30, edges, &volumes)
        .map_err(|error| WaterFlowCheckErrorV1::new("cost network", error.to_string()))?;
    let mut volumes = volumes;
    let mut maximum = 0_u128;
    let mut total = 0_u128;
    // Apparatus (plan 07 revision 2, recorded): G6 measures `step_in_place`,
    // the path the runtime's physics step runs.
    for _ in 0..COST_STEPS {
        let started = Instant::now();
        network
            .step_in_place(&mut volumes)
            .map_err(|error| WaterFlowCheckErrorV1::new("cost step", error.to_string()))?;
        let elapsed = started.elapsed().as_micros();
        maximum = maximum.max(elapsed);
        total += elapsed;
    }
    Ok((edge_count, maximum, total / u128::from(COST_STEPS.max(1))))
}

fn register_tool_principal(
    fixture: &mut ReferenceGameSession,
) -> Result<(IssuerPrincipal, CommandStreamId), WaterFlowCheckErrorV1> {
    let principal = IssuerPrincipal::Tool(
        ToolPrincipalId::new(TOOL_PRINCIPAL_ID)
            .map_err(|error| WaterFlowCheckErrorV1::new("tool principal id", error.to_string()))?,
    );
    let capability = CapabilityId::new(WATER_FLOW_CAPABILITY_ID).map_err(|error| {
        WaterFlowCheckErrorV1::new("water flow capability id", error.to_string())
    })?;
    fixture
        .bootstrap
        .principal_registry
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes(sha256(
                    b"nextengine.principal.water-flow-check.v1\0",
                )),
                capability_subject_id: SchemaId::new(
                    "nextengine.capability-subject.water-flow-check",
                )
                .map_err(|error| {
                    WaterFlowCheckErrorV1::new("capability subject id", error.to_string())
                })?,
                status: PrincipalStatus::Active,
            },
        )
        .map_err(|error| {
            WaterFlowCheckErrorV1::new("register tool principal", error.to_string())
        })?;
    let stream_id = fixture
        .bootstrap
        .stream_registry
        .allocate_stream(principal.clone())
        .map_err(|error| WaterFlowCheckErrorV1::new("allocate tool stream", error.to_string()))?;
    fixture
        .authority
        .register(principal.clone(), [capability])
        .map_err(|error| {
            WaterFlowCheckErrorV1::new("grant water flow capability", error.to_string())
        })?;
    Ok((principal, stream_id))
}

fn run_flow_command(
    runtime: &mut RuntimeState,
    stream_id: CommandStreamId,
    principal: &IssuerPrincipal,
    sequence: &mut u64,
    payload: WaterFlowCommandV1,
) -> Result<next_runtime::TickReport, WaterFlowCheckErrorV1> {
    let command = WorldCommand::water_flow(
        stream_id,
        principal.clone(),
        *sequence,
        runtime.next_tick(),
        payload,
    )
    .map_err(|error| WaterFlowCheckErrorV1::new("build flow command", error.to_string()))?;
    *sequence += 1;
    runtime
        .run_tick([command])
        .map_err(|error| WaterFlowCheckErrorV1::new("run flow command tick", error.to_string()))
}

fn require_disposition(
    report: &next_runtime::TickReport,
    expected: CommandDisposition,
    label: &str,
) -> Result<(), WaterFlowCheckErrorV1> {
    let actual = report
        .results
        .iter()
        .map(|result| result.disposition.clone())
        .collect::<Vec<_>>();
    if actual != [expected] {
        return Err(WaterFlowCheckErrorV1::new(label, format!("{actual:?}")));
    }
    Ok(())
}

fn flow_event_count(report: &next_runtime::TickReport) -> u64 {
    report
        .events
        .iter()
        .filter(|event| matches!(event.payload, EventPayload::WaterFlow(_)))
        .count() as u64
}

fn evidence_digest(evidence: &GenerationEvidenceV1) -> ContentHash {
    let mut preimage = b"nextengine.water-flow-check.v1\0".to_vec();
    preimage.extend_from_slice(&evidence.analytic_drain_ticks.to_le_bytes());
    preimage.extend_from_slice(&evidence.drained_by_tick.unwrap_or(u64::MAX).to_le_bytes());
    for value in [
        evidence.level_a_initial_micrometres,
        evidence.level_a_final_micrometres,
        evidence.level_b_final_micrometres,
    ] {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        evidence.total_volume_initial_cubic_millimetres,
        evidence.total_volume_final_cubic_millimetres,
        evidence.source_volume_cubic_millimetres,
        evidence.sink_volume_cubic_millimetres,
    ] {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for flag in [
        evidence.conservation_exact,
        evidence.gate_closed_flux_zero,
        evidence.gate_reopened_flux_positive,
        evidence.checkpoint_round_trip,
        evidence.restored_run_identical,
    ] {
        preimage.push(u8::from(flag));
    }
    preimage.extend_from_slice(&evidence.committed_commands.to_le_bytes());
    preimage.extend_from_slice(&evidence.rejected_commands.to_le_bytes());
    preimage.extend_from_slice(&evidence.flow_events.to_le_bytes());
    preimage.extend_from_slice(evidence.final_state_root.as_bytes());
    preimage.extend_from_slice(evidence.final_physics_checkpoint_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn require(condition: bool, context: &str) -> Result<(), WaterFlowCheckErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(WaterFlowCheckErrorV1::condition(context))
    }
}
