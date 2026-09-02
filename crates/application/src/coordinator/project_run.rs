use next_contracts::canonical::sha256;
use next_contracts::command::IssuerPrincipal;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
    RuntimeDeterminismBundleV1, WorldIdentityManifestV1,
};
use next_contracts::ids::{
    ApplicationSessionId, CapabilityId, ContentHash, SchemaId, SystemId, content_hash_from_bytes,
};
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
        self.run_project_headless_internal(ProjectHeadlessRunKind::SingleTick)
    }

    /// Runs the bounded public creator-scenario path. Every requested action is
    /// one ordinary Runtime + World Routine/Population tick. Activity and
    /// cognition owner snapshots remain unchanged when the generic project has
    /// no RPG aggregate; the existing one-tick contract remains unchanged.
    pub fn run_project_headless_scenario(
        &mut self,
        tick_actions: u32,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if !(1..=256).contains(&tick_actions) {
            return Err(ApplicationError::ProjectRuntimeTick);
        }
        self.run_project_headless_internal(ProjectHeadlessRunKind::Scenario { tick_actions })
    }

    fn run_project_headless_internal(
        &mut self,
        kind: ProjectHeadlessRunKind,
    ) -> Result<ApplicationRunOutcomeV1, ApplicationError> {
        if self.machine.state().state != ApplicationSessionStatusV1::Active
            || self.launch.presentation_target != PresentationTargetKindV1::None
        {
            return Err(ApplicationError::CloseStateInvalid);
        }
        if self.live_run.is_some() || self.prepared_run.is_some() {
            return Err(ApplicationError::LiveRunAlreadyActive);
        }
        let prepared = prepare_project_run(
            self.machine.state().session_id,
            self.activated_package(),
            kind,
        )?;
        self.publish_prepared_run(prepared)
    }
}

#[derive(Clone, Copy)]
enum ProjectHeadlessRunKind {
    SingleTick,
    Scenario { tick_actions: u32 },
}

fn prepare_project_run(
    session_id: ApplicationSessionId,
    package: next_project::ActivatedProjectPackage,
    kind: ProjectHeadlessRunKind,
) -> Result<PreparedRunV1, ApplicationError> {
    let next_project::ActivatedProjectPackage {
        project,
        content_generation,
    } = package;
    let lock = &project.project_lock;
    let determinism = RuntimeDeterminismBundleV1::core_r8c()
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

    let mut authority = AuthorityRegistry::new();
    if matches!(kind, ProjectHeadlessRunKind::Scenario { .. }) {
        register_world_service_route(
            &mut bootstrap,
            &mut authority,
            next_contracts::world_population::WORLD_POPULATION_SYSTEM_ID,
            next_contracts::world_population::WORLD_POPULATION_CAPABILITY_ID,
            next_contracts::world_population::WORLD_POPULATION_CAPABILITY_SUBJECT_ID,
            b"nextengine.principal.world-population-boundary.v1\0",
        )?;
        if project.world_routine_catalog_or_none.is_some() {
            register_world_service_route(
                &mut bootstrap,
                &mut authority,
                next_contracts::world_routine::WORLD_ROUTINE_SYSTEM_ID,
                next_contracts::world_routine::WORLD_ROUTINE_CAPABILITY_ID,
                next_contracts::world_routine::WORLD_ROUTINE_CAPABILITY_SUBJECT_ID,
                b"nextengine.principal.world-routine-boundary.v1\0",
            )?;
        }
    }

    let mut runtime = RuntimeState::with_rpg_snapshot(
        bootstrap,
        authority,
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
    let mut world_streamer =
        WorldStreamerV1::activate(project.clone(), content_generation, initial_chunk_id)
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let mut world_routine =
        WorldRoutineOwnerV1::activate(project.world_routine_catalog_or_none, runtime.next_tick())
            .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    let mut world_population = WorldPopulationOwnerV1::activate(
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

    let (events, rpg_events) = match kind {
        ProjectHeadlessRunKind::SingleTick => {
            let tick = runtime
                .run_tick(Vec::<next_contracts::command::WorldCommand>::new())
                .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
            (
                u64::try_from(tick.events.len())
                    .map_err(|_| ApplicationError::ProjectRuntimeTick)?,
                0,
            )
        }
        ProjectHeadlessRunKind::Scenario { tick_actions } => {
            let mut event_count = 0_u64;
            let mut rpg_event_count = 0_u64;
            for _ in 0..tick_actions {
                let prepared = runtime
                    .tick_preparation()
                    .prepare_with_world_services(
                        Vec::<next_contracts::command::WorldCommand>::new(),
                        &world_routine,
                        &world_population,
                        &world_streamer,
                    )
                    .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
                event_count = event_count
                    .checked_add(
                        u64::try_from(prepared.events().len())
                            .map_err(|_| ApplicationError::ProjectRuntimeTick)?,
                    )
                    .ok_or(ApplicationError::ProjectRuntimeTick)?;
                rpg_event_count = rpg_event_count
                    .checked_add(
                        u64::try_from(
                            prepared
                                .events()
                                .iter()
                                .filter(|event| {
                                    matches!(
                                        event.payload,
                                        next_contracts::command::EventPayload::Rpg(_)
                                    )
                                })
                                .count(),
                        )
                        .map_err(|_| ApplicationError::ProjectRuntimeTick)?,
                    )
                    .ok_or(ApplicationError::ProjectRuntimeTick)?;
                let validated = runtime
                    .validate_prepared_world_services_tick_without_application_evidence(
                        &world_routine,
                        &world_population,
                        &world_streamer,
                        prepared,
                    )
                    .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
                runtime
                    .commit_validated_world_services_tick_without_application_evidence(
                        &mut world_routine,
                        &mut world_population,
                        &mut world_streamer,
                        validated,
                    )
                    .map_err(|_| ApplicationError::ProjectRuntimeTick)?;
            }
            (event_count, rpg_event_count)
        }
    };
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
    let summary = ApplicationRunOutcomeV1 {
        session_id,
        project_composition_lock_hash: lock.project_lock_sha256,
        ticks: runtime.next_tick(),
        events,
        rpg_events,
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

fn register_world_service_route(
    bootstrap: &mut RuntimeBootstrapV4,
    authority: &mut AuthorityRegistry,
    system_id: &str,
    capability_id: &str,
    capability_subject_id: &str,
    provenance_domain: &[u8],
) -> Result<(), ApplicationError> {
    let principal = IssuerPrincipal::InternalSystem(
        SystemId::new(system_id).map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?,
    );
    bootstrap
        .principal_registry
        .register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash: content_hash_from_bytes(sha256(provenance_domain)),
                capability_subject_id: SchemaId::new(capability_subject_id)
                    .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?,
                status: PrincipalStatus::Active,
            },
        )
        .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    bootstrap
        .stream_registry
        .allocate_stream(principal.clone())
        .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?;
    authority
        .register(
            principal,
            [CapabilityId::new(capability_id)
                .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)?],
        )
        .map_err(|_| ApplicationError::ProjectRuntimeBootstrap)
}
