use super::*;

pub(super) fn application_closure(
    components: &WorldCheckpointCanonicalComponentsV1,
    streaming: &WorldStreamingSnapshotV1,
    routine_or_none: Option<&WorldRoutineSnapshotV1>,
    population_or_none: Option<&WorldPopulationSnapshotV1>,
    agent_or_none: Option<&AgentCognitionSnapshotV1>,
    memory_or_none: Option<&AgentMemorySnapshotV1>,
) -> Result<(Vec<SaveSegmentDescriptor>, StateRoot), RuntimeFatalError> {
    let streaming_bytes = streaming.canonical_bytes()?;
    let mut descriptors = vec![
        segment_descriptor(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            components.runtime_snapshot_bytes(),
        )?,
        segment_descriptor(
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
            components.rpg_snapshot_bytes(),
        )?,
        segment_descriptor(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
            components.physics_checkpoint_bytes(),
        )?,
        segment_descriptor(
            WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION,
            &streaming_bytes,
        )?,
    ];
    if let Some(routine) = routine_or_none {
        let routine_bytes = routine.canonical_bytes()?;
        descriptors.push(segment_descriptor(
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_ROUTINE_SCHEMA_VERSION),
            &routine_bytes,
        )?);
    }
    if let Some(population) = population_or_none {
        let population_bytes = population
            .canonical_bytes()
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        descriptors.push(segment_descriptor(
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_POPULATION_SCHEMA_VERSION),
            &population_bytes,
        )?);
    }
    match (agent_or_none, memory_or_none) {
        (Some(agent), Some(memory)) => {
            let agent_bytes = agent
                .canonical_bytes()
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
            descriptors.push(segment_descriptor(
                AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
                AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
                AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
                u32::from(COGNITION_SCHEMA_VERSION),
                &agent_bytes,
            )?);
            let memory_bytes = memory
                .canonical_bytes()
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
            descriptors.push(segment_descriptor(
                AGENT_MEMORY_SNAPSHOT_OWNER_ID,
                AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
                AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
                u32::from(COGNITION_SCHEMA_VERSION),
                &memory_bytes,
            )?);
        }
        (None, None) => {}
        _ => return Err(RuntimeFatalError::AgentCognitionInternalInvariant),
    }
    let application_state_root = match (agent_or_none, memory_or_none) {
        (Some(agent), Some(memory)) => {
            world_checkpoint_with_cognition_v1_state_root_from_canonical_components(
                components,
                streaming,
                routine_or_none,
                population_or_none,
                agent,
                memory,
            )?
        }
        (None, None) => {
            world_checkpoint_with_world_services_v1_state_root_from_canonical_components(
                components,
                streaming,
                routine_or_none,
                population_or_none,
            )?
        }
        _ => return Err(RuntimeFatalError::AgentCognitionInternalInvariant),
    };
    descriptors.sort();
    if descriptors.windows(2).any(|pair| {
        (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
            == (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
    }) {
        return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
    }
    Ok((descriptors, application_state_root))
}

fn segment_descriptor(
    owner_id: &str,
    schema_id: &str,
    segment_id: &str,
    schema_version: u32,
    bytes: &[u8],
) -> Result<SaveSegmentDescriptor, RuntimeFatalError> {
    Ok(SaveSegmentDescriptor::for_bytes(
        SchemaId::new(owner_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        SchemaId::new(schema_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        SchemaId::new(segment_id).map_err(next_contracts::canonical::CanonicalError::from)?,
        schema_version,
        bytes,
    )?)
}
