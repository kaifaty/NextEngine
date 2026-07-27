use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_STRUCT,
    CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_UTF8_NFC, CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, DecodedCanonicalSegment, decode_canonical_segment,
    encode_canonical_segment, sha256,
};
use crate::{
    CommandStreamId, ContentHash, IdentifierError, IssuerPrincipal, PlayerPrincipalId, ProjectId,
    SchemaId, WorldNamespaceId, content_hash_from_bytes,
};

pub const WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION: u16 = 1;
pub const PRINCIPAL_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const COMMAND_STREAM_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS: u32 = 3600;

const WORLD_IDENTITY_OWNER_ID: &str = "nextengine.runtime";
const WORLD_IDENTITY_SCHEMA_ID: &str = "nextengine.world-identity-manifest";
const WORLD_IDENTITY_SEGMENT_ID: &str = "v1";
const PRINCIPAL_REGISTRY_OWNER_ID: &str = "nextengine.runtime";
const PRINCIPAL_REGISTRY_SCHEMA_ID: &str = "nextengine.principal-registry";
const PRINCIPAL_REGISTRY_SEGMENT_ID: &str = "v1";
const STREAM_REGISTRY_OWNER_ID: &str = "nextengine.runtime";
const STREAM_REGISTRY_SCHEMA_ID: &str = "nextengine.command-stream-registry";
const STREAM_REGISTRY_SEGMENT_ID: &str = "v1";
const RUNTIME_PROFILE_OWNER_ID: &str = "nextengine.runtime";
const RUNTIME_PROFILE_SCHEMA_ID: &str = "nextengine.runtime-determinism-profile";
const RUNTIME_PROFILE_SEGMENT_ID: &str = "v1";

type EncodedMapEntries = Vec<(Vec<u8>, Vec<u8>)>;

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
                    crate::CANONICAL_TYPE_BYTES,
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
                CanonicalField::new(6, crate::CANONICAL_TYPE_BYTES, self.rng_root_seed.to_vec()),
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
                (3, crate::CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_U32),
                (6, crate::CANONICAL_TYPE_BYTES),
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandStreamKeyV1 {
    pub principal: IssuerPrincipal,
    pub stream_slot: u32,
    pub stream_epoch: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandStreamRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub entries: BTreeMap<CommandStreamKeyV1, CommandStreamId>,
    pub next_stream_slot: BTreeMap<IssuerPrincipal, u32>,
}

impl CommandStreamRegistryV1 {
    #[must_use]
    pub const fn empty(world_namespace: WorldNamespaceId) -> Self {
        Self {
            schema_version: COMMAND_STREAM_REGISTRY_SCHEMA_VERSION,
            world_namespace,
            entries: BTreeMap::new(),
            next_stream_slot: BTreeMap::new(),
        }
    }

    pub fn allocate_stream(
        &mut self,
        principal: IssuerPrincipal,
    ) -> Result<CommandStreamId, IdentityContractError> {
        let slot = self.next_stream_slot.get(&principal).copied().unwrap_or(0);
        let next = slot
            .checked_add(1)
            .ok_or(IdentityContractError::StreamSlotExhausted)?;
        let key = CommandStreamKeyV1 {
            principal: principal.clone(),
            stream_slot: slot,
            stream_epoch: 0,
        };
        let stream_id = derive_command_stream_id(
            self.world_namespace,
            &principal,
            key.stream_slot,
            key.stream_epoch,
        )?;
        if self.entries.insert(key, stream_id).is_some() {
            return Err(IdentityContractError::DuplicateKey);
        }
        self.next_stream_slot.insert(principal, next);
        Ok(stream_id)
    }

    pub fn binding(
        &self,
        stream_id: CommandStreamId,
    ) -> Option<(&CommandStreamKeyV1, CommandStreamId)> {
        self.entries
            .iter()
            .find(|(_, candidate)| **candidate == stream_id)
            .map(|(key, id)| (key, *id))
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != COMMAND_STREAM_REGISTRY_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "command stream registry",
                version: self.schema_version,
            });
        }
        let mut ids = BTreeMap::new();
        for (key, stream_id) in &self.entries {
            if derive_command_stream_id(
                self.world_namespace,
                &key.principal,
                key.stream_slot,
                key.stream_epoch,
            )? != *stream_id
            {
                return Err(IdentityContractError::CommandStreamMismatch);
            }
            if ids.insert(*stream_id, key).is_some() {
                return Err(IdentityContractError::CommandStreamCollision);
            }
            let next = self
                .next_stream_slot
                .get(&key.principal)
                .ok_or(IdentityContractError::NextStreamSlotMismatch)?;
            if *next <= key.stream_slot {
                return Err(IdentityContractError::NextStreamSlotMismatch);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .entries
            .iter()
            .map(|(key, stream_id)| {
                Ok((
                    nested_struct([
                        CanonicalField::new(
                            1,
                            CANONICAL_TYPE_TAGGED_UNION,
                            principal_union_payload(&key.principal)?,
                        ),
                        CanonicalField::new(
                            2,
                            CANONICAL_TYPE_U32,
                            key.stream_slot.to_le_bytes().to_vec(),
                        ),
                        CanonicalField::new(
                            3,
                            CANONICAL_TYPE_U32,
                            key.stream_epoch.to_le_bytes().to_vec(),
                        ),
                    ])?,
                    nested_value(CANONICAL_TYPE_ID128, stream_id.as_bytes())?,
                ))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        let next_slots = self
            .next_stream_slot
            .iter()
            .map(|(principal, next)| {
                Ok((
                    principal.canonical_bytes()?,
                    nested_value(CANONICAL_TYPE_U32, &next.to_le_bytes())?,
                ))
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        encode_canonical_segment(
            STREAM_REGISTRY_OWNER_ID,
            STREAM_REGISTRY_SCHEMA_ID,
            STREAM_REGISTRY_SEGMENT_ID,
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
                CanonicalField::new(4, CANONICAL_TYPE_MAP, encode_map(next_slots)?),
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
            STREAM_REGISTRY_OWNER_ID,
            STREAM_REGISTRY_SCHEMA_ID,
            STREAM_REGISTRY_SEGMENT_ID,
        )?;
        require_fields(
            &segment,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
                (4, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut entries = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 3)?.payload.as_slice(), limits)? {
            let key_fields = decode_nested_struct(&key, limits)?;
            require_nested_fields(
                &key_fields,
                &[
                    (1, CANONICAL_TYPE_TAGGED_UNION),
                    (2, CANONICAL_TYPE_U32),
                    (3, CANONICAL_TYPE_U32),
                ],
            )?;
            let stream_key = CommandStreamKeyV1 {
                principal: principal_from_union_payload(nested_payload(&key_fields, 1)?, limits)?,
                stream_slot: nested_u32(&key_fields, 2)?,
                stream_epoch: nested_u32(&key_fields, 3)?,
            };
            let stream_id =
                CommandStreamId::from_bytes(decode_nested_id(&value, CANONICAL_TYPE_ID128)?);
            if entries.insert(stream_key, stream_id).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let mut next_stream_slot = BTreeMap::new();
        for (key, value) in decode_map(field(&segment, 4)?.payload.as_slice(), limits)? {
            let principal = IssuerPrincipal::from_canonical_bytes(&key, limits)?;
            let next = u32::from_le_bytes(decode_nested_fixed(&value, CANONICAL_TYPE_U32)?);
            if next_stream_slot.insert(principal, next).is_some() {
                return Err(IdentityContractError::DuplicateKey);
            }
        }
        let registry = Self {
            schema_version: read_u16(&segment, 1)?,
            world_namespace: WorldNamespaceId::from_bytes(read_array(&segment, 2)?),
            entries,
            next_stream_slot,
        };
        registry.validate()?;
        if registry.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(registry)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeDeterminismProfileV1 {
    pub schema_version: u16,
    pub tick_rate_profile_hash: ContentHash,
    pub ingress_assignment_profile_hash: ContentHash,
    pub command_identity_profile_hash: ContentHash,
    pub command_ledger_profile_hash: ContentHash,
    pub rng_profile_hash: ContentHash,
    pub schedule_manifest_hash: ContentHash,
    pub task_merge_profile_hash: ContentHash,
    pub numeric_profile_hash: ContentHash,
    pub physics_quantization_profile_hash: ContentHash,
    pub command_kind_registry_hash: ContentHash,
    pub admission_limits_profile_hash: ContentHash,
    pub maximum_future_command_ticks: u32,
}

impl RuntimeDeterminismProfileV1 {
    #[must_use]
    pub fn bootstrap_default(command_kind_registry_hash: ContentHash) -> Self {
        let fixed = |name: &[u8]| content_hash_from_bytes(sha256(name));
        let admission_limits = crate::RuntimeAdmissionLimitsV1::default();
        let tick_rate_profile = crate::TickRateProfileV1::at_30_hz();
        let ingress_assignment_profile =
            crate::IngressAssignmentProfileV1::core_v1(&admission_limits)
                .expect("the built-in ingress profile is canonically representable");
        let physics_quantization_profile =
            crate::PhysicsQuantizationProfileV1::capsule_reference_v1()
                .expect("the built-in physics profile identifiers are valid");
        let authoritative_numeric_profile =
            crate::AuthoritativeNumericProfileV1::capsule_reference_v1(
                &physics_quantization_profile,
            )
            .expect("the built-in numeric profile is canonically representable");
        Self {
            schema_version: RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION,
            tick_rate_profile_hash: tick_rate_profile
                .profile_hash()
                .expect("the built-in tick profile is canonically representable"),
            ingress_assignment_profile_hash: ingress_assignment_profile
                .profile_hash()
                .expect("the built-in ingress profile is canonically representable"),
            command_identity_profile_hash: fixed(b"nextengine.bootstrap.command-identity.v2"),
            command_ledger_profile_hash: fixed(b"nextengine.bootstrap.command-ledger.v2"),
            rng_profile_hash: fixed(b"nextengine.bootstrap.rng.v1"),
            schedule_manifest_hash: fixed(b"nextengine.bootstrap.schedule.v1"),
            task_merge_profile_hash: fixed(b"nextengine.bootstrap.task-merge.v1"),
            numeric_profile_hash: authoritative_numeric_profile
                .profile_hash()
                .expect("the built-in numeric profile is canonically representable"),
            physics_quantization_profile_hash: physics_quantization_profile
                .profile_hash()
                .expect("the built-in physics profile is canonically representable"),
            command_kind_registry_hash,
            admission_limits_profile_hash: admission_limits
                .profile_hash()
                .expect("the built-in admission profile is canonically representable"),
            maximum_future_command_ticks: 120,
        }
    }

    pub fn validate(&self) -> Result<(), IdentityContractError> {
        if self.schema_version != RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION {
            return Err(IdentityContractError::UnsupportedVersion {
                contract: "runtime determinism profile",
                version: self.schema_version,
            });
        }
        if self.maximum_future_command_ticks > RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS {
            return Err(IdentityContractError::FutureHorizonExceeded(
                self.maximum_future_command_ticks,
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let hashes = [
            self.tick_rate_profile_hash,
            self.ingress_assignment_profile_hash,
            self.command_identity_profile_hash,
            self.command_ledger_profile_hash,
            self.rng_profile_hash,
            self.schedule_manifest_hash,
            self.task_merge_profile_hash,
            self.numeric_profile_hash,
            self.physics_quantization_profile_hash,
            self.command_kind_registry_hash,
            self.admission_limits_profile_hash,
        ];
        let mut fields = vec![CanonicalField::new(
            1,
            CANONICAL_TYPE_U16,
            self.schema_version.to_le_bytes().to_vec(),
        )];
        for (offset, hash) in hashes.into_iter().enumerate() {
            fields.push(CanonicalField::new(
                u32::try_from(offset).map_err(|_| CanonicalError::LengthOverflow)? + 2,
                CANONICAL_TYPE_HASH256,
                hash.as_bytes().to_vec(),
            ));
        }
        fields.push(CanonicalField::new(
            13,
            CANONICAL_TYPE_U32,
            self.maximum_future_command_ticks.to_le_bytes().to_vec(),
        ));
        encode_canonical_segment(
            RUNTIME_PROFILE_OWNER_ID,
            RUNTIME_PROFILE_SCHEMA_ID,
            RUNTIME_PROFILE_SEGMENT_ID,
            fields,
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, IdentityContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        require_envelope(
            &segment,
            RUNTIME_PROFILE_OWNER_ID,
            RUNTIME_PROFILE_SCHEMA_ID,
            RUNTIME_PROFILE_SEGMENT_ID,
        )?;
        let mut expected = vec![(1, CANONICAL_TYPE_U16)];
        expected.extend((2..=12).map(|id| (id, CANONICAL_TYPE_HASH256)));
        expected.push((13, CANONICAL_TYPE_U32));
        require_fields(&segment, &expected)?;
        let profile = Self {
            schema_version: read_u16(&segment, 1)?,
            tick_rate_profile_hash: read_hash(&segment, 2)?,
            ingress_assignment_profile_hash: read_hash(&segment, 3)?,
            command_identity_profile_hash: read_hash(&segment, 4)?,
            command_ledger_profile_hash: read_hash(&segment, 5)?,
            rng_profile_hash: read_hash(&segment, 6)?,
            schedule_manifest_hash: read_hash(&segment, 7)?,
            task_merge_profile_hash: read_hash(&segment, 8)?,
            numeric_profile_hash: read_hash(&segment, 9)?,
            physics_quantization_profile_hash: read_hash(&segment, 10)?,
            command_kind_registry_hash: read_hash(&segment, 11)?,
            admission_limits_profile_hash: read_hash(&segment, 12)?,
            maximum_future_command_ticks: read_u32(&segment, 13)?,
        };
        profile.validate()?;
        if profile.canonical_bytes()? != bytes {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }
        Ok(profile)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        runtime_profile_hash(self)
    }
}

pub fn derive_world_namespace(
    project_id: &ProjectId,
    world_creation_nonce: [u8; 32],
) -> Result<WorldNamespaceId, CanonicalError> {
    let project_bytes = project_id.as_str().as_bytes();
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.world-namespace.v1\0");
    extend_lp(&mut preimage, project_bytes)?;
    preimage.extend_from_slice(&world_creation_nonce);
    Ok(WorldNamespaceId::from_bytes(left128(sha256(&preimage))))
}

#[must_use]
pub fn derive_player_principal_id(
    world_namespace: WorldNamespaceId,
    player_slot: u32,
) -> PlayerPrincipalId {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.player-principal.v1\0");
    preimage.extend_from_slice(world_namespace.as_bytes());
    preimage.extend_from_slice(&player_slot.to_le_bytes());
    PlayerPrincipalId::from_bytes(left128(sha256(&preimage)))
}

pub fn derive_command_stream_id(
    world_namespace: WorldNamespaceId,
    principal: &IssuerPrincipal,
    stream_slot: u32,
    stream_epoch: u32,
) -> Result<CommandStreamId, CanonicalError> {
    let principal_bytes = principal.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-stream.v1\0");
    preimage.extend_from_slice(world_namespace.as_bytes());
    extend_lp(&mut preimage, &principal_bytes)?;
    preimage.extend_from_slice(&stream_slot.to_le_bytes());
    preimage.extend_from_slice(&stream_epoch.to_le_bytes());
    Ok(CommandStreamId::from_bytes(left128(sha256(&preimage))))
}

pub fn runtime_profile_hash(
    profile: &RuntimeDeterminismProfileV1,
) -> Result<ContentHash, CanonicalError> {
    let bytes = profile.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.runtime-profile.v1\0");
    extend_lp(&mut preimage, RUNTIME_PROFILE_SCHEMA_ID.as_bytes())?;
    extend_lp(&mut preimage, &bytes)?;
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IdentityContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    Principal(crate::PrincipalDecodeError),
    WrongEnvelope,
    MissingField(u32),
    UnknownField(u32),
    WrongFieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    InvalidFieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedVersion {
        contract: &'static str,
        version: u16,
    },
    IdentityEpochUnsupported(u32),
    WorldNamespaceMismatch,
    PrincipalCollision,
    CommandStreamMismatch,
    CommandStreamCollision,
    NextStreamSlotMismatch,
    StreamSlotExhausted,
    FutureHorizonExceeded(u32),
    UnknownTag(u8),
    DuplicateKey,
    NonCanonicalEncoding,
}

impl Display for IdentityContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => {
                write!(formatter, "identity canonicalization failed: {error}")
            }
            Self::Decode(error) => write!(formatter, "identity encoding is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "identity identifier is invalid: {error}"),
            Self::Principal(error) => write!(formatter, "identity principal is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("identity contract envelope is invalid"),
            Self::MissingField(id) => write!(formatter, "identity contract field {id} is missing"),
            Self::UnknownField(id) => write!(formatter, "identity contract field {id} is unknown"),
            Self::WrongFieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "identity field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidFieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "identity field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedVersion { contract, version } => {
                write!(formatter, "unsupported {contract} schema version {version}")
            }
            Self::IdentityEpochUnsupported(epoch) => {
                write!(formatter, "identity epoch {epoch} is unsupported")
            }
            Self::WorldNamespaceMismatch => {
                formatter.write_str("world namespace does not match project and creation nonce")
            }
            Self::PrincipalCollision => formatter.write_str("principal has conflicting provenance"),
            Self::CommandStreamMismatch => {
                formatter.write_str("command stream ID does not match its canonical provenance")
            }
            Self::CommandStreamCollision => {
                formatter.write_str("command stream ID has conflicting provenance")
            }
            Self::NextStreamSlotMismatch => {
                formatter.write_str("next command stream slot does not follow allocated slots")
            }
            Self::StreamSlotExhausted => formatter.write_str("command stream slot exhausted"),
            Self::FutureHorizonExceeded(value) => write!(
                formatter,
                "maximum future command horizon {value} exceeds the hard maximum"
            ),
            Self::UnknownTag(tag) => write!(formatter, "identity contract tag {tag} is unknown"),
            Self::DuplicateKey => formatter.write_str("identity contract contains a duplicate key"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("identity contract does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for IdentityContractError {}

impl From<CanonicalError> for IdentityContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for IdentityContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for IdentityContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::PrincipalDecodeError> for IdentityContractError {
    fn from(error: crate::PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

fn left128(digest: [u8; 32]) -> [u8; 16] {
    let mut result = [0; 16];
    result.copy_from_slice(&digest[..16]);
    result
}

fn extend_lp(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonicalError> {
    output.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}

fn require_envelope(
    segment: &DecodedCanonicalSegment,
    owner: &str,
    schema: &str,
    id: &str,
) -> Result<(), IdentityContractError> {
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != id {
        return Err(IdentityContractError::WrongEnvelope);
    }
    Ok(())
}

fn require_fields(
    segment: &DecodedCanonicalSegment,
    expected: &[(u32, u8)],
) -> Result<(), IdentityContractError> {
    for actual in &segment.fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(IdentityContractError::UnknownField(actual.field_id));
        }
    }
    for (id, expected_type) in expected {
        let actual = segment
            .field(*id)
            .ok_or(IdentityContractError::MissingField(*id))?;
        if actual.type_tag != *expected_type {
            return Err(IdentityContractError::WrongFieldType {
                field_id: *id,
                expected: *expected_type,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&CanonicalField, IdentityContractError> {
    segment
        .field(field_id)
        .ok_or(IdentityContractError::MissingField(field_id))
}

fn read_array<const N: usize>(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<[u8; N], IdentityContractError> {
    field(segment, field_id)?
        .payload
        .as_slice()
        .try_into()
        .map_err(|_| IdentityContractError::InvalidFieldLength {
            field_id,
            expected: N,
            actual: field(segment, field_id)
                .map(|field| field.payload.len())
                .unwrap_or_default(),
        })
}

fn read_u16(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u16, IdentityContractError> {
    Ok(u16::from_le_bytes(read_array(segment, field_id)?))
}

fn read_u32(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u32, IdentityContractError> {
    Ok(u32::from_le_bytes(read_array(segment, field_id)?))
}

fn read_hash(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<ContentHash, IdentityContractError> {
    Ok(ContentHash::from_bytes(read_array(segment, field_id)?))
}

fn read_text(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&str, IdentityContractError> {
    std::str::from_utf8(&field(segment, field_id)?.payload)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::InvalidUtf8))
}

fn nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![type_tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn nested_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        payload.extend_from_slice(&field.field_id.to_le_bytes());
        payload.push(field.type_tag);
        payload.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&field.payload);
    }
    nested_value(CANONICAL_TYPE_STRUCT, &payload)
}

fn encode_map(mut entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, CanonicalError> {
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (key, value) in entries {
        bytes.extend_from_slice(&key);
        bytes.extend_from_slice(&value);
    }
    Ok(bytes)
}

fn decode_nested_value<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), IdentityContractError> {
    let tag = cursor.read_u8()?;
    let length = usize::try_from(cursor.read_u64()?)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
    if length > limits.max_field_payload_bytes {
        return Err(IdentityContractError::Decode(
            CanonicalDecodeError::FieldPayloadTooLarge {
                field_id: 0,
                actual: length,
                limit: limits.max_field_payload_bytes,
            },
        ));
    }
    Ok((tag, cursor.read_exact(length)?))
}

fn decode_map(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<EncodedMapEntries, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut result = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let (key_tag, key_payload) = decode_nested_value(&mut cursor, limits)?;
        let key = nested_value(key_tag, key_payload)?;
        if previous.as_ref().is_some_and(|prior| prior >= &key) {
            return Err(IdentityContractError::DuplicateKey);
        }
        let (value_tag, value_payload) = decode_nested_value(&mut cursor, limits)?;
        let value = nested_value(value_tag, value_payload)?;
        previous = Some(key.clone());
        result.push((key, value));
    }
    cursor.finish()?;
    Ok(result)
}

fn decode_nested_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, IdentityContractError> {
    let mut outer = CanonicalCursor::new(bytes);
    let (tag, payload) = decode_nested_value(&mut outer, limits)?;
    outer.finish()?;
    if tag != CANONICAL_TYPE_STRUCT {
        return Err(IdentityContractError::WrongFieldType {
            field_id: 0,
            expected: CANONICAL_TYPE_STRUCT,
            actual: tag,
        });
    }
    let mut cursor = CanonicalCursor::new(payload);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let field_id = cursor.read_u32()?;
        if previous.is_some_and(|id| id >= field_id) {
            return Err(IdentityContractError::DuplicateKey);
        }
        let type_tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
        let payload = cursor.read_exact(length)?.to_vec();
        fields.push(CanonicalField::new(field_id, type_tag, payload));
        previous = Some(field_id);
    }
    cursor.finish()?;
    Ok(fields)
}

fn require_nested_fields(
    fields: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), IdentityContractError> {
    for actual in fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(IdentityContractError::UnknownField(actual.field_id));
        }
    }
    for (id, tag) in expected {
        let actual = fields
            .iter()
            .find(|field| field.field_id == *id)
            .ok_or(IdentityContractError::MissingField(*id))?;
        if actual.type_tag != *tag {
            return Err(IdentityContractError::WrongFieldType {
                field_id: *id,
                expected: *tag,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn nested_field(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&CanonicalField, IdentityContractError> {
    fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(IdentityContractError::MissingField(id))
}

fn nested_payload(fields: &[CanonicalField], id: u32) -> Result<&[u8], IdentityContractError> {
    Ok(&nested_field(fields, id)?.payload)
}

fn nested_array<const N: usize>(
    fields: &[CanonicalField],
    id: u32,
) -> Result<[u8; N], IdentityContractError> {
    nested_payload(fields, id)?
        .try_into()
        .map_err(|_| IdentityContractError::InvalidFieldLength {
            field_id: id,
            expected: N,
            actual: nested_payload(fields, id).map_or(0, <[u8]>::len),
        })
}

fn nested_text(fields: &[CanonicalField], id: u32) -> Result<&str, IdentityContractError> {
    std::str::from_utf8(nested_payload(fields, id)?)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::InvalidUtf8))
}

fn nested_u8(fields: &[CanonicalField], id: u32) -> Result<u8, IdentityContractError> {
    Ok(nested_array::<1>(fields, id)?[0])
}

fn nested_u32(fields: &[CanonicalField], id: u32) -> Result<u32, IdentityContractError> {
    Ok(u32::from_le_bytes(nested_array(fields, id)?))
}

fn decode_nested_fixed<const N: usize>(
    bytes: &[u8],
    expected_tag: u8,
) -> Result<[u8; N], IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let tag = cursor.read_u8()?;
    let length = cursor.read_u64()?;
    if tag != expected_tag {
        return Err(IdentityContractError::WrongFieldType {
            field_id: 0,
            expected: expected_tag,
            actual: tag,
        });
    }
    if length != N as u64 {
        return Err(IdentityContractError::InvalidFieldLength {
            field_id: 0,
            expected: N,
            actual: usize::try_from(length).unwrap_or(usize::MAX),
        });
    }
    let value = cursor.read_exact(N)?.try_into().map_err(|_| {
        IdentityContractError::InvalidFieldLength {
            field_id: 0,
            expected: N,
            actual: 0,
        }
    })?;
    cursor.finish()?;
    Ok(value)
}

fn decode_nested_id(bytes: &[u8], expected_tag: u8) -> Result<[u8; 16], IdentityContractError> {
    decode_nested_fixed(bytes, expected_tag)
}

fn principal_union_payload(principal: &IssuerPrincipal) -> Result<Vec<u8>, CanonicalError> {
    let full = principal.canonical_bytes()?;
    let mut cursor = CanonicalCursor::new(&full);
    let tag = cursor
        .read_u8()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    if tag != CANONICAL_TYPE_TAGGED_UNION {
        return Err(CanonicalError::LengthOverflow);
    }
    let length = usize::try_from(
        cursor
            .read_u64()
            .map_err(|_| CanonicalError::LengthOverflow)?,
    )
    .map_err(|_| CanonicalError::LengthOverflow)?;
    let payload = cursor
        .read_exact(length)
        .map_err(|_| CanonicalError::LengthOverflow)?
        .to_vec();
    cursor
        .finish()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    Ok(payload)
}

fn principal_from_union_payload(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<IssuerPrincipal, IdentityContractError> {
    IssuerPrincipal::from_canonical_bytes(
        &nested_value(CANONICAL_TYPE_TAGGED_UNION, payload)?,
        limits,
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> ContentHash {
        ContentHash::from_bytes([byte; 32])
    }

    #[test]
    fn world_player_and_stream_ids_are_stable_and_body_sensitive() {
        let project = ProjectId::new("nextengine.fixture").expect("fixture id");
        let world = WorldIdentityManifestV1::new(project, [1; 32], [2; 32], hash(3))
            .expect("world identity");
        assert_eq!(
            world.world_namespace.to_hex(),
            "b4911c2f5b2b7cc4371e28a522408982"
        );
        assert_eq!(
            derive_player_principal_id(world.world_namespace, 0).to_hex(),
            "7fd6241b679a81c7a1407665aa40f340"
        );
        let principal =
            IssuerPrincipal::Player(derive_player_principal_id(world.world_namespace, 0));
        let stream =
            derive_command_stream_id(world.world_namespace, &principal, 0, 0).expect("stream id");
        assert_eq!(stream.to_hex(), "d23d8b2c25c0744daab76ef54bd38038");
        assert_ne!(
            stream,
            derive_command_stream_id(world.world_namespace, &principal, 1, 0)
                .expect("second stream")
        );
    }

    #[test]
    fn identity_contracts_round_trip_and_reject_v1_drift() {
        let profile = RuntimeDeterminismProfileV1::bootstrap_default(hash(9));
        let profile_bytes = profile.canonical_bytes().expect("profile bytes");
        assert_eq!(
            RuntimeDeterminismProfileV1::from_canonical_bytes(
                &profile_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("profile round trip"),
            profile
        );
        let world = WorldIdentityManifestV1::new(
            ProjectId::new("nextengine.fixture").expect("fixture id"),
            [1; 32],
            [2; 32],
            profile.profile_hash().expect("profile hash"),
        )
        .expect("world");
        let world_bytes = world.canonical_bytes().expect("world bytes");
        assert_eq!(
            WorldIdentityManifestV1::from_canonical_bytes(
                &world_bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("world round trip"),
            world
        );
    }

    #[test]
    fn registries_are_order_independent_and_allocate_monotonic_slots() {
        let namespace = WorldNamespaceId::from_bytes([4; 16]);
        let first = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
        let second = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16]));
        let record = |byte| PrincipalRecordV1 {
            provenance_hash: hash(byte),
            capability_subject_id: SchemaId::new(format!("fixture.subject.{byte}"))
                .expect("subject id"),
            status: PrincipalStatus::Active,
        };
        let mut left = PrincipalRegistryV1::empty(namespace);
        left.register(first.clone(), record(1)).expect("first");
        left.register(second.clone(), record(2)).expect("second");
        let mut right = PrincipalRegistryV1::empty(namespace);
        right.register(second.clone(), record(2)).expect("second");
        right.register(first.clone(), record(1)).expect("first");
        assert_eq!(
            left.canonical_bytes().expect("left"),
            right.canonical_bytes().expect("right")
        );

        let mut streams = CommandStreamRegistryV1::empty(namespace);
        let stream0 = streams.allocate_stream(first.clone()).expect("slot zero");
        let stream1 = streams.allocate_stream(first).expect("slot one");
        assert_ne!(stream0, stream1);
        streams.validate().expect("registry closure");
        let bytes = streams.canonical_bytes().expect("stream bytes");
        assert_eq!(
            CommandStreamRegistryV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("stream round trip"),
            streams
        );
    }
}
