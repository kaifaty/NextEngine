use next_contracts::body::{
    BODY_PROJECTION_COMPILER_PROFILE_ID_V1, BodyInstanceProjectionV1, BodyProjectionRootsV1,
};
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::project::{ActivatedProjectV8, AssetRevisionRefV1};
use next_motor::{
    CompiledBodySchemaV1, body_projection_compiler_profile_hash_v1,
    neutral_body_instance_projection_v1,
};

use crate::ReferenceGameError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceBodyProjectionV1 {
    pub instance: BodyInstanceProjectionV1,
    pub roots: BodyProjectionRootsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceBodyProjectionSetV1 {
    pub body_schema_asset_revision: AssetRevisionRefV1,
    pub player: ReferenceBodyProjectionV1,
    pub npc: ReferenceBodyProjectionV1,
}

impl ReferenceBodyProjectionSetV1 {
    pub(crate) fn compile(
        project: &ActivatedProjectV8,
        player_subject_id: PersistentId,
        npc_subject_id: PersistentId,
    ) -> Result<Self, ReferenceGameError> {
        let asset = &project.body_schema_asset;
        asset.validate()?;
        if asset.compiler_profile_id.as_str() != BODY_PROJECTION_COMPILER_PROFILE_ID_V1
            || player_subject_id == PersistentId::default()
            || npc_subject_id == PersistentId::default()
            || player_subject_id == npc_subject_id
        {
            return Err(ReferenceGameError::BodyProjectionInvalid);
        }
        let asset_record_hash = asset.record_sha256()?;
        let matching_entries = project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .filter(|entry| {
                entry.asset_revision.asset_id == asset.asset_id
                    && entry.asset_revision.record_sha256 == asset_record_hash
            })
            .count();
        let matching_roots = project
            .content_manifest
            .body
            .root_assets
            .iter()
            .filter(|root| {
                root.asset_id == asset.asset_id && root.record_sha256 == asset_record_hash
            })
            .count();
        if matching_entries != 1 || matching_roots != 1 {
            return Err(ReferenceGameError::BodyProjectionInvalid);
        }
        let player_instance =
            neutral_body_instance_projection_v1(&asset.body_schema, player_subject_id)?;
        let npc_instance = neutral_body_instance_projection_v1(&asset.body_schema, npc_subject_id)?;
        let player_compiled =
            CompiledBodySchemaV1::compile_projection(&asset.body_schema, &player_instance)?;
        let npc_compiled =
            CompiledBodySchemaV1::compile_projection(&asset.body_schema, &npc_instance)?;
        let expected_compiler_profile = body_projection_compiler_profile_hash_v1();
        let shared_roots_match = player_compiled.body_schema_hash == npc_compiled.body_schema_hash
            && player_compiled.projection_roots.compiler_profile_hash == expected_compiler_profile
            && npc_compiled.projection_roots.compiler_profile_hash == expected_compiler_profile
            && player_compiled.projection_roots.observation_layout_hash
                == npc_compiled.projection_roots.observation_layout_hash
            && player_compiled.projection_roots.action_layout_hash
                == npc_compiled.projection_roots.action_layout_hash
            && player_compiled.projection_roots.actuator_safety_root
                == npc_compiled.projection_roots.actuator_safety_root;
        let instance_roots_are_distinct = player_compiled.body_instance_projection_hash
            != npc_compiled.body_instance_projection_hash
            && player_compiled.projection_roots.physics_descriptor_root
                != npc_compiled.projection_roots.physics_descriptor_root
            && player_compiled.projection_roots.projection_root()?
                != npc_compiled.projection_roots.projection_root()?;
        if !shared_roots_match || !instance_roots_are_distinct {
            return Err(ReferenceGameError::BodyProjectionInvalid);
        }
        Ok(Self {
            body_schema_asset_revision: AssetRevisionRefV1 {
                asset_id: asset.asset_id,
                record_sha256: asset_record_hash,
            },
            player: ReferenceBodyProjectionV1 {
                instance: player_instance,
                roots: player_compiled.projection_roots,
            },
            npc: ReferenceBodyProjectionV1 {
                instance: npc_instance,
                roots: npc_compiled.projection_roots,
            },
        })
    }

    pub fn roots_for(&self, subject_id: PersistentId) -> Option<&BodyProjectionRootsV1> {
        if self.player.instance.subject_id == subject_id {
            Some(&self.player.roots)
        } else if self.npc.instance.subject_id == subject_id {
            Some(&self.npc.roots)
        } else {
            None
        }
    }

    pub fn combined_root(&self) -> Result<ContentHash, ReferenceGameError> {
        let mut bytes = b"nextengine.reference-body-projection-set.v1\0".to_vec();
        bytes.extend_from_slice(self.body_schema_asset_revision.asset_id.as_bytes());
        bytes.extend_from_slice(self.body_schema_asset_revision.record_sha256.as_bytes());
        bytes.extend_from_slice(self.player.roots.projection_root()?.as_bytes());
        bytes.extend_from_slice(self.npc.roots.projection_root()?.as_bytes());
        Ok(next_contracts::ids::content_hash_from_bytes(
            next_contracts::canonical::sha256(&bytes),
        ))
    }
}
