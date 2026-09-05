use next_contracts::body::BodySchemaV2;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_physics_physx::{ExternalForceScheduleV1, PhysXAdapterError, PhysXArticulationWorldV3};
use next_physics_physx_ffi::PHYSX_FORCE_SCHEDULE_EXTENSION_VERSION;

use crate::{CompiledBodySchemaV3, MotorCompileError};

pub const BIOMECHANICS_FORCE_SCHEDULE_PROFILE_ID_V1: &str =
    "nextengine.physics.humanoid-per-iteration-external-forces.v1";

/// Material-complete body with explicitly selected per-iteration force scheduling.
/// Body/legacy compiled identities stay intact; consumers bind the outer hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBodySchemaV4 {
    pub base: CompiledBodySchemaV3,
    pub compiled_descriptor_hash: ContentHash,
}

impl CompiledBodySchemaV4 {
    /// Reset-time admission for consumers of the exact articulated body.
    pub(crate) fn articulated_subject(&self) -> Result<PersistentId, MotorCompileError> {
        self.subject_for_schema(&crate::biomechanics_humanoid_body_schema_v8())
    }

    pub(crate) fn sampled_damping_subject(&self) -> Result<PersistentId, MotorCompileError> {
        self.subject_for_schema(&crate::biomechanics_humanoid_body_schema_v9())
    }

    pub(crate) fn screened_damping_subject(&self) -> Result<PersistentId, MotorCompileError> {
        self.subject_for_schema(&crate::biomechanics_humanoid_body_schema_v10())
    }

    fn subject_for_schema(&self, schema: &BodySchemaV2) -> Result<PersistentId, MotorCompileError> {
        let subject = self
            .base
            .base
            .physics_descriptors
            .bodies
            .keys()
            .next()
            .ok_or(MotorCompileError::InvalidReference)?
            .subject_id;
        let expected = Self::compile(schema, subject)?;
        if *self != expected {
            return Err(MotorCompileError::InvalidReference);
        }
        Ok(subject)
    }

    pub fn compile(
        schema: &BodySchemaV2,
        subject_id: PersistentId,
    ) -> Result<Self, MotorCompileError> {
        let base = CompiledBodySchemaV3::compile(schema, subject_id)?;
        let mut bytes = b"nextengine.compiled-body-schema.v4\0".to_vec();
        bytes.extend_from_slice(base.compiled_descriptor_hash.as_bytes());
        bytes.extend_from_slice(BIOMECHANICS_FORCE_SCHEDULE_PROFILE_ID_V1.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&PHYSX_FORCE_SCHEDULE_EXTENSION_VERSION.to_le_bytes());
        bytes.extend_from_slice(
            &(ExternalForceScheduleV1::EverySolverPositionIteration as u32).to_le_bytes(),
        );
        Ok(Self {
            base,
            compiled_descriptor_hash: content_hash_from_bytes(sha256(&bytes)),
        })
    }

    pub fn create_world(&self) -> Result<PhysXArticulationWorldV3, PhysXAdapterError> {
        PhysXArticulationWorldV3::create_with_force_schedule_v1(
            self.base.base.physx_scene_profile,
            &self.base.physx_catalog,
            ExternalForceScheduleV1::EverySolverPositionIteration,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_schedule_has_new_lineage_without_changing_body_or_legacy_descriptor() {
        let schema = crate::biomechanics_humanoid_body_schema_v6();
        let subject = PersistentId::from_bytes([0; 16]);
        let legacy = CompiledBodySchemaV3::compile(&schema, subject).unwrap();
        let current = CompiledBodySchemaV4::compile(&schema, subject).unwrap();
        assert_eq!(current.base, legacy);
        assert_ne!(
            current.compiled_descriptor_hash,
            legacy.compiled_descriptor_hash
        );
        assert_eq!(
            current.compiled_descriptor_hash.to_hex(),
            "66d6a5b01ea1a26294b92e050bbdc79556cffd63cabdd7382d15ea0543d6c7ed"
        );
        assert_eq!(
            current,
            CompiledBodySchemaV4::compile(&schema, subject).unwrap()
        );
        let old_body =
            CompiledBodySchemaV4::compile(&crate::biomechanics_humanoid_body_schema_v5(), subject)
                .unwrap();
        assert_ne!(
            current.compiled_descriptor_hash,
            old_body.compiled_descriptor_hash
        );
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn native_force_schedule_changes_dynamics_and_reconstructs_from_same_efforts() {
        let compiled = CompiledBodySchemaV4::compile(
            &crate::biomechanics_humanoid_body_schema_v6(),
            PersistentId::from_bytes([0; 16]),
        )
        .unwrap();
        let mut current = compiled.create_world().unwrap();
        let mut reconstruction = compiled.create_world().unwrap();
        let mut legacy = PhysXArticulationWorldV3::create(
            compiled.base.base.physx_scene_profile,
            &compiled.base.physx_catalog,
        )
        .unwrap();
        assert_eq!(current.capture().unwrap(), legacy.capture().unwrap());
        let mut differs = false;
        for _ in 0..64 {
            let actual = current.apply_efforts_and_step(&[0; 23]).unwrap();
            assert_eq!(
                actual,
                reconstruction.apply_efforts_and_step(&[0; 23]).unwrap()
            );
            differs |= actual != legacy.apply_efforts_and_step(&[0; 23]).unwrap();
        }
        assert!(
            differs,
            "a profile label without native force scheduling is not a repair"
        );
    }
}
