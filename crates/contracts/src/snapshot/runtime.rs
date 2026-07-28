use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AuthoritativeNumericProfileV1, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, CausalIdentityKey, CausalIdentityKind, CommandBodyArchiveV1,
    CommandLedgerError, CommandLedgerHash, CommandLedgerV2, CommandStreamRegistryV1,
    IdentityContractError, IngressAssignmentProfileV1, IngressCheckpointV1, InputContractError,
    IssuerPrincipal, PhysicsContractError, PhysicsQuantizationProfileV1,
    PlayerControllerRegistryV1, PrincipalRegistryV1, RpgContractErrorV1, RpgRuntimeBindingsV1,
    RuntimeAdmissionLimitsV1, RuntimeDeterminismProfileV1, TickRateProfileV1,
    WorldIdentityManifestV1, causal_provenance_hash, decode_canonical_segment,
    encode_canonical_segment,
};

pub const RUNTIME_SNAPSHOT_SCHEMA_VERSION: u32 = 3;
pub const RUNTIME_SNAPSHOT_OWNER_ID: &str = "nextengine.runtime";
pub const RUNTIME_SNAPSHOT_SCHEMA_ID: &str = "nextengine.runtime-snapshot";
pub const RUNTIME_SNAPSHOT_SEGMENT_ID: &str = "v3";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshotV3 {
    pub next_tick: u64,
    pub committed_event_count: u64,
    pub authoritative_revision: u64,
    pub world_identity: WorldIdentityManifestV1,
    pub principal_registry: PrincipalRegistryV1,
    pub stream_registry: CommandStreamRegistryV1,
    pub runtime_profile: RuntimeDeterminismProfileV1,
    pub admission_limits: RuntimeAdmissionLimitsV1,
    pub tick_rate_profile: TickRateProfileV1,
    pub ingress_assignment_profile: IngressAssignmentProfileV1,
    pub authoritative_numeric_profile: AuthoritativeNumericProfileV1,
    pub physics_quantization_profile: PhysicsQuantizationProfileV1,
    pub player_controller_registry: PlayerControllerRegistryV1,
    pub ingress_checkpoint: IngressCheckpointV1,
    pub rpg_runtime_bindings: RpgRuntimeBindingsV1,
    pub command_ledger: CommandLedgerV2,
    pub body_archive: CommandBodyArchiveV1,
}

pub type RuntimeSnapshot = RuntimeSnapshotV3;

impl RuntimeSnapshotV3 {
    pub fn validate(&self) -> Result<(), SnapshotDecodeError> {
        self.world_identity.validate()?;
        self.principal_registry.validate()?;
        self.stream_registry.validate()?;
        self.runtime_profile.validate()?;
        self.admission_limits.validate()?;
        self.tick_rate_profile.validate()?;
        self.ingress_assignment_profile.validate()?;
        self.authoritative_numeric_profile.validate()?;
        self.physics_quantization_profile.validate()?;
        self.player_controller_registry.validate()?;
        self.ingress_checkpoint.validate(&self.admission_limits)?;
        self.rpg_runtime_bindings.validate()?;
        let profile_hash = self.runtime_profile.profile_hash()?;
        let world = self.world_identity.world_namespace;
        if self.world_identity.runtime_determinism_profile_hash != profile_hash
            || self.principal_registry.world_namespace != world
            || self.stream_registry.world_namespace != world
            || self.player_controller_registry.world_namespace != world
            || self.command_ledger.world_namespace != world
            || self.command_ledger.runtime_determinism_profile_hash != profile_hash
            || self.command_ledger.command_kind_registry_hash
                != self.runtime_profile.command_kind_registry_hash
        {
            return Err(SnapshotDecodeError::ClosureMismatch);
        }
        if self.runtime_profile.admission_limits_profile_hash
            != self.admission_limits.profile_hash()?
            || self.runtime_profile.tick_rate_profile_hash
                != self.tick_rate_profile.profile_hash()?
            || self.runtime_profile.ingress_assignment_profile_hash
                != self.ingress_assignment_profile.profile_hash()?
            || self.runtime_profile.numeric_profile_hash
                != self.authoritative_numeric_profile.profile_hash()?
            || self.runtime_profile.physics_quantization_profile_hash
                != self.physics_quantization_profile.profile_hash()?
            || self.ingress_assignment_profile.admission_limits_hash
                != self.admission_limits.profile_hash()?
            || self
                .authoritative_numeric_profile
                .physics_quantization_profile_hash
                != self.physics_quantization_profile.profile_hash()?
            || self.ingress_checkpoint.current_tick != self.next_tick
        {
            return Err(SnapshotDecodeError::ProfileClosureMismatch);
        }
        for binding in self.player_controller_registry.bindings.values() {
            if !self.principal_registry.is_active(&binding.principal)
                || self.stream_registry.entries.iter().all(|(key, stream)| {
                    key.principal != binding.principal || stream != &binding.command_stream_id
                })
            {
                return Err(SnapshotDecodeError::ControllerRegistryMismatch);
            }
        }
        for (stream_key, stream_id) in &self.stream_registry.entries {
            if !self.principal_registry.is_active(&stream_key.principal) {
                return Err(SnapshotDecodeError::InactivePrincipal);
            }
            let stream = self
                .command_ledger
                .streams
                .get(stream_id)
                .ok_or(SnapshotDecodeError::StreamRegistryMismatch)?;
            if stream.issuer != stream_key.principal
                || stream.stream_slot != stream_key.stream_slot
                || stream.stream_epoch != stream_key.stream_epoch
            {
                return Err(SnapshotDecodeError::StreamRegistryMismatch);
            }
            let principal_bytes = stream_key.principal.canonical_bytes()?;
            let mut provenance = Vec::new();
            provenance.extend_from_slice(world.as_bytes());
            provenance.extend_from_slice(
                &u64::try_from(principal_bytes.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            provenance.extend_from_slice(&principal_bytes);
            provenance.extend_from_slice(&stream_key.stream_slot.to_le_bytes());
            provenance.extend_from_slice(&stream_key.stream_epoch.to_le_bytes());
            let expected = causal_provenance_hash(CausalIdentityKind::CommandStream, &provenance)?;
            if self
                .command_ledger
                .causal_identity_registry
                .bindings
                .get(&CausalIdentityKey {
                    identity_kind: CausalIdentityKind::CommandStream,
                    identity_bytes: *stream_id.as_bytes(),
                })
                != Some(&expected)
            {
                return Err(SnapshotDecodeError::CausalRegistryMismatch);
            }
        }
        if self.command_ledger.streams.len() != self.stream_registry.entries.len() {
            return Err(SnapshotDecodeError::StreamRegistryMismatch);
        }
        for (principal, record) in &self.principal_registry.principals {
            if let IssuerPrincipal::Player(id) = principal
                && self
                    .command_ledger
                    .causal_identity_registry
                    .bindings
                    .get(&CausalIdentityKey {
                        identity_kind: CausalIdentityKind::PlayerPrincipal,
                        identity_bytes: *id.as_bytes(),
                    })
                    != Some(&record.provenance_hash)
            {
                return Err(SnapshotDecodeError::CausalRegistryMismatch);
            }
        }
        let domain_event_count = self
            .command_ledger
            .causal_identity_registry
            .bindings
            .keys()
            .filter(|key| key.identity_kind == CausalIdentityKind::DomainEvent)
            .count();
        if u64::try_from(domain_event_count).map_err(|_| CanonicalError::LengthOverflow)?
            != self.committed_event_count
        {
            return Err(SnapshotDecodeError::CausalRegistryMismatch);
        }
        self.command_ledger.validate(&self.body_archive)?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate().map_err(|error| match error {
            SnapshotDecodeError::Canonicalization(error) => error,
            SnapshotDecodeError::Identity(IdentityContractError::Canonical(error)) => error,
            SnapshotDecodeError::Ledger(CommandLedgerError::Canonical(error)) => error,
            _ => CanonicalError::DuplicateSequenceValue,
        })?;
        let ledger_bytes = self
            .command_ledger
            .canonical_bytes(&self.body_archive)
            .map_err(|error| match error {
                CommandLedgerError::Canonical(error) => error,
                _ => CanonicalError::DuplicateSequenceValue,
            })?;
        encode_canonical_segment(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    u16::try_from(RUNTIME_SNAPSHOT_SCHEMA_VERSION)
                        .map_err(|_| CanonicalError::LengthOverflow)?
                        .to_le_bytes()
                        .to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_U64, self.next_tick.to_le_bytes().to_vec()),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.committed_event_count.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_U64,
                    self.authoritative_revision.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_BYTES,
                    self.world_identity.canonical_bytes()?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_BYTES,
                    self.principal_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_BYTES,
                    self.stream_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_BYTES,
                    self.runtime_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_BYTES,
                    self.admission_limits.canonical_bytes()?,
                ),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_BYTES,
                    self.tick_rate_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_BYTES,
                    self.ingress_assignment_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_BYTES,
                    self.authoritative_numeric_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_BYTES,
                    self.physics_quantization_profile.canonical_bytes()?,
                ),
                CanonicalField::new(
                    14,
                    CANONICAL_TYPE_BYTES,
                    self.player_controller_registry.canonical_bytes()?,
                ),
                CanonicalField::new(
                    15,
                    CANONICAL_TYPE_BYTES,
                    self.ingress_checkpoint.canonical_bytes()?,
                ),
                CanonicalField::new(16, CANONICAL_TYPE_BYTES, ledger_bytes),
                CanonicalField::new(
                    17,
                    CANONICAL_TYPE_BYTES,
                    self.body_archive.canonical_bytes()?,
                ),
                CanonicalField::new(
                    18,
                    CANONICAL_TYPE_BYTES,
                    self.rpg_runtime_bindings.canonical_bytes()?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, SnapshotDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != RUNTIME_SNAPSHOT_OWNER_ID
            || segment.schema_id != RUNTIME_SNAPSHOT_SCHEMA_ID
            || segment.segment_id != RUNTIME_SNAPSHOT_SEGMENT_ID
        {
            return Err(SnapshotDecodeError::WrongEnvelope);
        }
        let version_field = segment
            .field(1)
            .ok_or(SnapshotDecodeError::MissingField(1))?;
        let version = match version_field.type_tag {
            CANONICAL_TYPE_U16 if version_field.payload.len() == 2 => u32::from(
                u16::from_le_bytes(version_field.payload.as_slice().try_into().map_err(|_| {
                    SnapshotDecodeError::FieldLength {
                        field_id: 1,
                        expected: 2,
                        actual: version_field.payload.len(),
                    }
                })?),
            ),
            CANONICAL_TYPE_U32 if version_field.payload.len() == 4 => {
                u32::from_le_bytes(version_field.payload.as_slice().try_into().map_err(|_| {
                    SnapshotDecodeError::FieldLength {
                        field_id: 1,
                        expected: 4,
                        actual: version_field.payload.len(),
                    }
                })?)
            }
            _ => {
                return Err(SnapshotDecodeError::FieldType {
                    field_id: 1,
                    expected: CANONICAL_TYPE_U16,
                    actual: version_field.type_tag,
                });
            }
        };
        if version != RUNTIME_SNAPSHOT_SCHEMA_VERSION {
            return Err(SnapshotDecodeError::UnsupportedSchemaVersion(version));
        }
        require_fields(&segment)?;
        let admission_limits =
            RuntimeAdmissionLimitsV1::from_canonical_bytes(field(&segment, 9)?, limits)?;
        let archive = CommandBodyArchiveV1::from_canonical_bytes(field(&segment, 17)?, limits)?;
        let snapshot = Self {
            next_tick: read_u64(&segment, 2)?,
            committed_event_count: read_u64(&segment, 3)?,
            authoritative_revision: read_u64(&segment, 4)?,
            world_identity: WorldIdentityManifestV1::from_canonical_bytes(
                field(&segment, 5)?,
                limits,
            )?,
            principal_registry: PrincipalRegistryV1::from_canonical_bytes(
                field(&segment, 6)?,
                limits,
            )?,
            stream_registry: CommandStreamRegistryV1::from_canonical_bytes(
                field(&segment, 7)?,
                limits,
            )?,
            runtime_profile: RuntimeDeterminismProfileV1::from_canonical_bytes(
                field(&segment, 8)?,
                limits,
            )?,
            admission_limits,
            tick_rate_profile: TickRateProfileV1::from_canonical_bytes(
                field(&segment, 10)?,
                limits,
            )?,
            ingress_assignment_profile: IngressAssignmentProfileV1::from_canonical_bytes(
                field(&segment, 11)?,
                limits,
            )?,
            authoritative_numeric_profile: AuthoritativeNumericProfileV1::from_canonical_bytes(
                field(&segment, 12)?,
                limits,
            )?,
            physics_quantization_profile: PhysicsQuantizationProfileV1::from_canonical_bytes(
                field(&segment, 13)?,
                limits,
            )?,
            player_controller_registry: PlayerControllerRegistryV1::from_canonical_bytes(
                field(&segment, 14)?,
                limits,
            )?,
            ingress_checkpoint: IngressCheckpointV1::from_canonical_bytes(
                field(&segment, 15)?,
                limits,
                &admission_limits,
            )?,
            rpg_runtime_bindings: RpgRuntimeBindingsV1::from_canonical_bytes(field(&segment, 18)?)?,
            command_ledger: CommandLedgerV2::from_canonical_bytes(
                field(&segment, 16)?,
                &archive,
                limits,
            )?,
            body_archive: archive,
        };
        snapshot.validate()?;
        if snapshot.canonical_bytes()? != bytes {
            return Err(SnapshotDecodeError::NonCanonicalEncoding);
        }
        Ok(snapshot)
    }

    pub fn command_ledger_hash(&self) -> Result<CommandLedgerHash, CanonicalError> {
        self.command_ledger
            .command_ledger_hash(&self.body_archive)
            .map_err(|error| match error {
                CommandLedgerError::Canonical(error) => error,
                _ => CanonicalError::DuplicateSequenceValue,
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SnapshotDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identity(IdentityContractError),
    Ledger(CommandLedgerError),
    Input(InputContractError),
    Physics(PhysicsContractError),
    RpgBindings(RpgContractErrorV1),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    FieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedSchemaVersion(u32),
    ClosureMismatch,
    ProfileClosureMismatch,
    ControllerRegistryMismatch,
    InactivePrincipal,
    StreamRegistryMismatch,
    CausalRegistryMismatch,
    NonCanonicalEncoding,
}

impl SnapshotDecodeError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "UNSUPPORTED_RUNTIME_SNAPSHOT_VERSION",
            Self::ClosureMismatch
            | Self::ProfileClosureMismatch
            | Self::ControllerRegistryMismatch
            | Self::InactivePrincipal
            | Self::StreamRegistryMismatch
            | Self::CausalRegistryMismatch
            | Self::Ledger(_)
            | Self::RpgBindings(_) => "RUNTIME_SNAPSHOT_CLOSURE_CORRUPT",
            _ => "RUNTIME_SNAPSHOT_INVALID",
        }
    }
}

impl Display for SnapshotDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "snapshot encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "snapshot canonicalization failed: {error}")
            }
            Self::Identity(error) => write!(formatter, "snapshot identity is invalid: {error}"),
            Self::Ledger(error) => write!(formatter, "snapshot ledger is invalid: {error}"),
            Self::Input(error) => write!(formatter, "snapshot ingress is invalid: {error}"),
            Self::Physics(error) => {
                write!(formatter, "snapshot physics profile is invalid: {error}")
            }
            Self::RpgBindings(error) => {
                write!(
                    formatter,
                    "snapshot RPG runtime bindings are invalid: {error}"
                )
            }
            Self::WrongEnvelope => formatter.write_str("snapshot envelope does not match V3"),
            Self::UnknownField(id) => write!(formatter, "unknown snapshot field {id}"),
            Self::MissingField(id) => write!(formatter, "missing snapshot field {id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "snapshot field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported snapshot schema version {version}")
            }
            Self::ClosureMismatch => {
                formatter.write_str("snapshot world/profile/registry closure does not match")
            }
            Self::ProfileClosureMismatch => {
                formatter.write_str("snapshot component profile closure does not match")
            }
            Self::ControllerRegistryMismatch => {
                formatter.write_str("snapshot controller registry closure does not match")
            }
            Self::InactivePrincipal => {
                formatter.write_str("snapshot stream references an inactive principal")
            }
            Self::StreamRegistryMismatch => {
                formatter.write_str("snapshot stream registry does not match ledger streams")
            }
            Self::CausalRegistryMismatch => {
                formatter.write_str("snapshot causal registry does not close identity provenance")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("snapshot does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for SnapshotDecodeError {}

impl From<CanonicalDecodeError> for SnapshotDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for SnapshotDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<IdentityContractError> for SnapshotDecodeError {
    fn from(error: IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<CommandLedgerError> for SnapshotDecodeError {
    fn from(error: CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<InputContractError> for SnapshotDecodeError {
    fn from(error: InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<PhysicsContractError> for SnapshotDecodeError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<RpgContractErrorV1> for SnapshotDecodeError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgBindings(error)
    }
}

fn require_fields(segment: &crate::DecodedCanonicalSegment) -> Result<(), SnapshotDecodeError> {
    const EXPECTED: [(u32, u8); 18] = [
        (1, CANONICAL_TYPE_U16),
        (2, CANONICAL_TYPE_U64),
        (3, CANONICAL_TYPE_U64),
        (4, CANONICAL_TYPE_U64),
        (5, CANONICAL_TYPE_BYTES),
        (6, CANONICAL_TYPE_BYTES),
        (7, CANONICAL_TYPE_BYTES),
        (8, CANONICAL_TYPE_BYTES),
        (9, CANONICAL_TYPE_BYTES),
        (10, CANONICAL_TYPE_BYTES),
        (11, CANONICAL_TYPE_BYTES),
        (12, CANONICAL_TYPE_BYTES),
        (13, CANONICAL_TYPE_BYTES),
        (14, CANONICAL_TYPE_BYTES),
        (15, CANONICAL_TYPE_BYTES),
        (16, CANONICAL_TYPE_BYTES),
        (17, CANONICAL_TYPE_BYTES),
        (18, CANONICAL_TYPE_BYTES),
    ];
    for actual in &segment.fields {
        if !EXPECTED.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(SnapshotDecodeError::UnknownField(actual.field_id));
        }
    }
    for (id, expected) in EXPECTED {
        let actual = segment
            .field(id)
            .ok_or(SnapshotDecodeError::MissingField(id))?;
        if actual.type_tag != expected {
            return Err(SnapshotDecodeError::FieldType {
                field_id: id,
                expected,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn field(segment: &crate::DecodedCanonicalSegment, id: u32) -> Result<&[u8], SnapshotDecodeError> {
    Ok(&segment
        .field(id)
        .ok_or(SnapshotDecodeError::MissingField(id))?
        .payload)
}

fn read_u64(segment: &crate::DecodedCanonicalSegment, id: u32) -> Result<u64, SnapshotDecodeError> {
    let payload = field(segment, id)?;
    Ok(u64::from_le_bytes(payload.try_into().map_err(|_| {
        SnapshotDecodeError::FieldLength {
            field_id: id,
            expected: 8,
            actual: payload.len(),
        }
    })?))
}
