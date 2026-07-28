use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AssetRevisionRefV1, CapabilityId, ContentHash, MechanicPackageId, SchemaId,
    content_hash_from_bytes, sha256,
};

pub const DATA_ONLY_PACKAGE_KIND_V1: &str = "nextengine.mechanic-package.data-only.v1";

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateTransitionV1 {
    pub transition_id: SchemaId,
    pub source_state_id: SchemaId,
    pub target_state_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogueDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub entry_node_id: SchemaId,
    pub transitions: Vec<StateTransitionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub entry_state_id: SchemaId,
    pub transitions: Vec<StateTransitionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub dimension_id: SchemaId,
    pub minimum_value: i32,
    pub maximum_value: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionDefinitionV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub interaction_id: SchemaId,
    pub dialogue_definition: AssetRevisionRefV1,
    pub dialogue_transition_id: SchemaId,
    pub quest_definition: AssetRevisionRefV1,
    pub quest_transition_id: SchemaId,
    pub relationship_definition: AssetRevisionRefV1,
    pub relationship_source_value: i32,
    pub relationship_delta: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MechanicPackageManifestV1 {
    pub package_id: MechanicPackageId,
    pub package_version: u32,
    pub package_kind: SchemaId,
    pub required_capabilities: Vec<CapabilityId>,
    pub interaction_definition_hashes: Vec<ContentHash>,
    pub package_manifest_sha256: ContentHash,
}

impl MechanicPackageManifestV1 {
    pub fn new(
        package_id: MechanicPackageId,
        package_version: u32,
        mut required_capabilities: Vec<CapabilityId>,
        mut interaction_definition_hashes: Vec<ContentHash>,
    ) -> Result<Self, MechanicsContractError> {
        if package_version == 0 {
            return Err(MechanicsContractError::ZeroVersion);
        }
        required_capabilities.sort();
        interaction_definition_hashes.sort();
        ensure_unique(&required_capabilities)?;
        ensure_unique(&interaction_definition_hashes)?;
        let mut manifest = Self {
            package_id,
            package_version,
            package_kind: SchemaId::new(DATA_ONLY_PACKAGE_KIND_V1)
                .expect("engine-owned package kind is valid"),
            required_capabilities,
            interaction_definition_hashes,
            package_manifest_sha256: ContentHash::default(),
        };
        manifest.package_manifest_sha256 = manifest.computed_hash();
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), MechanicsContractError> {
        if self.package_version == 0 {
            return Err(MechanicsContractError::ZeroVersion);
        }
        if self.package_kind.as_str() != DATA_ONLY_PACKAGE_KIND_V1
            || !strictly_sorted(&self.required_capabilities)
            || !strictly_sorted(&self.interaction_definition_hashes)
            || self.computed_hash() != self.package_manifest_sha256
        {
            return Err(MechanicsContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        let mut bytes = Vec::new();
        extend_text(&mut bytes, self.package_id.as_str());
        bytes.extend_from_slice(&self.package_version.to_le_bytes());
        extend_text(&mut bytes, self.package_kind.as_str());
        bytes.extend_from_slice(&(self.required_capabilities.len() as u32).to_le_bytes());
        for capability in &self.required_capabilities {
            extend_text(&mut bytes, capability.as_str());
        }
        bytes.extend_from_slice(&(self.interaction_definition_hashes.len() as u32).to_le_bytes());
        for hash in &self.interaction_definition_hashes {
            bytes.extend_from_slice(hash.as_bytes());
        }
        domain_hash("nextengine.mechanic-package-manifest.v1", &bytes)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LockedMechanicPackageV1 {
    pub package_id: MechanicPackageId,
    pub package_manifest_sha256: ContentHash,
    pub granted_capabilities: Vec<CapabilityId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MechanicsLockV1 {
    pub packages: Vec<LockedMechanicPackageV1>,
    pub mechanics_lock_sha256: ContentHash,
}

impl MechanicsLockV1 {
    pub fn new(mut packages: Vec<LockedMechanicPackageV1>) -> Result<Self, MechanicsContractError> {
        for package in &mut packages {
            package.granted_capabilities.sort();
            ensure_unique(&package.granted_capabilities)?;
        }
        packages.sort();
        ensure_unique_by(&packages, |package| package.package_id.as_str())?;
        let mut lock = Self {
            packages,
            mechanics_lock_sha256: ContentHash::default(),
        };
        lock.mechanics_lock_sha256 = lock.computed_hash();
        Ok(lock)
    }

    pub fn validate(&self) -> Result<(), MechanicsContractError> {
        if !strictly_sorted(&self.packages)
            || self
                .packages
                .iter()
                .any(|package| !strictly_sorted(&package.granted_capabilities))
            || self.computed_hash() != self.mechanics_lock_sha256
        {
            return Err(MechanicsContractError::HashMismatch);
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.packages.len() as u32).to_le_bytes());
        for package in &self.packages {
            extend_text(&mut bytes, package.package_id.as_str());
            bytes.extend_from_slice(package.package_manifest_sha256.as_bytes());
            bytes.extend_from_slice(&(package.granted_capabilities.len() as u32).to_le_bytes());
            for capability in &package.granted_capabilities {
                extend_text(&mut bytes, capability.as_str());
            }
        }
        domain_hash("nextengine.mechanics-lock.v1", &bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpgDefinitionRegistryV1 {
    pub dialogues: Vec<DialogueDefinitionV1>,
    pub quests: Vec<QuestDefinitionV1>,
    pub relationships: Vec<RelationshipDefinitionV1>,
    pub interactions: Vec<InteractionDefinitionV1>,
    pub packages: Vec<MechanicPackageManifestV1>,
    pub mechanics_lock: MechanicsLockV1,
    pub registry_sha256: ContentHash,
}

impl RpgDefinitionRegistryV1 {
    pub fn empty() -> Result<Self, MechanicsContractError> {
        Self::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            MechanicsLockV1::new(Vec::new())?,
        )
    }

    pub fn new(
        mut dialogues: Vec<DialogueDefinitionV1>,
        mut quests: Vec<QuestDefinitionV1>,
        mut relationships: Vec<RelationshipDefinitionV1>,
        mut interactions: Vec<InteractionDefinitionV1>,
        mut packages: Vec<MechanicPackageManifestV1>,
        mechanics_lock: MechanicsLockV1,
    ) -> Result<Self, MechanicsContractError> {
        dialogues.sort_by_key(|definition| definition.asset_revision);
        quests.sort_by_key(|definition| definition.asset_revision);
        relationships.sort_by_key(|definition| definition.asset_revision);
        interactions.sort_by_key(|definition| definition.asset_revision);
        packages.sort_by(|left, right| left.package_id.cmp(&right.package_id));
        let mut registry = Self {
            dialogues,
            quests,
            relationships,
            interactions,
            packages,
            mechanics_lock,
            registry_sha256: ContentHash::default(),
        };
        registry.validate_without_hash()?;
        registry.registry_sha256 = registry.computed_hash();
        Ok(registry)
    }

    pub fn validate(&self) -> Result<(), MechanicsContractError> {
        self.validate_without_hash()?;
        if self.computed_hash() != self.registry_sha256 {
            return Err(MechanicsContractError::HashMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn dialogue(&self, asset: AssetRevisionRefV1) -> Option<&DialogueDefinitionV1> {
        self.dialogues
            .binary_search_by_key(&asset, |definition| definition.asset_revision)
            .ok()
            .map(|index| &self.dialogues[index])
    }

    #[must_use]
    pub fn quest(&self, asset: AssetRevisionRefV1) -> Option<&QuestDefinitionV1> {
        self.quests
            .binary_search_by_key(&asset, |definition| definition.asset_revision)
            .ok()
            .map(|index| &self.quests[index])
    }

    #[must_use]
    pub fn relationship(&self, asset: AssetRevisionRefV1) -> Option<&RelationshipDefinitionV1> {
        self.relationships
            .binary_search_by_key(&asset, |definition| definition.asset_revision)
            .ok()
            .map(|index| &self.relationships[index])
    }

    fn validate_without_hash(&self) -> Result<(), MechanicsContractError> {
        if !strictly_sorted_by(&self.dialogues, |value| value.asset_revision)
            || !strictly_sorted_by(&self.quests, |value| value.asset_revision)
            || !strictly_sorted_by(&self.relationships, |value| value.asset_revision)
            || !strictly_sorted_by(&self.interactions, |value| value.asset_revision)
            || !self
                .packages
                .windows(2)
                .all(|pair| pair[0].package_id < pair[1].package_id)
        {
            return Err(MechanicsContractError::DuplicateIdentity);
        }
        self.mechanics_lock.validate()?;
        for definition in &self.dialogues {
            validate_transitions(&definition.transitions, &definition.entry_node_id)?;
        }
        for definition in &self.quests {
            validate_transitions(&definition.transitions, &definition.entry_state_id)?;
        }
        for definition in &self.relationships {
            if definition.minimum_value > definition.maximum_value {
                return Err(MechanicsContractError::InvalidBounds);
            }
        }
        let package_hashes: BTreeSet<_> = self
            .packages
            .iter()
            .map(|package| {
                package.validate()?;
                Ok((package.package_id.clone(), package.package_manifest_sha256))
            })
            .collect::<Result<_, MechanicsContractError>>()?;
        let locked: BTreeSet<_> = self
            .mechanics_lock
            .packages
            .iter()
            .map(|package| (package.package_id.clone(), package.package_manifest_sha256))
            .collect();
        if package_hashes != locked {
            return Err(MechanicsContractError::PackageLockMismatch);
        }
        for package in &self.packages {
            let locked = self
                .mechanics_lock
                .packages
                .iter()
                .find(|locked| locked.package_id == package.package_id)
                .ok_or(MechanicsContractError::PackageLockMismatch)?;
            if package.required_capabilities.iter().any(|capability| {
                locked
                    .granted_capabilities
                    .binary_search(capability)
                    .is_err()
            }) {
                return Err(MechanicsContractError::CapabilityDenied);
            }
        }
        let interaction_hashes: BTreeSet<_> = self
            .interactions
            .iter()
            .map(interaction_definition_hash)
            .collect();
        for package in &self.packages {
            let declared: BTreeSet<_> = package
                .interaction_definition_hashes
                .iter()
                .copied()
                .collect();
            if !declared.is_subset(&interaction_hashes) {
                return Err(MechanicsContractError::MissingDefinition);
            }
        }
        for interaction in &self.interactions {
            let dialogue = self
                .dialogue(interaction.dialogue_definition)
                .ok_or(MechanicsContractError::MissingDefinition)?;
            let quest = self
                .quest(interaction.quest_definition)
                .ok_or(MechanicsContractError::MissingDefinition)?;
            let relationship = self
                .relationship(interaction.relationship_definition)
                .ok_or(MechanicsContractError::MissingDefinition)?;
            if !dialogue
                .transitions
                .iter()
                .any(|transition| transition.transition_id == interaction.dialogue_transition_id)
                || !quest
                    .transitions
                    .iter()
                    .any(|transition| transition.transition_id == interaction.quest_transition_id)
            {
                return Err(MechanicsContractError::MissingTransition);
            }
            let target = interaction
                .relationship_source_value
                .checked_add(interaction.relationship_delta)
                .ok_or(MechanicsContractError::InvalidBounds)?;
            if interaction.relationship_source_value < relationship.minimum_value
                || target > relationship.maximum_value
                || target < relationship.minimum_value
            {
                return Err(MechanicsContractError::InvalidBounds);
            }
        }
        Ok(())
    }

    fn computed_hash(&self) -> ContentHash {
        let mut bytes = Vec::new();
        for dialogue in &self.dialogues {
            bytes.extend_from_slice(dialogue_definition_hash(dialogue).as_bytes());
        }
        for quest in &self.quests {
            bytes.extend_from_slice(quest_definition_hash(quest).as_bytes());
        }
        for relationship in &self.relationships {
            bytes.extend_from_slice(relationship_definition_hash(relationship).as_bytes());
        }
        for interaction in &self.interactions {
            bytes.extend_from_slice(interaction_definition_hash(interaction).as_bytes());
        }
        bytes.extend_from_slice(self.mechanics_lock.mechanics_lock_sha256.as_bytes());
        domain_hash("nextengine.rpg-definition-registry.v1", &bytes)
    }
}

#[must_use]
pub fn dialogue_definition_hash(definition: &DialogueDefinitionV1) -> ContentHash {
    definition_with_transitions_hash(
        "nextengine.dialogue-definition.v1",
        definition.asset_revision,
        &definition.entry_node_id,
        &definition.transitions,
    )
}

#[must_use]
pub fn quest_definition_hash(definition: &QuestDefinitionV1) -> ContentHash {
    definition_with_transitions_hash(
        "nextengine.quest-definition.v1",
        definition.asset_revision,
        &definition.entry_state_id,
        &definition.transitions,
    )
}

#[must_use]
pub fn relationship_definition_hash(definition: &RelationshipDefinitionV1) -> ContentHash {
    let mut bytes = asset_revision_bytes(definition.asset_revision);
    extend_text(&mut bytes, definition.dimension_id.as_str());
    bytes.extend_from_slice(&definition.minimum_value.to_le_bytes());
    bytes.extend_from_slice(&definition.maximum_value.to_le_bytes());
    domain_hash("nextengine.relationship-definition.v1", &bytes)
}

#[must_use]
pub fn interaction_definition_hash(definition: &InteractionDefinitionV1) -> ContentHash {
    let mut bytes = asset_revision_bytes(definition.asset_revision);
    extend_text(&mut bytes, definition.interaction_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.dialogue_definition));
    extend_text(&mut bytes, definition.dialogue_transition_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.quest_definition));
    extend_text(&mut bytes, definition.quest_transition_id.as_str());
    bytes.extend_from_slice(&asset_revision_bytes(definition.relationship_definition));
    bytes.extend_from_slice(&definition.relationship_source_value.to_le_bytes());
    bytes.extend_from_slice(&definition.relationship_delta.to_le_bytes());
    domain_hash("nextengine.interaction-definition.v1", &bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MechanicsContractError {
    ZeroVersion,
    DuplicateIdentity,
    HashMismatch,
    PackageLockMismatch,
    MissingDefinition,
    MissingTransition,
    InvalidTransitionGraph,
    InvalidBounds,
    CapabilityDenied,
}

impl Display for MechanicsContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ZeroVersion => "mechanic package version must be positive",
            Self::DuplicateIdentity => "mechanic identity is duplicated or unordered",
            Self::HashMismatch => "mechanic hash mismatch",
            Self::PackageLockMismatch => "mechanic package/lock mismatch",
            Self::MissingDefinition => "mechanic definition is missing",
            Self::MissingTransition => "mechanic transition is missing",
            Self::InvalidTransitionGraph => "mechanic transition graph is invalid",
            Self::InvalidBounds => "mechanic bounds are invalid",
            Self::CapabilityDenied => "mechanic capability denied",
        })
    }
}

impl Error for MechanicsContractError {}

fn validate_transitions(
    transitions: &[StateTransitionV1],
    entry: &SchemaId,
) -> Result<(), MechanicsContractError> {
    if transitions.is_empty()
        || !strictly_sorted(transitions)
        || !transitions
            .iter()
            .any(|transition| &transition.source_state_id == entry)
    {
        return Err(MechanicsContractError::InvalidTransitionGraph);
    }
    let mut sources = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for transition in transitions {
        if transition.source_state_id == transition.target_state_id
            || !sources.insert(transition.source_state_id.clone())
            || !ids.insert(transition.transition_id.clone())
        {
            return Err(MechanicsContractError::InvalidTransitionGraph);
        }
    }
    Ok(())
}

fn definition_with_transitions_hash(
    domain: &str,
    asset: AssetRevisionRefV1,
    entry: &SchemaId,
    transitions: &[StateTransitionV1],
) -> ContentHash {
    let mut bytes = asset_revision_bytes(asset);
    extend_text(&mut bytes, entry.as_str());
    for transition in transitions {
        extend_text(&mut bytes, transition.transition_id.as_str());
        extend_text(&mut bytes, transition.source_state_id.as_str());
        extend_text(&mut bytes, transition.target_state_id.as_str());
    }
    domain_hash(domain, &bytes)
}

fn asset_revision_bytes(asset: AssetRevisionRefV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(asset.asset_id.as_bytes());
    bytes.extend_from_slice(asset.record_sha256.as_bytes());
    bytes
}

fn domain_hash(domain: &str, body: &[u8]) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(body);
    content_hash_from_bytes(sha256(&bytes))
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn ensure_unique<T: Ord>(values: &[T]) -> Result<(), MechanicsContractError> {
    if strictly_sorted(values) || values.len() < 2 {
        Ok(())
    } else {
        Err(MechanicsContractError::DuplicateIdentity)
    }
}

fn ensure_unique_by<T>(
    values: &[T],
    key: impl Fn(&T) -> &str,
) -> Result<(), MechanicsContractError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        Err(MechanicsContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn strictly_sorted_by<T, K: Ord>(values: &[T], key: impl Fn(&T) -> K) -> bool {
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RPG_COMMAND_CAPABILITY_ID;

    #[test]
    fn invalid_transition_graph_is_rejected() {
        let transition_a = StateTransitionV1 {
            transition_id: SchemaId::new("nextengine.test.transition.a").expect("ID"),
            source_state_id: SchemaId::new("nextengine.test.state.entry").expect("ID"),
            target_state_id: SchemaId::new("nextengine.test.state.a").expect("ID"),
        };
        let transition_b = StateTransitionV1 {
            transition_id: SchemaId::new("nextengine.test.transition.b").expect("ID"),
            source_state_id: SchemaId::new("nextengine.test.state.entry").expect("ID"),
            target_state_id: SchemaId::new("nextengine.test.state.b").expect("ID"),
        };

        assert_eq!(
            validate_transitions(
                &[transition_a, transition_b],
                &SchemaId::new("nextengine.test.state.entry").expect("ID"),
            ),
            Err(MechanicsContractError::InvalidTransitionGraph)
        );
    }

    #[test]
    fn package_capability_denial_is_rejected_before_activation() {
        let manifest = package_manifest();
        let lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
            package_id: manifest.package_id.clone(),
            package_manifest_sha256: manifest.package_manifest_sha256,
            granted_capabilities: Vec::new(),
        }])
        .expect("lock");

        assert_eq!(
            RpgDefinitionRegistryV1::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![manifest],
                lock,
            ),
            Err(MechanicsContractError::CapabilityDenied)
        );
    }

    #[test]
    fn package_hash_mismatch_is_rejected_before_activation() {
        let manifest = package_manifest();
        let lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
            package_id: manifest.package_id.clone(),
            package_manifest_sha256: ContentHash::from_bytes([0x7f; 32]),
            granted_capabilities: vec![
                CapabilityId::new(RPG_COMMAND_CAPABILITY_ID).expect("capability"),
            ],
        }])
        .expect("lock");

        assert_eq!(
            RpgDefinitionRegistryV1::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![manifest],
                lock,
            ),
            Err(MechanicsContractError::PackageLockMismatch)
        );
    }

    #[test]
    fn mutated_grant_order_invalidates_lock() {
        let manifest = package_manifest();
        let mut lock = MechanicsLockV1::new(vec![LockedMechanicPackageV1 {
            package_id: manifest.package_id,
            package_manifest_sha256: manifest.package_manifest_sha256,
            granted_capabilities: vec![
                CapabilityId::new("nextengine.capability.z").expect("capability"),
                CapabilityId::new("nextengine.capability.a").expect("capability"),
            ],
        }])
        .expect("lock");
        lock.packages[0].granted_capabilities.swap(0, 1);

        assert_eq!(lock.validate(), Err(MechanicsContractError::HashMismatch));
    }

    fn package_manifest() -> MechanicPackageManifestV1 {
        MechanicPackageManifestV1::new(
            MechanicPackageId::new("org.nextengine.test.package").expect("package ID"),
            1,
            vec![CapabilityId::new(RPG_COMMAND_CAPABILITY_ID).expect("capability")],
            Vec::new(),
        )
        .expect("manifest")
    }
}
