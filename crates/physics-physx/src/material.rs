use std::collections::BTreeMap;

use next_contracts::physics::{
    PhysicsMaterialCombineProfileV1, PhysicsMaterialCombineRuleV1, PhysicsMaterialDescriptorV2,
    PhysicsSurfaceVelocityCombineRuleV1,
};
use next_physics_physx_ffi::{
    MATERIAL_COEFFICIENT_ENCODING_F32_BITS, MATERIAL_COEFFICIENT_ENCODING_Q16,
    MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN,
    MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER, MaterialProfileInput,
};

use crate::PhysXAdapterError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysXSharedMaterialProfileV1 {
    pub material_ids: Vec<next_contracts::ids::SchemaId>,
    pub static_friction_q16: u32,
    pub dynamic_friction_q16: u32,
    pub restitution_q16: u32,
}

impl PhysXSharedMaterialProfileV1 {
    pub fn from_material_catalog(
        materials: &BTreeMap<next_contracts::ids::SchemaId, PhysicsMaterialDescriptorV2>,
        combine: &PhysicsMaterialCombineProfileV1,
    ) -> Result<Self, PhysXAdapterError> {
        combine
            .validate()
            .map_err(|_| PhysXAdapterError::UnsupportedProfile)?;
        if materials.is_empty()
            || [
                combine.static_friction,
                combine.dynamic_friction,
                combine.restitution,
                combine.rolling_friction,
                combine.spinning_friction,
            ]
            .into_iter()
            .any(|rule| rule != PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven)
            || combine.surface_velocity
                != PhysicsSurfaceVelocityCombineRuleV1::CanonicalParticipantOrder
        {
            return Err(PhysXAdapterError::UnsupportedProfile);
        }
        let mut descriptors = materials.iter();
        let (first_id, first) = descriptors
            .next()
            .ok_or(PhysXAdapterError::UnsupportedProfile)?;
        if first_id != &first.base.material_id {
            return Err(PhysXAdapterError::ProfileMismatch);
        }
        first
            .validate()
            .map_err(|_| PhysXAdapterError::UnsupportedProfile)?;
        if first.rolling_friction_q16 != 0
            || first.spinning_friction_q16 != 0
            || first.surface_velocity_micrometres_per_second != [0; 3]
        {
            return Err(PhysXAdapterError::UnsupportedProfile);
        }
        for (id, descriptor) in descriptors {
            descriptor
                .validate()
                .map_err(|_| PhysXAdapterError::UnsupportedProfile)?;
            if id != &descriptor.base.material_id {
                return Err(PhysXAdapterError::ProfileMismatch);
            }
            if descriptor.base.static_friction_q16 != first.base.static_friction_q16
                || descriptor.base.dynamic_friction_q16 != first.base.dynamic_friction_q16
                || descriptor.base.restitution_q16 != first.base.restitution_q16
                || descriptor.rolling_friction_q16 != first.rolling_friction_q16
                || descriptor.spinning_friction_q16 != first.spinning_friction_q16
                || descriptor.surface_velocity_micrometres_per_second
                    != first.surface_velocity_micrometres_per_second
            {
                return Err(PhysXAdapterError::UnsupportedProfile);
            }
        }
        Ok(Self {
            material_ids: materials.keys().cloned().collect(),
            static_friction_q16: first.base.static_friction_q16,
            dynamic_friction_q16: first.base.dynamic_friction_q16,
            restitution_q16: first.base.restitution_q16,
        })
    }

    pub(crate) fn ffi(&self) -> MaterialProfileInput {
        MaterialProfileInput {
            coefficient_encoding: MATERIAL_COEFFICIENT_ENCODING_Q16,
            static_friction: self.static_friction_q16,
            dynamic_friction: self.dynamic_friction_q16,
            restitution: self.restitution_q16,
            rolling_friction: 0,
            spinning_friction: 0,
            surface_velocity_micrometres_per_second: [0; 3],
            coefficient_combine_rules: [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5],
            surface_velocity_combine_rule: MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER,
        }
    }
}

pub(crate) fn legacy_stage0_material_input() -> MaterialProfileInput {
    MaterialProfileInput {
        coefficient_encoding: MATERIAL_COEFFICIENT_ENCODING_F32_BITS,
        static_friction: 0.8_f32.to_bits(),
        dynamic_friction: 0.7_f32.to_bits(),
        restitution: 0.0_f32.to_bits(),
        rolling_friction: 0,
        spinning_friction: 0,
        surface_velocity_micrometres_per_second: [0; 3],
        coefficient_combine_rules: [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5],
        surface_velocity_combine_rule: MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER,
    }
}
