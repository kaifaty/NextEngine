use std::collections::{BTreeMap, BTreeSet};

use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::cognition::{
    AGENT_COGNITION_CAPABILITY_ID, AGENT_COGNITION_CAPABILITY_SUBJECT_ID, AGENT_COGNITION_SYSTEM_ID,
};
use next_contracts::command::IssuerPrincipal;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRegistryV1, RuntimeDeterminismBundleV1,
    RuntimeDeterminismProfileV1, WorldIdentityManifestV1,
};
use next_contracts::ids::{
    CapabilityId, ContentHash, PhysicsWorldId, ProjectId, SchemaId, SystemId,
    content_hash_from_bytes,
};
use next_contracts::input::{
    IngressAssignmentProfileV1, PlayerControllerRegistryV1, RuntimeAdmissionLimitsV1,
    TickRateProfileV1,
};
use next_contracts::ledger::{
    CausalIdentityKey, CausalIdentityKind, CommandLedgerV2, IdentityInsertResult,
};
use next_contracts::mechanics::{RpgDefinitionRegistryV2, interaction_definition_hash_v2};
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsMotionKindV1, PhysicsQuantizationProfileV1,
    PhysicsWorldCheckpointV1,
};
use next_contracts::rpg::{
    CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CoreDialogueQuestClosureError, RPG_COMMAND_CAPABILITY_ID,
};
use next_contracts::rpg::{RpgRuntimeBindingsV1, RpgSnapshotV2};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_CAPABILITY_ID, WORLD_ACTIVITY_CAPABILITY_SUBJECT_ID, WORLD_ACTIVITY_SYSTEM_ID,
};
use next_contracts::world_population::{
    WORLD_POPULATION_CAPABILITY_ID, WORLD_POPULATION_CAPABILITY_SUBJECT_ID,
    WORLD_POPULATION_SYSTEM_ID,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_CAPABILITY_ID, WORLD_ROUTINE_CAPABILITY_SUBJECT_ID, WORLD_ROUTINE_SYSTEM_ID,
};
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;
use crate::registry::{CommandKindRegistry, command_kind_registry_hash};

use super::affordance::resolve_dialogue_quest_binding_v2;
use super::error::SnapshotRestoreError;
use super::interaction::resolve_interaction_outcome_route;
use super::physics::empty_physics_checkpoint;
use super::policy::{core_rpg_policy_hash, domain_hash};

fn bootstrap_rpg_bindings(
    world_identity: &WorldIdentityManifestV1,
    runtime_profile: &RuntimeDeterminismProfileV1,
) -> RpgRuntimeBindingsV1 {
    let project_bytes = world_identity
        .canonical_bytes()
        .expect("validated world identity is canonical");
    let budget_bytes = runtime_profile
        .canonical_bytes()
        .expect("validated runtime profile is canonical");
    RpgRuntimeBindingsV1 {
        project_composition_lock_hash: domain_hash(
            b"nextengine.bootstrap-project-composition.v1\0",
            &project_bytes,
        ),
        schema_registry_hash: runtime_profile.command_kind_registry_hash,
        budget_policy_hash: domain_hash(b"nextengine.bootstrap-rpg-budget.v1\0", &budget_bytes),
        active_definition_policy_hashes: {
            let mut hashes = vec![
                core_rpg_policy_hash(),
                bootstrap_equipment_slot_policy_hash_v1(),
            ];
            hashes.sort_unstable();
            hashes
        },
    }
}

#[must_use]
pub fn bootstrap_equipment_slot_policy_hash_v1() -> ContentHash {
    domain_hash(
        b"nextengine.bootstrap-equipment-slot-policy.v1\0",
        CORE_EQUIPMENT_MAIN_HAND_SLOT_ID.as_bytes(),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeBootstrapV4 {
    pub world_identity: WorldIdentityManifestV1,
    pub principal_registry: PrincipalRegistryV1,
    pub stream_registry: CommandStreamRegistryV1,
    pub runtime_profile: RuntimeDeterminismProfileV1,
    pub admission_limits: RuntimeAdmissionLimitsV1,
    pub tick_rate_profile: TickRateProfileV1,
    pub ingress_assignment_profile: IngressAssignmentProfileV1,
    pub authoritative_numeric_profile: AuthoritativeNumericProfileV1,
    pub physics_quantization_profile: PhysicsQuantizationProfileV1,
    pub player_controller_registry: PlayerControllerRegistryV1,
    pub physics_checkpoint: PhysicsWorldCheckpointV1,
    pub rpg_bindings: RpgRuntimeBindingsV1,
    pub rpg_definitions: RpgDefinitionRegistryV2,
}

impl RuntimeBootstrapV4 {
    pub fn new(
        world_identity: WorldIdentityManifestV1,
        principal_registry: PrincipalRegistryV1,
        stream_registry: CommandStreamRegistryV1,
        runtime_profile: RuntimeDeterminismProfileV1,
    ) -> Self {
        let rpg_bindings = bootstrap_rpg_bindings(&world_identity, &runtime_profile);
        let rpg_definitions =
            RpgDefinitionRegistryV2::empty().expect("empty RPG definition registry is canonical");
        let admission_limits = RuntimeAdmissionLimitsV1::default();
        let tick_rate_profile = TickRateProfileV1::at_30_hz();
        let ingress_assignment_profile = IngressAssignmentProfileV1::core_v1(&admission_limits)
            .expect("built-in ingress profile is canonical");
        let physics_quantization_profile = PhysicsQuantizationProfileV1::capsule_reference_v1()
            .expect("built-in quantization profile identifiers are valid");
        let authoritative_numeric_profile =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&physics_quantization_profile)
                .expect("built-in numeric profile is canonical");
        let player_controller_registry = PlayerControllerRegistryV1 {
            schema_version: 1,
            world_namespace: world_identity.world_namespace,
            bindings: BTreeMap::new(),
        };
        let physics_checkpoint = empty_physics_checkpoint(
            PhysicsWorldId::from_bytes(*world_identity.world_namespace.as_bytes()),
            &tick_rate_profile,
            &authoritative_numeric_profile,
            &physics_quantization_profile,
        )
        .expect("built-in empty physical snapshot is canonical");
        Self {
            world_identity,
            principal_registry,
            stream_registry,
            runtime_profile,
            admission_limits,
            tick_rate_profile,
            ingress_assignment_profile,
            authoritative_numeric_profile,
            physics_quantization_profile,
            player_controller_registry,
            physics_checkpoint,
            rpg_bindings,
            rpg_definitions,
        }
    }

    #[must_use]
    pub fn with_player_input_and_physics(
        mut self,
        player_controller_registry: PlayerControllerRegistryV1,
        physics_checkpoint: PhysicsWorldCheckpointV1,
    ) -> Self {
        self.player_controller_registry = player_controller_registry;
        self.physics_checkpoint = physics_checkpoint;
        self
    }

    #[must_use]
    pub fn with_rpg_bindings(mut self, rpg_bindings: RpgRuntimeBindingsV1) -> Self {
        self.rpg_bindings = rpg_bindings;
        self
    }

    pub fn neutral_empty() -> Result<Self, SnapshotRestoreError> {
        let profile = RuntimeDeterminismBundleV1::core_r4d()?.runtime_profile();
        let world_identity = WorldIdentityManifestV1::new(
            ProjectId::new("nextengine.runtime-empty")
                .expect("built-in neutral project identifier is valid"),
            [0; 32],
            [0; 32],
            profile.profile_hash()?,
        )?;
        Ok(Self::new(
            world_identity.clone(),
            PrincipalRegistryV1::empty(world_identity.world_namespace),
            CommandStreamRegistryV1::empty(world_identity.world_namespace),
            profile,
        ))
    }
}

pub(super) fn validate_bootstrap(
    bootstrap: &RuntimeBootstrapV4,
    authority: &AuthorityRegistry,
    registry: &CommandKindRegistry,
) -> Result<(), SnapshotRestoreError> {
    bootstrap.world_identity.validate()?;
    bootstrap.principal_registry.validate()?;
    bootstrap.stream_registry.validate()?;
    bootstrap.runtime_profile.validate()?;
    bootstrap.admission_limits.validate()?;
    bootstrap.tick_rate_profile.validate()?;
    bootstrap.ingress_assignment_profile.validate()?;
    bootstrap.authoritative_numeric_profile.validate()?;
    bootstrap.physics_quantization_profile.validate()?;
    bootstrap.player_controller_registry.validate()?;
    bootstrap.rpg_bindings.validate()?;
    bootstrap.rpg_definitions.validate()?;
    bootstrap.physics_checkpoint.validate()?;
    let determinism = RuntimeDeterminismBundleV1::core_r4d()?;
    bootstrap
        .physics_checkpoint
        .snapshot
        .validate_profile_closure(
            &bootstrap.physics_checkpoint.catalog,
            &bootstrap.tick_rate_profile,
            &bootstrap.authoritative_numeric_profile,
            &bootstrap.physics_quantization_profile,
        )?;
    let world = bootstrap.world_identity.world_namespace;
    let profile_hash = bootstrap.runtime_profile.profile_hash()?;
    if bootstrap.world_identity.runtime_determinism_profile_hash != profile_hash
        || bootstrap.runtime_profile != determinism.runtime_profile()
        || registry != determinism.command_kind_registry()
        || bootstrap.principal_registry.world_namespace != world
        || bootstrap.stream_registry.world_namespace != world
        || bootstrap.player_controller_registry.world_namespace != world
        || bootstrap.runtime_profile.command_kind_registry_hash
            != command_kind_registry_hash(registry)
        || bootstrap.runtime_profile.schedule_manifest_hash != determinism.schedule_manifest_hash()
        || bootstrap.runtime_profile.admission_limits_profile_hash
            != bootstrap.admission_limits.profile_hash()?
        || bootstrap.runtime_profile.tick_rate_profile_hash
            != bootstrap.tick_rate_profile.profile_hash()?
        || bootstrap.runtime_profile.ingress_assignment_profile_hash
            != bootstrap.ingress_assignment_profile.profile_hash()?
        || bootstrap.runtime_profile.numeric_profile_hash
            != bootstrap.authoritative_numeric_profile.profile_hash()?
        || bootstrap.runtime_profile.physics_quantization_profile_hash
            != bootstrap.physics_quantization_profile.profile_hash()?
        || bootstrap.ingress_assignment_profile.admission_limits_hash
            != bootstrap.admission_limits.profile_hash()?
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    if bootstrap
        .rpg_definitions
        .interactions
        .iter()
        .any(|definition| {
            bootstrap
                .rpg_bindings
                .active_definition_policy_hashes
                .binary_search(&interaction_definition_hash_v2(definition))
                .is_err()
        })
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    validate_world_routine_bootstrap_closure(bootstrap, authority)?;
    validate_world_population_bootstrap_closure(bootstrap, authority)?;
    validate_world_activity_bootstrap_closure(bootstrap, authority)?;
    validate_agent_cognition_bootstrap_closure(bootstrap, authority)?;
    for (principal, _) in authority.entries() {
        if !bootstrap.principal_registry.is_active(principal) {
            return Err(SnapshotRestoreError::InactivePrincipal);
        }
    }
    for key in bootstrap.stream_registry.entries.keys() {
        if !bootstrap.principal_registry.is_active(&key.principal)
            || !authority.is_authenticated(&key.principal)
        {
            return Err(SnapshotRestoreError::InactivePrincipal);
        }
    }
    for binding in bootstrap.player_controller_registry.bindings.values() {
        if !bootstrap.principal_registry.is_active(&binding.principal)
            || !authority.is_authenticated(&binding.principal)
            || bootstrap
                .stream_registry
                .binding(binding.command_stream_id)
                .is_none_or(|(key, _)| key.principal != binding.principal)
            || bootstrap
                .physics_checkpoint
                .catalog
                .avatar_bindings
                .get(&binding.controlled_body_id)
                .is_none_or(|body_id| {
                    !bootstrap
                        .physics_checkpoint
                        .snapshot
                        .sorted_body_states
                        .contains_key(body_id)
                })
        {
            return Err(SnapshotRestoreError::ControllerClosureMismatch);
        }
    }
    Ok(())
}

fn validate_world_routine_bootstrap_closure(
    bootstrap: &RuntimeBootstrapV4,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let conditioned = bootstrap
        .rpg_definitions
        .interactions
        .iter()
        .filter(|definition| definition.availability_condition_or_none.is_some())
        .count();
    if conditioned > 1 {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    let principal = IssuerPrincipal::InternalSystem(
        SystemId::new(WORLD_ROUTINE_SYSTEM_ID).expect("engine-owned routine system id is valid"),
    );
    let principal_record = bootstrap.principal_registry.principals.get(&principal);
    let stream_entries = bootstrap
        .stream_registry
        .entries
        .iter()
        .filter(|(key, _)| key.principal == principal)
        .collect::<Vec<_>>();
    if conditioned == 0 {
        if principal_record.is_some()
            || !stream_entries.is_empty()
            || authority.is_authenticated(&principal)
        {
            return Err(SnapshotRestoreError::BootstrapClosureMismatch);
        }
        return Ok(());
    }
    let expected_provenance =
        content_hash_from_bytes(sha256(b"nextengine.principal.world-routine-boundary.v1\0"));
    let expected_capability = CapabilityId::new(WORLD_ROUTINE_CAPABILITY_ID)
        .expect("engine-owned routine capability id is valid");
    let expected_subject = SchemaId::new(WORLD_ROUTINE_CAPABILITY_SUBJECT_ID)
        .expect("engine-owned routine capability subject id is valid");
    let Some(record) = principal_record else {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    };
    if record.status != next_contracts::identity::PrincipalStatus::Active
        || record.provenance_hash != expected_provenance
        || record.capability_subject_id != expected_subject
        || authority.grants(&principal) != Some(&BTreeSet::from([expected_capability]))
        || stream_entries.len() != 1
        || stream_entries[0].0.stream_slot != 0
        || stream_entries[0].0.stream_epoch != 0
        || bootstrap.stream_registry.next_stream_slot.get(&principal) != Some(&1)
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    Ok(())
}

fn validate_world_population_bootstrap_closure(
    bootstrap: &RuntimeBootstrapV4,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let principal = IssuerPrincipal::InternalSystem(
        SystemId::new(WORLD_POPULATION_SYSTEM_ID)
            .expect("engine-owned population system id is valid"),
    );
    let principal_record = bootstrap.principal_registry.principals.get(&principal);
    let stream_entries = bootstrap
        .stream_registry
        .entries
        .iter()
        .filter(|(key, _)| key.principal == principal)
        .collect::<Vec<_>>();
    if principal_record.is_none() {
        if !stream_entries.is_empty() || authority.is_authenticated(&principal) {
            return Err(SnapshotRestoreError::BootstrapClosureMismatch);
        }
        return Ok(());
    }
    let expected_provenance = content_hash_from_bytes(sha256(
        b"nextengine.principal.world-population-boundary.v1\0",
    ));
    let expected_capability = CapabilityId::new(WORLD_POPULATION_CAPABILITY_ID)
        .expect("engine-owned population capability id is valid");
    let expected_subject = SchemaId::new(WORLD_POPULATION_CAPABILITY_SUBJECT_ID)
        .expect("engine-owned population capability subject id is valid");
    let record = principal_record.expect("presence was checked");
    if record.status != next_contracts::identity::PrincipalStatus::Active
        || record.provenance_hash != expected_provenance
        || record.capability_subject_id != expected_subject
        || authority.grants(&principal) != Some(&BTreeSet::from([expected_capability]))
        || stream_entries.len() != 1
        || stream_entries[0].0.stream_slot != 0
        || stream_entries[0].0.stream_epoch != 0
        || bootstrap.stream_registry.next_stream_slot.get(&principal) != Some(&1)
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    Ok(())
}

fn validate_agent_cognition_bootstrap_closure(
    bootstrap: &RuntimeBootstrapV4,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let principal = IssuerPrincipal::InternalSystem(
        SystemId::new(AGENT_COGNITION_SYSTEM_ID)
            .expect("engine-owned cognition system id is valid"),
    );
    let principal_record = bootstrap.principal_registry.principals.get(&principal);
    let stream_entries = bootstrap
        .stream_registry
        .entries
        .iter()
        .filter(|(key, _)| key.principal == principal)
        .collect::<Vec<_>>();
    if principal_record.is_none() {
        if !stream_entries.is_empty() || authority.is_authenticated(&principal) {
            return Err(SnapshotRestoreError::BootstrapClosureMismatch);
        }
        return Ok(());
    }
    let expected_provenance = content_hash_from_bytes(sha256(
        b"nextengine.principal.agent-cognition-boundary.v2\0",
    ));
    let cognition_capability = CapabilityId::new(AGENT_COGNITION_CAPABILITY_ID)
        .expect("engine-owned cognition capability id is valid");
    let rpg_capability = CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
        .expect("engine-owned RPG capability id is valid");
    let expected_subject = SchemaId::new(AGENT_COGNITION_CAPABILITY_SUBJECT_ID)
        .expect("engine-owned cognition capability subject id is valid");
    let record = principal_record.expect("presence was checked");
    let grants = authority
        .grants(&principal)
        .ok_or(SnapshotRestoreError::BootstrapClosureMismatch)?;
    let systemic = grants == &BTreeSet::from([cognition_capability.clone(), rpg_capability]);
    let cognition_only = grants == &BTreeSet::from([cognition_capability]);
    let expected_streams = if systemic { 2_usize } else { 1_usize };
    let expected_next_slot = if systemic { 2_u32 } else { 1_u32 };
    if record.status != next_contracts::identity::PrincipalStatus::Active
        || record.provenance_hash != expected_provenance
        || record.capability_subject_id != expected_subject
        || !systemic && !cognition_only
        || stream_entries.len() != expected_streams
        || stream_entries[0].0.stream_slot != 0
        || stream_entries[0].0.stream_epoch != 0
        || systemic
            && (stream_entries[1].0.stream_slot != 1 || stream_entries[1].0.stream_epoch != 0)
        || bootstrap.stream_registry.next_stream_slot.get(&principal) != Some(&expected_next_slot)
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    Ok(())
}

fn validate_world_activity_bootstrap_closure(
    bootstrap: &RuntimeBootstrapV4,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let principal = IssuerPrincipal::InternalSystem(
        SystemId::new(WORLD_ACTIVITY_SYSTEM_ID).expect("engine-owned activity system id is valid"),
    );
    let principal_record = bootstrap.principal_registry.principals.get(&principal);
    let stream_entries = bootstrap
        .stream_registry
        .entries
        .iter()
        .filter(|(key, _)| key.principal == principal)
        .collect::<Vec<_>>();
    if principal_record.is_none() {
        if !stream_entries.is_empty() || authority.is_authenticated(&principal) {
            return Err(SnapshotRestoreError::BootstrapClosureMismatch);
        }
        return Ok(());
    }
    let expected_provenance =
        content_hash_from_bytes(sha256(b"nextengine.principal.world-activity-boundary.v1\0"));
    let expected_capability = CapabilityId::new(WORLD_ACTIVITY_CAPABILITY_ID)
        .expect("engine-owned activity capability id is valid");
    let expected_subject = SchemaId::new(WORLD_ACTIVITY_CAPABILITY_SUBJECT_ID)
        .expect("engine-owned activity capability subject id is valid");
    let record = principal_record.expect("presence was checked");
    if record.status != next_contracts::identity::PrincipalStatus::Active
        || record.provenance_hash != expected_provenance
        || record.capability_subject_id != expected_subject
        || authority.grants(&principal) != Some(&BTreeSet::from([expected_capability]))
        || stream_entries.len() != 1
        || stream_entries[0].0.stream_slot != 0
        || stream_entries[0].0.stream_epoch != 0
        || bootstrap.stream_registry.next_stream_slot.get(&principal) != Some(&1)
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
    Ok(())
}

pub(super) fn validate_core_interaction_runtime_closure(
    bootstrap: &RuntimeBootstrapV4,
    rpg: &RpgState,
    authority: &AuthorityRegistry,
) -> Result<(), SnapshotRestoreError> {
    let binding = match bootstrap.player_controller_registry.bindings.len() {
        0 => None,
        1 => {
            let controlled_character_id = bootstrap
                .player_controller_registry
                .bindings
                .values()
                .next()
                .expect("single controller exists")
                .controlled_body_id;
            resolve_dialogue_quest_binding_v2(
                rpg,
                &bootstrap.rpg_definitions,
                controlled_character_id,
            )?
        }
        _ => return Err(CoreDialogueQuestClosureError.into()),
    };
    if let Some(binding) = binding {
        if !bootstrap
            .physics_checkpoint
            .catalog
            .avatar_bindings
            .contains_key(&binding.player_id)
        {
            return Err(CoreDialogueQuestClosureError.into());
        }
        let mut npc_bodies = bootstrap
            .physics_checkpoint
            .catalog
            .bodies
            .values()
            .filter(|body| body.body_id.subject_id == binding.npc_id);
        if npc_bodies
            .next()
            .is_none_or(|body| body.motion_kind != PhysicsMotionKindV1::Static)
            || npc_bodies.next().is_some()
        {
            return Err(CoreDialogueQuestClosureError.into());
        }
    }
    if binding.is_some()
        && resolve_interaction_outcome_route(
            &bootstrap.principal_registry,
            &bootstrap.stream_registry,
            authority,
        )
        .is_none()
    {
        return Err(SnapshotRestoreError::CoreInteractionClosure(
            CoreDialogueQuestClosureError,
        ));
    }
    Ok(())
}

pub(super) fn register_bootstrap_identities(
    ledger: &mut CommandLedgerV2,
    principals: &PrincipalRegistryV1,
    streams: &CommandStreamRegistryV1,
) -> Result<(), SnapshotRestoreError> {
    for (principal, record) in &principals.principals {
        if let IssuerPrincipal::Player(id) = principal {
            let result = ledger.causal_identity_registry.compare_or_insert(
                CausalIdentityKey {
                    identity_kind: CausalIdentityKind::PlayerPrincipal,
                    identity_bytes: *id.as_bytes(),
                },
                record.provenance_hash,
            )?;
            if result == IdentityInsertResult::Collision {
                return Err(SnapshotRestoreError::IdentityCollision);
            }
        }
    }
    for (key, stream_id) in &streams.entries {
        let principal = key.principal.canonical_bytes()?;
        let mut provenance = Vec::new();
        provenance.extend_from_slice(streams.world_namespace.as_bytes());
        provenance.extend_from_slice(&(principal.len() as u64).to_le_bytes());
        provenance.extend_from_slice(&principal);
        provenance.extend_from_slice(&key.stream_slot.to_le_bytes());
        provenance.extend_from_slice(&key.stream_epoch.to_le_bytes());
        let result = ledger
            .causal_identity_registry
            .compare_or_insert_provenance(
                CausalIdentityKind::CommandStream,
                *stream_id.as_bytes(),
                &provenance,
            )?;
        if result == IdentityInsertResult::Collision {
            return Err(SnapshotRestoreError::IdentityCollision);
        }
    }
    Ok(())
}

pub(super) fn validate_rpg_snapshot(
    snapshot: RpgSnapshotV2,
) -> Result<RpgState, SnapshotRestoreError> {
    let canonical_bytes = snapshot.canonical_bytes()?;
    let snapshot =
        RpgSnapshotV2::from_canonical_bytes(&canonical_bytes, CanonicalDecodeLimits::default())?;
    Ok(RpgState::from_snapshot(snapshot)?)
}
