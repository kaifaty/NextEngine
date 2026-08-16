use std::collections::BTreeMap;

use super::*;

impl ScheduleManifestV1 {
    pub fn core_r4b() -> Result<Self, IdentityContractError> {
        let routine_system_id = SystemId::new(WORLD_ROUTINE_SYSTEM_ID)?;
        let routine_shard_plan_id = SchemaId::new(WORLD_ROUTINE_SHARD_PLAN_ID)?;
        let population_system_id = SystemId::new(WORLD_POPULATION_SYSTEM_ID)?;
        let population_shard_plan_id = SchemaId::new(WORLD_POPULATION_SHARD_PLAN_ID)?;
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
        let mut routine_reads = vec![
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
        routine_reads.sort();
        let routine_writes = vec![access(
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            "nextengine.world-routine-proposal",
            1,
        )?];
        let routine_descriptor = SystemDescriptorV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            system_id: routine_system_id.clone(),
            owner_id: SchemaId::new("nextengine.world-services")?,
            stage_id: RuntimeStageId::WorldStreamingCommit,
            before: vec![population_system_id.clone()],
            after: Vec::new(),
            access: AccessSetV1 {
                reads: routine_reads,
                writes: routine_writes,
            },
            query_order: QueryOrderV1::PersistentId,
            shard_plan_id: routine_shard_plan_id.clone(),
            reducer_ids: Vec::new(),
        };
        let routine_shard_plan = LogicalShardPlanV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            shard_plan_id: routine_shard_plan_id.clone(),
            system_id: routine_system_id.clone(),
            logical_shard_count: 1,
            partition_rule: ShardPartitionRuleV1::Sha256StableKeyFirstU64LeModulo,
            record_order: ShardRecordOrderV1::CanonicalStableRecordKey,
            merge_order: DeltaMergeOrderV1::OwnerSchemaRecordFieldSystemShard,
        };
        let mut population_reads = vec![
            access("nextengine.runtime", "nextengine.runtime-snapshot", 2)?,
            access(
                WORLD_POPULATION_CATALOG_OWNER_ID,
                WORLD_POPULATION_CATALOG_SCHEMA_ID,
                7,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                4,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                5,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                6,
            )?,
            access(
                WORLD_POPULATION_SNAPSHOT_OWNER_ID,
                WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
                6,
            )?,
        ];
        population_reads.sort();
        let population_writes = vec![access(
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            "nextengine.world-population-proposal",
            1,
        )?];
        let population_descriptor = SystemDescriptorV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            system_id: population_system_id.clone(),
            owner_id: SchemaId::new("nextengine.world-services")?,
            stage_id: RuntimeStageId::WorldStreamingCommit,
            before: Vec::new(),
            after: vec![routine_system_id.clone()],
            access: AccessSetV1 {
                reads: population_reads,
                writes: population_writes,
            },
            query_order: QueryOrderV1::PersistentId,
            shard_plan_id: population_shard_plan_id.clone(),
            reducer_ids: Vec::new(),
        };
        let population_shard_plan = LogicalShardPlanV1 {
            schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
            shard_plan_id: population_shard_plan_id.clone(),
            system_id: population_system_id.clone(),
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
            systems: BTreeMap::from([
                (routine_system_id, routine_descriptor),
                (population_system_id, population_descriptor),
            ]),
            reducers: BTreeMap::new(),
            shard_plans: BTreeMap::from([
                (routine_shard_plan_id, routine_shard_plan),
                (population_shard_plan_id, population_shard_plan),
            ]),
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

    pub fn core_r4c() -> Result<Self, IdentityContractError> {
        let mut value = Self::core_r4b()?;
        let system_id = SystemId::new(AGENT_COGNITION_SYSTEM_ID)?;
        let shard_plan_id = SchemaId::new(AGENT_COGNITION_SHARD_PLAN_ID)?;
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
                AGENT_COGNITION_CATALOG_OWNER_ID,
                AGENT_COGNITION_CATALOG_SCHEMA_ID,
                4,
            )?,
            access(
                AGENT_MEMORY_SNAPSHOT_OWNER_ID,
                AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
                3,
            )?,
            access(
                AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
                AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
                3,
            )?,
            access("rpg", "nextengine.rpg.snapshot", 2)?,
            access(
                WORLD_POPULATION_SNAPSHOT_OWNER_ID,
                WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
                6,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                4,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                5,
            )?,
            access(
                WORLD_NAVIGATION_CATALOG_OWNER_ID,
                WORLD_NAVIGATION_CATALOG_SCHEMA_ID,
                6,
            )?,
        ];
        reads.sort();
        let mut writes = vec![
            access(
                AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
                "nextengine.agent-cognition-proposal",
                1,
            )?,
            access(
                AGENT_MEMORY_SNAPSHOT_OWNER_ID,
                "nextengine.agent-memory-proposal",
                1,
            )?,
        ];
        writes.sort();
        value.systems.insert(
            system_id.clone(),
            SystemDescriptorV1 {
                schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
                system_id: system_id.clone(),
                owner_id: SchemaId::new(AGENT_RUNTIME_SNAPSHOT_OWNER_ID)?,
                stage_id: RuntimeStageId::AgentPlanning,
                before: Vec::new(),
                after: Vec::new(),
                access: AccessSetV1 { reads, writes },
                query_order: QueryOrderV1::PersistentId,
                shard_plan_id: shard_plan_id.clone(),
                reducer_ids: Vec::new(),
            },
        );
        value.shard_plans.insert(
            shard_plan_id.clone(),
            LogicalShardPlanV1 {
                schema_version: SCHEDULE_MANIFEST_SCHEMA_VERSION,
                shard_plan_id,
                system_id,
                logical_shard_count: 1,
                partition_rule: ShardPartitionRuleV1::Sha256StableKeyFirstU64LeModulo,
                record_order: ShardRecordOrderV1::CanonicalStableRecordKey,
                merge_order: DeltaMergeOrderV1::OwnerSchemaRecordFieldSystemShard,
            },
        );
        value.validate()?;
        Ok(value)
    }
}
