use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::BodyContactRoleV2;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_physics_physx::{CanonicalPhysXContactV2, CanonicalPhysXSnapshotV2};

use crate::CompiledBodySchemaV2;

pub const HUMANOID_SAFETY_CONTACT_PROFILE_SHA256: [u8; 32] = [
    0xad, 0x20, 0xd7, 0xa4, 0xab, 0xd5, 0xcc, 0x8b, 0x59, 0x06, 0x9e, 0xcd, 0xb5, 0x91, 0x61, 0x49,
    0x9c, 0xe7, 0x75, 0x49, 0x53, 0xcb, 0xff, 0x24, 0x77, 0xf2, 0x85, 0x03, 0x95, 0xad, 0xb4, 0x2c,
];
pub const HUMANOID_GROUND_ACTOR_TOKEN: u64 = 1;
pub const HUMANOID_GROUND_SHAPE_TOKEN: u64 = 0;
pub const ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS: u64 = 50_000;
pub const CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS: u64 = 250_000;
pub const LOW_IMPULSE_GRACE_SUBSTEPS: u64 = 4;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BiomechanicsSkillContactProfileV1 {
    Locomotion = 1,
    BraceFall = 2,
    GetUp = 3,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BiomechanicsContactClassV1 {
    SoleSupport = 1,
    TransientAllowed = 2,
    ForbiddenLocomotion = 3,
    BraceSupport = 4,
    GetUpSupport = 5,
    SelfCollisionViolation = 6,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContactPairKeyV1 {
    pub actor_a_token: u64,
    pub shape_a_token: u64,
    pub actor_b_token: u64,
    pub shape_b_token: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClassifiedBiomechanicsContactV1 {
    pub pair: ContactPairKeyV1,
    pub primary_role: BodyContactRoleV2,
    pub secondary_role: Option<BodyContactRoleV2>,
    pub impulse_micronewton_seconds: [i64; 3],
    pub impulse_magnitude_squared: u128,
    pub minimum_separation_micrometres: i64,
    pub consecutive_active_substeps: u64,
    pub material: bool,
    pub hard_impact_violation: bool,
    pub class: BiomechanicsContactClassV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsContactFrameV1 {
    pub safety_contact_profile_hash: ContentHash,
    pub skill_profile: BiomechanicsSkillContactProfileV1,
    pub contacts: Vec<ClassifiedBiomechanicsContactV1>,
    pub classification_root: ContentHash,
    pub continuity_root: ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ContactAggregate {
    impulse: [i128; 3],
    minimum_separation: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsContactClassifier {
    shape_roles: BTreeMap<u64, BodyContactRoleV2>,
    shape_actors: BTreeMap<u64, u64>,
    body_actors: BTreeSet<u64>,
    continuity: BTreeMap<ContactPairKeyV1, u64>,
}

impl BiomechanicsContactClassifier {
    pub fn new(compiled: &CompiledBodySchemaV2) -> Result<Self, ContactClassificationError> {
        let body_actors = compiled
            .body_tokens
            .values()
            .copied()
            .collect::<BTreeSet<_>>();
        if compiled.collider_contact_roles.len() != compiled.collider_body_tokens.len()
            || body_actors.contains(&HUMANOID_GROUND_ACTOR_TOKEN)
        {
            return Err(ContactClassificationError::ProfileMismatch);
        }
        for (shape, actor) in &compiled.collider_body_tokens {
            if !compiled.collider_contact_roles.contains_key(shape)
                || !body_actors.contains(actor)
                || *shape == HUMANOID_GROUND_ACTOR_TOKEN
            {
                return Err(ContactClassificationError::ProfileMismatch);
            }
        }
        Ok(Self {
            shape_roles: compiled.collider_contact_roles.clone(),
            shape_actors: compiled.collider_body_tokens.clone(),
            body_actors,
            continuity: BTreeMap::new(),
        })
    }

    pub fn classify_substep(
        &mut self,
        snapshot: &CanonicalPhysXSnapshotV2,
        profile: BiomechanicsSkillContactProfileV1,
    ) -> Result<BiomechanicsContactFrameV1, ContactClassificationError> {
        let aggregates = aggregate_contacts(&snapshot.contacts)?;
        let mut next_continuity = BTreeMap::new();
        let mut classified = Vec::new();
        for (pair, aggregate) in aggregates {
            let (primary_role, secondary_role, self_contact) = self.roles_for_pair(pair)?;
            let impulse = aggregate
                .impulse
                .map(|value| {
                    i64::try_from(value).map_err(|_| ContactClassificationError::NumericOverflow)
                })
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            let impulse: [i64; 3] = impulse
                .try_into()
                .map_err(|_| ContactClassificationError::NumericOverflow)?;
            let magnitude_squared = impulse_magnitude_squared(aggregate.impulse)?;
            let active = aggregate.minimum_separation < 0
                || magnitude_squared
                    >= u128::from(ACTIVE_CONTACT_IMPULSE_MICRONEWTON_SECONDS).pow(2);
            if !active {
                continue;
            }
            let consecutive = self
                .continuity
                .get(&pair)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(ContactClassificationError::NumericOverflow)?;
            next_continuity.insert(pair, consecutive);
            let material = magnitude_squared
                > u128::from(CONTACT_BRUSH_CEILING_MICRONEWTON_SECONDS).pow(2)
                || consecutive > LOW_IMPULSE_GRACE_SUBSTEPS;
            let hard_limit = hard_impact_limit(primary_role)
                .min(secondary_role.map(hard_impact_limit).unwrap_or(u64::MAX));
            let hard_impact_violation = magnitude_squared > u128::from(hard_limit).pow(2);
            classified.push(ClassifiedBiomechanicsContactV1 {
                pair,
                primary_role,
                secondary_role,
                impulse_micronewton_seconds: impulse,
                impulse_magnitude_squared: magnitude_squared,
                minimum_separation_micrometres: aggregate.minimum_separation,
                consecutive_active_substeps: consecutive,
                material,
                hard_impact_violation,
                class: classify_role(primary_role, self_contact, material, profile),
            });
        }
        classified.sort_unstable();
        let classification_root = classification_root(profile, &classified);
        let continuity_root = continuity_root(&next_continuity);
        self.continuity = next_continuity;
        Ok(BiomechanicsContactFrameV1 {
            safety_contact_profile_hash: content_hash_from_bytes(
                HUMANOID_SAFETY_CONTACT_PROFILE_SHA256,
            ),
            skill_profile: profile,
            contacts: classified,
            classification_root,
            continuity_root,
        })
    }

    pub fn reset(&mut self) {
        self.continuity.clear();
    }

    #[must_use]
    pub fn continuity_root(&self) -> ContentHash {
        continuity_root(&self.continuity)
    }

    fn roles_for_pair(
        &self,
        pair: ContactPairKeyV1,
    ) -> Result<(BodyContactRoleV2, Option<BodyContactRoleV2>, bool), ContactClassificationError>
    {
        let a_is_ground = pair.actor_a_token == HUMANOID_GROUND_ACTOR_TOKEN;
        let b_is_ground = pair.actor_b_token == HUMANOID_GROUND_ACTOR_TOKEN;
        if a_is_ground == b_is_ground {
            if a_is_ground {
                return Err(ContactClassificationError::InvalidGroundPair);
            }
            let role_a = self.validated_role(pair.actor_a_token, pair.shape_a_token)?;
            let role_b = self.validated_role(pair.actor_b_token, pair.shape_b_token)?;
            if pair.actor_a_token == pair.actor_b_token {
                return Err(ContactClassificationError::ActorShapeMismatch);
            }
            return Ok((role_a, Some(role_b), true));
        }
        let (ground_shape, actor, shape) = if a_is_ground {
            (pair.shape_a_token, pair.actor_b_token, pair.shape_b_token)
        } else {
            (pair.shape_b_token, pair.actor_a_token, pair.shape_a_token)
        };
        if ground_shape != HUMANOID_GROUND_SHAPE_TOKEN {
            return Err(ContactClassificationError::InvalidGroundPair);
        }
        Ok((self.validated_role(actor, shape)?, None, false))
    }

    fn validated_role(
        &self,
        actor: u64,
        shape: u64,
    ) -> Result<BodyContactRoleV2, ContactClassificationError> {
        if !self.body_actors.contains(&actor) {
            return Err(ContactClassificationError::UnknownActor);
        }
        let expected_actor = self
            .shape_actors
            .get(&shape)
            .ok_or(ContactClassificationError::UnknownShape)?;
        if *expected_actor != actor {
            return Err(ContactClassificationError::ActorShapeMismatch);
        }
        self.shape_roles
            .get(&shape)
            .copied()
            .ok_or(ContactClassificationError::UnknownShape)
    }
}

fn aggregate_contacts(
    contacts: &[CanonicalPhysXContactV2],
) -> Result<BTreeMap<ContactPairKeyV1, ContactAggregate>, ContactClassificationError> {
    let mut aggregates = BTreeMap::<ContactPairKeyV1, ContactAggregate>::new();
    for contact in contacts {
        let (pair, sign) = canonical_pair(contact);
        let entry = aggregates.entry(pair).or_insert(ContactAggregate {
            impulse: [0; 3],
            minimum_separation: contact.separation_micrometres,
        });
        for (output, value) in entry
            .impulse
            .iter_mut()
            .zip(contact.impulse_micronewton_seconds)
        {
            *output = output
                .checked_add(i128::from(value) * sign)
                .ok_or(ContactClassificationError::NumericOverflow)?;
        }
        entry.minimum_separation = entry.minimum_separation.min(contact.separation_micrometres);
    }
    Ok(aggregates)
}

fn canonical_pair(contact: &CanonicalPhysXContactV2) -> (ContactPairKeyV1, i128) {
    let first = (contact.actor_a_token, contact.shape_a_token);
    let second = (contact.actor_b_token, contact.shape_b_token);
    if first <= second {
        (
            ContactPairKeyV1 {
                actor_a_token: first.0,
                shape_a_token: first.1,
                actor_b_token: second.0,
                shape_b_token: second.1,
            },
            1,
        )
    } else {
        (
            ContactPairKeyV1 {
                actor_a_token: second.0,
                shape_a_token: second.1,
                actor_b_token: first.0,
                shape_b_token: first.1,
            },
            -1,
        )
    }
}

fn impulse_magnitude_squared(impulse: [i128; 3]) -> Result<u128, ContactClassificationError> {
    impulse.into_iter().try_fold(0_u128, |sum, value| {
        sum.checked_add(value.unsigned_abs().pow(2))
            .ok_or(ContactClassificationError::NumericOverflow)
    })
}

const fn hard_impact_limit(role: BodyContactRoleV2) -> u64 {
    match role {
        BodyContactRoleV2::FootWithSoleFeature => 6_000_000,
        BodyContactRoleV2::ThighGround
        | BodyContactRoleV2::ShankGround
        | BodyContactRoleV2::KneeGround
        | BodyContactRoleV2::PelvisGround
        | BodyContactRoleV2::TorsoGround => 4_000_000,
        BodyContactRoleV2::AnkleGround
        | BodyContactRoleV2::UpperArmGround
        | BodyContactRoleV2::ForearmGround
        | BodyContactRoleV2::HandGround => 3_000_000,
        BodyContactRoleV2::HeadGround => 1_000_000,
    }
}

const fn classify_role(
    role: BodyContactRoleV2,
    self_contact: bool,
    material: bool,
    profile: BiomechanicsSkillContactProfileV1,
) -> BiomechanicsContactClassV1 {
    if self_contact {
        return if material {
            BiomechanicsContactClassV1::SelfCollisionViolation
        } else {
            BiomechanicsContactClassV1::TransientAllowed
        };
    }
    if matches!(role, BodyContactRoleV2::FootWithSoleFeature) {
        return BiomechanicsContactClassV1::SoleSupport;
    }
    match profile {
        BiomechanicsSkillContactProfileV1::Locomotion if material => {
            BiomechanicsContactClassV1::ForbiddenLocomotion
        }
        BiomechanicsSkillContactProfileV1::Locomotion => {
            BiomechanicsContactClassV1::TransientAllowed
        }
        BiomechanicsSkillContactProfileV1::BraceFall
            if matches!(
                role,
                BodyContactRoleV2::HandGround | BodyContactRoleV2::ForearmGround
            ) =>
        {
            BiomechanicsContactClassV1::BraceSupport
        }
        BiomechanicsSkillContactProfileV1::GetUp
            if matches!(
                role,
                BodyContactRoleV2::HandGround
                    | BodyContactRoleV2::ForearmGround
                    | BodyContactRoleV2::KneeGround
            ) =>
        {
            BiomechanicsContactClassV1::GetUpSupport
        }
        BiomechanicsSkillContactProfileV1::BraceFall | BiomechanicsSkillContactProfileV1::GetUp => {
            BiomechanicsContactClassV1::TransientAllowed
        }
    }
}

fn classification_root(
    profile: BiomechanicsSkillContactProfileV1,
    contacts: &[ClassifiedBiomechanicsContactV1],
) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.humanoid-contact-frame.v1\0");
    bytes.extend_from_slice(&HUMANOID_SAFETY_CONTACT_PROFILE_SHA256);
    bytes.push(profile as u8);
    bytes.extend_from_slice(&(contacts.len() as u64).to_le_bytes());
    for contact in contacts {
        push_pair(&mut bytes, contact.pair);
        bytes.push(contact.primary_role as u8);
        bytes.push(contact.secondary_role.map_or(0, |role| role as u8));
        for value in contact.impulse_micronewton_seconds {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&contact.impulse_magnitude_squared.to_le_bytes());
        bytes.extend_from_slice(&contact.minimum_separation_micrometres.to_le_bytes());
        bytes.extend_from_slice(&contact.consecutive_active_substeps.to_le_bytes());
        bytes.push(u8::from(contact.material));
        bytes.push(u8::from(contact.hard_impact_violation));
        bytes.push(contact.class as u8);
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn continuity_root(continuity: &BTreeMap<ContactPairKeyV1, u64>) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.humanoid-contact-continuity.v1\0");
    bytes.extend_from_slice(&HUMANOID_SAFETY_CONTACT_PROFILE_SHA256);
    bytes.extend_from_slice(&(continuity.len() as u64).to_le_bytes());
    for (pair, consecutive) in continuity {
        push_pair(&mut bytes, *pair);
        bytes.extend_from_slice(&consecutive.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn push_pair(bytes: &mut Vec<u8>, pair: ContactPairKeyV1) {
    bytes.extend_from_slice(&pair.actor_a_token.to_le_bytes());
    bytes.extend_from_slice(&pair.shape_a_token.to_le_bytes());
    bytes.extend_from_slice(&pair.actor_b_token.to_le_bytes());
    bytes.extend_from_slice(&pair.shape_b_token.to_le_bytes());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactClassificationError {
    ProfileMismatch,
    UnknownActor,
    UnknownShape,
    ActorShapeMismatch,
    InvalidGroundPair,
    NumericOverflow,
}

impl ContactClassificationError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_CONTACT_PROFILE_MISMATCH",
            Self::UnknownActor => "MOTOR_CONTACT_ACTOR_UNKNOWN",
            Self::UnknownShape => "MOTOR_CONTACT_SHAPE_UNKNOWN",
            Self::ActorShapeMismatch => "MOTOR_CONTACT_ACTOR_SHAPE_MISMATCH",
            Self::InvalidGroundPair => "MOTOR_CONTACT_GROUND_PAIR_INVALID",
            Self::NumericOverflow => "MOTOR_CONTACT_NUMERIC_OVERFLOW",
        }
    }
}

impl Display for ContactClassificationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for ContactClassificationError {}
