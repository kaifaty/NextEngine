#![forbid(unsafe_code)]

mod canonical;
mod command;
mod identity;
mod ids;
mod input;
mod ledger;
mod manifest_jcs;
mod persistence;
mod physics;
mod rpg;
mod rpg_v1;
mod snapshot;

pub use canonical::{
    CANONICAL_BINARY_V1_MAGIC, CANONICAL_BINARY_V1_VERSION, CANONICAL_TYPE_BOOL,
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_F32_BITS, CANONICAL_TYPE_F64_BITS, CANONICAL_TYPE_HASH256,
    CANONICAL_TYPE_I8, CANONICAL_TYPE_I16, CANONICAL_TYPE_I32, CANONICAL_TYPE_I64,
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_SET, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UNIT,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment,
    sha256,
};
pub use command::{
    COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SCHEMA_VERSION,
    COMMAND_BODY_SEGMENT_ID, COMMAND_ENVELOPE_SCHEMA_VERSION, COMMAND_SCHEMA_VERSION,
    CanonicalCommandBodyV2, CapabilityRefV1, CommandDecodeError, CommandPayload, CommandPhase,
    CommandPreconditionV1, DomainEvent, DomainEventEnvelopeV2, EventPayload, IssuerPrincipal,
    IssuerPrincipalV2, NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID, PrincipalDecodeError,
    WorldCommand, WorldCommandEnvelopeV2, compute_command_id_from_body_bytes,
};
pub use identity::{
    COMMAND_STREAM_REGISTRY_SCHEMA_VERSION, CommandStreamKeyV1, CommandStreamRegistryV1,
    IdentityContractError, PRINCIPAL_REGISTRY_SCHEMA_VERSION, PrincipalRecordV1,
    PrincipalRegistryV1, PrincipalStatus, RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION,
    RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS, RuntimeDeterminismProfileV1,
    WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION, WorldIdentityManifestV1, derive_command_stream_id,
    derive_player_principal_id, derive_world_namespace, runtime_profile_hash,
};
pub use ids::{
    AssetId, CapabilityId, CommandBodyHash, CommandId, CommandLedgerHash, CommandStreamId,
    ContentHash, EventId, IdentifierError, InputSourceId, MechanicPackageId, PersistentId,
    PhysicsContactId, PhysicsWorldId, PlayerPrincipalId, PluginId, ProjectId, SchemaId,
    ScriptPrincipalId, StateRoot, SystemId, ToolPrincipalId, WorldNamespaceId,
    command_body_hash_from_bytes, command_ledger_hash_from_bytes, content_hash_from_bytes,
};
pub use input::{
    CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION, CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
    CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID, ClosedCommandAdmissionBatchBodyV2,
    ClosedCommandAdmissionBatchV2, ClosedIngressBatchBodyV1, ClosedIngressBatchV1,
    INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION, INGRESS_ASSIGNMENT_SCHEMA_VERSION,
    INGRESS_CHECKPOINT_SCHEMA_VERSION, INPUT_SAMPLE_SCHEMA_VERSION, IngressAssignmentProfileV1,
    IngressAssignmentV1, IngressCheckpointV1, IngressEquivalenceReceiptV1, IngressResultCodeV1,
    IngressSubjectKindV1, InputContractError, InputMappingCodeV1, InputMappingReceiptV1,
    InputSampleV1, MAX_PLAYER_ACTIONS_PER_FRAME, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS,
    PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION, PLAYER_INTERACTION_SYSTEM_ID, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1, PlayerControllerBindingV1,
    PlayerControllerRegistryV1, RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION, RuntimeAdmissionLimitsV1,
    TICK_RATE_PROFILE_SCHEMA_VERSION, TickRateProfileV1, core_player_action_map_v1_hash,
};
pub use ledger::{
    ArchiveInsertResult, CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION,
    COMMAND_BODY_ARCHIVE_SCHEMA_VERSION, COMMAND_IDENTITY_INDEX_BODY_OWNER_ID,
    COMMAND_IDENTITY_INDEX_BODY_SCHEMA_ID, COMMAND_IDENTITY_INDEX_BODY_SEGMENT_ID,
    COMMAND_IDENTITY_INDEX_SCHEMA_VERSION, COMMAND_LEDGER_SCHEMA_VERSION, COMMAND_PENDING_CAPACITY,
    COMMAND_RECEIPT_OWNER_ID, COMMAND_RECEIPT_SCHEMA_ID, COMMAND_RECEIPT_SCHEMA_VERSION,
    COMMAND_RECEIPT_SEGMENT_ID, COMMAND_RECEIPT_WINDOW_CAPACITY,
    COMMAND_RESERVATION_SCHEMA_VERSION, COMMAND_STREAM_LEDGER_SCHEMA_VERSION, CausalIdentityKey,
    CausalIdentityKind, CausalIdentityRegistryV1, CommandBodyArchiveManifestV1,
    CommandBodyArchiveV1, CommandCollisionCandidateV1, CommandCollisionIncidentV1,
    CommandFinalResultV1, CommandIdentityBindingState, CommandIdentityBindingV1,
    CommandIdentityIndexBodyV1, CommandIdentityIndexV1, CommandIdentityOccurrenceV1,
    CommandLedgerError, CommandLedgerV2, CommandReceiptSubjectV1, CommandReceiptV1,
    CommandReservationV1, CommandStreamLedgerV2, CommandStreamStateV1, IdentityInsertResult,
    causal_provenance_hash, command_body_archive_root, command_collision_candidates_root,
    command_collision_incident_digest, command_identity_index_root, command_receipt_chain_genesis,
    command_receipt_chain_next, command_receipt_digest,
};
pub use persistence::{
    AuthorityGrant, CommandLedgerDescriptorV2, DecodedReplayTickV4, HashBinding,
    ManifestCodecError, ManifestValidationError, REPLAY_MANIFEST_V4_SCHEMA_VERSION,
    ReplayCommandRecord, ReplayCommandResultV2, ReplayComparePointV4, ReplayManifestV4,
    ReplayOwnerSegmentV2, ReplayTickManifestV4, SAVE_MANIFEST_SCHEMA_VERSION, SaveCompatibility,
    SaveManifestV2, SaveSegmentDescriptor, SchemaBinding, TickSettings,
};
pub use physics::{
    AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION, AcceptedLocomotionIntentV2,
    AppliedLocomotionResultV1, AuthoritativeNumericProfileV1,
    CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND, CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION,
    ClosedPhysicsContactBatchV1, ContactEventV1, ContactPhaseV1, FixedPointDescriptorV1,
    LEGACY_PHYSICS_SNAPSHOT_SCHEMA_VERSION, LEGACY_PHYSICS_SNAPSHOT_SEGMENT_ID,
    PHYSICAL_COMMAND_CAPABILITY_ID, PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION,
    PHYSICS_CONTACT_NORMAL_X_FIELD_ID, PHYSICS_CONTACT_NORMAL_Y_FIELD_ID,
    PHYSICS_CONTACT_NORMAL_Z_FIELD_ID, PHYSICS_METRES_UNIT_ID, PHYSICS_MICROMETRES_FIXED_POINT_ID,
    PHYSICS_MICROMETRES_UNIT_ID, PHYSICS_Q1_30_FIXED_POINT_ID,
    PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION, PHYSICS_SCALAR_UNIT_ID, PHYSICS_SNAPSHOT_OWNER_ID,
    PHYSICS_SNAPSHOT_SCHEMA_ID, PHYSICS_SNAPSHOT_SCHEMA_VERSION, PHYSICS_SNAPSHOT_SEGMENT_ID,
    PHYSICS_STEP_INPUT_SCHEMA_VERSION, PHYSICS_SWEEP_DISTANCE_FIELD_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION,
    PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PhysicalCommandV1, PhysicalEventV1,
    PhysicsBodyDescriptorV1, PhysicsBodyIdV1, PhysicsBodyStateV2, PhysicsCanonicalSnapshotV2,
    PhysicsContactContinuityStateV1, PhysicsContactReportingV1, PhysicsContractError,
    PhysicsCoordinateProfileV1, PhysicsGeometryV1, PhysicsLimitsProfileV1,
    PhysicsMaterialDescriptorV1, PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1,
    PhysicsQuantizationProfileV1, PhysicsQuantizationRuleV1, PhysicsShapeDescriptorV1,
    PhysicsShapeIdV1, PhysicsSolverSemanticsProfileV1, PhysicsSourceFormatV1, PhysicsStepInputV2,
    PhysicsStepResultV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1, PhysicsWorldDescriptorV1,
    REFERENCE_GRAVITY_MICROMETRES_PER_SECOND_SQUARED, derive_physics_contact_id,
};
pub use rpg::{
    CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID, CORE_DIALOGUE_QUEST_TRUST_DELTA,
    CORE_HELP_DIALOGUE_DEFINITION_ID, CORE_HELP_QUEST_DEFINITION_ID,
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_INTERACTIVE_OBJECT_ARCHETYPE_ID,
    CORE_INTERACTIVE_OBJECT_READY_STATE_ID, CORE_QUEST_ACTIVE_STATE_ID,
    CORE_QUEST_AVAILABLE_STATE_ID, CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CharacterSnapshot, CoreDialogueQuestClosureError,
    CoreDialogueQuestProfileState, DialogueSnapshot, FactionSnapshot, InteractiveObjectSnapshot,
    ItemSnapshot, QuestSnapshot, RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID,
    RPG_EVENT_DIALOGUE_QUEST_ADVANCED_SCHEMA_ID,
    RPG_EVENT_INTERACTIVE_OBJECT_STATE_CHANGED_SCHEMA_ID, RPG_EVENT_ITEM_TRANSFERRED_SCHEMA_ID,
    RPG_EVENT_SKILL_LEARNED_SCHEMA_ID, RPG_SNAPSHOT_OWNER_ID, RPG_SNAPSHOT_SCHEMA_ID,
    RPG_SNAPSHOT_SCHEMA_VERSION, RPG_SNAPSHOT_SEGMENT_ID, RelationshipEntry,
    ResolvedCoreDialogueQuestBinding, RpgCommand, RpgDecodeError, RpgEvent, RpgSnapshot,
    SKILL_PROFICIENCY_MAX, SkillProficiency, SkillProficiencyEntry, SkillProficiencyError,
    WorldChunkRecordSnapshot,
};
pub use rpg_v1::{
    CharacterPayloadV1, DefinitionRefV1, DialoguePayloadV1, DivineStandingPayloadV1,
    EquipmentPayloadV1, EquipmentSlotAssignmentV1, FactionMembershipPayloadV1, FactionPayloadV1,
    InteractiveObjectPayloadV1, InventoryPayloadV1, InventoryReservationV1, ItemPayloadV1,
    ProvenanceBindingV1, QuestPayloadV1, RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
    RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RPG_MAX_AGGREGATES_PER_SNAPSHOT, RPG_MAX_COLLECTION_ENTRIES,
    RPG_MAX_OPERATIONS_PER_COMMAND, RPG_TRANSACTION_COMMAND_SCHEMA_VERSION,
    RPG_TRANSACTION_PLAN_SCHEMA_ID, RelationshipDimensionV1, RelationshipPayloadV1,
    RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1, RpgAggregateRefV1,
    RpgCommandV1, RpgContractErrorV1, RpgEventDraftV1, RpgEventV1, RpgOperationPayloadV1,
    RpgOperationV1, RpgReadSetEntryV1, RpgRuntimeBindingsV1, RpgSnapshotV2, RpgTransactionPlanV1,
    RpgWriteSetEntryV1, SkillProficiencyEntryV1,
};
pub use snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, RuntimeSnapshot, RuntimeSnapshotV3, SnapshotDecodeError,
    WorldCheckpointError, WorldCheckpointV3, WorldCheckpointV4,
    validate_core_dialogue_quest_world_closure, world_checkpoint_v3_state_root,
    world_checkpoint_v4_state_root,
};
