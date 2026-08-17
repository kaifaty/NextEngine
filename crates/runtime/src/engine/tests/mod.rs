mod fixtures;
mod input;
mod input_configuration;
mod ledger;
mod root_motion;
mod world_routine;

use std::collections::BTreeMap;

use next_contracts::command::{
    CommandPayload, CommandPhase, DomainEvent, IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID,
    WorldCommand,
};
use next_contracts::identity::{
    CommandStreamKeyV1, CommandStreamRegistryV1, PrincipalRecordV1, PrincipalRegistryV1,
    PrincipalStatus, RuntimeDeterminismBundleV1, WorldIdentityManifestV1,
};
use next_contracts::ids::{
    CapabilityId, CommandId, CommandStreamId, ContentHash, InputSourceId, PersistentId,
    PhysicsWorldId, PlayerPrincipalId, ProjectId, SchemaId, SystemId, WorldNamespaceId,
    content_hash_from_bytes,
};
use next_contracts::input::{
    ActionBindingTransformV1, ActionMapManifestV1, CORE_CAMERA_ORBIT_ACTION_ID,
    CORE_GAMEPLAY_CONTEXT_STACK_ID, CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID,
    CORE_UI_BACK_ACTION_ID, CORE_UI_CONFIRM_ACTION_ID, CORE_UI_MENU_CONTEXT_ID,
    CORE_UI_NAVIGATE_ACTION_ID, InputContextCapturePolicyV1, InputContextStackV1, InputContextV1,
    InputContractError, InputMappingCodeV1, InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1, PlayerControllerBindingV1,
    TickRateProfileV1,
};
use next_contracts::ledger::{CausalIdentityKey, CausalIdentityKind, CommandStreamStateV1};
use next_contracts::physical_animation::{
    ROOT_MOTION_INTENT_SCHEMA_VERSION, ROOT_MOTION_MOVE_PERFORMED_PHASE_ID, RootMotionIntentV1,
    capsule_root_motion_profile_hash_v1,
};
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PHYSICAL_COMMAND_CAPABILITY_ID, PhysicsBodyDescriptorV1,
    PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2, PhysicsContactReportingV1, PhysicsGeometryV1,
    PhysicsMaterialDescriptorV1, PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1,
    PhysicsQuantizationProfileV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1,
    PhysicsWorldCheckpointV1,
};
use next_physics_api::{PhysicsBackendKind, PhysicsBackendPolicy};

use super::*;
use crate::AuthorityRegistry;
use fixtures::{
    PhysicalFixture, clipped_physical_fixture, command, fixture, fixture_for, movement_sample,
    physical_fixture,
};

fn exact_player_sample(
    fixture: &PhysicalFixture,
    sequence: u64,
    actions: Vec<PlayerActionV1>,
) -> InputSampleV1 {
    let binding = fixture
        .runtime
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("fixture controller binding");
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: binding.action_map_hash,
        action_map_revision: binding.action_map_revision,
        context_stack_hash: binding.context_stack_hash,
        context_stack_revision: binding.context_stack_revision,
        actions,
    };
    InputSampleV1 {
        schema_version: 1,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes().expect("frame"),
        sampled_wall_time: None,
    }
}
