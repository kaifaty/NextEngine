use std::collections::BTreeMap;

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::{ContentHash, IssuerPrincipal, SchemaId, WorldNamespaceId};

use super::codec::{
    decode_map, decode_nested_struct, encode_map, field, nested_array, nested_struct, nested_text,
    nested_u8, read_array, read_u16, require_envelope, require_fields, require_nested_fields,
};
use super::error::IdentityContractError;

pub const PRINCIPAL_REGISTRY_SCHEMA_VERSION: u16 = 1;

const PRINCIPAL_REGISTRY_OWNER_ID: &str = "nextengine.runtime";
const PRINCIPAL_REGISTRY_SCHEMA_ID: &str = "nextengine.principal-registry";
const PRINCIPAL_REGISTRY_SEGMENT_ID: &str = "v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PrincipalStatus {
    Active = 0,
    Disabled = 1,
    Retired = 2,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PrincipalRecordV1 {
    pub provenance_hash: ContentHash,
    pub capability_subject_id: SchemaId,
    pub status: PrincipalStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrincipalRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub principals: BTreeMap<IssuerPrincipal, PrincipalRecordV1>,
}

impl PrincipalRegistryV1 {
    #[must_use]
    pub const fn empty(world_namespace: WorldNamespaceId) -> Self {
        Self {
            schema_version: PRINCIPAL_REGISTRY_SCHEMA_VERSION,
            world_namespace,
            principals: BTreeMap::new(),
        }
    }

    pub fn register(
        &mut self,
        principal: IssuerPrincipal,
        record: PrincipalRecordV1,
    ) -> Result<bool, IdentityContractError> {
        match self.principals.get(&principal) {
            Some(existing) if existing == &record => Ok(false),
            Some(_) => Err(IdentityContractError::PrincipalCollision),
            None => {
                self.principals.insert(principal, record);
                Ok(true)
            }
        }
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != PRINCIPAL_REGISTRY_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "principal registry",
                version: self.schema_version,
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn is_active(&self, principal: &IssuerPrincipal) -> bool {
        self.principals
            .get(principal)
            .is_some_and(|record| record.status == PrincipalStatus::Active)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .principals
            .iter()
            .map(|(principal, record)| {
                Ok((
                    principal.canonical_bytes()?,
                    nested_struct([
                        CanonicalField::new(
                            1,
                            CANONICAL_TYPE_HASH256,
                            record.provenance_hash.as_bytes().to_vec(),
                        ),
                        CanonicalField::new(
                            2,
                            CANONICAL_TYPE_UTF8_NFC,
                            record.capability_subject_id.as_str().as_bytes().to_vec(),
                        ),
                        CanonicalField::new(3, CANONICAL_TYPE_U8, vec![record.status as u8]),
                    ])?,
                ))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        encode_canonical_segment(
            PRINCIPAL_REGISTRY_OWNER_ID,
            PRINCIPAL_REGISTRY_SCHEMA_ID,
            PRINCIPAL_REGISTRY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    self.world_namespace.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_map(entries)?),
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
            PRINCIPAL_REGISTRY_OWNER_ID,
            PRINCIPAL_REGISTRY_SCHEMA_ID,
            PRINCIPAL_REGISTRY_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut principals = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 3)?.payload.as_slice(), limits)? {
            let principal = IssuerPrincipal::from_canonical_bytes(&key, limits)?;
            let fields = decode_nested_struct(&value, limits)?;
            require_nested_fields(
                &fields,
                &[
                    (1, CANONICAL_TYPE_HASH256),
                    (2, CANONICAL_TYPE_UTF8_NFC),
                    (3, CANONICAL_TYPE_U8),
                ],
            )?;
            let record = PrincipalRecordV1 {
                provenance_hash: ContentHash::from_bytes(nested_array(&fields, 1)?),
                capability_subject_id: SchemaId::new(nested_text(&fields, 2)?)?,
                status: match nested_u8(&fields, 3)? {
                    0 => PrincipalStatus::Active,
                    1 => PrincipalStatus::Disabled,
                    2 => PrincipalStatus::Retired,
                    value => return Err(IdentityContractError::UnknownTag(value)),
                },
            };
            if principals.insert(principal, record).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let registry = Self {
            schema_version: read_u16(&segment, 1)?,
            world_namespace: WorldNamespaceId::from_bytes(read_array(&segment, 2)?),
            principals,
        };
        registry.validate()?;
        if registry.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(registry)
    }
}
