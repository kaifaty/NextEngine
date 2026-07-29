use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{ContentHash, ProjectId, WorldNamespaceId};

use super::codec::{read_array, read_text, read_u16, read_u32, require_envelope, require_fields};
use super::derivation::derive_world_namespace;
use super::error::IdentityContractError;

pub const WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION: u16 = 1;

const WORLD_IDENTITY_OWNER_ID: &str = "nextengine.runtime";
const WORLD_IDENTITY_SCHEMA_ID: &str = "nextengine.world-identity-manifest";
const WORLD_IDENTITY_SEGMENT_ID: &str = "v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldIdentityManifestV1 {
    pub schema_version: u16,
    pub project_id: ProjectId,
    pub world_creation_nonce: [u8; 32],
    pub world_namespace: WorldNamespaceId,
    pub identity_epoch: u32,
    pub rng_root_seed: [u8; 32],
    pub runtime_determinism_profile_hash: ContentHash,
}

impl WorldIdentityManifestV1 {
    pub fn new(
        project_id: ProjectId,
        world_creation_nonce: [u8; 32],
        rng_root_seed: [u8; 32],
        runtime_determinism_profile_hash: ContentHash,
    ) -> Result<Self, IdentityContractError> {
        let world_namespace = derive_world_namespace(&project_id, world_creation_nonce)?;
        Ok(Self {
            schema_version: WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION,
            project_id,
            world_creation_nonce,
            world_namespace,
            identity_epoch: 0,
            rng_root_seed,
            runtime_determinism_profile_hash,
        })
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "world identity",
                version: self.schema_version,
            });
        }
        if self.identity_epoch != 0 {
            return Err(IdentityContractError::IdentityEpochUnsupported(
                self.identity_epoch,
            ));
        }
        if derive_world_namespace(&self.project_id, self.world_creation_nonce)?
            != self.world_namespace
        {
            return Err(IdentityContractError::WorldNamespaceMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            WORLD_IDENTITY_OWNER_ID,
            WORLD_IDENTITY_SCHEMA_ID,
            WORLD_IDENTITY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.project_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    3,
                    crate::canonical::CANONICAL_TYPE_BYTES,
                    self.world_creation_nonce.to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_ID128,
                    self.world_namespace.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_U32,
                    self.identity_epoch.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    crate::canonical::CANONICAL_TYPE_BYTES,
                    self.rng_root_seed.to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_HASH256,
                    self.runtime_determinism_profile_hash.as_bytes().to_vec(),
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, IdentityContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            WORLD_IDENTITY_OWNER_ID,
            WORLD_IDENTITY_SCHEMA_ID,
            WORLD_IDENTITY_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, crate::canonical::CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_U32),
                (6, crate::canonical::CANONICAL_TYPE_BYTES),
                (7, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let manifest = Self {
            schema_version: read_u16(&segment, 1)?,
            project_id: ProjectId::new(read_text(&segment, 2)?)?,
            world_creation_nonce: read_array(&segment, 3)?,
            world_namespace: WorldNamespaceId::from_bytes(read_array(&segment, 4)?),
            identity_epoch: read_u32(&segment, 5)?,
            rng_root_seed: read_array(&segment, 6)?,
            runtime_determinism_profile_hash: ContentHash::from_bytes(read_array(&segment, 7)?),
        };
        manifest.validate()?;
        if manifest.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(manifest)
    }
}
