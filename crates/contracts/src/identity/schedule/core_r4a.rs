use std::collections::BTreeMap;

use super::*;

impl ScheduleManifestV1 {
    pub fn core_r4a() -> Result<Self, IdentityContractError> {
        let system_id = SystemId::new(WORLD_ROUTINE_SYSTEM_ID)?;
        let shard_plan_id = SchemaId::new(WORLD_ROUTINE_SHARD_PLAN_ID)?;
        let access = |owner: &str,
                      schema: &str,
                      field_id: u32|
         -> Result<AccessKeyV1, IdentityContractError> {
            Ok(AccessKeyV1 {
                owner_id: SchemaId::new(owner)?,
                schema_id: SchemaId::new(schema)?,
                field_id,
            })
        };
        let mut reads = vec![
            access("nextengine.runtime", "nextengine.runtime-snapshot", 2)?,
            access(
                WORLD_ROUTINE_CATALOG_OWNER_ID,
                WORLD_ROUTINE_CATALOG_SCHEMA_ID,
                3,
            )?,
            access(
                WORLD_ROUTINE_CATALOG_OWNER_ID,
                WORLD_ROUTINE_CATALOG_SCHEMA_ID,
                4,
            )?,
            access(
                WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
                WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
                2,
            )?,
        ];
        reads.sort();
        let writes = vec![access(
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            "nextengine.world-routine-proposal",
            1,
        )?];
        let descriptor = SystemDescriptorV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            system_id: system_id.clone(),
            owner_id: SchemaId::new("nextengine.world-services")?,
            stage_id: RuntimeStageId::WorldStreamingCommit,
            before: Vec::new(),
            after: Vec::new(),
            access: AccessSetV1 { reads, writes },
            query_order: QueryOrderV1::PersistentId,
            shard_plan_id: shard_plan_id.clone(),
            reducer_ids: Vec::new(),
        };
        let shard_plan = LogicalShardPlanV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            shard_plan_id: shard_plan_id.clone(),
            system_id: system_id.clone(),
            logical_shard_count: 1,
            partition_rule: ShardPartitionRuleV1::Sha256StableKeyFirstU64LeModulo,
            record_order: ShardRecordOrderV1::CanonicalStableRecordKey,
            merge_order: DeltaMergeOrderV1::OwnerSchemaRecordFieldSystemShard,
        };
        let value = Self {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            stage_order: vec![
                RuntimeStageId::InputIngest,
                RuntimeStageId::CandidateAuthentication,
                RuntimeStageId::IngressValidationAndPlan,
                RuntimeStageId::IngressAdmission,
                RuntimeStageId::IngressCommit,
                RuntimeStageId::WorldStreamingCommit,
                RuntimeStageId::AgentPlanning,
                RuntimeStageId::PhysicalStep,
                RuntimeStageId::OutcomeCommit,
                RuntimeStageId::ResidencyCommit,
                RuntimeStageId::StateHash,
                RuntimeStageId::SnapshotPublication,
            ],
            systems: BTreeMap::from([(system_id, descriptor)]),
            reducers: BTreeMap::new(),
            shard_plans: BTreeMap::from([(shard_plan_id, shard_plan)]),
            command_admission_barriers: vec![
                CommandAdmissionBarrierV1 {
                    schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
                    phase: CommandPhase::Ingress,
                    stage_index: RuntimeStageId::CandidateAuthentication as u8,
                    batch_ordinal: 0,
                    source: CommandBarrierSourceV1::AuthenticatedExternalAndQueuedInternal,
                },
                CommandAdmissionBarrierV1 {
                    schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
                    phase: CommandPhase::Outcome,
                    stage_index: RuntimeStageId::OutcomeCommit as u8,
                    batch_ordinal: 0,
                    source: CommandBarrierSourceV1::InternalSystemOnly,
                },
            ],
        };
        value.validate()?;
        Ok(value)
    }
}
