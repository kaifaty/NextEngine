use next_contracts::cognition::{
    AGENT_MEMORY_SNAPSHOT_OWNER_ID, AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
    AGENT_MEMORY_SNAPSHOT_SEGMENT_ID, AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
    AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID, AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID, COGNITION_SCHEMA_VERSION,
};
use next_contracts::persistence::SaveSegmentDescriptor;
use next_contracts::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
    PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID, PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_SCHEMA_VERSION, WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
    WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::world_population::{
    WORLD_POPULATION_SCHEMA_VERSION, WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
    WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
    WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
};

use super::super::error::SaveStoreError;

pub(super) fn physical_animation_segment_index(
    descriptors: &[SaveSegmentDescriptor],
) -> Result<Option<usize>, SaveStoreError> {
    let mut index = None;
    for (candidate, descriptor) in descriptors.iter().enumerate().filter(|(_, descriptor)| {
        descriptor.owner_id.as_str() == PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID
    }) {
        if descriptor.schema_id.as_str() != PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID
            || descriptor.segment_id.as_str() != PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID
            || descriptor.schema_version != u32::from(PHYSICAL_ANIMATION_SCHEMA_VERSION)
        {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_PHYSICAL_ANIMATION_SCHEMA_UNSUPPORTED",
            ));
        }
        if index.replace(candidate).is_some() {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_PHYSICAL_ANIMATION_SEGMENT_DUPLICATE",
            ));
        }
    }
    Ok(index)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CognitionSegmentIndices {
    pub(super) agent: Option<usize>,
    pub(super) memory: Option<usize>,
}

pub(super) fn cognition_segment_indices(
    descriptors: &[SaveSegmentDescriptor],
) -> Result<CognitionSegmentIndices, SaveStoreError> {
    let mut agent = None;
    let mut memory = None;
    for (index, descriptor) in descriptors.iter().enumerate().filter(|(_, descriptor)| {
        matches!(
            descriptor.owner_id.as_str(),
            AGENT_RUNTIME_SNAPSHOT_OWNER_ID | AGENT_MEMORY_SNAPSHOT_OWNER_ID
        )
    }) {
        let target = if descriptor.owner_id.as_str() == AGENT_RUNTIME_SNAPSHOT_OWNER_ID
            && descriptor.schema_id.as_str() == AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == u32::from(COGNITION_SCHEMA_VERSION)
        {
            &mut agent
        } else if descriptor.owner_id.as_str() == AGENT_MEMORY_SNAPSHOT_OWNER_ID
            && descriptor.schema_id.as_str() == AGENT_MEMORY_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == AGENT_MEMORY_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == u32::from(COGNITION_SCHEMA_VERSION)
        {
            &mut memory
        } else {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_COGNITION_SCHEMA_UNSUPPORTED",
            ));
        };
        if target.replace(index).is_some() {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_COGNITION_SEGMENT_DUPLICATE",
            ));
        }
    }
    if agent.is_some() != memory.is_some() {
        return Err(SaveStoreError::InvalidImage(
            "SAVE_COGNITION_SEGMENT_INCOMPLETE",
        ));
    }
    Ok(CognitionSegmentIndices { agent, memory })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorldServicesSegmentIndices {
    pub(super) streaming: Option<usize>,
    pub(super) routine: Option<usize>,
    pub(super) population: Option<usize>,
    pub(super) activity: Option<usize>,
}

pub(super) fn world_services_segment_indices(
    descriptors: &[SaveSegmentDescriptor],
) -> Result<WorldServicesSegmentIndices, SaveStoreError> {
    let mut streaming = None;
    let mut routine = None;
    let mut population = None;
    let mut activity = None;
    for (index, descriptor) in descriptors
        .iter()
        .enumerate()
        .filter(|(_, descriptor)| descriptor.owner_id.as_str() == WORLD_STREAMING_SNAPSHOT_OWNER_ID)
    {
        let target = if descriptor.schema_id.as_str() == WORLD_STREAMING_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == WORLD_STREAMING_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION
        {
            &mut streaming
        } else if descriptor.schema_id.as_str() == WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == u32::from(WORLD_ROUTINE_SCHEMA_VERSION)
        {
            &mut routine
        } else if descriptor.schema_id.as_str() == WORLD_POPULATION_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == WORLD_POPULATION_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == u32::from(WORLD_POPULATION_SCHEMA_VERSION)
        {
            &mut population
        } else if descriptor.schema_id.as_str() == WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID
            && descriptor.segment_id.as_str() == WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID
            && descriptor.schema_version == u32::from(WORLD_ACTIVITY_SCHEMA_VERSION)
        {
            &mut activity
        } else {
            return Err(SaveStoreError::InvalidImage(
                "WORLD_SERVICES_SCHEMA_UNSUPPORTED",
            ));
        };
        if target.replace(index).is_some() {
            return Err(SaveStoreError::InvalidImage(
                "SAVE_WORLD_SERVICES_SEGMENT_DUPLICATE",
            ));
        }
    }
    if (routine.is_some() || population.is_some() || activity.is_some()) && streaming.is_none() {
        return Err(SaveStoreError::InvalidImage(
            "SAVE_WORLD_ROUTINE_STREAMING_MISSING",
        ));
    }
    Ok(WorldServicesSegmentIndices {
        streaming,
        routine,
        population,
        activity,
    })
}
