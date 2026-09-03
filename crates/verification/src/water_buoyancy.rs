//! `CONTINUUM-WATER-BUOYANCY-P1` (ADR-105, plan `continuum-water/08`): the
//! reference crate floats on the exact basin level through one exact
//! impulse batch per tick inside the physics step input, follows a raised
//! level, and the batch never touches a body outside the water.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Instant;

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::identity::{PrincipalRecordV1, PrincipalStatus};
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ContentHash, PersistentId, PhysicsWorldId, SchemaId, StateRoot,
    ToolPrincipalId, content_hash_from_bytes,
};
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsBodyStateV2,
    PhysicsCanonicalSnapshotV2, PhysicsContactReportingV1, PhysicsCoordinateProfileV1,
    PhysicsGeometryV1, PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsQuantizationProfileV1, PhysicsShapeDescriptorV1,
    PhysicsShapeIdV1, PhysicsSolverSemanticsProfileV1, PhysicsStepInputV2,
    PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1, WATER_VOLUME_CAPABILITY_ID,
    WaterBuoyancyBatchV1, WaterBuoyancyProfileV1, WaterExchangeContextV1, WaterVolumeCommandV1,
    WaterVolumeDefinitionV1, WaterVolumeSetV1,
};
use next_reference_game::{
    REFERENCE_WATER_BASIN_ID, REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
    REFERENCE_WATER_CRATE_BODY_ID, REFERENCE_WATER_CRATE_HALF_EXTENTS_MICROMETRES,
    REFERENCE_WATER_FLOW_TICKS_PER_SECOND, ReferenceGameSession, cooked_project_rpg_snapshot,
};
use next_runtime::{CommandDisposition, RuntimeState};

use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

const TOOL_PRINCIPAL_ID: &str = "nextengine.tool.water-buoyancy-check";
/// Plan `continuum-water/08`: `600` ticks to settle, the level command at
/// tick `600`, `600` more ticks, the save at tick `900`.
const RUN_TICKS: u64 = 1_200;
const LEVEL_COMMAND_TICK: u64 = 600;
const SAVE_TICK: u64 = 900;
const RAISED_LEVEL_MICROMETRES: i64 = 1_500_000;
/// G1/G2: equilibrium immersion `0.20 +- 0.05 m`.
const EXPECTED_IMMERSION_MICROMETRES: i64 = 200_000;
const IMMERSION_TOLERANCE_MICROMETRES: i64 = 50_000;
/// G6 synthetic batch: the record bounds of ADR-105.
const COST_BODIES: usize = 64;
const COST_VOLUMES: usize = 64;
const COST_SAMPLES: u32 = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterBuoyancyCheckReportV1 {
    pub crate_body_id: PhysicsBodyIdV1,
    pub dry_body_id: PhysicsBodyIdV1,
    pub run_ticks: u64,
    pub level_initial_micrometres: i64,
    pub level_raised_micrometres: i64,
    pub immersion_settled_micrometres: i64,
    pub immersion_raised_micrometres: i64,
    pub crate_bottom_raised_micrometres: i64,
    pub crate_max_velocity_micrometres_per_second: i64,
    pub ticks_with_crate_record: u64,
    pub max_displaced_volume_cubic_millimetres: i64,
    pub dry_body_records: u64,
    pub dry_body_trajectory_identical: bool,
    pub step_inputs_round_trip: bool,
    pub checkpoint_round_trip: bool,
    pub restored_run_identical: bool,
    pub repeated_run_identical: bool,
    pub batch_cost_bodies: usize,
    pub batch_cost_volumes: usize,
    pub batch_cost_max_microseconds: u128,
    pub batch_cost_debug_build: bool,
    pub final_state_root: StateRoot,
    pub final_physics_checkpoint_hash: ContentHash,
    pub matrix_digest: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaterBuoyancyCheckErrorV1 {
    context: String,
    detail: String,
}

impl WaterBuoyancyCheckErrorV1 {
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

impl Display for WaterBuoyancyCheckErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for WaterBuoyancyCheckErrorV1 {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GenerationEvidenceV1 {
    dry_body_id: PhysicsBodyIdV1,
    immersion_settled_micrometres: i64,
    immersion_raised_micrometres: i64,
    crate_bottom_raised_micrometres: i64,
    crate_max_velocity_micrometres_per_second: i64,
    ticks_with_crate_record: u64,
    max_displaced_volume_cubic_millimetres: i64,
    dry_body_records: u64,
    dry_body_trajectory_identical: bool,
    step_inputs_round_trip: bool,
    checkpoint_round_trip: bool,
    restored_run_identical: bool,
    final_state_root: StateRoot,
    final_physics_checkpoint_hash: ContentHash,
}

pub fn run_water_buoyancy_check() -> Result<WaterBuoyancyCheckReportV1, WaterBuoyancyCheckErrorV1> {
    let scratch = ScratchContext::new(&std::env::temp_dir()).map_err(|error| {
        WaterBuoyancyCheckErrorV1::new("create scratch context", error.to_string())
    })?;
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.water-buoyancy-check")
            .map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("activate reference package", error.to_string())
        })?;
    let result = (|| {
        let first = run_generation(&prepared.package)?;
        let repeated = run_generation(&prepared.package)?;
        if repeated != first {
            return Err(WaterBuoyancyCheckErrorV1::condition(
                "repeated water buoyancy generation is identical",
            ));
        }
        let batch_cost_max_microseconds = batch_cost()?;
        let matrix_digest = evidence_digest(&first);
        Ok(WaterBuoyancyCheckReportV1 {
            crate_body_id: REFERENCE_WATER_CRATE_BODY_ID,
            dry_body_id: first.dry_body_id,
            run_ticks: RUN_TICKS,
            level_initial_micrometres: REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
            level_raised_micrometres: RAISED_LEVEL_MICROMETRES,
            immersion_settled_micrometres: first.immersion_settled_micrometres,
            immersion_raised_micrometres: first.immersion_raised_micrometres,
            crate_bottom_raised_micrometres: first.crate_bottom_raised_micrometres,
            crate_max_velocity_micrometres_per_second: first
                .crate_max_velocity_micrometres_per_second,
            ticks_with_crate_record: first.ticks_with_crate_record,
            max_displaced_volume_cubic_millimetres: first.max_displaced_volume_cubic_millimetres,
            dry_body_records: first.dry_body_records,
            dry_body_trajectory_identical: first.dry_body_trajectory_identical,
            step_inputs_round_trip: first.step_inputs_round_trip,
            checkpoint_round_trip: first.checkpoint_round_trip,
            restored_run_identical: first.restored_run_identical,
            repeated_run_identical: true,
            batch_cost_bodies: COST_BODIES,
            batch_cost_volumes: COST_VOLUMES,
            batch_cost_max_microseconds,
            batch_cost_debug_build: cfg!(debug_assertions),
            final_state_root: first.final_state_root,
            final_physics_checkpoint_hash: first.final_physics_checkpoint_hash,
            matrix_digest,
        })
    })();
    prepared.finish(result, |error| {
        WaterBuoyancyCheckErrorV1::new("remove reference package", error.to_string())
    })
}

fn run_generation(
    package: &next_project::ActivatedProjectPackage,
) -> Result<GenerationEvidenceV1, WaterBuoyancyCheckErrorV1> {
    let mut fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("build reference session", error.to_string())
        })?;
    let (principal, stream_id) = register_tool_principal(&mut fixture)?;
    require(
        fixture
            .bootstrap
            .physics_checkpoint
            .water_buoyancy
            .is_some(),
        "reference scene declares the buoyancy profile",
    )?;
    let crate_id = REFERENCE_WATER_CRATE_BODY_ID;
    // G4: the R5b push box, a dynamic body outside every water volume.
    let dry_id = fixture.r5b_course.dynamic_body_id;

    let mut runtime = activate(&fixture, fixture.bootstrap.clone())?;
    // G4 apparatus: the same scene without the batch profile.
    let mut dry_bootstrap = fixture.bootstrap.clone();
    dry_bootstrap.physics_checkpoint.water_buoyancy = None;
    let mut without_batch = activate(&fixture, dry_bootstrap)?;

    let mut sequence = 0_u64;
    let mut ticks_with_crate_record = 0_u64;
    let mut max_displaced_volume: i64 = 0;
    let mut dry_body_records = 0_u64;
    let mut dry_body_trajectory_identical = true;
    let mut step_inputs_round_trip = true;
    let mut crate_max_velocity: i64 = 0;
    let mut immersion_settled = 0_i64;
    let mut restored: Option<RuntimeState> = None;
    let mut checkpoint_round_trip = false;
    let mut restored_run_identical = true;

    for tick in 0..RUN_TICKS {
        // The runtime's own tick counter binds the tuple, not the loop index.
        let expected = expected_batch(&runtime, runtime.next_tick())?;
        let report = if tick == LEVEL_COMMAND_TICK {
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
            report
        } else {
            runtime
                .run_tick(Vec::<WorldCommand>::new())
                .map_err(|error| WaterBuoyancyCheckErrorV1::new("run tick", error.to_string()))?
        };
        without_batch
            .run_tick(Vec::<WorldCommand>::new())
            .map_err(|error| WaterBuoyancyCheckErrorV1::new("run dry tick", error.to_string()))?;

        // The batch of this tick rides the step input; it round-trips and
        // binds the committed roots (G7).
        let input = &report.physics_step_input;
        let bytes = input.canonical_bytes().map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("encode step input", error.to_string())
        })?;
        if PhysicsStepInputV2::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .ok()
            .as_ref()
            != Some(input)
        {
            step_inputs_round_trip = false;
        }
        if input
            .external_impulses
            .iter()
            .any(|impulse| impulse.body_id == crate_id)
        {
            ticks_with_crate_record += 1;
        }
        if input
            .external_impulses
            .iter()
            .any(|impulse| impulse.body_id == dry_id)
        {
            dry_body_records += 1;
        }
        if input.external_impulses.iter().any(|impulse| {
            impulse.exchange.destination_root != input.expected_snapshot_hash
                || impulse.exchange.destination_revision != input.expected_world_revision
                || impulse.exchange.gameplay_tick != input.gameplay_tick
        }) {
            return Err(WaterBuoyancyCheckErrorV1::condition(
                "every record binds the step input's roots",
            ));
        }
        if let Some(record) = expected.record(crate_id) {
            max_displaced_volume =
                max_displaced_volume.max(record.displaced_volume_cubic_millimetres);
        }
        // The physics owner's batch equals the independent recomputation
        // from the committed state (same exact law, same bound roots). On
        // the command tick the level command commits before the physical
        // step, so the owner's batch binds the raised table (revision 1);
        // that tick verifies the tuple roots below instead.
        if tick != LEVEL_COMMAND_TICK && expected.external_impulses() != input.external_impulses {
            return Err(WaterBuoyancyCheckErrorV1::new(
                "the step input carries exactly the recomputed batch",
                format!(
                    "tick {tick}: expected {:?} vs input {:?}",
                    expected.external_impulses(),
                    input.external_impulses
                ),
            ));
        }

        let crate_state = body_state(&runtime, crate_id)?;
        crate_max_velocity =
            crate_max_velocity.max(crate_state.linear_velocity_micrometres_per_second[1].abs());
        if body_state(&runtime, dry_id)? != body_state(&without_batch, dry_id)? {
            dry_body_trajectory_identical = false;
        }

        if tick + 1 == LEVEL_COMMAND_TICK {
            immersion_settled = immersion(&runtime, crate_id, tick + 1)?;
        }

        if let Some(restored) = restored.as_mut() {
            let restored_report =
                restored
                    .run_tick(Vec::<WorldCommand>::new())
                    .map_err(|error| {
                        WaterBuoyancyCheckErrorV1::new("run restored tick", error.to_string())
                    })?;
            if restored_report.physics_checkpoint_hash != report.physics_checkpoint_hash
                || restored_report.physics_step_input != report.physics_step_input
                || restored_report.events != report.events
            {
                restored_run_identical = false;
            }
        }
        if tick == SAVE_TICK {
            let checkpoint = runtime.world_checkpoint().map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("build world checkpoint", error.to_string())
            })?;
            let physics_bytes =
                checkpoint
                    .physics_checkpoint
                    .canonical_bytes()
                    .map_err(|error| {
                        WaterBuoyancyCheckErrorV1::new(
                            "encode physics checkpoint",
                            error.to_string(),
                        )
                    })?;
            let decoded = next_contracts::physics::PhysicsWorldCheckpointV1::from_canonical_bytes(
                &physics_bytes,
                CanonicalDecodeLimits::default(),
            )
            .map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("decode physics checkpoint", error.to_string())
            })?;
            checkpoint_round_trip =
                decoded == checkpoint.physics_checkpoint && decoded.water_buoyancy.is_some();
            require(
                checkpoint_round_trip,
                "physics checkpoint with the buoyancy profile round-trips",
            )?;
            let candidate = RuntimeState::restore_world_checkpoint_with_definitions(
                checkpoint,
                fixture.authority.clone(),
                fixture.activated_project.rpg_definitions.clone(),
            )
            .map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("restore runtime", error.to_string())
            })?;
            restored = Some(candidate);
        }
    }

    let immersion_raised = immersion(&runtime, crate_id, RUN_TICKS)?;
    let crate_bottom_raised = bottom(&runtime, crate_id)?;
    require(
        (immersion_settled - EXPECTED_IMMERSION_MICROMETRES).abs()
            <= IMMERSION_TOLERANCE_MICROMETRES,
        "G1: settled immersion within 0.20 +- 0.05 m",
    )?;
    require(
        (immersion_raised - EXPECTED_IMMERSION_MICROMETRES).abs()
            <= IMMERSION_TOLERANCE_MICROMETRES
            && crate_bottom_raised > REFERENCE_WATER_BASIN_INITIAL_LEVEL_MICROMETRES,
        "G2: the crate follows the raised level",
    )?;
    require(dry_body_records == 0, "G4: the dry body gets no record")?;
    require(
        dry_body_trajectory_identical,
        "G4: the dry body's trajectory equals the run without the batch",
    )?;
    require(step_inputs_round_trip, "G7: every step input round-trips")?;
    require(ticks_with_crate_record > 0, "the crate receives records")?;
    let restored = restored
        .ok_or_else(|| WaterBuoyancyCheckErrorV1::condition("a restored runtime exists"))?;
    let live_final = runtime.world_checkpoint().map_err(|error| {
        WaterBuoyancyCheckErrorV1::new("build final checkpoint", error.to_string())
    })?;
    let restored_final = restored.world_checkpoint().map_err(|error| {
        WaterBuoyancyCheckErrorV1::new("build restored final checkpoint", error.to_string())
    })?;
    restored_run_identical =
        restored_run_identical && live_final.state_root == restored_final.state_root;
    require(
        restored_run_identical,
        "G3: restored and live runs reach the same roots",
    )?;
    let final_physics_checkpoint_hash =
        live_final
            .physics_checkpoint
            .checkpoint_hash()
            .map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("hash final physics checkpoint", error.to_string())
            })?;
    Ok(GenerationEvidenceV1 {
        dry_body_id: dry_id,
        immersion_settled_micrometres: immersion_settled,
        immersion_raised_micrometres: immersion_raised,
        crate_bottom_raised_micrometres: crate_bottom_raised,
        crate_max_velocity_micrometres_per_second: crate_max_velocity,
        ticks_with_crate_record,
        max_displaced_volume_cubic_millimetres: max_displaced_volume,
        dry_body_records,
        dry_body_trajectory_identical,
        step_inputs_round_trip,
        checkpoint_round_trip,
        restored_run_identical,
        final_state_root: live_final.state_root,
        final_physics_checkpoint_hash,
    })
}

fn activate(
    fixture: &ReferenceGameSession,
    bootstrap: next_runtime::RuntimeBootstrapV4,
) -> Result<RuntimeState, WaterBuoyancyCheckErrorV1> {
    RuntimeState::with_rpg_snapshot(
        bootstrap,
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(fixture),
    )
    .map_err(|error| WaterBuoyancyCheckErrorV1::new("activate runtime", error.to_string()))
}

/// Recomputes the batch of the coming tick from the committed state the
/// runtime will build it from (the committed table and the committed
/// poses), with the roots the runtime binds.
fn expected_batch(
    runtime: &RuntimeState,
    tick: u64,
) -> Result<WaterBuoyancyBatchV1, WaterBuoyancyCheckErrorV1> {
    let checkpoint = runtime.physics_checkpoint();
    let profile = checkpoint
        .water_buoyancy
        .as_ref()
        .ok_or_else(|| WaterBuoyancyCheckErrorV1::condition("buoyancy profile present"))?;
    let context =
        WaterExchangeContextV1 {
            world_id: checkpoint.snapshot.world_id,
            source_revision: checkpoint
                .water_volumes
                .states
                .values()
                .map(|state| state.record_revision)
                .max()
                .unwrap_or(0),
            source_root: checkpoint.water_volumes.set_hash().map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("water set hash", error.to_string())
            })?,
            destination_revision: checkpoint.snapshot.world_revision,
            destination_root: checkpoint.snapshot.snapshot_hash().map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("snapshot hash", error.to_string())
            })?,
        };
    WaterBuoyancyBatchV1::compute(
        profile,
        &checkpoint.water_volumes,
        &checkpoint.catalog,
        &checkpoint.snapshot,
        tick,
        // The gameplay rate of the world profile; the flow network
        // cross-checks it against the tick rate at activation.
        REFERENCE_WATER_FLOW_TICKS_PER_SECOND,
        &context,
    )
    .map_err(|error| WaterBuoyancyCheckErrorV1::new("recompute batch", error.to_string()))
}

fn body_state(
    runtime: &RuntimeState,
    body_id: PhysicsBodyIdV1,
) -> Result<PhysicsBodyStateV2, WaterBuoyancyCheckErrorV1> {
    runtime
        .physics_checkpoint()
        .snapshot
        .sorted_body_states
        .get(&body_id)
        .cloned()
        .ok_or_else(|| WaterBuoyancyCheckErrorV1::condition("body exists in the snapshot"))
}

fn bottom(
    runtime: &RuntimeState,
    body_id: PhysicsBodyIdV1,
) -> Result<i64, WaterBuoyancyCheckErrorV1> {
    Ok(
        body_state(runtime, body_id)?.pose.translation_micrometres[1]
            - REFERENCE_WATER_CRATE_HALF_EXTENTS_MICROMETRES[1],
    )
}

/// Level minus the crate's bottom face.
fn immersion(
    runtime: &RuntimeState,
    body_id: PhysicsBodyIdV1,
    tick: u64,
) -> Result<i64, WaterBuoyancyCheckErrorV1> {
    let level = runtime
        .physics_checkpoint()
        .water_volumes
        .effective_level(REFERENCE_WATER_BASIN_ID, tick)
        .ok_or_else(|| WaterBuoyancyCheckErrorV1::condition("basin has a level"))?;
    Ok(level - bottom(runtime, body_id)?)
}

/// G6: one batch computation at the record bounds (64 bodies over 64
/// volumes, every body immersed).
fn batch_cost() -> Result<u128, WaterBuoyancyCheckErrorV1> {
    let tick = TickRateProfileV1::at_30_hz();
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1()
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("quantization", error.to_string()))?;
    let numeric = AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization)
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("numeric", error.to_string()))?;
    let material_id = SchemaId::new("nextengine.physics.material.buoyancy-cost")
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("material id", error.to_string()))?;
    let material = PhysicsMaterialDescriptorV1 {
        material_id: material_id.clone(),
        descriptor_revision: 1,
        static_friction_q16: 0,
        dynamic_friction_q16: 0,
        restitution_q16: 0,
        canonical_material_tags: Vec::new(),
    };
    let mut bodies = BTreeMap::new();
    let mut definitions = Vec::with_capacity(COST_VOLUMES);
    for index in 0..COST_BODIES {
        let x = i64::try_from(index).unwrap_or(0) * 3_000_000;
        let body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([u8::try_from(index).unwrap_or(0); 16]),
            body_slot: 0,
        };
        let shape_id = PhysicsShapeIdV1 {
            body_id,
            shape_slot: 0,
        };
        bodies.insert(
            body_id,
            PhysicsBodyDescriptorV1 {
                body_id,
                descriptor_revision: 1,
                motion_kind: PhysicsMotionKindV1::Dynamic,
                initial_pose: PhysicsPoseV1 {
                    translation_micrometres: [x + 1_000_000, 250_000, 1_000_000],
                    ..PhysicsPoseV1::default()
                },
                initial_linear_velocity_micrometres_per_second: [0, -100_000, 0],
                initial_angular_velocity_q16: [0; 3],
                active: true,
                mass_microkilograms: 50_000_000,
                shapes: BTreeMap::from([(
                    shape_id,
                    PhysicsShapeDescriptorV1 {
                        shape_id,
                        descriptor_revision: 1,
                        local_pose: PhysicsPoseV1::default(),
                        geometry: PhysicsGeometryV1::Box {
                            half_extents_micrometres: [250_000; 3],
                        },
                        material_id: material_id.clone(),
                        collision_layer: 1,
                        collision_mask: u64::MAX,
                        participation: PhysicsParticipationV1::Solid,
                        contact_reporting: PhysicsContactReportingV1::Disabled,
                    },
                )]),
            },
        );
        definitions.push(WaterVolumeDefinitionV1 {
            volume_id: PersistentId::from_bytes([u8::try_from(index).unwrap_or(0); 16]),
            minimum_micrometres: [x, 0, 0],
            maximum_micrometres: [x + 2_000_000, 2_000_000, 2_000_000],
            initial_level_micrometres: 500_000,
            swimming_depth_micrometres: 1_200_000,
            level_ramp: None,
            profile_revision: 1,
        });
    }
    let catalog = PhysicsWorldCatalogV1::new(
        PhysicsWorldId::from_bytes([0xb1; 16]),
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()
                .map_err(|error| WaterBuoyancyCheckErrorV1::new("coordinate", error.to_string()))?,
            limits: PhysicsLimitsProfileV1::reference_v1()
                .map_err(|error| WaterBuoyancyCheckErrorV1::new("limits", error.to_string()))?,
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()
                .map_err(|error| WaterBuoyancyCheckErrorV1::new("solver", error.to_string()))?,
            tick_rate_hash: tick
                .profile_hash()
                .map_err(|error| WaterBuoyancyCheckErrorV1::new("tick hash", error.to_string()))?,
            authoritative_numeric_hash: numeric.profile_hash().map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("numeric hash", error.to_string())
            })?,
            quantization_hash: quantization.profile_hash().map_err(|error| {
                WaterBuoyancyCheckErrorV1::new("quantization hash", error.to_string())
            })?,
        },
        BTreeMap::from([(material_id, material)]),
        bodies,
        BTreeMap::new(),
    )
    .map_err(|error| WaterBuoyancyCheckErrorV1::new("cost catalog", error.to_string()))?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick, &numeric, &quantization)
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("cost snapshot", error.to_string()))?;
    let volumes = WaterVolumeSetV1::from_definitions(definitions)
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("cost volumes", error.to_string()))?;
    let profile = WaterBuoyancyProfileV1::reference_v1()
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("cost profile", error.to_string()))?;
    let context = WaterExchangeContextV1 {
        world_id: snapshot.world_id,
        source_revision: 0,
        source_root: ContentHash::default(),
        destination_revision: 0,
        destination_root: ContentHash::default(),
    };
    let mut maximum = 0_u128;
    for sample in 0..COST_SAMPLES {
        let started = Instant::now();
        let batch = WaterBuoyancyBatchV1::compute(
            &profile,
            &volumes,
            &catalog,
            &snapshot,
            u64::from(sample),
            tick.gameplay_hz,
            &context,
        )
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("cost batch", error.to_string()))?;
        maximum = maximum.max(started.elapsed().as_micros());
        if batch.records.len() != COST_BODIES {
            return Err(WaterBuoyancyCheckErrorV1::condition(
                "every cost body receives a record",
            ));
        }
    }
    Ok(maximum)
}

fn register_tool_principal(
    fixture: &mut ReferenceGameSession,
) -> Result<(IssuerPrincipal, CommandStreamId), WaterBuoyancyCheckErrorV1> {
    let principal =
        IssuerPrincipal::Tool(ToolPrincipalId::new(TOOL_PRINCIPAL_ID).map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("tool principal id", error.to_string())
        })?);
    let capability = CapabilityId::new(WATER_VOLUME_CAPABILITY_ID).map_err(|error| {
        WaterBuoyancyCheckErrorV1::new("water capability id", error.to_string())
    })?;
    fixture
        .bootstrap
        .principal_registry
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes(sha256(
                    b"nextengine.principal.water-buoyancy-check.v1\0",
                )),
                capability_subject_id: SchemaId::new(
                    "nextengine.capability-subject.water-buoyancy-check",
                )
                .map_err(|error| {
                    WaterBuoyancyCheckErrorV1::new("capability subject id", error.to_string())
                })?,
                status: PrincipalStatus::Active,
            },
        )
        .map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("register tool principal", error.to_string())
        })?;
    let stream_id = fixture
        .bootstrap
        .stream_registry
        .allocate_stream(principal.clone())
        .map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("allocate tool stream", error.to_string())
        })?;
    fixture
        .authority
        .register(principal.clone(), [capability])
        .map_err(|error| {
            WaterBuoyancyCheckErrorV1::new("grant water capability", error.to_string())
        })?;
    Ok((principal, stream_id))
}

fn run_water_command(
    runtime: &mut RuntimeState,
    stream_id: CommandStreamId,
    principal: &IssuerPrincipal,
    sequence: &mut u64,
    payload: WaterVolumeCommandV1,
) -> Result<next_runtime::TickReport, WaterBuoyancyCheckErrorV1> {
    let command = WorldCommand::water_volume(
        stream_id,
        principal.clone(),
        *sequence,
        runtime.next_tick(),
        payload,
    )
    .map_err(|error| WaterBuoyancyCheckErrorV1::new("build water command", error.to_string()))?;
    *sequence += 1;
    runtime
        .run_tick([command])
        .map_err(|error| WaterBuoyancyCheckErrorV1::new("run water tick", error.to_string()))
}

fn require_disposition(
    report: &next_runtime::TickReport,
    expected: CommandDisposition,
    label: &str,
) -> Result<(), WaterBuoyancyCheckErrorV1> {
    let actual = report
        .results
        .iter()
        .map(|result| result.disposition.clone())
        .collect::<Vec<_>>();
    if actual != [expected] {
        return Err(WaterBuoyancyCheckErrorV1::new(label, format!("{actual:?}")));
    }
    Ok(())
}

fn evidence_digest(evidence: &GenerationEvidenceV1) -> ContentHash {
    let mut preimage = b"nextengine.water-buoyancy-check.v1\0".to_vec();
    for value in [
        evidence.immersion_settled_micrometres,
        evidence.immersion_raised_micrometres,
        evidence.crate_bottom_raised_micrometres,
        evidence.crate_max_velocity_micrometres_per_second,
        evidence.max_displaced_volume_cubic_millimetres,
    ] {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    preimage.extend_from_slice(&evidence.ticks_with_crate_record.to_le_bytes());
    preimage.extend_from_slice(&evidence.dry_body_records.to_le_bytes());
    for flag in [
        evidence.dry_body_trajectory_identical,
        evidence.step_inputs_round_trip,
        evidence.checkpoint_round_trip,
        evidence.restored_run_identical,
    ] {
        preimage.push(u8::from(flag));
    }
    preimage.extend_from_slice(evidence.final_state_root.as_bytes());
    preimage.extend_from_slice(evidence.final_physics_checkpoint_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn require(condition: bool, context: &str) -> Result<(), WaterBuoyancyCheckErrorV1> {
    if condition {
        Ok(())
    } else {
        Err(WaterBuoyancyCheckErrorV1::condition(context))
    }
}
