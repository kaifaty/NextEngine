use super::*;

impl PreparedRuntimeSnapshotFields {
    pub(super) fn snapshot(
        &self,
        next_tick: u64,
        staged: &StagedAuthoritativeState,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> RuntimeSnapshotV3 {
        RuntimeSnapshotV3 {
            next_tick,
            committed_event_count: staged.event_count,
            authoritative_revision: staged.revision,
            world_identity: self.world_identity.clone(),
            principal_registry: self.principal_registry.clone(),
            stream_registry: self.stream_registry.clone(),
            runtime_profile: self.runtime_profile,
            admission_limits: self.admission_limits,
            tick_rate_profile: self.tick_rate_profile,
            ingress_assignment_profile: self.ingress_assignment_profile,
            authoritative_numeric_profile: self.authoritative_numeric_profile.clone(),
            physics_quantization_profile: self.physics_quantization_profile.clone(),
            player_controller_registry: self.player_controller_registry.clone(),
            ingress_checkpoint: staged.ingress.clone(),
            rpg_runtime_bindings: self.rpg_runtime_bindings.clone(),
            command_ledger,
            body_archive,
        }
    }

    pub(super) fn into_snapshot(
        self,
        next_tick: u64,
        staged: &StagedAuthoritativeState,
        command_ledger: next_contracts::ledger::CommandLedgerV2,
        body_archive: next_contracts::ledger::CommandBodyArchiveV1,
    ) -> RuntimeSnapshotV3 {
        RuntimeSnapshotV3 {
            next_tick,
            committed_event_count: staged.event_count,
            authoritative_revision: staged.revision,
            world_identity: self.world_identity,
            principal_registry: self.principal_registry,
            stream_registry: self.stream_registry,
            runtime_profile: self.runtime_profile,
            admission_limits: self.admission_limits,
            tick_rate_profile: self.tick_rate_profile,
            ingress_assignment_profile: self.ingress_assignment_profile,
            authoritative_numeric_profile: self.authoritative_numeric_profile,
            physics_quantization_profile: self.physics_quantization_profile,
            player_controller_registry: self.player_controller_registry,
            ingress_checkpoint: staged.ingress.clone(),
            rpg_runtime_bindings: self.rpg_runtime_bindings,
            command_ledger,
            body_archive,
        }
    }
}
