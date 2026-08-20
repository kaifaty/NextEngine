use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC,
    CanonicalCursor, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment, sha256,
};
use crate::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use std::collections::{BTreeMap, BTreeSet};

pub const WORLD_POPULATION_SCHEMA_VERSION: u16 = 1;
pub const WORLD_POPULATION_COUNT_V1: usize = 100;
pub const WORLD_POPULATION_ACTIVE_COUNT_V1: usize = 16;
pub const WORLD_POPULATION_NEAR_COUNT_V1: usize = 32;
pub const WORLD_POPULATION_BACKGROUND_COUNT_V1: usize = 52;

pub const WORLD_POPULATION_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const WORLD_POPULATION_CATALOG_SCHEMA_ID: &str = "nextengine.content.world-population-catalog";
pub const WORLD_POPULATION_CATALOG_SEGMENT_ID: &str = "nextengine.world-population-catalog.v1";
pub const WORLD_NAVIGATION_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const WORLD_NAVIGATION_CATALOG_SCHEMA_ID: &str = "nextengine.content.world-navigation-catalog";
pub const WORLD_NAVIGATION_CATALOG_SEGMENT_ID: &str = "nextengine.world-navigation-catalog.v1";

pub const WORLD_POPULATION_SNAPSHOT_OWNER_ID: &str = "nextengine.world-services";
pub const WORLD_POPULATION_SNAPSHOT_SCHEMA_ID: &str = "nextengine.world-population-snapshot";
pub const WORLD_POPULATION_SNAPSHOT_SEGMENT_ID: &str = "world-population";
pub const WORLD_POPULATION_COMMAND_SCHEMA_ID: &str = "nextengine.command.world-population";
pub const WORLD_POPULATION_COMMAND_KIND_ID: &str = "nextengine.command-kind.world-population";
pub const WORLD_POPULATION_EVENT_SCHEMA_ID: &str = "nextengine.event.world-population-changed";
pub const WORLD_POPULATION_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const WORLD_POPULATION_CAPABILITY_ID: &str = "nextengine.capability.world-population-commit";
pub const WORLD_POPULATION_SYSTEM_ID: &str = "nextengine.system.world-population-boundary";
pub const WORLD_POPULATION_CAPABILITY_SUBJECT_ID: &str =
    "nextengine.capability-subject.world-population-boundary";
pub const WORLD_POPULATION_SHARD_PLAN_ID: &str = "nextengine.shard-plan.world-population-single";
pub const WORLD_POPULATION_PRIORITY_CLASS: u16 = 260;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PopulationTierV1 {
    Dormant = 1,
    Abstract = 2,
    Simulated = 3,
    Active = 4,
}

impl PopulationTierV1 {
    fn from_tag(tag: u8) -> Result<Self, WorldPopulationContractError> {
        match tag {
            1 => Ok(Self::Dormant),
            2 => Ok(Self::Abstract),
            3 => Ok(Self::Simulated),
            4 => Ok(Self::Active),
            value => Err(WorldPopulationContractError::UnknownTier(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PopulationCadenceClassV1 {
    Active = 1,
    Near = 2,
    Background = 3,
}

impl PopulationCadenceClassV1 {
    #[must_use]
    pub const fn period_ticks(self) -> u64 {
        match self {
            Self::Active => 3,
            Self::Near => 15,
            Self::Background => 60,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, WorldPopulationContractError> {
        match tag {
            1 => Ok(Self::Active),
            2 => Ok(Self::Near),
            3 => Ok(Self::Background),
            value => Err(WorldPopulationContractError::UnknownCadence(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NavigationCapabilityV1 {
    AbstractTransfer = 1,
}

impl NavigationCapabilityV1 {
    fn from_tag(tag: u8) -> Result<Self, WorldPopulationContractError> {
        match tag {
            1 => Ok(Self::AbstractTransfer),
            value => Err(WorldPopulationContractError::UnknownNavigationCapability(
                value,
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldNavigationTileV1 {
    pub tile_id: SchemaId,
    pub region_id: SchemaId,
    pub tile_revision: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldNavigationNodeV1 {
    pub node_id: SchemaId,
    pub tile_id: SchemaId,
    pub chunk_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldNavigationEdgeV1 {
    pub node_low: SchemaId,
    pub node_high: SchemaId,
    pub cost: u32,
    pub capability: NavigationCapabilityV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldNavigationCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub topology_revision: u64,
    pub tiles: Vec<WorldNavigationTileV1>,
    pub nodes: Vec<WorldNavigationNodeV1>,
    pub edges: Vec<WorldNavigationEdgeV1>,
}

mod navigation_impl;
mod snapshot_validation;

pub use navigation_impl::navigation_tile_revision;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldPopulationDefinitionV1 {
    pub subject_id: PersistentId,
    pub home_node_id: SchemaId,
    pub initial_node_id: SchemaId,
    pub navigation_goal_node_id: SchemaId,
    pub cadence_class: PopulationCadenceClassV1,
    pub cadence_phase: u64,
    pub initial_tier: PopulationTierV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPopulationCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub navigation_catalog_asset_id: AssetId,
    pub navigation_catalog_revision: ContentHash,
    pub courier_subject_id: PersistentId,
    pub courier_transition_start_tick: u64,
    pub records: Vec<WorldPopulationDefinitionV1>,
}

impl WorldPopulationCatalogV1 {
    pub fn validate_against_navigation(
        &self,
        navigation: &WorldNavigationCatalogV1,
    ) -> Result<(), WorldPopulationContractError> {
        navigation.validate()?;
        if self.schema_version != WORLD_POPULATION_SCHEMA_VERSION
            || self.records.len() != WORLD_POPULATION_COUNT_V1
            || self.courier_transition_start_tick == 0
            || !strictly_sorted(&self.records)
            || self.navigation_catalog_asset_id != navigation.catalog_asset_id
            || self.navigation_catalog_revision != navigation.revision()?
        {
            return Err(WorldPopulationContractError::PopulationContentInvalid);
        }
        let mut active = 0_usize;
        let mut near = 0_usize;
        let mut background = 0_usize;
        let mut active_tier = 0_usize;
        let mut simulated_tier = 0_usize;
        let mut abstract_tier = 0_usize;
        let mut dormant_tier = 0_usize;
        for record in &self.records {
            if navigation.node(&record.home_node_id).is_none()
                || navigation.node(&record.initial_node_id).is_none()
                || navigation.node(&record.navigation_goal_node_id).is_none()
                || record.cadence_phase
                    != population_cadence_phase(record.subject_id, record.cadence_class)
            {
                return Err(WorldPopulationContractError::PopulationContentInvalid);
            }
            match record.cadence_class {
                PopulationCadenceClassV1::Active => active += 1,
                PopulationCadenceClassV1::Near => near += 1,
                PopulationCadenceClassV1::Background => background += 1,
            }
            match record.initial_tier {
                PopulationTierV1::Dormant => dormant_tier += 1,
                PopulationTierV1::Abstract => abstract_tier += 1,
                PopulationTierV1::Simulated => simulated_tier += 1,
                PopulationTierV1::Active => active_tier += 1,
            }
        }
        let courier = self
            .definition(self.courier_subject_id)
            .ok_or(WorldPopulationContractError::PopulationContentInvalid)?;
        if active != WORLD_POPULATION_ACTIVE_COUNT_V1
            || near != WORLD_POPULATION_NEAR_COUNT_V1
            || background != WORLD_POPULATION_BACKGROUND_COUNT_V1
            || active_tier != WORLD_POPULATION_ACTIVE_COUNT_V1
            || simulated_tier != WORLD_POPULATION_NEAR_COUNT_V1
            || abstract_tier != WORLD_POPULATION_BACKGROUND_COUNT_V1 - 1
            || dormant_tier != 1
            || courier.cadence_class != PopulationCadenceClassV1::Background
            || courier.initial_tier != PopulationTierV1::Dormant
            || courier.initial_node_id == courier.navigation_goal_node_id
            || navigation.region_for_node(&courier.initial_node_id)
                == navigation.region_for_node(&courier.navigation_goal_node_id)
        {
            return Err(WorldPopulationContractError::PopulationContentInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let records = encode_sequence(
            self.records
                .iter()
                .map(|record| {
                    encode_struct([
                        field_persistent_id(1, record.subject_id),
                        field_text(2, &record.home_node_id),
                        field_text(3, &record.initial_node_id),
                        field_text(4, &record.navigation_goal_node_id),
                        field_u8(5, record.cadence_class as u8),
                        field_u64(6, record.cadence_phase),
                        field_u8(7, record.initial_tier as u8),
                    ])
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            WORLD_POPULATION_CATALOG_OWNER_ID,
            WORLD_POPULATION_CATALOG_SCHEMA_ID,
            WORLD_POPULATION_CATALOG_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.catalog_asset_id),
                field_id(3, self.navigation_catalog_asset_id),
                field_hash(4, self.navigation_catalog_revision),
                field_persistent_id(5, self.courier_subject_id),
                field_u64(6, self.courier_transition_start_tick),
                CanonicalField::new(7, CANONICAL_TYPE_SEQUENCE, records),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldPopulationContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            WORLD_POPULATION_CATALOG_OWNER_ID,
            WORLD_POPULATION_CATALOG_SCHEMA_ID,
            WORLD_POPULATION_CATALOG_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let records = decode_sequence(field(&segment, 7)?, limits)?
            .into_iter()
            .map(|payload| {
                let fields = decode_struct(&payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_ID128),
                        (2, CANONICAL_TYPE_UTF8_NFC),
                        (3, CANONICAL_TYPE_UTF8_NFC),
                        (4, CANONICAL_TYPE_UTF8_NFC),
                        (5, CANONICAL_TYPE_U8),
                        (6, CANONICAL_TYPE_U64),
                        (7, CANONICAL_TYPE_U8),
                    ],
                )?;
                Ok(WorldPopulationDefinitionV1 {
                    subject_id: PersistentId::from_bytes(read_exact(nested_field(&fields, 1)?)?),
                    home_node_id: read_schema_id(nested_field(&fields, 2)?)?,
                    initial_node_id: read_schema_id(nested_field(&fields, 3)?)?,
                    navigation_goal_node_id: read_schema_id(nested_field(&fields, 4)?)?,
                    cadence_class: PopulationCadenceClassV1::from_tag(read_u8(nested_field(
                        &fields, 5,
                    )?)?)?,
                    cadence_phase: read_u64(nested_field(&fields, 6)?)?,
                    initial_tier: PopulationTierV1::from_tag(read_u8(nested_field(&fields, 7)?)?)?,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 2)?)?),
            navigation_catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 3)?)?),
            navigation_catalog_revision: read_hash(field(&segment, 4)?)?,
            courier_subject_id: PersistentId::from_bytes(read_exact(field(&segment, 5)?)?),
            courier_transition_start_tick: read_u64(field(&segment, 6)?)?,
            records,
        };
        if value.schema_version != WORLD_POPULATION_SCHEMA_VERSION {
            return Err(WorldPopulationContractError::UnsupportedVersion(
                value.schema_version,
            ));
        }
        if value.canonical_bytes()? != bytes {
            return Err(WorldPopulationContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(
        &self,
        navigation: &WorldNavigationCatalogV1,
    ) -> Result<ContentHash, WorldPopulationContractError> {
        self.validate_against_navigation(navigation)?;
        Ok(domain_hash(
            WORLD_POPULATION_CATALOG_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    pub fn residency_policy_revision(
        &self,
        navigation: &WorldNavigationCatalogV1,
    ) -> Result<ContentHash, WorldPopulationContractError> {
        self.validate_against_navigation(navigation)?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(WORLD_POPULATION_ACTIVE_COUNT_V1 as u32).to_le_bytes());
        bytes.extend_from_slice(&(WORLD_POPULATION_NEAR_COUNT_V1 as u32).to_le_bytes());
        bytes.extend_from_slice(&(WORLD_POPULATION_BACKGROUND_COUNT_V1 as u32).to_le_bytes());
        bytes.extend_from_slice(
            &PopulationCadenceClassV1::Active
                .period_ticks()
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&PopulationCadenceClassV1::Near.period_ticks().to_le_bytes());
        bytes.extend_from_slice(
            &PopulationCadenceClassV1::Background
                .period_ticks()
                .to_le_bytes(),
        );
        bytes.extend_from_slice(self.courier_subject_id.as_bytes());
        bytes.extend_from_slice(&self.courier_transition_start_tick.to_le_bytes());
        bytes.extend_from_slice(self.navigation_catalog_revision.as_bytes());
        Ok(domain_hash(
            "nextengine.population-residency-policy.v1",
            &bytes,
        ))
    }

    #[must_use]
    pub fn definition(&self, subject_id: PersistentId) -> Option<&WorldPopulationDefinitionV1> {
        self.records
            .binary_search_by_key(&subject_id, |record| record.subject_id)
            .ok()
            .and_then(|index| self.records.get(index))
    }
}

#[must_use]
pub fn population_cadence_phase(
    subject_id: PersistentId,
    cadence_class: PopulationCadenceClassV1,
) -> u64 {
    let mut preimage = b"nextengine.npc-cadence-phase.v1\0".to_vec();
    preimage.extend_from_slice(subject_id.as_bytes());
    let digest = sha256(&preimage);
    u64::from_le_bytes(
        digest[..8]
            .try_into()
            .expect("SHA-256 prefix is exactly eight bytes"),
    ) % cadence_class.period_ticks()
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NavigationQueryV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub graph_revision: ContentHash,
    pub start_node_id: SchemaId,
    pub goal_node_id: SchemaId,
    pub capability: NavigationCapabilityV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NavigationRoutePlanV1 {
    pub schema_version: u16,
    pub query: NavigationQueryV1,
    pub ordered_node_ids: Vec<SchemaId>,
    pub total_cost: u64,
    pub plan_hash: ContentHash,
}

impl NavigationRoutePlanV1 {
    pub fn new(
        query: NavigationQueryV1,
        ordered_node_ids: Vec<SchemaId>,
        total_cost: u64,
    ) -> Result<Self, WorldPopulationContractError> {
        let mut value = Self {
            schema_version: WORLD_POPULATION_SCHEMA_VERSION,
            query,
            ordered_node_ids,
            total_cost,
            plan_hash: ContentHash::default(),
        };
        value.plan_hash = value.compute_hash()?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), WorldPopulationContractError> {
        if self.schema_version != WORLD_POPULATION_SCHEMA_VERSION
            || self.query.schema_version != WORLD_POPULATION_SCHEMA_VERSION
            || self.ordered_node_ids.is_empty()
            || self.ordered_node_ids.first() != Some(&self.query.start_node_id)
            || self.ordered_node_ids.last() != Some(&self.query.goal_node_id)
            || self
                .ordered_node_ids
                .windows(2)
                .any(|pair| pair[0] == pair[1])
            || self.plan_hash != self.compute_hash()?
        {
            return Err(WorldPopulationContractError::RoutePlanInvalid);
        }
        Ok(())
    }

    fn compute_hash(&self) -> Result<ContentHash, WorldPopulationContractError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.schema_version.to_le_bytes());
        bytes.extend_from_slice(&self.query.schema_version.to_le_bytes());
        bytes.extend_from_slice(self.query.catalog_asset_id.as_bytes());
        bytes.extend_from_slice(self.query.graph_revision.as_bytes());
        append_text(&mut bytes, self.query.start_node_id.as_str())?;
        append_text(&mut bytes, self.query.goal_node_id.as_str())?;
        bytes.push(self.query.capability as u8);
        bytes.extend_from_slice(
            &u32::try_from(self.ordered_node_ids.len())
                .map_err(|_| WorldPopulationContractError::RoutePlanInvalid)?
                .to_le_bytes(),
        );
        for node_id in &self.ordered_node_ids {
            append_text(&mut bytes, node_id.as_str())?;
        }
        bytes.extend_from_slice(&self.total_cost.to_le_bytes());
        Ok(domain_hash("nextengine.navigation-route-plan.v1", &bytes))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldPopulationRecordV1 {
    pub subject_id: PersistentId,
    pub record_revision: u64,
    pub home_region_id: SchemaId,
    pub home_node_id: SchemaId,
    pub current_region_id: SchemaId,
    pub current_node_id: SchemaId,
    pub tier: PopulationTierV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPopulationSnapshotV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub catalog_revision: ContentHash,
    pub navigation_catalog_asset_id: AssetId,
    pub navigation_catalog_revision: ContentHash,
    pub records: Vec<WorldPopulationRecordV1>,
}

impl WorldPopulationSnapshotV1 {
    pub fn initial(
        catalog: &WorldPopulationCatalogV1,
        navigation: &WorldNavigationCatalogV1,
    ) -> Result<Self, WorldPopulationContractError> {
        catalog.validate_against_navigation(navigation)?;
        let records = catalog
            .records
            .iter()
            .map(|definition| {
                let home_region_id = navigation
                    .region_for_node(&definition.home_node_id)
                    .ok_or(WorldPopulationContractError::PopulationContentInvalid)?
                    .clone();
                let current_region_id = navigation
                    .region_for_node(&definition.initial_node_id)
                    .ok_or(WorldPopulationContractError::PopulationContentInvalid)?
                    .clone();
                Ok(WorldPopulationRecordV1 {
                    subject_id: definition.subject_id,
                    record_revision: 0,
                    home_region_id,
                    home_node_id: definition.home_node_id.clone(),
                    current_region_id,
                    current_node_id: definition.initial_node_id.clone(),
                    tier: definition.initial_tier,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let value = Self {
            schema_version: WORLD_POPULATION_SCHEMA_VERSION,
            catalog_asset_id: catalog.catalog_asset_id,
            catalog_revision: catalog.revision(navigation)?,
            navigation_catalog_asset_id: navigation.catalog_asset_id,
            navigation_catalog_revision: navigation.revision()?,
            records,
        };
        value.validate_against(catalog, navigation, 0)?;
        Ok(value)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let records = encode_sequence(
            self.records
                .iter()
                .map(|record| {
                    encode_struct([
                        field_persistent_id(1, record.subject_id),
                        field_u64(2, record.record_revision),
                        field_text(3, &record.home_region_id),
                        field_text(4, &record.home_node_id),
                        field_text(5, &record.current_region_id),
                        field_text(6, &record.current_node_id),
                        field_u8(7, record.tier as u8),
                    ])
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.catalog_asset_id),
                field_hash(3, self.catalog_revision),
                field_id(4, self.navigation_catalog_asset_id),
                field_hash(5, self.navigation_catalog_revision),
                CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, records),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldPopulationContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_HASH256),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let records = decode_sequence(field(&segment, 6)?, limits)?
            .into_iter()
            .map(|payload| {
                let fields = decode_struct(&payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_ID128),
                        (2, CANONICAL_TYPE_U64),
                        (3, CANONICAL_TYPE_UTF8_NFC),
                        (4, CANONICAL_TYPE_UTF8_NFC),
                        (5, CANONICAL_TYPE_UTF8_NFC),
                        (6, CANONICAL_TYPE_UTF8_NFC),
                        (7, CANONICAL_TYPE_U8),
                    ],
                )?;
                Ok(WorldPopulationRecordV1 {
                    subject_id: PersistentId::from_bytes(read_exact(nested_field(&fields, 1)?)?),
                    record_revision: read_u64(nested_field(&fields, 2)?)?,
                    home_region_id: read_schema_id(nested_field(&fields, 3)?)?,
                    home_node_id: read_schema_id(nested_field(&fields, 4)?)?,
                    current_region_id: read_schema_id(nested_field(&fields, 5)?)?,
                    current_node_id: read_schema_id(nested_field(&fields, 6)?)?,
                    tier: PopulationTierV1::from_tag(read_u8(nested_field(&fields, 7)?)?)?,
                })
            })
            .collect::<Result<Vec<_>, WorldPopulationContractError>>()?;
        let value = Self {
            schema_version: read_u16(field(&segment, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 2)?)?),
            catalog_revision: read_hash(field(&segment, 3)?)?,
            navigation_catalog_asset_id: AssetId::from_bytes(read_exact(field(&segment, 4)?)?),
            navigation_catalog_revision: read_hash(field(&segment, 5)?)?,
            records,
        };
        if value.schema_version != WORLD_POPULATION_SCHEMA_VERSION {
            return Err(WorldPopulationContractError::UnsupportedVersion(
                value.schema_version,
            ));
        }
        if value.canonical_bytes()? != bytes {
            return Err(WorldPopulationContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    #[must_use]
    pub fn record(&self, subject_id: PersistentId) -> Option<&WorldPopulationRecordV1> {
        self.records
            .binary_search_by_key(&subject_id, |record| record.subject_id)
            .ok()
            .and_then(|index| self.records.get(index))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorldPopulationCommandV1 {
    TransitionTier {
        subject_id: PersistentId,
        expected_record_revision: u64,
        catalog_asset_id: AssetId,
        catalog_revision: ContentHash,
        transition_tick: u64,
        previous_tier: PopulationTierV1,
        current_tier: PopulationTierV1,
    },
    CommitAbstractTransfer {
        subject_id: PersistentId,
        expected_record_revision: u64,
        catalog_asset_id: AssetId,
        catalog_revision: ContentHash,
        navigation_catalog_asset_id: AssetId,
        navigation_catalog_revision: ContentHash,
        route_plan_hash: ContentHash,
        source_region_id: SchemaId,
        source_node_id: SchemaId,
        target_region_id: SchemaId,
        target_node_id: SchemaId,
    },
}

impl WorldPopulationCommandV1 {
    #[must_use]
    pub const fn subject_id(&self) -> PersistentId {
        match self {
            Self::TransitionTier { subject_id, .. }
            | Self::CommitAbstractTransfer { subject_id, .. } => *subject_id,
        }
    }

    pub fn validate_shape(&self) -> Result<(), WorldPopulationContractError> {
        match self {
            Self::TransitionTier {
                previous_tier,
                current_tier,
                ..
            } if matches!(
                (*previous_tier, *current_tier),
                (PopulationTierV1::Dormant, PopulationTierV1::Abstract)
                    | (PopulationTierV1::Abstract, PopulationTierV1::Simulated)
                    | (PopulationTierV1::Simulated, PopulationTierV1::Active)
                    | (PopulationTierV1::Active, PopulationTierV1::Simulated)
                    | (PopulationTierV1::Simulated, PopulationTierV1::Abstract)
                    | (PopulationTierV1::Abstract, PopulationTierV1::Dormant)
            ) =>
            {
                Ok(())
            }
            Self::CommitAbstractTransfer {
                source_region_id,
                source_node_id,
                target_region_id,
                target_node_id,
                ..
            } if source_region_id != target_region_id && source_node_id != target_node_id => Ok(()),
            _ => Err(WorldPopulationContractError::CommandInvalid),
        }
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::TransitionTier {
                subject_id,
                expected_record_revision,
                catalog_asset_id,
                catalog_revision,
                transition_tick,
                previous_tier,
                current_tier,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(subject_id.as_bytes());
                bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
                bytes.extend_from_slice(catalog_asset_id.as_bytes());
                bytes.extend_from_slice(catalog_revision.as_bytes());
                bytes.extend_from_slice(&transition_tick.to_le_bytes());
                bytes.push(*previous_tier as u8);
                bytes.push(*current_tier as u8);
            }
            Self::CommitAbstractTransfer {
                subject_id,
                expected_record_revision,
                catalog_asset_id,
                catalog_revision,
                navigation_catalog_asset_id,
                navigation_catalog_revision,
                route_plan_hash,
                source_region_id,
                source_node_id,
                target_region_id,
                target_node_id,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(subject_id.as_bytes());
                bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
                bytes.extend_from_slice(catalog_asset_id.as_bytes());
                bytes.extend_from_slice(catalog_revision.as_bytes());
                bytes.extend_from_slice(navigation_catalog_asset_id.as_bytes());
                bytes.extend_from_slice(navigation_catalog_revision.as_bytes());
                bytes.extend_from_slice(route_plan_hash.as_bytes());
                append_text(&mut bytes, source_region_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, source_node_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, target_region_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, target_node_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldPopulationContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let value = match cursor.read_u8()? {
            1 => Self::TransitionTier {
                subject_id: PersistentId::from_bytes(read_cursor_exact(&mut cursor)?),
                expected_record_revision: cursor.read_u64()?,
                catalog_asset_id: AssetId::from_bytes(read_cursor_exact(&mut cursor)?),
                catalog_revision: ContentHash::from_bytes(read_cursor_exact(&mut cursor)?),
                transition_tick: cursor.read_u64()?,
                previous_tier: PopulationTierV1::from_tag(cursor.read_u8()?)?,
                current_tier: PopulationTierV1::from_tag(cursor.read_u8()?)?,
            },
            2 => Self::CommitAbstractTransfer {
                subject_id: PersistentId::from_bytes(read_cursor_exact(&mut cursor)?),
                expected_record_revision: cursor.read_u64()?,
                catalog_asset_id: AssetId::from_bytes(read_cursor_exact(&mut cursor)?),
                catalog_revision: ContentHash::from_bytes(read_cursor_exact(&mut cursor)?),
                navigation_catalog_asset_id: AssetId::from_bytes(read_cursor_exact(&mut cursor)?),
                navigation_catalog_revision: ContentHash::from_bytes(read_cursor_exact(
                    &mut cursor,
                )?),
                route_plan_hash: ContentHash::from_bytes(read_cursor_exact(&mut cursor)?),
                source_region_id: read_cursor_schema_id(&mut cursor, limits)?,
                source_node_id: read_cursor_schema_id(&mut cursor, limits)?,
                target_region_id: read_cursor_schema_id(&mut cursor, limits)?,
                target_node_id: read_cursor_schema_id(&mut cursor, limits)?,
            },
            tag => return Err(WorldPopulationContractError::UnknownCommand(tag)),
        };
        cursor.finish()?;
        value.validate_shape()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(WorldPopulationContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn owner_delta_bytes(&self) -> Result<Vec<u8>, WorldPopulationContractError> {
        self.validate_shape()?;
        let mut bytes = b"nextengine.world-population-owner-write-set.v1\0".to_vec();
        let payload = self.canonical_payload_bytes()?;
        bytes.extend_from_slice(
            &u64::try_from(payload.len())
                .map_err(|_| WorldPopulationContractError::CommandInvalid)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorldPopulationChangedV1 {
    TierTransitioned {
        subject_id: PersistentId,
        previous_tier: PopulationTierV1,
        current_tier: PopulationTierV1,
        transition_tick: u64,
        record_revision: u64,
    },
    AbstractTransferred {
        subject_id: PersistentId,
        source_region_id: SchemaId,
        source_node_id: SchemaId,
        target_region_id: SchemaId,
        target_node_id: SchemaId,
        route_plan_hash: ContentHash,
        record_revision: u64,
    },
}

impl WorldPopulationChangedV1 {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = Vec::new();
        match self {
            Self::TierTransitioned {
                subject_id,
                previous_tier,
                current_tier,
                transition_tick,
                record_revision,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(subject_id.as_bytes());
                bytes.push(*previous_tier as u8);
                bytes.push(*current_tier as u8);
                bytes.extend_from_slice(&transition_tick.to_le_bytes());
                bytes.extend_from_slice(&record_revision.to_le_bytes());
            }
            Self::AbstractTransferred {
                subject_id,
                source_region_id,
                source_node_id,
                target_region_id,
                target_node_id,
                route_plan_hash,
                record_revision,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(subject_id.as_bytes());
                append_text(&mut bytes, source_region_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, source_node_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, target_region_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                append_text(&mut bytes, target_node_id.as_str())
                    .map_err(|_| CanonicalError::LengthOverflow)?;
                bytes.extend_from_slice(route_plan_hash.as_bytes());
                bytes.extend_from_slice(&record_revision.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}

mod wire;

pub use wire::WorldPopulationContractError;
use wire::*;

#[cfg(test)]
mod tests;
