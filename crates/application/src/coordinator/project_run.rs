use next_contracts::canonical::sha256;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRegistryV1, RuntimeDeterminismBundleV1,
    WorldIdentityManifestV1,
};
use next_contracts::ids::{ApplicationSessionId, ContentHash};
use next_contracts::mechanics::{ability_definition_hash, interaction_definition_hash_v2};
use next_contracts::rpg::RpgSnapshotV2;
use next_contracts::session::{ApplicationSessionStatusV1, PresentationTargetKindV1};
use next_runtime::{AuthorityRegistry, RuntimeBootstrapV4, RuntimeState};
use next_world::{
    WorldActivityOwnerV1, WorldPopulationOwnerV1, WorldRoutineOwnerV1, WorldStreamerV1,
};

use crate::ApplicationError;

use super::{ApplicationCoordinator, ApplicationRunOutcomeV1, PreparedRunV1};

/// Starts the current external project through the production application and
/// runtime boundaries, executes one deterministic headless tick, and stages
/// the complete owner state for the ordinary save-on-close path.
impl ApplicationCoordinator {
    pub fn run_project_headless(&mut self) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active
            || self.launch.presentation_target != PresentationTargetKindV1::None
        {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if self.live_run.is_some() || self.prepared_run.is_some() {
            return Err(ApplicationError::LiveRunAlreadyActive);
        }
        let prepared =
            prepare_project_run(self.machine.state().session_id, self.activated_package())?;
        self.publish_prepared_run(prepared)
    }
}

fn prepare_project_run(
    session_id: ApplicationSessionId,
    package: next_project::ActivatedProjectPackage,
) -> Result<PreparedRunV1, ApplicationError> {
    let next_project::ActivatedProjectPackage {
        project,
        content_generation,
    } = package;
    let lock = &project.project_lock;
    let determinism = RuntimeDeterminismBundleV1::core_r5c()
        .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let profile = determinism.runtime_profile();
    if determinism.runtime_profile_hash() != lock.runtime_determinism_profile_sha256 {
        return Err(ApplicationError::ProjectRuntimeBootstrap);
    }

    let nonce = project_seed(
        b"nextengine.project-runtime-instance.v1\0",
        lock.project_lock_sha256,
    );
    let rng = project_seed(
        b"nextengine.project-runtime-rng.v1\0",
        lock.project_lock_sha256,
    );
    let world_identity = WorldIdentityManifestV1::new(
        lock.project_id.clone(),
        nonce,
        rng,
        profile
            .profile_hash()
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?,
    )
    .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let mut bootstrap = RuntimeBootstrapV4::new(
        world_identity.clone(),
        PrincipalRegistryV1::empty(world_identity.world_namespace),
        CommandStreamRegistryV1::empty(world_identity.world_namespace),
        profile,
    );
    bootstrap.rpg_definitions = project.rpg_definitions.clone();
    bootstrap.rpg_bindings.project_composition_lock_hash = lock.project_lock_sha256;
    bootstrap.rpg_bindings.schema_registry_hash = lock.schema_registry_manifest_sha256;
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .extend(
            project
                .rpg_definitions
                .interactions
                .iter()
                .map(interaction_definition_hash_v2),
        );
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .extend(
            project
                .rpg_definitions
                .abilities
                .iter()
                .map(ability_definition_hash),
        );
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .sort_unstable();
    bootstrap
        .rpg_bindings
        .active_definition_policy_hashes
        .dedup();

    let mut runtime = RuntimeState::with_rpg_snapshot(
        bootstrap,
        AuthorityRegistry::new(),
        RpgSnapshotV2 {
            aggregates: Vec::new(),
        },
    )
    .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let initial_chunk_id = project
        .world_population_catalog
        .definition(project.world_population_catalog.courier_subject_id)
        .ok_or(ApplicationError::ProjectRuntimeBootstrap)?
        .initial_node_id
        .clone();
    let world_streamer =
        WorldStreamerV1::activate(project.clone(), content_generation, initial_chunk_id)
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let world_routine =
        WorldRoutineOwnerV1::activate(project.world_routine_catalog_or_none, runtime.next_tick())
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let world_population = WorldPopulationOwnerV1::activate(
        project.world_population_catalog.clone(),
        project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )
    .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let world_activity =
        WorldActivityOwnerV1::activate(project.world_activity_catalog.clone(), runtime.next_tick())
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let cognition_seed = u64::from_le_bytes(rng[..8].try_into().unwrap_or_else(|_| unreachable!()));
    let cognition = next_agent::cognition::StrategicAgentOwnersV1::initial(
        project.agent_cognition_catalog.clone(),
        cognition_seed,
    )
    .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;

    let tick = runtime
        .run_tick(Vec::<next_contracts::command::WorldCommand>::new())
        .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
    world_routine
        .validate(runtime.next_tick())
        .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
    world_population
        .validate(runtime.next_tick())
        .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
    world_activity
        .validate(runtime.next_tick())
        .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
    cognition
        .validate()
        .map_err(|_| ApplicationError::ProjectRuntimeTick)?;

    let (checkpoint, components) = runtime.world_checkpoint_with_canonical_components()?;
    let streaming = world_streamer.snapshot().clone();
    let routine = world_routine.snapshot_or_none().cloned();
    let population = world_population
        .snapshot_or_none()
        .ok_or(ApplicationError::ProjectRuntimeBootstrap)?
        .clone();
    let activity = world_activity.snapshot().clone();
    let agent = cognition.agent_snapshot().clone();
    let memory = cognition.memory_snapshot().clone();
    let authoritative_state_root = next_contracts::snapshot::world_checkpoint_with_systemic_cognition_v1_state_root_from_canonical_components(
        &components,
        &streaming,
        routine.as_ref(),
        &population,
        &activity,
        &agent,
        &memory,
    )?;
    let events =
        u64::try_from(tick.events.len()).map_err(|_| ApplicationError::ProjectRuntimeTick)?;
    let summary = ApplicationRunOutcomeV1 {
        session_id,
        project_composition_lock_hash: lock.project_lock_sha256,
        ticks: runtime.next_tick(),
        events,
        rpg_events: 0,
        authoritative_revision: runtime.authoritative_revision(),
        authoritative_state_root: ContentHash::from_bytes(*authoritative_state_root.as_bytes()),
        command_archive_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .body_archive
            .archive_root,
        command_identity_index_root: checkpoint
            .runtime_snapshot
            .command_ledger
            .identity_index
            .index_root,
        command_ledger_hash: components.command_ledger_hash()?,
        presentation_input_count: 0,
        presentation_snapshot: None,
    };
    Ok(PreparedRunV1 {
        checkpoint,
        streaming,
        routine,
        population,
        activity,
        agent,
        memory,
        physical_animation: None,
        summary,
    })
}

fn project_seed(domain: &[u8], project_lock: ContentHash) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(domain.len() + project_lock.as_bytes().len());
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(project_lock.as_bytes());
    sha256(&preimage)
}
