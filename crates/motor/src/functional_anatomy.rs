use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::{
    BODY_CAPABILITY_ENVELOPE_VERSION_V1, BodyActuatorCapabilityV1, BodyCapabilityEnvelopeV1,
    FUNCTIONAL_CAPACITY_FULL_Q16, FunctionalActuatorDirectionV1, FunctionalAnatomyProfileV1,
};
use next_contracts::rpg::{
    BodyImpairmentV1, BodyRecoveryStageV1, RpgAggregateEnvelopeV1, RpgAggregatePayloadV1,
};

use crate::CompiledBodySchemaV1;

pub fn compile_body_capability_envelope_v1(
    compiled: &CompiledBodySchemaV1,
    anatomy_profile: &FunctionalAnatomyProfileV1,
    condition: &RpgAggregateEnvelopeV1,
) -> Result<BodyCapabilityEnvelopeV1, FunctionalAnatomyCompileError> {
    anatomy_profile
        .validate()
        .map_err(|_| FunctionalAnatomyCompileError::InvalidProfile)?;
    condition
        .validate()
        .map_err(|_| FunctionalAnatomyCompileError::InvalidCondition)?;
    let RpgAggregatePayloadV1::BodyCondition(condition_payload) = &condition.payload else {
        return Err(FunctionalAnatomyCompileError::InvalidCondition);
    };
    let anatomy_profile_hash = anatomy_profile
        .profile_hash()
        .map_err(|_| FunctionalAnatomyCompileError::InvalidProfile)?;
    if compiled.subject_id != condition_payload.character_id
        || compiled.body_schema_hash != anatomy_profile.body_schema_hash
        || condition_payload.body_schema_hash != anatomy_profile.body_schema_hash
        || condition_payload.anatomy_profile_hash != anatomy_profile_hash
        || condition_payload.region_id != anatomy_profile.region_id
        || compiled
            .actuator_definitions
            .binary_search_by(|actuator| actuator.actuator_id.cmp(&anatomy_profile.actuator_id))
            .is_err()
    {
        return Err(FunctionalAnatomyCompileError::ProfileMismatch);
    }

    let affected_capacity = match condition_payload.impairment {
        BodyImpairmentV1::Intact => FUNCTIONAL_CAPACITY_FULL_Q16,
        BodyImpairmentV1::PartialKneeExtensor => {
            if condition_payload.recovery_stage == BodyRecoveryStageV1::Repaired {
                anatomy_profile.post_repair_capacity_q16
            } else {
                anatomy_profile.partial_capacity_q16
            }
        }
        BodyImpairmentV1::TendonTransmissionLost | BodyImpairmentV1::NerveControlLost => 0,
    };
    let (negative_capacity_q16, positive_capacity_q16) = match anatomy_profile.affected_direction {
        FunctionalActuatorDirectionV1::Negative => {
            (affected_capacity, FUNCTIONAL_CAPACITY_FULL_Q16)
        }
        FunctionalActuatorDirectionV1::Positive => {
            (FUNCTIONAL_CAPACITY_FULL_Q16, affected_capacity)
        }
    };
    let envelope = BodyCapabilityEnvelopeV1 {
        schema_version: BODY_CAPABILITY_ENVELOPE_VERSION_V1,
        subject_id: compiled.subject_id,
        body_schema_hash: compiled.body_schema_hash,
        anatomy_profile_hash,
        condition_state_hash: condition
            .state_hash()
            .map_err(|_| FunctionalAnatomyCompileError::InvalidCondition)?,
        condition_revision: condition.revision,
        actuator_capabilities: vec![BodyActuatorCapabilityV1 {
            actuator_id: anatomy_profile.actuator_id.clone(),
            negative_capacity_q16,
            positive_capacity_q16,
        }],
    };
    envelope
        .validate()
        .map_err(|_| FunctionalAnatomyCompileError::InvalidEnvelope)?;
    Ok(envelope)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionalAnatomyCompileError {
    InvalidProfile,
    InvalidCondition,
    ProfileMismatch,
    InvalidEnvelope,
}

impl FunctionalAnatomyCompileError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::InvalidProfile => "MOTOR_FUNCTIONAL_ANATOMY_PROFILE_INVALID",
            Self::InvalidCondition => "MOTOR_FUNCTIONAL_ANATOMY_CONDITION_INVALID",
            Self::ProfileMismatch => "MOTOR_FUNCTIONAL_ANATOMY_PROFILE_MISMATCH",
            Self::InvalidEnvelope => "MOTOR_FUNCTIONAL_ANATOMY_ENVELOPE_INVALID",
        }
    }
}

impl Display for FunctionalAnatomyCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for FunctionalAnatomyCompileError {}

#[cfg(test)]
mod tests {
    use next_contracts::body::{
        REFERENCE_LEFT_KNEE_ACTUATOR_ID, reference_lower_limb_anatomy_profile_v1,
    };
    use next_contracts::ids::{AssetId, ContentHash, PersistentId};
    use next_contracts::rpg::{
        BodyConditionPayloadV1, DefinitionRefV1, ProvenanceBindingV1, RpgAggregatePayloadV1,
        SystemicConditionV1,
    };

    use super::*;
    use crate::{
        ACTUATOR_CAPABILITY_CLAMPED, FixedPdController, JointControlStateV1,
        reference_humanoid_body_schema_v1,
    };

    fn condition(
        compiled: &CompiledBodySchemaV1,
        profile: &FunctionalAnatomyProfileV1,
        impairment: BodyImpairmentV1,
        recovery_stage: BodyRecoveryStageV1,
    ) -> RpgAggregateEnvelopeV1 {
        let systemic_condition = if impairment == BodyImpairmentV1::Intact {
            SystemicConditionV1::Stable
        } else {
            SystemicConditionV1::Impaired
        };
        RpgAggregateEnvelopeV1::new(
            PersistentId::from_bytes([0xc0; 16]),
            1,
            7,
            DefinitionRefV1::Exact {
                asset_id: AssetId::from_bytes([0xc0; 16]),
                content_hash: ContentHash::from_bytes([0xc0; 32]),
            },
            ProvenanceBindingV1::None,
            RpgAggregatePayloadV1::BodyCondition(BodyConditionPayloadV1 {
                character_id: compiled.subject_id,
                body_schema_hash: compiled.body_schema_hash,
                anatomy_profile_hash: profile.profile_hash().expect("profile hash"),
                region_id: profile.region_id.clone(),
                impairment,
                recovery_stage,
                systemic_condition,
            }),
        )
        .expect("condition")
    }

    fn positive_knee_effort(
        compiled: &CompiledBodySchemaV1,
        capability: &BodyCapabilityEnvelopeV1,
    ) -> (i64, u16) {
        let mut controller = FixedPdController::new(compiled).expect("controller");
        let targets = vec![10_000_000; controller.channel_count()];
        let states = vec![
            JointControlStateV1 {
                position_microradians: -1_500_000,
                velocity_microradians_per_second: -20_000_000,
            };
            controller.channel_count()
        ];
        let mut result = Vec::new();
        for _ in 0..512 {
            result = controller
                .step_substep_with_capability(&targets, &states, Some(capability))
                .expect("bounded substep");
        }
        let knee = result
            .iter()
            .find(|effort| effort.actuator_id.as_str() == REFERENCE_LEFT_KNEE_ACTUATOR_ID)
            .expect("left knee channel");
        (knee.effort_micronewton_metres, knee.clamp_flags)
    }

    #[test]
    fn intact_partial_and_zero_loss_compile_to_distinct_positive_knee_capacity() {
        let schema = reference_humanoid_body_schema_v1();
        let subject_id = PersistentId::from_bytes([0x54; 16]);
        let compiled = CompiledBodySchemaV1::compile(&schema, subject_id).expect("compile");
        let profile = reference_lower_limb_anatomy_profile_v1(&schema).expect("profile");

        let envelope = |impairment, recovery_stage| {
            compile_body_capability_envelope_v1(
                &compiled,
                &profile,
                &condition(&compiled, &profile, impairment, recovery_stage),
            )
            .expect("capability")
        };
        let intact = envelope(BodyImpairmentV1::Intact, BodyRecoveryStageV1::Untreated);
        let partial = envelope(
            BodyImpairmentV1::PartialKneeExtensor,
            BodyRecoveryStageV1::Untreated,
        );
        let tendon = envelope(
            BodyImpairmentV1::TendonTransmissionLost,
            BodyRecoveryStageV1::Untreated,
        );
        let nerve = envelope(
            BodyImpairmentV1::NerveControlLost,
            BodyRecoveryStageV1::Untreated,
        );

        let (intact_effort, _) = positive_knee_effort(&compiled, &intact);
        let (partial_effort, partial_flags) = positive_knee_effort(&compiled, &partial);
        let (tendon_effort, tendon_flags) = positive_knee_effort(&compiled, &tendon);
        let (nerve_effort, nerve_flags) = positive_knee_effort(&compiled, &nerve);
        assert!(intact_effort > partial_effort && partial_effort > 0);
        assert_ne!(partial_flags & ACTUATOR_CAPABILITY_CLAMPED, 0);
        assert_eq!(tendon_effort, 0);
        assert_eq!(nerve_effort, 0);
        assert_ne!(tendon_flags & ACTUATOR_CAPABILITY_CLAMPED, 0);
        assert_ne!(nerve_flags & ACTUATOR_CAPABILITY_CLAMPED, 0);
    }

    #[test]
    fn positive_direction_loss_preserves_negative_knee_effort() {
        let schema = reference_humanoid_body_schema_v1();
        let compiled = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0x54; 16]))
            .expect("compile");
        let profile = reference_lower_limb_anatomy_profile_v1(&schema).expect("profile");
        let capability = compile_body_capability_envelope_v1(
            &compiled,
            &profile,
            &condition(
                &compiled,
                &profile,
                BodyImpairmentV1::NerveControlLost,
                BodyRecoveryStageV1::Untreated,
            ),
        )
        .expect("capability");
        let mut controller = FixedPdController::new(&compiled).expect("controller");
        let targets = vec![-10_000_000; controller.channel_count()];
        let states = vec![
            JointControlStateV1 {
                position_microradians: 1_500_000,
                velocity_microradians_per_second: 20_000_000,
            };
            controller.channel_count()
        ];
        let result = controller
            .step_substep_with_capability(&targets, &states, Some(&capability))
            .expect("bounded substep");
        let knee = result
            .iter()
            .find(|effort| effort.actuator_id.as_str() == REFERENCE_LEFT_KNEE_ACTUATOR_ID)
            .expect("left knee channel");
        assert!(knee.effort_micronewton_metres < 0);
        assert_eq!(knee.clamp_flags & ACTUATOR_CAPABILITY_CLAMPED, 0);
    }
}
