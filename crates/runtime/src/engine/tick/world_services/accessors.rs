use super::*;

impl PreparedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.runtime.events()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.runtime.physics_snapshot()
    }

    #[must_use]
    pub fn physics_checkpoint(&self) -> &next_contracts::physics::PhysicsWorldCheckpointV1 {
        self.runtime.physics_checkpoint()
    }

    /// Immutable contact candidate at the same prepared publication boundary.
    #[must_use]
    pub fn contact_batch(&self) -> &next_contracts::physics::ClosedPhysicsContactBatchV1 {
        self.runtime.contact_batch()
    }

    #[must_use]
    pub fn rpg_snapshot(&self) -> next_contracts::rpg::RpgSnapshotV2 {
        self.runtime.rpg_snapshot()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.memory_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn decision_trace_or_none(&self) -> Option<&DecisionTraceV1> {
        self.decision_trace_or_none.as_ref()
    }

    #[must_use]
    pub const fn tier_cognition_report_or_none(
        &self,
    ) -> Option<&next_agent::TierCognitionServiceReportV1> {
        self.tier_cognition_report_or_none.as_ref()
    }

    #[must_use]
    pub const fn population_service_report_or_none(
        &self,
    ) -> Option<&next_world::PopulationNavigationServiceReportV1> {
        self.population.service_report_or_none()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.runtime.world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.generation.runtime.events()
    }

    #[must_use]
    pub fn report(&self) -> &TickReport {
        self.generation.runtime.report()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.generation.runtime.physics_snapshot()
    }

    #[must_use]
    pub fn physics_checkpoint(&self) -> &next_contracts::physics::PhysicsWorldCheckpointV1 {
        self.generation.runtime.physics_checkpoint()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.generation.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.generation.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.generation.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.generation.memory_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn decision_trace_or_none(&self) -> Option<&DecisionTraceV1> {
        self.generation.decision_trace_or_none.as_ref()
    }

    #[must_use]
    pub const fn tier_cognition_report_or_none(
        &self,
    ) -> Option<&next_agent::TierCognitionServiceReportV1> {
        self.generation.tier_cognition_report_or_none.as_ref()
    }

    #[must_use]
    pub const fn population_service_report_or_none(
        &self,
    ) -> Option<&next_world::PopulationNavigationServiceReportV1> {
        self.generation.population.service_report_or_none()
    }

    #[must_use]
    pub const fn application_state_root(&self) -> StateRoot {
        self.application_state_root
    }

    #[must_use]
    pub fn application_owner_segments(&self) -> &[SaveSegmentDescriptor] {
        &self.application_owner_segments
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.generation.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.generation.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.generation.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.generation.memory_snapshot_or_none.as_ref()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}
