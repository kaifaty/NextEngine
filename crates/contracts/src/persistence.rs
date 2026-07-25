use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    CanonicalDecodeLimits, CanonicalError, CapabilityId, CommandDecodeError, CommandId,
    CommandLedgerHash, CommandLedgerSnapshot, CommandStreamId, ContentHash, IssuerPrincipal,
    RuntimeSnapshot, SchemaId, StateRoot, WorldCommand, content_hash_from_bytes, sha256,
};

pub const SAVE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const REPLAY_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickSettings {
    pub gameplay_hz: u32,
    pub physics_hz: u32,
    pub motor_hz: u32,
}

impl TickSettings {
    pub fn validate(self) -> Result<(), ManifestValidationError> {
        if self.gameplay_hz == 0 || self.physics_hz == 0 || self.motor_hz == 0 {
            return Err(ManifestValidationError::InvalidTickSettings);
        }
        if !self.physics_hz.is_multiple_of(self.gameplay_hz)
            || !self.physics_hz.is_multiple_of(self.motor_hz)
        {
            return Err(ManifestValidationError::InvalidTickSettings);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HashBinding {
    pub binding_id: SchemaId,
    pub content_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaBinding {
    pub schema_id: SchemaId,
    pub schema_version: u32,
    pub content_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandLedgerDescriptor {
    pub stream_id: CommandStreamId,
    pub issuer: IssuerPrincipal,
    pub last_sequence: u64,
    pub command_id: CommandId,
}

impl From<&CommandLedgerSnapshot> for CommandLedgerDescriptor {
    fn from(ledger: &CommandLedgerSnapshot) -> Self {
        Self {
            stream_id: ledger.stream_id,
            issuer: ledger.issuer.clone(),
            last_sequence: ledger.last_sequence,
            command_id: ledger.command_id,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SaveSegmentDescriptor {
    pub owner_id: SchemaId,
    pub schema_id: SchemaId,
    pub segment_id: SchemaId,
    pub schema_version: u32,
    pub byte_length: u64,
    pub content_hash: ContentHash,
}

impl SaveSegmentDescriptor {
    pub fn for_bytes(
        owner_id: SchemaId,
        schema_id: SchemaId,
        segment_id: SchemaId,
        schema_version: u32,
        bytes: &[u8],
    ) -> Result<Self, CanonicalError> {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.state-segment.v1\0");
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(bytes);
        Ok(Self {
            owner_id,
            schema_id,
            segment_id,
            schema_version,
            byte_length: u64::try_from(bytes.len()).map_err(|_| CanonicalError::LengthOverflow)?,
            content_hash: content_hash_from_bytes(sha256(&preimage)),
        })
    }

    #[must_use]
    pub fn matches_bytes(&self, bytes: &[u8]) -> bool {
        let Ok(actual) = Self::for_bytes(
            self.owner_id.clone(),
            self.schema_id.clone(),
            self.segment_id.clone(),
            self.schema_version,
            bytes,
        ) else {
            return false;
        };
        actual.byte_length == self.byte_length && actual.content_hash == self.content_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveCompatibility {
    pub engine_build_hash: ContentHash,
    pub game_build_hash: ContentHash,
    pub project_id: SchemaId,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub mechanics_lock_hash: ContentHash,
    pub tick_settings: TickSettings,
    pub loaded_chunk_revisions: Vec<HashBinding>,
    pub rng_stream_states: Vec<HashBinding>,
    pub physical_bindings: Vec<HashBinding>,
    pub policy_state_schemas: Vec<SchemaBinding>,
    pub plugin_script_bindings: Vec<HashBinding>,
}

impl SaveCompatibility {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        self.tick_settings.validate()?;
        validate_sorted_unique_hash_bindings(&self.loaded_chunk_revisions)?;
        validate_sorted_unique_hash_bindings(&self.rng_stream_states)?;
        validate_sorted_unique_hash_bindings(&self.physical_bindings)?;
        validate_sorted_unique_schema_bindings(&self.policy_state_schemas)?;
        validate_sorted_unique_hash_bindings(&self.plugin_script_bindings)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveManifestV1 {
    pub schema_version: u32,
    pub generation: u64,
    pub world_revision: u64,
    pub compatibility: SaveCompatibility,
    pub command_ledgers: Vec<CommandLedgerDescriptor>,
    pub segments: Vec<SaveSegmentDescriptor>,
}

impl SaveManifestV1 {
    pub fn for_runtime_snapshot(
        generation: u64,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshot,
        snapshot_bytes: &[u8],
    ) -> Result<Self, ManifestValidationError> {
        let mut command_ledgers = snapshot
            .command_ledgers
            .iter()
            .map(CommandLedgerDescriptor::from)
            .collect::<Vec<_>>();
        command_ledgers.sort();
        let segment = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(crate::RUNTIME_SNAPSHOT_OWNER_ID)?,
            SchemaId::new(crate::RUNTIME_SNAPSHOT_SCHEMA_ID)?,
            SchemaId::new(crate::RUNTIME_SNAPSHOT_SEGMENT_ID)?,
            crate::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            snapshot_bytes,
        )?;
        let manifest = Self {
            schema_version: SAVE_MANIFEST_SCHEMA_VERSION,
            generation,
            world_revision: snapshot.authoritative_revision,
            compatibility,
            command_ledgers,
            segments: vec![segment],
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != SAVE_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedSaveVersion(
                self.schema_version,
            ));
        }
        self.compatibility.validate()?;
        if self.command_ledgers.windows(2).any(|pair| {
            (&pair[0].stream_id, &pair[0].issuer) >= (&pair[1].stream_id, &pair[1].issuer)
        }) {
            return Err(ManifestValidationError::CommandLedgersNotStrictlySorted);
        }
        if self.segments.is_empty() {
            return Err(ManifestValidationError::MissingRequiredSegment);
        }
        if self.segments.windows(2).any(|pair| {
            (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                >= (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
        }) {
            return Err(ManifestValidationError::SegmentsNotStrictlySorted);
        }
        Ok(())
    }

    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ManifestCodecError> {
        crate::manifest_jcs::encode_save_manifest(self)
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ManifestCodecError> {
        crate::manifest_jcs::decode_save_manifest(bytes, limits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityGrant {
    pub principal: IssuerPrincipal,
    pub capabilities: Vec<CapabilityId>,
}

impl AuthorityGrant {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.capabilities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ManifestValidationError::CapabilitiesNotStrictlySorted);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCommandRecord {
    pub command_id: CommandId,
    pub canonical_command_bytes: Vec<u8>,
}

impl ReplayCommandRecord {
    pub fn from_command(command: &WorldCommand) -> Result<Self, CanonicalError> {
        Ok(Self {
            command_id: command.compute_command_id()?,
            canonical_command_bytes: command.canonical_bytes()?,
        })
    }

    pub fn decode_command(
        &self,
        limits: CanonicalDecodeLimits,
    ) -> Result<WorldCommand, ManifestValidationError> {
        let command = WorldCommand::from_canonical_bytes(&self.canonical_command_bytes, limits)?;
        if command.compute_command_id()? != self.command_id {
            return Err(ManifestValidationError::ReplayCommandIdMismatch);
        }
        Ok(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTickManifest {
    pub tick: u64,
    pub commands: Vec<ReplayCommandRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayComparePoint {
    pub tick: u64,
    pub state_root: StateRoot,
    pub command_ledger_hash: CommandLedgerHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayManifestV1 {
    pub schema_version: u32,
    pub compatibility: SaveCompatibility,
    pub initial_snapshot_bytes: Vec<u8>,
    pub initial_state_root: StateRoot,
    pub authority: Vec<AuthorityGrant>,
    pub ticks: Vec<ReplayTickManifest>,
    pub compare_points: Vec<ReplayComparePoint>,
}

impl ReplayManifestV1 {
    pub fn validate_and_decode(
        &self,
        limits: CanonicalDecodeLimits,
    ) -> Result<(RuntimeSnapshot, Vec<Vec<WorldCommand>>), ManifestValidationError> {
        if self.schema_version != REPLAY_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedReplayVersion(
                self.schema_version,
            ));
        }
        self.compatibility.validate()?;
        let snapshot = RuntimeSnapshot::from_canonical_bytes(&self.initial_snapshot_bytes, limits)?;
        if self
            .authority
            .windows(2)
            .any(|pair| pair[0].principal >= pair[1].principal)
        {
            return Err(ManifestValidationError::AuthorityNotStrictlySorted);
        }
        for grant in &self.authority {
            grant.validate()?;
        }
        if self.ticks.len() != self.compare_points.len() {
            return Err(ManifestValidationError::ComparePointCountMismatch);
        }

        let mut expected_tick = snapshot.next_tick;
        let mut decoded_ticks = Vec::with_capacity(self.ticks.len());
        for (tick, compare_point) in self.ticks.iter().zip(&self.compare_points) {
            if tick.tick != expected_tick || compare_point.tick != expected_tick {
                return Err(ManifestValidationError::ReplayTickSequenceMismatch);
            }
            let mut commands = Vec::with_capacity(tick.commands.len());
            for record in &tick.commands {
                let command = record.decode_command(limits)?;
                if command.target_tick != tick.tick {
                    return Err(ManifestValidationError::ReplayCommandTickMismatch);
                }
                commands.push(command);
            }
            decoded_ticks.push(commands);
            expected_tick = expected_tick
                .checked_add(1)
                .ok_or(ManifestValidationError::ReplayTickExhausted)?;
        }
        Ok((snapshot, decoded_ticks))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ManifestValidationError {
    Identifier(crate::IdentifierError),
    Canonicalization(CanonicalError),
    Snapshot(crate::SnapshotDecodeError),
    Command(CommandDecodeError),
    InvalidTickSettings,
    HashBindingsNotStrictlySorted,
    SchemaBindingsNotStrictlySorted,
    CommandLedgersNotStrictlySorted,
    SegmentsNotStrictlySorted,
    CapabilitiesNotStrictlySorted,
    AuthorityNotStrictlySorted,
    MissingRequiredSegment,
    UnsupportedSaveVersion(u32),
    UnsupportedReplayVersion(u32),
    ReplayCommandIdMismatch,
    ComparePointCountMismatch,
    ReplayTickSequenceMismatch,
    ReplayCommandTickMismatch,
    ReplayTickExhausted,
}

impl Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "manifest identifier is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "manifest canonicalization failed: {error}")
            }
            Self::Snapshot(error) => write!(formatter, "manifest snapshot is invalid: {error}"),
            Self::Command(error) => write!(formatter, "manifest command is invalid: {error}"),
            Self::InvalidTickSettings => formatter.write_str("manifest tick settings are invalid"),
            Self::HashBindingsNotStrictlySorted => {
                formatter.write_str("manifest hash bindings are not strictly sorted")
            }
            Self::SchemaBindingsNotStrictlySorted => {
                formatter.write_str("manifest schema bindings are not strictly sorted")
            }
            Self::CommandLedgersNotStrictlySorted => {
                formatter.write_str("manifest command ledgers are not strictly sorted")
            }
            Self::SegmentsNotStrictlySorted => {
                formatter.write_str("manifest segments are not strictly sorted")
            }
            Self::CapabilitiesNotStrictlySorted => {
                formatter.write_str("manifest capabilities are not strictly sorted")
            }
            Self::AuthorityNotStrictlySorted => {
                formatter.write_str("manifest authority grants are not strictly sorted")
            }
            Self::MissingRequiredSegment => formatter.write_str("manifest has no state segment"),
            Self::UnsupportedSaveVersion(version) => {
                write!(formatter, "unsupported save manifest version {version}")
            }
            Self::UnsupportedReplayVersion(version) => {
                write!(formatter, "unsupported replay manifest version {version}")
            }
            Self::ReplayCommandIdMismatch => {
                formatter.write_str("replay command id does not match canonical command bytes")
            }
            Self::ComparePointCountMismatch => {
                formatter.write_str("replay tick and compare-point counts differ")
            }
            Self::ReplayTickSequenceMismatch => {
                formatter.write_str("replay ticks or compare points are not contiguous")
            }
            Self::ReplayCommandTickMismatch => {
                formatter.write_str("replay command target tick does not match its tick record")
            }
            Self::ReplayTickExhausted => formatter.write_str("replay tick sequence overflowed"),
        }
    }
}

impl Error for ManifestValidationError {}

impl From<crate::IdentifierError> for ManifestValidationError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<CanonicalError> for ManifestValidationError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::SnapshotDecodeError> for ManifestValidationError {
    fn from(error: crate::SnapshotDecodeError) -> Self {
        Self::Snapshot(error)
    }
}

impl From<CommandDecodeError> for ManifestValidationError {
    fn from(error: CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

fn validate_sorted_unique_hash_bindings(
    bindings: &[HashBinding],
) -> Result<(), ManifestValidationError> {
    if bindings
        .windows(2)
        .any(|pair| pair[0].binding_id >= pair[1].binding_id)
    {
        return Err(ManifestValidationError::HashBindingsNotStrictlySorted);
    }
    Ok(())
}

fn validate_sorted_unique_schema_bindings(
    bindings: &[SchemaBinding],
) -> Result<(), ManifestValidationError> {
    if bindings.windows(2).any(|pair| {
        (&pair[0].schema_id, pair[0].schema_version) >= (&pair[1].schema_id, pair[1].schema_version)
    }) {
        return Err(ManifestValidationError::SchemaBindingsNotStrictlySorted);
    }
    Ok(())
}

pub use crate::manifest_jcs::ManifestCodecError;

#[cfg(test)]
mod tests {
    use crate::{
        CanonicalDecodeLimits, CommandStreamId, IssuerPrincipal, PlayerPrincipalId,
        ReplayCommandRecord, ReplayComparePoint, ReplayManifestV1, ReplayTickManifest,
        RuntimeSnapshot, SchemaId, StateRoot, TickSettings, WorldCommand,
        command_ledger_hash_from_bytes, content_hash_from_bytes,
    };

    use super::{AuthorityGrant, SaveCompatibility};

    fn compatibility() -> SaveCompatibility {
        SaveCompatibility {
            engine_build_hash: content_hash_from_bytes([1; 32]),
            game_build_hash: content_hash_from_bytes([2; 32]),
            project_id: SchemaId::new("nextengine.test-project").expect("valid project"),
            schema_registry_hash: content_hash_from_bytes([3; 32]),
            content_manifest_hash: content_hash_from_bytes([4; 32]),
            mechanics_lock_hash: content_hash_from_bytes([5; 32]),
            tick_settings: TickSettings {
                gameplay_hz: 30,
                physics_hz: 120,
                motor_hz: 60,
            },
            loaded_chunk_revisions: vec![],
            rng_stream_states: vec![],
            physical_bindings: vec![],
            policy_state_schemas: vec![],
            plugin_script_bindings: vec![],
        }
    }

    #[test]
    fn replay_manifest_decodes_all_commands_before_execution() {
        let principal = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
        let command = WorldCommand::noop(
            CommandStreamId::from_bytes([2; 16]),
            principal.clone(),
            0,
            0,
        )
        .expect("canonical command");
        let snapshot = RuntimeSnapshot {
            next_tick: 0,
            committed_event_count: 0,
            authoritative_revision: 0,
            command_ledgers: vec![],
        };
        let manifest = ReplayManifestV1 {
            schema_version: super::REPLAY_MANIFEST_SCHEMA_VERSION,
            compatibility: compatibility(),
            initial_snapshot_bytes: snapshot.canonical_bytes().expect("canonical snapshot"),
            initial_state_root: StateRoot::from_bytes([0; 32]),
            authority: vec![AuthorityGrant {
                principal,
                capabilities: command
                    .capability_claims
                    .iter()
                    .map(|capability| capability.capability_id.clone())
                    .collect(),
            }],
            ticks: vec![ReplayTickManifest {
                tick: 0,
                commands: vec![
                    ReplayCommandRecord::from_command(&command).expect("canonical replay command"),
                ],
            }],
            compare_points: vec![ReplayComparePoint {
                tick: 0,
                state_root: StateRoot::from_bytes([0; 32]),
                command_ledger_hash: command_ledger_hash_from_bytes([0; 32]),
            }],
        };

        let (decoded_snapshot, decoded_ticks) = manifest
            .validate_and_decode(CanonicalDecodeLimits::default())
            .expect("manifest validates");
        assert_eq!(decoded_snapshot, snapshot);
        assert_eq!(decoded_ticks, vec![vec![command]]);
    }
}
