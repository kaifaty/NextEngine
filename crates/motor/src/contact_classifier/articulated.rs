use super::*;

#[cfg(test)]
mod tests;

/// Exact V8 diagnostic contact semantics; not a training-environment identity.
#[must_use]
pub fn articulated_foot_contact_profile_hash() -> ContentHash {
    content_hash_from_bytes(sha256(b"nextengine.articulated-foot-contact.v1\0body=v8;ground-oriented-raw-vector-sum-before-active-filter;limit=6000000uns;retain-pair-limits;left-right-separate;anatomical-active-continuity;legacy-nonfoot-roles"))
}

/// Same contact law, explicitly bound to the V9 diagnostic body.
#[must_use]
pub fn sampled_damping_contact_profile_hash() -> ContentHash {
    let mut bytes = b"nextengine.articulated-foot-contact.v2\0body=v9\0".to_vec();
    bytes.extend_from_slice(articulated_foot_contact_profile_hash().as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

/// Same contact law, explicitly bound to the V10 diagnostic body.
#[must_use]
pub fn screened_damping_contact_profile_hash() -> ContentHash {
    let mut bytes = b"nextengine.articulated-foot-contact.v3\0body=v10\0".to_vec();
    bytes.extend_from_slice(articulated_foot_contact_profile_hash().as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

/// Same contact law, explicitly bound to the V11 diagnostic body.
#[must_use]
pub fn bandwidth_contact_profile_hash() -> ContentHash {
    let mut bytes = b"nextengine.articulated-foot-contact.v4\0body=v11\0".to_vec();
    bytes.extend_from_slice(articulated_foot_contact_profile_hash().as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsContactClassifierV2 {
    base: BiomechanicsContactClassifier,
    feet: BTreeMap<u64, usize>,
    binding: ContentHash,
    profile_hash: ContentHash,
    foot_active_substeps: [u64; 2],
    foot_impulses: [[i128; 3]; 2],
}

impl BiomechanicsContactClassifierV2 {
    pub fn new(compiled: &crate::CompiledBodySchemaV4) -> Result<Self, ContactClassificationError> {
        let subject = compiled
            .articulated_subject()
            .map_err(|_| ContactClassificationError::ProfileMismatch)?;
        Self::new_bound(compiled, subject, articulated_foot_contact_profile_hash())
    }

    pub fn new_sampled_damping(
        compiled: &crate::CompiledBodySchemaV4,
    ) -> Result<Self, ContactClassificationError> {
        let subject = compiled
            .sampled_damping_subject()
            .map_err(|_| ContactClassificationError::ProfileMismatch)?;
        Self::new_bound(compiled, subject, sampled_damping_contact_profile_hash())
    }

    pub fn new_screened_damping(
        compiled: &crate::CompiledBodySchemaV4,
    ) -> Result<Self, ContactClassificationError> {
        let subject = compiled
            .screened_damping_subject()
            .map_err(|_| ContactClassificationError::ProfileMismatch)?;
        Self::new_bound(compiled, subject, screened_damping_contact_profile_hash())
    }

    pub fn new_bandwidth(
        compiled: &crate::CompiledBodySchemaV4,
    ) -> Result<Self, ContactClassificationError> {
        let subject = compiled
            .bandwidth_subject()
            .map_err(|_| ContactClassificationError::ProfileMismatch)?;
        Self::new_bound(compiled, subject, bandwidth_contact_profile_hash())
    }

    fn new_bound(
        compiled: &crate::CompiledBodySchemaV4,
        subject: next_contracts::ids::PersistentId,
        profile_hash: ContentHash,
    ) -> Result<Self, ContactClassificationError> {
        let base = &compiled.base.base;
        let mut feet = BTreeMap::new();
        for (side_index, side) in ["left", "right"].iter().enumerate() {
            for suffix in ["ankle-roll", "mtp"] {
                let name = format!("body.{side}-{suffix}");
                let actor = base
                    .body_tokens
                    .iter()
                    .find(|(id, _)| id.as_str() == name)
                    .ok_or(ContactClassificationError::ProfileMismatch)?
                    .1;
                feet.insert(*actor, side_index);
            }
        }
        let mut bytes = compiled.compiled_descriptor_hash.as_bytes().to_vec();
        bytes.extend_from_slice(subject.as_bytes());
        Ok(Self {
            base: BiomechanicsContactClassifier::new(base)?,
            feet,
            binding: content_hash_from_bytes(sha256(&bytes)),
            profile_hash,
            foot_active_substeps: [0; 2],
            foot_impulses: [[0; 3]; 2],
        })
    }

    pub fn classify_substep(
        &mut self,
        snapshot: &CanonicalPhysXSnapshotV2,
        profile: BiomechanicsSkillContactProfileV1,
    ) -> Result<BiomechanicsContactFrameV1, ContactClassificationError> {
        // Validate/aggregate every raw pair before filtering. Ground token 1 is
        // always the first canonical endpoint, hence all vectors share orientation.
        let aggregates = self.base.aggregate_contacts(&snapshot.contacts)?;
        let mut totals = [[0_i128; 3]; 2];
        let mut active = [false; 2];
        for (pair, aggregate) in &aggregates {
            if let Some(side) = self.foot_side(*pair) {
                for (total, value) in totals[side].iter_mut().zip(aggregate.impulse) {
                    *total = total
                        .checked_add(value)
                        .ok_or(ContactClassificationError::NumericOverflow)?;
                }
                active[side] |= aggregate.minimum_separation < 0
                    || impulse_magnitude_squared(aggregate.impulse)?
                        >= u128::from(ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS).pow(2);
            }
        }
        let mut violation = [false; 2];
        let mut counts = [0; 2];
        for side in 0..2 {
            violation[side] = impulse_magnitude_squared(totals[side])? > 6_000_000_u128.pow(2);
            if active[side] {
                counts[side] = self.foot_active_substeps[side]
                    .checked_add(1)
                    .ok_or(ContactClassificationError::NumericOverflow)?;
            }
        }
        // Base classification is transactional too. No fallible work follows it.
        let mut frame = self.base.classify_substep(snapshot, profile)?;
        for contact in &mut frame.contacts {
            if let Some(side) = self.foot_side(contact.pair) {
                contact.hard_impact_violation |= violation[side];
            }
        }
        self.foot_active_substeps = counts;
        self.foot_impulses = totals;
        frame.safety_contact_profile_hash = self.profile_hash;
        let mut bytes = b"nextengine.articulated-foot-contact-frame.v1\0".to_vec();
        bytes.extend_from_slice(frame.safety_contact_profile_hash.as_bytes());
        bytes.extend_from_slice(self.binding.as_bytes());
        bytes.extend_from_slice(classification_root(profile, &frame.contacts).as_bytes());
        for total in totals {
            for component in total {
                bytes.extend_from_slice(&component.to_le_bytes());
            }
        }
        frame.classification_root = content_hash_from_bytes(sha256(&bytes));
        frame.continuity_root = self.continuity_root();
        Ok(frame)
    }

    fn foot_side(&self, pair: ContactPairKeyV1) -> Option<usize> {
        (pair.actor_a_token == HUMANOID_GROUND_ACTOR_TOKEN)
            .then(|| self.feet.get(&pair.actor_b_token).copied())
            .flatten()
    }

    #[must_use]
    pub const fn foot_active_substeps(&self) -> [u64; 2] {
        self.foot_active_substeps
    }

    #[must_use]
    pub const fn foot_impulses_micronewton_seconds(&self) -> [[i128; 3]; 2] {
        self.foot_impulses
    }

    #[must_use]
    pub fn continuity_root(&self) -> ContentHash {
        let mut bytes = b"nextengine.articulated-foot-contact-continuity.v1\0".to_vec();
        bytes.extend_from_slice(self.profile_hash.as_bytes());
        bytes.extend_from_slice(self.binding.as_bytes());
        bytes.extend_from_slice(self.base.continuity_root().as_bytes());
        for count in self.foot_active_substeps {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        content_hash_from_bytes(sha256(&bytes))
    }

    pub fn reset(&mut self) {
        self.base.reset();
        self.foot_active_substeps = [0; 2];
        self.foot_impulses = [[0; 3]; 2];
    }
}
