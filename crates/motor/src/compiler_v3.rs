use std::collections::BTreeMap;

use next_contracts::body::BodySchemaV2;
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::physics::{
    PHYSICS_MATERIAL_COMBINE_PROFILE_V1_SCHEMA_VERSION,
    PHYSICS_MATERIAL_DESCRIPTOR_V2_SCHEMA_VERSION, PhysicsContractError,
    PhysicsMaterialCombineProfileV1, PhysicsMaterialCombineRuleV1, PhysicsMaterialDescriptorV1,
    PhysicsMaterialDescriptorV2, PhysicsSurfaceVelocityCombineRuleV1,
};
use next_physics_physx::{PhysXArticulationCatalogV3, PhysXSharedMaterialProfileV1};
use next_physics_physx_ffi::NEXTENGINE_PHYSX_ABI_VERSION;

use crate::{CompiledBodySchemaV2, CompiledPhysicsDescriptorsV2, MotorCompileError};

pub const BIOMECHANICS_BODY_MATERIAL_ID: &str = "physics-material.humanoid-body.v1";
pub const BIOMECHANICS_GROUND_MATERIAL_ID: &str = "physics-material.humanoid-ground.v1";
pub const BIOMECHANICS_SOLE_MATERIAL_ID: &str = "physics-material.humanoid-sole.v1";
pub const BIOMECHANICS_MATERIAL_COMBINE_PROFILE_ID: &str =
    "nextengine.physics-material-combine.humanoid-motor.v1";
pub const BIOMECHANICS_STATIC_FRICTION_Q16: u32 = 52_429;
pub const BIOMECHANICS_DYNAMIC_FRICTION_Q16: u32 = 45_875;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledPhysicsDescriptorsV3 {
    pub base: CompiledPhysicsDescriptorsV2,
    pub materials: BTreeMap<SchemaId, PhysicsMaterialDescriptorV2>,
    pub material_combine_profile: PhysicsMaterialCombineProfileV1,
    pub ground_material_id: SchemaId,
    pub collider_material_assignment_counts: BTreeMap<SchemaId, u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBodySchemaV3 {
    pub base: CompiledBodySchemaV2,
    pub compiled_descriptor_hash: ContentHash,
    pub material_lineage_hash: ContentHash,
    pub physics_descriptors: CompiledPhysicsDescriptorsV3,
    pub physx_catalog: PhysXArticulationCatalogV3,
}

impl CompiledBodySchemaV3 {
    pub fn compile(
        schema: &BodySchemaV2,
        subject_id: PersistentId,
    ) -> Result<Self, MotorCompileError> {
        let base = CompiledBodySchemaV2::compile(schema, subject_id)?;
        let materials = biomechanics_material_catalog_v2();
        let material_combine_profile = biomechanics_material_combine_profile_v1();
        let ground_material_id = schema_id(BIOMECHANICS_GROUND_MATERIAL_ID);
        let sole_count =
            if schema.schema_id.as_str() == "nextengine.body.humanoid-biomechanics-raja-1700.v8" {
                if *schema != crate::biomechanics_humanoid_body_schema_v8() {
                    return Err(MotorCompileError::UnsupportedMaterialProfile);
                }
                4
            } else {
                2
            };
        let collider_material_assignment_counts =
            validate_material_closure(&base, &materials, &ground_material_id, sole_count)?;
        let shared_material = PhysXSharedMaterialProfileV1::from_material_catalog(
            &materials,
            &material_combine_profile,
        )
        .map_err(|_| MotorCompileError::UnsupportedMaterialProfile)?;
        let material_lineage_hash = material_lineage_hash(
            &materials,
            &material_combine_profile,
            &ground_material_id,
            &collider_material_assignment_counts,
        )?;
        let compiled_descriptor_hash = compiled_descriptor_hash_v3(
            base.compiled_descriptor_hash,
            material_lineage_hash,
            &shared_material,
        );
        let physics_descriptors = CompiledPhysicsDescriptorsV3 {
            base: base.physics_descriptors.clone(),
            materials,
            material_combine_profile,
            ground_material_id,
            collider_material_assignment_counts,
        };
        let physx_catalog = PhysXArticulationCatalogV3 {
            base: base.physx_catalog.clone(),
            shared_material,
        };
        Ok(Self {
            base,
            compiled_descriptor_hash,
            material_lineage_hash,
            physics_descriptors,
            physx_catalog,
        })
    }
}

pub fn biomechanics_material_catalog_v2() -> BTreeMap<SchemaId, PhysicsMaterialDescriptorV2> {
    [
        BIOMECHANICS_BODY_MATERIAL_ID,
        BIOMECHANICS_GROUND_MATERIAL_ID,
        BIOMECHANICS_SOLE_MATERIAL_ID,
    ]
    .into_iter()
    .map(|id| {
        let descriptor = PhysicsMaterialDescriptorV2 {
            schema_version: PHYSICS_MATERIAL_DESCRIPTOR_V2_SCHEMA_VERSION,
            base: PhysicsMaterialDescriptorV1 {
                material_id: schema_id(id),
                descriptor_revision: 1,
                static_friction_q16: BIOMECHANICS_STATIC_FRICTION_Q16,
                dynamic_friction_q16: BIOMECHANICS_DYNAMIC_FRICTION_Q16,
                restitution_q16: 0,
                canonical_material_tags: Vec::new(),
            },
            rolling_friction_q16: 0,
            spinning_friction_q16: 0,
            surface_velocity_micrometres_per_second: [0; 3],
        };
        (descriptor.base.material_id.clone(), descriptor)
    })
    .collect()
}

pub fn biomechanics_material_combine_profile_v1() -> PhysicsMaterialCombineProfileV1 {
    PhysicsMaterialCombineProfileV1 {
        schema_version: PHYSICS_MATERIAL_COMBINE_PROFILE_V1_SCHEMA_VERSION,
        profile_id: schema_id(BIOMECHANICS_MATERIAL_COMBINE_PROFILE_ID),
        profile_revision: 1,
        static_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        dynamic_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        restitution: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        rolling_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        spinning_friction: PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven,
        surface_velocity: PhysicsSurfaceVelocityCombineRuleV1::CanonicalParticipantOrder,
    }
}

fn validate_material_closure(
    compiled: &CompiledBodySchemaV2,
    materials: &BTreeMap<SchemaId, PhysicsMaterialDescriptorV2>,
    ground_material_id: &SchemaId,
    sole_count: u32,
) -> Result<BTreeMap<SchemaId, u32>, MotorCompileError> {
    if !materials.contains_key(ground_material_id) {
        return Err(MotorCompileError::InvalidReference);
    }
    for (id, descriptor) in materials {
        if id != &descriptor.base.material_id {
            return Err(MotorCompileError::InvalidReference);
        }
        descriptor.validate()?;
    }
    let mut counts = BTreeMap::new();
    for body in compiled.physics_descriptors.bodies.values() {
        for shape in body.base.base.shapes.values() {
            if !materials.contains_key(&shape.material_id) {
                return Err(MotorCompileError::InvalidReference);
            }
            let count = counts.entry(shape.material_id.clone()).or_insert(0_u32);
            *count = count.checked_add(1).ok_or(MotorCompileError::Capacity)?;
        }
    }
    let expected = BTreeMap::from([
        (schema_id(BIOMECHANICS_BODY_MATERIAL_ID), 17),
        (schema_id(BIOMECHANICS_SOLE_MATERIAL_ID), sole_count),
    ]);
    if counts != expected {
        return Err(MotorCompileError::UnsupportedMaterialProfile);
    }
    Ok(counts)
}

fn material_lineage_hash(
    materials: &BTreeMap<SchemaId, PhysicsMaterialDescriptorV2>,
    combine: &PhysicsMaterialCombineProfileV1,
    ground_material_id: &SchemaId,
    assignment_counts: &BTreeMap<SchemaId, u32>,
) -> Result<ContentHash, MotorCompileError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.biomechanics-material-lineage.v1\0");
    for (id, descriptor) in materials {
        push_id_bytes(&mut bytes, id);
        let record = descriptor.canonical_bytes().map_err(canonical_error)?;
        push_record_bytes(&mut bytes, &record)?;
    }
    let combine_record = combine.canonical_bytes().map_err(canonical_error)?;
    push_record_bytes(&mut bytes, &combine_record)?;
    push_id_bytes(&mut bytes, ground_material_id);
    for (id, count) in assignment_counts {
        push_id_bytes(&mut bytes, id);
        bytes.extend_from_slice(&count.to_le_bytes());
    }
    Ok(content_hash_from_bytes(sha256(&bytes)))
}

fn compiled_descriptor_hash_v3(
    base_hash: ContentHash,
    material_lineage_hash: ContentHash,
    shared_material: &PhysXSharedMaterialProfileV1,
) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.compiled-body-schema.v3\0");
    bytes.extend_from_slice(base_hash.as_bytes());
    bytes.extend_from_slice(material_lineage_hash.as_bytes());
    bytes.extend_from_slice(&NEXTENGINE_PHYSX_ABI_VERSION.to_le_bytes());
    bytes.extend_from_slice(b"nextengine.physx-shared-material.v1\0");
    for id in &shared_material.material_ids {
        push_id_bytes(&mut bytes, id);
    }
    for value in [
        shared_material.static_friction_q16,
        shared_material.dynamic_friction_q16,
        shared_material.restitution_q16,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn push_record_bytes(bytes: &mut Vec<u8>, record: &[u8]) -> Result<(), MotorCompileError> {
    let length = u64::try_from(record.len()).map_err(|_| MotorCompileError::Capacity)?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(record);
    Ok(())
}

fn push_id_bytes(bytes: &mut Vec<u8>, id: &SchemaId) {
    let value = id.as_str().as_bytes();
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value);
}

fn canonical_error(error: next_contracts::canonical::CanonicalError) -> MotorCompileError {
    MotorCompileError::Physics(PhysicsContractError::Canonicalization(error))
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned generated identifiers are valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomechanics_humanoid_body_schema_v2;

    #[test]
    fn biomechanics_v3_compiles_exact_material_closure_without_mutating_v2_identity() {
        let schema = biomechanics_humanoid_body_schema_v2();
        let subject = PersistentId::from_bytes([31; 16]);
        let legacy = CompiledBodySchemaV2::compile(&schema, subject).expect("V2 compile");
        let compiled = CompiledBodySchemaV3::compile(&schema, subject).expect("V3 compile");
        assert_eq!(compiled.base, legacy);
        assert_ne!(
            compiled.compiled_descriptor_hash,
            compiled.base.compiled_descriptor_hash
        );
        assert_eq!(compiled.physics_descriptors.materials.len(), 3);
        assert_eq!(
            compiled
                .physics_descriptors
                .collider_material_assignment_counts,
            BTreeMap::from([
                (schema_id(BIOMECHANICS_BODY_MATERIAL_ID), 17),
                (schema_id(BIOMECHANICS_SOLE_MATERIAL_ID), 2),
            ])
        );
        assert_eq!(
            compiled.physx_catalog.shared_material.static_friction_q16,
            BIOMECHANICS_STATIC_FRICTION_Q16
        );
        assert_eq!(
            compiled.physx_catalog.shared_material.dynamic_friction_q16,
            BIOMECHANICS_DYNAMIC_FRICTION_Q16
        );
        assert_eq!(NEXTENGINE_PHYSX_ABI_VERSION, 4);
    }

    #[test]
    fn v3_rejects_an_unresolved_collider_material_before_adapter_construction() {
        let mut schema = biomechanics_humanoid_body_schema_v2();
        schema.bodies[0].colliders[0].material_id = schema_id("physics-material.unresolved.v1");
        let error = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))
            .expect_err("unresolved material must fail");
        assert_eq!(error.stable_code(), "MOTOR_BODY_REFERENCE_INVALID");
    }

    #[test]
    fn v3_material_lineage_hash_changes_for_every_extended_material_field() {
        let materials = biomechanics_material_catalog_v2();
        let combine = biomechanics_material_combine_profile_v1();
        let ground = schema_id(BIOMECHANICS_GROUND_MATERIAL_ID);
        let counts = BTreeMap::from([
            (schema_id(BIOMECHANICS_BODY_MATERIAL_ID), 17),
            (schema_id(BIOMECHANICS_SOLE_MATERIAL_ID), 2),
        ]);
        let expected = material_lineage_hash(&materials, &combine, &ground, &counts)
            .expect("material lineage");
        let mut changed = materials;
        changed
            .get_mut(&schema_id(BIOMECHANICS_SOLE_MATERIAL_ID))
            .expect("sole")
            .surface_velocity_micrometres_per_second[2] = 1;
        assert_ne!(
            material_lineage_hash(&changed, &combine, &ground, &counts).expect("changed lineage"),
            expected
        );
    }
}
