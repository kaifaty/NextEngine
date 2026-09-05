use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};
use next_physics_physx::CanonicalPhysXSnapshotV2;

use crate::{
    CompiledBodySchemaV2, HUMANOID_SAFETY_CONTACT_PROFILE_SHA256, NORMALIZED_RESIDUAL_ONE_Q1_30,
};

pub const PROCEDURAL_STANDING_SCENARIO_MOTOR_TICKS: u64 = 1_800;
pub const PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS: i64 = 100_000;
pub const PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS: i64 = -140_000;
pub const PROCEDURAL_STANDING_REFERENCE_PROFILE_ID_V2: &str =
    "nextengine.motor.procedural-standing-reference.v2";
pub const PROCEDURAL_STANDING_REFERENCE_PROFILE_ID_V3: &str =
    "nextengine.motor.procedural-standing-reference.v3";
pub const PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1: &str =
    "nextengine.motor.procedural-walking-reference.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProceduralReferenceProfileV1 {
    Standing,
    WalkingTranslationInvariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StandingChannelKind {
    Knee,
    AnklePitch,
    Neutral,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StandingChannel {
    neutral_microradians: i64,
    kind: StandingChannelKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsProceduralStandingControllerV1 {
    root_actor_token: u64,
    root_target_z_micrometres: i64,
    channels: Vec<StandingChannel>,
    reference_profile: ProceduralReferenceProfileV1,
}

impl BiomechanicsProceduralStandingControllerV1 {
    pub fn new(
        compiled: &CompiledBodySchemaV2,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Self, ProceduralStandingError> {
        Self::new_with_profile(
            compiled,
            reset_snapshot,
            ProceduralReferenceProfileV1::Standing,
        )
    }

    pub fn new_walking_translation_invariant(
        compiled: &CompiledBodySchemaV2,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Self, ProceduralStandingError> {
        Self::new_with_profile(
            compiled,
            reset_snapshot,
            ProceduralReferenceProfileV1::WalkingTranslationInvariant,
        )
    }

    fn new_with_profile(
        compiled: &CompiledBodySchemaV2,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
        reference_profile: ProceduralReferenceProfileV1,
    ) -> Result<Self, ProceduralStandingError> {
        let root_id = compiled
            .construction_order
            .first()
            .ok_or(ProceduralStandingError::ProfileMismatch)?;
        let root_actor_token = *compiled
            .body_tokens
            .get(root_id)
            .ok_or(ProceduralStandingError::ProfileMismatch)?;
        let root = unique_root(reset_snapshot, root_actor_token)
            .ok_or(ProceduralStandingError::RootState)?;
        let channels = compiled
            .physics_descriptors
            .actuators
            .iter()
            .map(|actuator| {
                let joint_id = actuator.base.joint_id.as_str();
                let kind = if joint_id.ends_with("-knee") {
                    StandingChannelKind::Knee
                } else if joint_id.ends_with("-ankle-pitch") {
                    StandingChannelKind::AnklePitch
                } else {
                    StandingChannelKind::Neutral
                };
                StandingChannel {
                    neutral_microradians: actuator.base.neutral_position_microradians,
                    kind,
                }
            })
            .collect::<Vec<_>>();
        if channels.len() != compiled.actuator_definitions.len()
            || channels
                .iter()
                .filter(|channel| channel.kind == StandingChannelKind::Knee)
                .count()
                != 2
            || channels
                .iter()
                .filter(|channel| channel.kind == StandingChannelKind::AnklePitch)
                .count()
                != 2
        {
            return Err(ProceduralStandingError::ProfileMismatch);
        }
        Ok(Self {
            root_actor_token,
            root_target_z_micrometres: root.position_micrometres[2],
            channels,
            reference_profile,
        })
    }

    pub fn reference_targets(
        &self,
        snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Vec<i64>, ProceduralStandingError> {
        let root = unique_root(snapshot, self.root_actor_token)
            .ok_or(ProceduralStandingError::RootState)?;
        if root
            .rotation_q1_30
            .iter()
            .any(|value| value.unsigned_abs() > NORMALIZED_RESIDUAL_ONE_Q1_30 as u64)
        {
            return Err(ProceduralStandingError::RootState);
        }
        let pitch_proxy = round_div_ties_even(
            i128::from(root.rotation_q1_30[0])
                .checked_mul(2_000_000)
                .ok_or(ProceduralStandingError::NumericOverflow)?,
            i128::from(NORMALIZED_RESIDUAL_ONE_Q1_30),
        );
        let z_error = match self.reference_profile {
            ProceduralReferenceProfileV1::Standing => root.position_micrometres[2]
                .checked_sub(self.root_target_z_micrometres)
                .ok_or(ProceduralStandingError::NumericOverflow)?,
            ProceduralReferenceProfileV1::WalkingTranslationInvariant => 0,
        };
        let ankle_pitch = i128::from(PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS)
            .checked_add(round_div_ties_even(pitch_proxy, 2))
            .and_then(|value| {
                value.checked_add(round_div_ties_even(
                    i128::from(root.angular_velocity_microradians_per_second[0]),
                    20,
                ))
            })
            .and_then(|value| value.checked_add(round_div_ties_even(i128::from(z_error), 10)))
            .and_then(|value| {
                value.checked_add(round_div_ties_even(
                    i128::from(root.linear_velocity_micrometres_per_second[2]),
                    50,
                ))
            })
            .ok_or(ProceduralStandingError::NumericOverflow)?;
        let ankle_pitch =
            i64::try_from(ankle_pitch).map_err(|_| ProceduralStandingError::NumericOverflow)?;
        Ok(self
            .channels
            .iter()
            .map(|channel| match channel.kind {
                StandingChannelKind::Knee => PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
                StandingChannelKind::AnklePitch => ankle_pitch,
                StandingChannelKind::Neutral => channel.neutral_microradians,
            })
            .collect())
    }

    #[must_use]
    pub fn state_root(&self) -> ContentHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(match self.reference_profile {
            ProceduralReferenceProfileV1::Standing => {
                b"nextengine.humanoid-procedural-standing.v1\0".as_slice()
            }
            ProceduralReferenceProfileV1::WalkingTranslationInvariant => {
                b"nextengine.humanoid-procedural-walking-reference.v1\0".as_slice()
            }
        });
        bytes.extend_from_slice(&HUMANOID_SAFETY_CONTACT_PROFILE_SHA256);
        bytes.extend_from_slice(&self.root_actor_token.to_le_bytes());
        if self.reference_profile == ProceduralReferenceProfileV1::Standing {
            bytes.extend_from_slice(&self.root_target_z_micrometres.to_le_bytes());
        }
        bytes.extend_from_slice(&(self.channels.len() as u64).to_le_bytes());
        content_hash_from_bytes(sha256(&bytes))
    }
}

/// Explicit V7-body / compiled-V4 standing successor. It retains the V1 ankle
/// law and adds only the verified hip position term, without hip rate feedback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsProceduralStandingControllerV2 {
    base: BiomechanicsProceduralStandingControllerV1,
    hip_channels: [usize; 2],
    compiled_hash: ContentHash,
    subject_id: next_contracts::ids::PersistentId,
}

impl BiomechanicsProceduralStandingControllerV2 {
    pub fn new(
        compiled: &crate::CompiledBodySchemaV4,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Self, ProceduralStandingError> {
        Self::new_with_schema(
            compiled,
            reset_snapshot,
            &crate::biomechanics_humanoid_body_schema_v7(),
        )
    }

    fn new_with_schema(
        compiled: &crate::CompiledBodySchemaV4,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
        schema: &next_contracts::body::BodySchemaV2,
    ) -> Result<Self, ProceduralStandingError> {
        let base = &compiled.base.base;
        let subject = base
            .physics_descriptors
            .bodies
            .keys()
            .next()
            .ok_or(ProceduralStandingError::ProfileMismatch)?
            .subject_id;
        // Validate the complete supplied compilation, not merely an editable
        // hash label. This is reset-time work, not part of the substep loop.
        let expected = crate::CompiledBodySchemaV4::compile(schema, subject)
            .map_err(|_| ProceduralStandingError::ProfileMismatch)?;
        if *compiled != expected {
            return Err(ProceduralStandingError::ProfileMismatch);
        }
        let hip_channels = base
            .physics_descriptors
            .actuators
            .iter()
            .enumerate()
            .filter_map(|(index, actuator)| {
                actuator
                    .base
                    .joint_id
                    .as_str()
                    .ends_with("-hip-pitch")
                    .then_some(index)
            })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| ProceduralStandingError::ProfileMismatch)?;
        Ok(Self {
            base: BiomechanicsProceduralStandingControllerV1::new(base, reset_snapshot)?,
            hip_channels,
            compiled_hash: compiled.compiled_descriptor_hash,
            subject_id: subject,
        })
    }

    pub fn reference_targets(
        &self,
        snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Vec<i64>, ProceduralStandingError> {
        let mut targets = self.base.reference_targets(snapshot)?;
        let root = unique_root(snapshot, self.base.root_actor_token)
            .ok_or(ProceduralStandingError::RootState)?;
        // The discriminator uses truncation towards zero here, independently
        // of the preserved V1 ankle law's ties-to-even arithmetic.
        let pitch = i128::from(root.rotation_q1_30[0]) * 2_000_000 / (1_i128 << 30);
        let correction =
            i64::try_from(-2 * pitch).map_err(|_| ProceduralStandingError::NumericOverflow)?;
        for index in self.hip_channels {
            targets[index] = targets[index]
                .checked_add(correction)
                .ok_or(ProceduralStandingError::NumericOverflow)?;
        }
        Ok(targets)
    }

    #[must_use]
    pub fn state_root(&self) -> ContentHash {
        let mut bytes = b"nextengine.humanoid-procedural-standing.v2\0".to_vec();
        bytes.extend_from_slice(PROCEDURAL_STANDING_REFERENCE_PROFILE_ID_V2.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.compiled_hash.as_bytes());
        bytes.extend_from_slice(self.subject_id.as_bytes());
        bytes.extend_from_slice(self.base.state_root().as_bytes());
        content_hash_from_bytes(sha256(&bytes))
    }
}

/// V8-bound diagnostic standing: retained upright law, neutral MTP targets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsProceduralStandingControllerV3 {
    base: BiomechanicsProceduralStandingControllerV2,
}

impl BiomechanicsProceduralStandingControllerV3 {
    pub fn new(
        compiled: &crate::CompiledBodySchemaV4,
        reset_snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Self, ProceduralStandingError> {
        Ok(Self {
            base: BiomechanicsProceduralStandingControllerV2::new_with_schema(
                compiled,
                reset_snapshot,
                &crate::biomechanics_humanoid_body_schema_v8(),
            )?,
        })
    }

    pub fn reference_targets(
        &self,
        snapshot: &CanonicalPhysXSnapshotV2,
    ) -> Result<Vec<i64>, ProceduralStandingError> {
        self.base.reference_targets(snapshot)
    }

    #[must_use]
    pub fn state_root(&self) -> ContentHash {
        let mut bytes = b"nextengine.humanoid-procedural-standing.v3\0".to_vec();
        bytes.extend_from_slice(PROCEDURAL_STANDING_REFERENCE_PROFILE_ID_V3.as_bytes());
        bytes.extend_from_slice(self.base.state_root().as_bytes());
        bytes.extend_from_slice(crate::articulated_foot_contact_profile_hash().as_bytes());
        content_hash_from_bytes(sha256(&bytes))
    }
}

fn unique_root(
    snapshot: &CanonicalPhysXSnapshotV2,
    root_actor_token: u64,
) -> Option<&next_physics_physx::CanonicalPhysXLinkState> {
    let mut matching = snapshot
        .links
        .iter()
        .filter(|link| link.user_token == root_actor_token);
    let root = matching.next()?;
    matching.next().is_none().then_some(root)
}

fn round_div_ties_even(numerator: i128, denominator: i128) -> i128 {
    debug_assert!(denominator > 0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let doubled_remainder = remainder.unsigned_abs().saturating_mul(2);
    let denominator = denominator as u128;
    if doubled_remainder < denominator {
        quotient
    } else if doubled_remainder > denominator || quotient.unsigned_abs() % 2 == 1 {
        quotient + numerator.signum()
    } else {
        quotient
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProceduralStandingError {
    ProfileMismatch,
    RootState,
    NumericOverflow,
}

impl ProceduralStandingError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::ProfileMismatch => "MOTOR_STANDING_PROFILE_MISMATCH",
            Self::RootState => "MOTOR_STANDING_ROOT_STATE_INVALID",
            Self::NumericOverflow => "MOTOR_STANDING_NUMERIC_OVERFLOW",
        }
    }
}

impl Display for ProceduralStandingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for ProceduralStandingError {}
