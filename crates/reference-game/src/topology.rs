use next_contracts::content::NeutralRecordKindV1;
use next_contracts::ids::SchemaId;
use next_contracts::project::ActivatedProjectV4;

use crate::ReferenceGameError;

const REFERENCE_ROLE_PROPERTY_ID: &str = "nextengine.reference.role";
const INITIAL_CHUNK_ROLE_ID: &str = "nextengine.reference-alpha.world-chunk.relay-station";
const GAMEPLAY_TARGET_CHUNK_ROLE_ID: &str = "nextengine.reference-alpha.world-chunk.frontier";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceChunkRouteEntryV1 {
    pub region_id: SchemaId,
    pub chunk_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceWorldTopologyV1 {
    initial_chunk_id: SchemaId,
    gameplay_target_chunk_id: SchemaId,
    ordered_route: Vec<ReferenceChunkRouteEntryV1>,
}

impl ReferenceWorldTopologyV1 {
    pub fn from_activated_project(
        project: &ActivatedProjectV4,
    ) -> Result<Self, ReferenceGameError> {
        if project.world_partition.body.chunk_bindings.is_empty() {
            return Err(ReferenceGameError::WorldPartitionEmpty);
        }

        let mut initial_chunks = Vec::new();
        let mut gameplay_target_chunks = Vec::new();
        let mut ordered_route =
            Vec::with_capacity(project.world_partition.body.chunk_bindings.len());
        for binding in &project.world_partition.body.chunk_bindings {
            let record = project
                .neutral_records
                .binary_search_by_key(&binding.chunk_asset.asset_id, |record| record.asset_id)
                .ok()
                .and_then(|index| project.neutral_records.get(index))
                .ok_or(ReferenceGameError::WorldChunkRecordMissing)?;
            if record.kind != NeutralRecordKindV1::WorldChunk {
                return Err(ReferenceGameError::WorldChunkRecordKindMismatch);
            }
            if let Some(role) = record
                .properties
                .iter()
                .find(|property| property.property_id.as_str() == REFERENCE_ROLE_PROPERTY_ID)
                .map(|property| property.value_id.as_str())
            {
                match role {
                    INITIAL_CHUNK_ROLE_ID => initial_chunks.push(binding.chunk_id.clone()),
                    GAMEPLAY_TARGET_CHUNK_ROLE_ID => {
                        gameplay_target_chunks.push(binding.chunk_id.clone());
                    }
                    _ => {}
                }
            }
            ordered_route.push(ReferenceChunkRouteEntryV1 {
                region_id: binding.region_id.clone(),
                chunk_id: binding.chunk_id.clone(),
            });
        }

        if initial_chunks.len() > 1 {
            return Err(ReferenceGameError::WorldChunkRoleDuplicate("initial"));
        }
        if gameplay_target_chunks.len() > 1 {
            return Err(ReferenceGameError::WorldChunkRoleDuplicate(
                "gameplay-target",
            ));
        }
        let initial_chunk_id = initial_chunks
            .pop()
            .ok_or(ReferenceGameError::WorldChunkRoleMissing("initial"))?;
        let gameplay_target_chunk_id = gameplay_target_chunks
            .pop()
            .ok_or(ReferenceGameError::WorldChunkRoleMissing("gameplay-target"))?;

        ordered_route.sort_by(|left, right| {
            (&left.region_id, &left.chunk_id).cmp(&(&right.region_id, &right.chunk_id))
        });
        let initial_index = ordered_route
            .iter()
            .position(|entry| entry.chunk_id == initial_chunk_id)
            .expect("initial role is resolved from a manifest binding");
        let initial_entry = ordered_route.remove(initial_index);
        ordered_route.insert(0, initial_entry);

        Ok(Self {
            initial_chunk_id,
            gameplay_target_chunk_id,
            ordered_route,
        })
    }

    #[must_use]
    pub fn initial_chunk_id(&self) -> &SchemaId {
        &self.initial_chunk_id
    }

    #[must_use]
    pub fn gameplay_target_chunk_id(&self) -> &SchemaId {
        &self.gameplay_target_chunk_id
    }

    #[must_use]
    pub fn ordered_multiregion_route(&self) -> &[ReferenceChunkRouteEntryV1] {
        &self.ordered_route
    }
}
