use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CORE_MOVE_ACTION_ID, CapabilityId, CommandLedgerHash, CommandStreamId, ContactPhaseV1,
    ContentHash, InputSampleV1, InputSourceId, IssuerPrincipal, PHYSICAL_COMMAND_CAPABILITY_ID,
    PLAYER_ACTION_FRAME_SCHEMA_ID, PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS,
    PersistentId, PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsCoordinateProfileV1, PhysicsGeometryV1,
    PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1,
    PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1, PhysicsWorldId, PlayerActionFrameV1, PlayerActionPhaseV1,
    PlayerActionV1, PlayerActionValueV1, PlayerControllerBindingV1, PlayerPrincipalId,
    RPG_COMMAND_CAPABILITY_ID, SchemaId, StateRoot,
};
use next_runtime::{RuntimeFatalError, RuntimeState, SnapshotRestoreError};

use crate::{NeutralFixtureError, build_neutral_runtime_fixture, compute_world_checkpoint_root};

#[derive(Clone, Debug)]
pub struct NeutralPlayerFixture {
    pub bootstrap: next_runtime::RuntimeBootstrapV3,
    pub authority: next_runtime::AuthorityRegistry,
    pub principal: IssuerPrincipal,
    pub movement_stream_id: CommandStreamId,
    pub rpg_stream_id: CommandStreamId,
    pub source_id: InputSourceId,
    pub controller_id: PersistentId,
    pub body_id: PersistentId,
    pub physics_body_id: PhysicsBodyIdV1,
    pub action_map_hash: ContentHash,
    pub context_stack_hash: ContentHash,
}

pub fn build_neutral_player_fixture(
    project_id: &str,
) -> Result<NeutralPlayerFixture, NeutralFixtureError> {
    let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([0x51; 16]));
    let base = build_neutral_runtime_fixture(
        project_id,
        [(
            principal.clone(),
            vec![
                CapabilityId::new(PHYSICAL_COMMAND_CAPABILITY_ID)?,
                CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?,
            ],
        )],
    )?;
    let movement_stream_id = base
        .stream_for(&principal)
        .expect("neutral fixture allocates its declared principal stream");
    let mut bootstrap = base.bootstrap;
    let rpg_stream_id = bootstrap
        .stream_registry
        .allocate_stream(principal.clone())?;
    let source_id = InputSourceId::from_bytes([0x52; 16]);
    let controller_id = PersistentId::from_bytes([0x53; 16]);
    let body_id = PersistentId::from_bytes([0x54; 16]);
    let action_map_hash = ContentHash::from_bytes([0x55; 32]);
    let context_stack_hash = ContentHash::from_bytes([0x56; 32]);
    bootstrap.player_controller_registry.bindings.insert(
        source_id,
        PlayerControllerBindingV1 {
            principal: principal.clone(),
            source_id,
            controller_id,
            controlled_body_id: body_id,
            command_stream_id: movement_stream_id,
            action_map_hash,
            action_map_revision: 1,
            context_stack_hash,
            context_stack_revision: 1,
        },
    );
    let physics_body_id = PhysicsBodyIdV1 {
        subject_id: body_id,
        body_slot: 0,
    };
    bootstrap.physics_checkpoint = grounded_capsule_checkpoint(
        PhysicsWorldId::from_bytes(*bootstrap.world_identity.world_namespace.as_bytes()),
        physics_body_id,
        &bootstrap.tick_rate_profile,
        &bootstrap.authoritative_numeric_profile,
        &bootstrap.physics_quantization_profile,
    )?;
    Ok(NeutralPlayerFixture {
        bootstrap,
        authority: base.authority,
        principal,
        movement_stream_id,
        rpg_stream_id,
        source_id,
        controller_id,
        body_id,
        physics_body_id,
        action_map_hash,
        context_stack_hash,
    })
}

fn grounded_capsule_checkpoint(
    world_id: PhysicsWorldId,
    capsule_body_id: PhysicsBodyIdV1,
    tick_rate: &next_contracts::TickRateProfileV1,
    numeric: &next_contracts::AuthoritativeNumericProfileV1,
    quantization: &next_contracts::PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldCheckpointV1, NeutralFixtureError> {
    let material_id = SchemaId::new("nextengine.physics.material.reference-zero")?;
    let material = PhysicsMaterialDescriptorV1 {
        material_id: material_id.clone(),
        descriptor_revision: 1,
        static_friction_q16: 0,
        dynamic_friction_q16: 0,
        restitution_q16: 0,
        canonical_material_tags: Vec::new(),
    };
    let capsule_shape_id = PhysicsShapeIdV1 {
        body_id: capsule_body_id,
        shape_slot: 0,
    };
    let capsule_shape = PhysicsShapeDescriptorV1 {
        shape_id: capsule_shape_id,
        descriptor_revision: 1,
        local_pose: PhysicsPoseV1::default(),
        geometry: PhysicsGeometryV1::Capsule {
            radius_micrometres: 300_000,
            half_segment_micrometres: 600_000,
        },
        material_id: material_id.clone(),
        collision_layer: 0,
        collision_mask: 1,
        participation: PhysicsParticipationV1::Solid,
        contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
    };
    let capsule_pose = PhysicsPoseV1 {
        translation_micrometres: [0, 900_000, 0],
        ..PhysicsPoseV1::default()
    };
    let capsule = PhysicsBodyDescriptorV1 {
        body_id: capsule_body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Kinematic,
        initial_pose: capsule_pose,
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(capsule_shape_id, capsule_shape)]),
    };
    let floor_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x57; 16]),
        body_slot: 0,
    };
    let floor_shape_id = PhysicsShapeIdV1 {
        body_id: floor_body_id,
        shape_slot: 0,
    };
    let floor = static_box_descriptor(
        floor_body_id,
        floor_shape_id,
        &material_id,
        [0, -100_000, 0],
        [10_000_000, 100_000, 10_000_000],
    );
    let wall_body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x58; 16]),
        body_slot: 0,
    };
    let wall_shape_id = PhysicsShapeIdV1 {
        body_id: wall_body_id,
        shape_slot: 0,
    };
    let wall = static_box_descriptor(
        wall_body_id,
        wall_shape_id,
        &material_id,
        [0, 900_000, 700_000],
        [10_000_000, 10_000_000, 100_000],
    );
    let catalog = PhysicsWorldCatalogV1::new(
        world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()?,
            limits: PhysicsLimitsProfileV1::reference_v1()?,
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()?,
            tick_rate_hash: tick_rate.profile_hash()?,
            authoritative_numeric_hash: numeric.profile_hash()?,
            quantization_hash: quantization.profile_hash()?,
        },
        BTreeMap::from([(material_id, material)]),
        BTreeMap::from([
            (capsule_body_id, capsule),
            (floor_body_id, floor),
            (wall_body_id, wall),
        ]),
        BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
    )?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(&catalog, tick_rate, numeric, quantization)?;
    Ok(PhysicsWorldCheckpointV1::new(catalog, snapshot)?)
}

fn static_box_descriptor(
    body_id: PhysicsBodyIdV1,
    shape_id: PhysicsShapeIdV1,
    material_id: &SchemaId,
    translation_micrometres: [i64; 3],
    half_extents_micrometres: [i64; 3],
) -> PhysicsBodyDescriptorV1 {
    PhysicsBodyDescriptorV1 {
        body_id,
        descriptor_revision: 1,
        motion_kind: PhysicsMotionKindV1::Static,
        initial_pose: PhysicsPoseV1 {
            translation_micrometres,
            ..PhysicsPoseV1::default()
        },
        initial_linear_velocity_micrometres_per_second: [0; 3],
        initial_angular_velocity_q16: [0; 3],
        active: true,
        shapes: BTreeMap::from([(
            shape_id,
            PhysicsShapeDescriptorV1 {
                shape_id,
                descriptor_revision: 1,
                local_pose: PhysicsPoseV1::default(),
                geometry: PhysicsGeometryV1::Box {
                    half_extents_micrometres,
                },
                material_id: material_id.clone(),
                collision_layer: 0,
                collision_mask: 1,
                participation: PhysicsParticipationV1::Solid,
                contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
            },
        )]),
    }
}

pub fn player_action_sample(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    phase: PlayerActionPhaseV1,
    direction_q15: [i16; 2],
    sampled_wall_time: Option<i64>,
) -> Result<InputSampleV1, CanonicalFixtureError> {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions: vec![PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID)?,
            phase,
            value: PlayerActionValueV1::Vector2Q15(direction_q15),
            semantic_occurrence_ordinal: 0,
        }],
    };
    Ok(InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS)?,
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)?,
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes()?,
        sampled_wall_time,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayCheckReport {
    pub ticks: u64,
    pub final_pose: PhysicsPoseV1,
    pub events: u64,
    pub final_command_ledger_hash: CommandLedgerHash,
    pub final_state_root: StateRoot,
}

pub fn run_play_check() -> Result<PlayCheckReport, PlayCheckError> {
    let scenario = run_grounded_collision_scenario()?;
    let checkpoint = scenario.runtime.world_checkpoint()?;
    Ok(PlayCheckReport {
        ticks: scenario.ticks,
        final_pose: scenario.final_pose,
        events: scenario.events,
        final_command_ledger_hash: checkpoint.runtime_snapshot.command_ledger_hash()?,
        final_state_root: compute_world_checkpoint_root(&checkpoint)?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCollisionCheckReport {
    pub gameplay_ticks: u64,
    pub physics_substeps: u64,
    pub begin_contacts: u64,
    pub persist_contacts: u64,
    pub end_contacts: u64,
    pub final_pose: PhysicsPoseV1,
    pub contact_batches_hash: ContentHash,
    pub physics_checkpoint_hash: ContentHash,
}

pub fn run_physics_collision_check() -> Result<PhysicsCollisionCheckReport, PlayCheckError> {
    let scenario = run_grounded_collision_scenario()?;
    Ok(PhysicsCollisionCheckReport {
        gameplay_ticks: scenario.ticks,
        physics_substeps: scenario.runtime.physics_snapshot().physics_tick,
        begin_contacts: scenario.begin_contacts,
        persist_contacts: scenario.persist_contacts,
        end_contacts: scenario.end_contacts,
        final_pose: scenario.final_pose,
        contact_batches_hash: scenario.contact_batches_hash,
        physics_checkpoint_hash: scenario.runtime.physics_checkpoint().checkpoint_hash()?,
    })
}

struct GroundedCollisionScenario {
    runtime: RuntimeState,
    ticks: u64,
    final_pose: PhysicsPoseV1,
    events: u64,
    begin_contacts: u64,
    persist_contacts: u64,
    end_contacts: u64,
    contact_batches_hash: ContentHash,
}

fn run_grounded_collision_scenario() -> Result<GroundedCollisionScenario, PlayCheckError> {
    let fixture = build_neutral_player_fixture("nextengine.play")?;
    let mut runtime = RuntimeState::new(fixture.bootstrap.clone(), fixture.authority.clone())?;
    let inputs = [
        (PlayerActionPhaseV1::Started, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, -32_767]),
        (PlayerActionPhaseV1::Completed, [0, 0]),
    ];
    let mut events = 0_u64;
    let mut begin_contacts = 0_u64;
    let mut persist_contacts = 0_u64;
    let mut end_contacts = 0_u64;
    let mut contact_preimage = b"nextengine.physics-collision-check.contacts.v1\0".to_vec();
    for (sequence, (phase, direction)) in inputs.into_iter().enumerate() {
        let sample = player_action_sample(
            &fixture,
            u64::try_from(sequence).map_err(|_| PlayCheckError::CountOverflow)?,
            phase,
            direction,
            Some(i64::try_from(sequence).map_err(|_| PlayCheckError::CountOverflow)? * 1000),
        )?;
        runtime.enqueue_input_sample(&fixture.principal, sample)?;
        let report = runtime.run_tick([])?;
        events = events
            .checked_add(
                u64::try_from(report.events.len()).map_err(|_| PlayCheckError::CountOverflow)?,
            )
            .ok_or(PlayCheckError::CountOverflow)?;
        for contact in &report.contact_batch.events {
            let count = match contact.phase {
                ContactPhaseV1::Begin => &mut begin_contacts,
                ContactPhaseV1::Persist => &mut persist_contacts,
                ContactPhaseV1::End => &mut end_contacts,
            };
            *count = count.checked_add(1).ok_or(PlayCheckError::CountOverflow)?;
        }
        contact_preimage.extend_from_slice(report.contact_batch.batch_hash.as_bytes());
    }
    let final_pose = runtime
        .physics_snapshot()
        .sorted_body_states
        .get(&fixture.physics_body_id)
        .ok_or(PlayCheckError::BodyMissing)?
        .pose;
    if final_pose.translation_micrometres != [0, 900_000, 200_000]
        || runtime.physics_snapshot().physics_tick != 12
        || events != 4
        || begin_contacts != 2
        || persist_contacts != 13
        || end_contacts != 1
    {
        return Err(PlayCheckError::AcceptanceMismatch);
    }
    Ok(GroundedCollisionScenario {
        ticks: 6,
        final_pose,
        events,
        runtime,
        begin_contacts,
        persist_contacts,
        end_contacts,
        contact_batches_hash: ContentHash::from_bytes(next_contracts::sha256(&contact_preimage)),
    })
}

#[derive(Debug)]
pub enum CanonicalFixtureError {
    Identifier(next_contracts::IdentifierError),
    Canonical(next_contracts::CanonicalError),
}

impl Display for CanonicalFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for CanonicalFixtureError {}

impl From<next_contracts::IdentifierError> for CanonicalFixtureError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<next_contracts::CanonicalError> for CanonicalFixtureError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[derive(Debug)]
pub enum PlayCheckError {
    Fixture(NeutralFixtureError),
    CanonicalFixture(CanonicalFixtureError),
    Input(next_runtime::InputAdmissionError),
    Runtime(RuntimeFatalError),
    Restore(SnapshotRestoreError),
    Checkpoint(next_contracts::WorldCheckpointError),
    Replay(crate::ReplayError),
    Ledger(next_contracts::CommandLedgerError),
    Canonical(next_contracts::CanonicalError),
    CountOverflow,
    BodyMissing,
    AcceptanceMismatch,
}

impl Display for PlayCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixture(error) => write!(formatter, "{error}"),
            Self::CanonicalFixture(error) => write!(formatter, "{error}"),
            Self::Input(error) => write!(formatter, "{error}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
            Self::Checkpoint(error) => write!(formatter, "{error}"),
            Self::Replay(error) => write!(formatter, "{error}"),
            Self::Ledger(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::CountOverflow => formatter.write_str("play check count overflow"),
            Self::BodyMissing => formatter.write_str("play check capsule body is missing"),
            Self::AcceptanceMismatch => formatter.write_str("play check result did not match"),
        }
    }
}

impl Error for PlayCheckError {}

impl From<NeutralFixtureError> for PlayCheckError {
    fn from(error: NeutralFixtureError) -> Self {
        Self::Fixture(error)
    }
}

impl From<CanonicalFixtureError> for PlayCheckError {
    fn from(error: CanonicalFixtureError) -> Self {
        Self::CanonicalFixture(error)
    }
}

impl From<next_runtime::InputAdmissionError> for PlayCheckError {
    fn from(error: next_runtime::InputAdmissionError) -> Self {
        Self::Input(error)
    }
}

impl From<RuntimeFatalError> for PlayCheckError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<SnapshotRestoreError> for PlayCheckError {
    fn from(error: SnapshotRestoreError) -> Self {
        Self::Restore(error)
    }
}

impl From<next_contracts::WorldCheckpointError> for PlayCheckError {
    fn from(error: next_contracts::WorldCheckpointError) -> Self {
        Self::Checkpoint(error)
    }
}

impl From<crate::ReplayError> for PlayCheckError {
    fn from(error: crate::ReplayError) -> Self {
        Self::Replay(error)
    }
}

impl From<next_contracts::CommandLedgerError> for PlayCheckError {
    fn from(error: next_contracts::CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<next_contracts::CanonicalError> for PlayCheckError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}
