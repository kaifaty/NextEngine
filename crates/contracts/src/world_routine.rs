use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_STRUCT,
    CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes,
};

mod error_impl;

pub const WORLD_ROUTINE_SCHEMA_VERSION: u16 = 1;
pub const WORLD_ROUTINE_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const WORLD_ROUTINE_EVENT_SCHEMA_VERSION: u32 = 1;
pub const WORLD_ROUTINE_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const WORLD_ROUTINE_CATALOG_SCHEMA_ID: &str = "nextengine.content.world-routine-catalog";
pub const WORLD_ROUTINE_CATALOG_SEGMENT_ID: &str = "nextengine.world-routine-catalog.v1";
pub const WORLD_ROUTINE_SNAPSHOT_OWNER_ID: &str = "nextengine.world-services";
pub const WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID: &str = "nextengine.world-routine-snapshot";
pub const WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID: &str = "world-routine";
pub const WORLD_ROUTINE_COMMAND_SCHEMA_ID: &str = "nextengine.command.world-routine";
pub const WORLD_ROUTINE_COMMAND_KIND_ID: &str = "nextengine.command-kind.world-routine";
pub const WORLD_ROUTINE_EVENT_SCHEMA_ID: &str = "nextengine.event.world-routine-activity-changed";
pub const WORLD_ROUTINE_CAPABILITY_ID: &str = "nextengine.capability.world-routine-commit";
pub const WORLD_ROUTINE_SYSTEM_ID: &str = "nextengine.system.world-routine-boundary";
pub const WORLD_ROUTINE_CAPABILITY_SUBJECT_ID: &str =
    "nextengine.capability-subject.world-routine-boundary";
pub const WORLD_ROUTINE_SHARD_PLAN_ID: &str = "nextengine.shard-plan.world-routine-single";
pub const WORLD_ROUTINE_PRIORITY_CLASS: u16 = 250;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WorldRoutineActivityV1 {
    Duty = 1,
    Rest = 2,
}

impl WorldRoutineActivityV1 {
    fn from_tag(tag: u8) -> Result<Self, WorldRoutineContractError> {
        match tag {
            1 => Ok(Self::Duty),
            2 => Ok(Self::Rest),
            value => Err(WorldRoutineContractError::UnknownActivity(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRoutineProfileV1 {
    pub schema_version: u16,
    pub anchor_simulation_tick: u64,
    pub anchor_world_tick: u64,
    pub world_ticks_per_simulation_tick_num: u64,
    pub world_ticks_per_simulation_tick_den: u64,
}

impl WorldRoutineProfileV1 {
    pub fn validate(&self) -> Result<(), WorldRoutineContractError> {
        if self.schema_version != WORLD_ROUTINE_SCHEMA_VERSION
            || self.world_ticks_per_simulation_tick_num == 0
            || self.world_ticks_per_simulation_tick_den == 0
        {
            return Err(WorldRoutineContractError::CalendarProfileInvalid);
        }
        Ok(())
    }

    pub fn world_tick(&self, simulation_tick: u64) -> Result<u64, WorldRoutineContractError> {
        self.validate()?;
        let delta = simulation_tick
            .checked_sub(self.anchor_simulation_tick)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        let scaled = u128::from(delta)
            .checked_mul(u128::from(self.world_ticks_per_simulation_tick_num))
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        let projected = scaled / u128::from(self.world_ticks_per_simulation_tick_den);
        let projected = u64::try_from(projected)
            .map_err(|_| WorldRoutineContractError::CalendarProfileInvalid)?;
        self.anchor_world_tick
            .checked_add(projected)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)
    }

    pub fn due_simulation_tick(
        &self,
        transition_world_tick: u64,
    ) -> Result<u64, WorldRoutineContractError> {
        self.validate()?;
        let world_delta = transition_world_tick
            .checked_sub(self.anchor_world_tick)
            .filter(|delta| *delta > 0)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        let numerator = u128::from(world_delta)
            .checked_mul(u128::from(self.world_ticks_per_simulation_tick_den))
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        let divisor = u128::from(self.world_ticks_per_simulation_tick_num);
        let due_delta = numerator
            .checked_add(divisor - 1)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?
            / divisor;
        let due_delta = u64::try_from(due_delta)
            .map_err(|_| WorldRoutineContractError::CalendarProfileInvalid)?;
        if due_delta == 0 {
            return Err(WorldRoutineContractError::CalendarProfileInvalid);
        }
        let due_tick = self
            .anchor_simulation_tick
            .checked_add(due_delta)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        let previous_tick = due_tick
            .checked_sub(1)
            .ok_or(WorldRoutineContractError::CalendarProfileInvalid)?;
        if self.world_tick(previous_tick)? >= transition_world_tick
            || self.world_tick(due_tick)? < transition_world_tick
        {
            return Err(WorldRoutineContractError::CalendarProfileInvalid);
        }
        Ok(due_tick)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRoutineDefinitionV1 {
    pub schema_version: u16,
    pub subject_id: PersistentId,
    pub initial_activity: WorldRoutineActivityV1,
    pub transition_world_tick: u64,
    pub next_activity: WorldRoutineActivityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRoutineCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub profile: WorldRoutineProfileV1,
    pub routine: WorldRoutineDefinitionV1,
}

impl WorldRoutineCatalogV1 {
    pub fn validate(&self) -> Result<(), WorldRoutineContractError> {
        if self.schema_version != WORLD_ROUTINE_SCHEMA_VERSION
            || self.routine.schema_version != WORLD_ROUTINE_SCHEMA_VERSION
            || self.routine.initial_activity != WorldRoutineActivityV1::Duty
            || self.routine.next_activity != WorldRoutineActivityV1::Rest
        {
            return Err(WorldRoutineContractError::ContentInvalid);
        }
        self.profile.validate()?;
        self.profile
            .due_simulation_tick(self.routine.transition_world_tick)?;
        Ok(())
    }

    pub fn due_simulation_tick(&self) -> Result<u64, WorldRoutineContractError> {
        self.validate()?;
        self.profile
            .due_simulation_tick(self.routine.transition_world_tick)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let profile = encode_struct([
            field_u16(1, self.profile.schema_version),
            field_u64(2, self.profile.anchor_simulation_tick),
            field_u64(3, self.profile.anchor_world_tick),
            field_u64(4, self.profile.world_ticks_per_simulation_tick_num),
            field_u64(5, self.profile.world_ticks_per_simulation_tick_den),
        ])?;
        let routine = encode_struct([
            field_u16(1, self.routine.schema_version),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                self.routine.subject_id.as_bytes().to_vec(),
            ),
            field_u8(3, self.routine.initial_activity as u8),
            field_u64(4, self.routine.transition_world_tick),
            field_u8(5, self.routine.next_activity as u8),
        ])?;
        encode_canonical_segment(
            WORLD_ROUTINE_CATALOG_OWNER_ID,
            WORLD_ROUTINE_CATALOG_SCHEMA_ID,
            WORLD_ROUTINE_CATALOG_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    self.catalog_asset_id.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_STRUCT, profile),
                CanonicalField::new(4, CANONICAL_TYPE_STRUCT, routine),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldRoutineContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            WORLD_ROUTINE_CATALOG_OWNER_ID,
            WORLD_ROUTINE_CATALOG_SCHEMA_ID,
            WORLD_ROUTINE_CATALOG_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_STRUCT),
            ],
        )?;
        let profile = decode_struct(field(&segment, 3)?, limits)?;
        require_fields(
            &profile,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_U64),
            ],
        )?;
        let routine = decode_struct(field(&segment, 4)?, limits)?;
        require_fields(
            &routine,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U8),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 2)?)?),
            profile: WorldRoutineProfileV1 {
                schema_version: read_u16(nested_field(&profile, 1)?)?,
                anchor_simulation_tick: read_u64(nested_field(&profile, 2)?)?,
                anchor_world_tick: read_u64(nested_field(&profile, 3)?)?,
                world_ticks_per_simulation_tick_num: read_u64(nested_field(&profile, 4)?)?,
                world_ticks_per_simulation_tick_den: read_u64(nested_field(&profile, 5)?)?,
            },
            routine: WorldRoutineDefinitionV1 {
                schema_version: read_u16(nested_field(&routine, 1)?)?,
                subject_id: PersistentId::from_bytes(read_exact(nested_field(&routine, 2)?)?),
                initial_activity: WorldRoutineActivityV1::from_tag(read_u8(nested_field(
                    &routine, 3,
                )?)?)?,
                transition_world_tick: read_u64(nested_field(&routine, 4)?)?,
                next_activity: WorldRoutineActivityV1::from_tag(read_u8(nested_field(
                    &routine, 5,
                )?)?)?,
            },
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(WorldRoutineContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(&self) -> Result<ContentHash, WorldRoutineContractError> {
        self.validate()?;
        let bytes = self.canonical_bytes()?;
        Ok(domain_hash(WORLD_ROUTINE_CATALOG_SEGMENT_ID, &bytes))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRoutineInteractionBindingV1 {
    pub interaction_id: SchemaId,
    pub subject_id: PersistentId,
    pub required_activity: WorldRoutineActivityV1,
}

impl WorldRoutineInteractionBindingV1 {
    pub fn validate_against(
        &self,
        catalog: &WorldRoutineCatalogV1,
    ) -> Result<(), WorldRoutineContractError> {
        catalog.validate()?;
        if self.subject_id != catalog.routine.subject_id
            || self.required_activity != WorldRoutineActivityV1::Duty
        {
            return Err(WorldRoutineContractError::BindingInvalid);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRoutineRecordV1 {
    pub subject_id: PersistentId,
    pub record_revision: u64,
    pub catalog_asset_id: AssetId,
    pub catalog_revision: ContentHash,
    pub current_activity: WorldRoutineActivityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRoutineSnapshotV1 {
    pub schema_version: u16,
    pub record: WorldRoutineRecordV1,
}

impl WorldRoutineSnapshotV1 {
    pub fn initial(catalog: &WorldRoutineCatalogV1) -> Result<Self, WorldRoutineContractError> {
        catalog.validate()?;
        Ok(Self {
            schema_version: WORLD_ROUTINE_SCHEMA_VERSION,
            record: WorldRoutineRecordV1 {
                subject_id: catalog.routine.subject_id,
                record_revision: 0,
                catalog_asset_id: catalog.catalog_asset_id,
                catalog_revision: catalog.revision()?,
                current_activity: WorldRoutineActivityV1::Duty,
            },
        })
    }

    pub fn validate_against(
        &self,
        catalog: &WorldRoutineCatalogV1,
        next_simulation_tick: u64,
    ) -> Result<(), WorldRoutineContractError> {
        catalog.validate()?;
        if self.schema_version != WORLD_ROUTINE_SCHEMA_VERSION
            || self.record.subject_id != catalog.routine.subject_id
            || self.record.catalog_asset_id != catalog.catalog_asset_id
            || self.record.catalog_revision != catalog.revision()?
            || next_simulation_tick < catalog.profile.anchor_simulation_tick
        {
            return Err(WorldRoutineContractError::SnapshotClosureInvalid);
        }
        let boundary_complete = next_simulation_tick
            .checked_sub(1)
            .filter(|tick| *tick >= catalog.profile.anchor_simulation_tick)
            .map(|last_complete| catalog.profile.world_tick(last_complete))
            .transpose()?
            .is_some_and(|world_tick| world_tick >= catalog.routine.transition_world_tick);
        let (expected_activity, expected_revision) = if boundary_complete {
            (WorldRoutineActivityV1::Rest, 1)
        } else {
            (WorldRoutineActivityV1::Duty, 0)
        };
        if self.record.current_activity != expected_activity
            || self.record.record_revision != expected_revision
        {
            return Err(WorldRoutineContractError::SnapshotClosureInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let record = encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_ID128,
                self.record.subject_id.as_bytes().to_vec(),
            ),
            field_u64(2, self.record.record_revision),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_ID128,
                self.record.catalog_asset_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_HASH256,
                self.record.catalog_revision.as_bytes().to_vec(),
            ),
            field_u8(5, self.record.current_activity as u8),
        ])?;
        encode_canonical_segment(
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, record),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldRoutineContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_STRUCT)],
        )?;
        let record = decode_struct(field(&segment, 2)?, limits)?;
        require_fields(
            &record,
            &[
                (1, CANONICAL_TYPE_ID128),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            record: WorldRoutineRecordV1 {
                subject_id: PersistentId::from_bytes(read_exact(nested_field(&record, 1)?)?),
                record_revision: read_u64(nested_field(&record, 2)?)?,
                catalog_asset_id: AssetId::from_bytes(read_exact(nested_field(&record, 3)?)?),
                catalog_revision: ContentHash::from_bytes(read_exact(nested_field(&record, 4)?)?),
                current_activity: WorldRoutineActivityV1::from_tag(read_u8(nested_field(
                    &record, 5,
                )?)?)?,
            },
        };
        if value.schema_version != WORLD_ROUTINE_SCHEMA_VERSION {
            return Err(WorldRoutineContractError::UnsupportedVersion(
                value.schema_version,
            ));
        }
        if value.canonical_bytes()? != bytes {
            return Err(WorldRoutineContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorldRoutineCommandV1 {
    CommitActivityBoundary {
        subject_id: PersistentId,
        expected_record_revision: u64,
        catalog_asset_id: AssetId,
        catalog_revision: ContentHash,
        boundary_world_tick: u64,
        previous_activity: WorldRoutineActivityV1,
        current_activity: WorldRoutineActivityV1,
    },
}

impl WorldRoutineCommandV1 {
    pub fn commit_boundary(
        snapshot: &WorldRoutineSnapshotV1,
        catalog: &WorldRoutineCatalogV1,
    ) -> Result<Self, WorldRoutineContractError> {
        catalog.validate()?;
        if snapshot.record.subject_id != catalog.routine.subject_id
            || snapshot.record.catalog_asset_id != catalog.catalog_asset_id
            || snapshot.record.catalog_revision != catalog.revision()?
            || snapshot.record.current_activity != WorldRoutineActivityV1::Duty
        {
            return Err(WorldRoutineContractError::SnapshotClosureInvalid);
        }
        Ok(Self::CommitActivityBoundary {
            subject_id: snapshot.record.subject_id,
            expected_record_revision: snapshot.record.record_revision,
            catalog_asset_id: snapshot.record.catalog_asset_id,
            catalog_revision: snapshot.record.catalog_revision,
            boundary_world_tick: catalog.routine.transition_world_tick,
            previous_activity: WorldRoutineActivityV1::Duty,
            current_activity: WorldRoutineActivityV1::Rest,
        })
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::CommitActivityBoundary {
                subject_id,
                expected_record_revision,
                catalog_asset_id,
                catalog_revision,
                boundary_world_tick,
                previous_activity,
                current_activity,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(subject_id.as_bytes());
                bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
                bytes.extend_from_slice(catalog_asset_id.as_bytes());
                bytes.extend_from_slice(catalog_revision.as_bytes());
                bytes.extend_from_slice(&boundary_world_tick.to_le_bytes());
                bytes.push(*previous_activity as u8);
                bytes.push(*current_activity as u8);
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        _limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldRoutineContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let value = match cursor.read_u8()? {
            1 => Self::CommitActivityBoundary {
                subject_id: PersistentId::from_bytes(
                    cursor
                        .read_exact(16)?
                        .try_into()
                        .map_err(|_| WorldRoutineContractError::ContentInvalid)?,
                ),
                expected_record_revision: cursor.read_u64()?,
                catalog_asset_id: AssetId::from_bytes(
                    cursor
                        .read_exact(16)?
                        .try_into()
                        .map_err(|_| WorldRoutineContractError::ContentInvalid)?,
                ),
                catalog_revision: ContentHash::from_bytes(
                    cursor
                        .read_exact(32)?
                        .try_into()
                        .map_err(|_| WorldRoutineContractError::ContentInvalid)?,
                ),
                boundary_world_tick: cursor.read_u64()?,
                previous_activity: WorldRoutineActivityV1::from_tag(cursor.read_u8()?)?,
                current_activity: WorldRoutineActivityV1::from_tag(cursor.read_u8()?)?,
            },
            tag => return Err(WorldRoutineContractError::UnknownCommand(tag)),
        };
        cursor.finish()?;
        value.validate_shape()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(WorldRoutineContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn validate_shape(&self) -> Result<(), WorldRoutineContractError> {
        match self {
            Self::CommitActivityBoundary {
                previous_activity,
                current_activity,
                ..
            } if *previous_activity == WorldRoutineActivityV1::Duty
                && *current_activity == WorldRoutineActivityV1::Rest =>
            {
                Ok(())
            }
            _ => Err(WorldRoutineContractError::ContentInvalid),
        }
    }

    pub fn owner_delta_bytes(&self) -> Result<Vec<u8>, WorldRoutineContractError> {
        self.validate_shape()?;
        let mut bytes = b"nextengine.world-routine-owner-write-set.v1\0".to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        match self {
            Self::CommitActivityBoundary {
                subject_id,
                expected_record_revision,
                catalog_asset_id,
                catalog_revision,
                boundary_world_tick,
                previous_activity,
                current_activity,
            } => {
                let after = expected_record_revision
                    .checked_add(1)
                    .ok_or(WorldRoutineContractError::RevisionExhausted)?;
                bytes.extend_from_slice(subject_id.as_bytes());
                bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
                bytes.extend_from_slice(&after.to_le_bytes());
                bytes.extend_from_slice(catalog_asset_id.as_bytes());
                bytes.extend_from_slice(catalog_revision.as_bytes());
                bytes.push(*previous_activity as u8);
                bytes.push(*current_activity as u8);
                bytes.extend_from_slice(&boundary_world_tick.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldRoutineActivityChangedV1 {
    pub subject_id: PersistentId,
    pub previous_activity: WorldRoutineActivityV1,
    pub current_activity: WorldRoutineActivityV1,
    pub boundary_world_tick: u64,
    pub record_revision: u64,
}

impl WorldRoutineActivityChangedV1 {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, WorldRoutineContractError> {
        if self.previous_activity != WorldRoutineActivityV1::Duty
            || self.current_activity != WorldRoutineActivityV1::Rest
            || self.record_revision == 0
        {
            return Err(WorldRoutineContractError::ContentInvalid);
        }
        let mut bytes = Vec::with_capacity(34);
        bytes.extend_from_slice(self.subject_id.as_bytes());
        bytes.push(self.previous_activity as u8);
        bytes.push(self.current_activity as u8);
        bytes.extend_from_slice(&self.boundary_world_tick.to_le_bytes());
        bytes.extend_from_slice(&self.record_revision.to_le_bytes());
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldRoutineActivityConditionV1 {
    pub subject_id: PersistentId,
    pub required_activity: WorldRoutineActivityV1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum InteractionAvailabilityCodeV1 {
    Available = 0,
    WorldRoutineActivityUnavailable = 1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InteractionRoutineRevisionBindingV1 {
    pub subject_id: PersistentId,
    pub routine_record_revision: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InteractionAvailabilityV1 {
    pub schema_version: u16,
    pub interaction_id: SchemaId,
    pub interaction_definition_hash_v2: ContentHash,
    pub routine_binding_or_none: Option<InteractionRoutineRevisionBindingV1>,
    pub code: InteractionAvailabilityCodeV1,
}

impl InteractionAvailabilityV1 {
    pub fn validate(&self) -> Result<(), WorldRoutineContractError> {
        if self.schema_version != WORLD_ROUTINE_SCHEMA_VERSION
            || matches!(
                (self.code, self.routine_binding_or_none),
                (
                    InteractionAvailabilityCodeV1::WorldRoutineActivityUnavailable,
                    None
                )
            )
        {
            return Err(WorldRoutineContractError::AvailabilityInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, WorldRoutineContractError> {
        self.validate()?;
        let binding = match self.routine_binding_or_none {
            None => vec![0],
            Some(binding) => {
                let payload = encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_ID128,
                        binding.subject_id.as_bytes().to_vec(),
                    ),
                    field_u64(2, binding.routine_record_revision),
                ])?;
                let mut value = vec![1];
                value.extend_from_slice(&nested(CANONICAL_TYPE_STRUCT, &payload)?);
                value
            }
        };
        Ok(encode_canonical_segment(
            "nextengine.mechanics",
            "nextengine.interaction-availability",
            "v1",
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.interaction_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_HASH256,
                    self.interaction_definition_hash_v2.as_bytes().to_vec(),
                ),
                CanonicalField::new(4, CANONICAL_TYPE_OPTIONAL, binding),
                field_u8(5, self.code as u8),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldRoutineContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            "nextengine.mechanics",
            "nextengine.interaction-availability",
            "v1",
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_HASH256),
                (4, CANONICAL_TYPE_OPTIONAL),
                (5, CANONICAL_TYPE_U8),
            ],
        )?;
        let routine_binding_or_none = {
            let mut cursor = CanonicalCursor::new(field(&segment, 4)?);
            let value = match cursor.read_u8()? {
                0 => None,
                1 => {
                    if cursor.read_u8()? != CANONICAL_TYPE_STRUCT {
                        return Err(WorldRoutineContractError::FieldType);
                    }
                    let length = usize::try_from(cursor.read_u64()?)
                        .map_err(|_| WorldRoutineContractError::FieldLength)?;
                    if length > limits.max_field_payload_bytes {
                        return Err(WorldRoutineContractError::FieldLength);
                    }
                    let fields = decode_struct(cursor.read_exact(length)?, limits)?;
                    require_fields(
                        &fields,
                        &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_U64)],
                    )?;
                    Some(InteractionRoutineRevisionBindingV1 {
                        subject_id: PersistentId::from_bytes(read_exact(nested_field(
                            &fields, 1,
                        )?)?),
                        routine_record_revision: read_u64(nested_field(&fields, 2)?)?,
                    })
                }
                _ => return Err(WorldRoutineContractError::FieldType),
            };
            cursor.finish()?;
            value
        };
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            interaction_id: SchemaId::new(
                std::str::from_utf8(field(&segment, 2)?)
                    .map_err(|_| WorldRoutineContractError::ContentInvalid)?,
            )?,
            interaction_definition_hash_v2: ContentHash::from_bytes(read_exact(field(
                &segment, 3,
            )?)?),
            routine_binding_or_none,
            code: match read_u8(field(&segment, 5)?)? {
                0 => InteractionAvailabilityCodeV1::Available,
                1 => InteractionAvailabilityCodeV1::WorldRoutineActivityUnavailable,
                _ => return Err(WorldRoutineContractError::AvailabilityInvalid),
            },
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(WorldRoutineContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn canonical_hash(&self) -> Result<ContentHash, WorldRoutineContractError> {
        let bytes = self.canonical_bytes()?;
        let mut preimage = b"nextengine.interaction-availability.v1\0".to_vec();
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
        Ok(content_hash_from_bytes(sha256(&preimage)))
    }
}

pub fn interaction_availability_batch_hash(
    values: &[InteractionAvailabilityV1],
) -> Result<ContentHash, WorldRoutineContractError> {
    let mut preimage = b"nextengine.interaction-availability-batch.v1\0".to_vec();
    preimage.extend_from_slice(
        &u32::try_from(values.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for value in values {
        let bytes = value.canonical_bytes()?;
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&bytes);
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldRoutineContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    UnsupportedVersion(u16),
    UnknownActivity(u8),
    UnknownCommand(u8),
    CalendarProfileInvalid,
    ContentInvalid,
    BindingInvalid,
    SnapshotClosureInvalid,
    AvailabilityInvalid,
    RevisionExhausted,
    WrongEnvelope,
    MissingField(u32),
    UnknownField(u32),
    FieldType,
    FieldLength,
    NonCanonicalEncoding,
}

fn domain_hash(domain: &str, bytes: &[u8]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("in-memory world routine bytes fit u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    content_hash_from_bytes(sha256(&preimage))
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, WorldRoutineContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(WorldRoutineContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

fn field(segment: &DecodedCanonicalSegment, id: u32) -> Result<&[u8], WorldRoutineContractError> {
    Ok(&segment
        .field(id)
        .ok_or(WorldRoutineContractError::MissingField(id))?
        .payload)
}

fn nested_field(fields: &[CanonicalField], id: u32) -> Result<&[u8], WorldRoutineContractError> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(WorldRoutineContractError::MissingField(id))?
        .payload)
}

fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), WorldRoutineContractError> {
    for field in actual {
        let Some((_, tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(WorldRoutineContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *tag {
            return Err(WorldRoutineContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(WorldRoutineContractError::MissingField(*id));
        }
    }
    Ok(())
}

fn encode_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if fields
        .windows(2)
        .any(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(0));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(&field.field_id.to_le_bytes());
        bytes.push(field.type_tag);
        bytes.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&field.payload);
    }
    Ok(bytes)
}

fn decode_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, WorldRoutineContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let id = cursor.read_u32()?;
        if previous.is_some_and(|previous| id <= previous) {
            return Err(WorldRoutineContractError::NonCanonicalEncoding);
        }
        previous = Some(id);
        let tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| WorldRoutineContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(WorldRoutineContractError::FieldLength);
        }
        fields.push(CanonicalField::new(
            id,
            tag,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(fields)
}

fn nested(tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn field_u8(id: u32, value: u8) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U8, vec![value])
}

fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

fn field_u64(id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

fn read_exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], WorldRoutineContractError> {
    bytes
        .try_into()
        .map_err(|_| WorldRoutineContractError::FieldLength)
}

fn read_u8(bytes: &[u8]) -> Result<u8, WorldRoutineContractError> {
    Ok(read_exact::<1>(bytes)?[0])
}

fn read_u16(bytes: &[u8]) -> Result<u16, WorldRoutineContractError> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

fn read_u64(bytes: &[u8]) -> Result<u64, WorldRoutineContractError> {
    Ok(u64::from_le_bytes(read_exact(bytes)?))
}

#[cfg(test)]
mod tests;
