use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MechanicPackageManifestV1 {
    pub package_id: MechanicPackageId,
    pub package_version: u32,
    pub package_kind: SchemaId,
    pub required_capabilities: Vec<CapabilityId>,
    pub interaction_definition_hashes: Vec<ContentHash>,
    pub ability_definition_hashes: Vec<ContentHash>,
    pub package_manifest_sha256: ContentHash,
}

impl MechanicPackageManifestV1 {
    pub fn new(
        package_id: MechanicPackageId,
        package_version: u32,
        mut required_capabilities: Vec<CapabilityId>,
        mut interaction_definition_hashes: Vec<ContentHash>,
        mut ability_definition_hashes: Vec<ContentHash>,
    ) -> Result<Self, MechanicsContractError> {
        if package_version == 0 {
            return Err(MechanicsContractError::ZeroVersion);
        }
        required_capabilities.sort();
        interaction_definition_hashes.sort();
        ability_definition_hashes.sort();
        ensure_unique(&required_capabilities)?;
        ensure_unique(&interaction_definition_hashes)?;
        ensure_unique(&ability_definition_hashes)?;
        let mut manifest = Self {
            package_id,
            package_version,
            package_kind: SchemaId::new(DATA_ONLY_PACKAGE_KIND_V1)
                .expect("engine-owned package kind is valid"),
            required_capabilities,
            interaction_definition_hashes,
            ability_definition_hashes,
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
            || !strictly_sorted(&self.ability_definition_hashes)
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
        bytes.extend_from_slice(&(self.ability_definition_hashes.len() as u32).to_le_bytes());
        for hash in &self.ability_definition_hashes {
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
pub struct RpgDefinitionRegistryV2 {
    pub dialogues: Vec<DialogueDefinitionV1>,
    pub quests: Vec<QuestDefinitionV1>,
    pub relationships: Vec<RelationshipDefinitionV1>,
    pub interactions: Vec<InteractionDefinitionV2>,
    pub abilities: Vec<AbilityDefinitionV1>,
    pub packages: Vec<MechanicPackageManifestV1>,
    pub mechanics_lock: MechanicsLockV1,
    pub registry_sha256: ContentHash,
}

impl RpgDefinitionRegistryV2 {
    pub fn empty() -> Result<Self, MechanicsContractError> {
        Self::new(
            Vec::new(),
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
        mut interactions: Vec<InteractionDefinitionV2>,
        mut abilities: Vec<AbilityDefinitionV1>,
        mut packages: Vec<MechanicPackageManifestV1>,
        mechanics_lock: MechanicsLockV1,
    ) -> Result<Self, MechanicsContractError> {
        dialogues.sort_by_key(|definition| definition.asset_revision);
        quests.sort_by_key(|definition| definition.asset_revision);
        relationships.sort_by_key(|definition| definition.asset_revision);
        interactions.sort_by_key(|definition| definition.asset_revision);
        abilities.sort_by_key(|definition| definition.asset_revision);
        packages.sort_by(|left, right| left.package_id.cmp(&right.package_id));
        let mut registry = Self {
            dialogues,
            quests,
            relationships,
            interactions,
            abilities,
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

    #[must_use]
    pub fn ability(&self, asset: AssetRevisionRefV1) -> Option<&AbilityDefinitionV1> {
        self.abilities
            .binary_search_by_key(&asset, |definition| definition.asset_revision)
            .ok()
            .map(|index| &self.abilities[index])
    }

    fn validate_without_hash(&self) -> Result<(), MechanicsContractError> {
        if !strictly_sorted_by(&self.dialogues, |value| value.asset_revision)
            || !strictly_sorted_by(&self.quests, |value| value.asset_revision)
            || !strictly_sorted_by(&self.relationships, |value| value.asset_revision)
            || !strictly_sorted_by(&self.interactions, |value| value.asset_revision)
            || !strictly_sorted_by(&self.abilities, |value| value.asset_revision)
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
        for ability in &self.abilities {
            ability.validate()?;
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
            .map(interaction_definition_hash_v2)
            .collect();
        for package in &self.packages {
            let locked_package = self
                .mechanics_lock
                .packages
                .iter()
                .find(|locked| locked.package_id == package.package_id)
                .ok_or(MechanicsContractError::PackageLockMismatch)?;
            let declared: BTreeSet<_> = package
                .interaction_definition_hashes
                .iter()
                .copied()
                .collect();
            if !declared.is_subset(&interaction_hashes) {
                return Err(MechanicsContractError::MissingDefinition);
            }
            let ability_hashes: BTreeSet<_> = self
                .abilities
                .iter()
                .filter(|ability| ability.package_id == package.package_id)
                .map(ability_definition_hash)
                .collect();
            if package
                .ability_definition_hashes
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                != ability_hashes
            {
                return Err(MechanicsContractError::MissingDefinition);
            }
            for ability in self
                .abilities
                .iter()
                .filter(|ability| ability.package_id == package.package_id)
            {
                if ability.required_capabilities.iter().any(|capability| {
                    locked_package
                        .granted_capabilities
                        .binary_search(capability)
                        .is_err()
                }) {
                    return Err(MechanicsContractError::CapabilityDenied);
                }
            }
        }
        if self.abilities.iter().any(|ability| {
            self.packages
                .iter()
                .all(|package| package.package_id != ability.package_id)
        }) {
            return Err(MechanicsContractError::MissingDefinition);
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
            bytes.extend_from_slice(interaction_definition_hash_v2(interaction).as_bytes());
        }
        for ability in &self.abilities {
            bytes.extend_from_slice(ability_definition_hash(ability).as_bytes());
        }
        bytes.extend_from_slice(self.mechanics_lock.mechanics_lock_sha256.as_bytes());
        domain_hash("nextengine.rpg-definition-registry.v2", &bytes)
    }
}
