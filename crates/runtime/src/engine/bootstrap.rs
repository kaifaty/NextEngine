use std::collections::BTreeMap;

use next_contracts::{
    AuthoritativeNumericProfileV1, CORE_EQUIPMENT_MAIN_HAND_SLOT_ID, CanonicalDecodeLimits,
    CausalIdentityKey, CausalIdentityKind, CommandLedgerV2, CommandStreamRegistryV1, ContentHash,
    CoreDialogueQuestClosureError, IdentityInsertResult, IngressAssignmentProfileV1,
    IssuerPrincipal, PhysicsMotionKindV1, PhysicsQuantizationProfileV1, PhysicsWorldCheckpointV1,
    PhysicsWorldId, PlayerControllerRegistryV1, PrincipalRegistryV1, ProjectId,
    RpgDefinitionRegistryV1, RpgRuntimeBindingsV1, RpgSnapshotV2, RuntimeAdmissionLimitsV1,
    RuntimeDeterminismProfileV1, TickRateProfileV1, WorldIdentityManifestV1,
    interaction_definition_hash,
};
use next_rpg::RpgState;

use crate::authority::AuthorityRegistry;
use crate::registry::CommandKindRegistry;

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
        schema_registry_hash: CommandKindRegistry::core_v1().canonical_hash(),
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
pub struct RuntimeBootstrapV3 {
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
    pub rpg_definitions: RpgDefinitionRegistryV1,
}

impl RuntimeBootstrapV3 {
    pub fn new(
        world_identity: WorldIdentityManifestV1,
        principal_registry: PrincipalRegistryV1,
        stream_registry: CommandStreamRegistryV1,
        runtime_profile: RuntimeDeterminismProfileV1,
    ) -> Self {
        let rpg_bindings = bootstrap_rpg_bindings(&world_identity, &runtime_profile);
        let rpg_definitions =
            RpgDefinitionRegistryV1::empty().expect("empty RPG definition registry is canonical");
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
        let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
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
    bootstrap: &RuntimeBootstrapV3,
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
        || bootstrap.principal_registry.world_namespace != world
        || bootstrap.stream_registry.world_namespace != world
        || bootstrap.player_controller_registry.world_namespace != world
        || bootstrap.runtime_profile.command_kind_registry_hash != registry.canonical_hash()
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
                .binary_search(&interaction_definition_hash(definition))
                .is_err()
        })
    {
        return Err(SnapshotRestoreError::BootstrapClosureMismatch);
    }
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

pub(super) fn validate_core_interaction_runtime_closure(
    bootstrap: &RuntimeBootstrapV3,
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
