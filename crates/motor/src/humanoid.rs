pub use next_contracts::body::{
    REFERENCE_HUMANOID_DOF, REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES,
    reference_humanoid_body_schema_v1,
};

use next_contracts::body::{
    BODY_INSTANCE_PROJECTION_VERSION_V1, BodyInstanceProjectionV1, BodySchemaV1,
};
use next_contracts::ids::PersistentId;
use next_contracts::project::domain_hash;

pub fn neutral_body_instance_projection_v1(
    schema: &BodySchemaV1,
    subject_id: PersistentId,
) -> Result<BodyInstanceProjectionV1, next_contracts::body::BodyContractError> {
    let body_schema_hash = schema.schema_hash()?;
    let owner_hash = |owner: &str| domain_hash(owner, body_schema_hash.as_bytes());
    let value = BodyInstanceProjectionV1 {
        schema_version: BODY_INSTANCE_PROJECTION_VERSION_V1,
        subject_id,
        body_schema_id: schema.schema_id.clone(),
        body_schema_revision: schema.schema_revision,
        body_schema_hash,
        morphology_revision: 0,
        morphology_hash: owner_hash("nextengine.body-projection.neutral-morphology.v1"),
        equipment_revision: 0,
        equipment_hash: owner_hash("nextengine.body-projection.no-equipment.v1"),
        stats_revision: 0,
        stats_hash: owner_hash("nextengine.body-projection.neutral-stats.v1"),
        damage_revision: 0,
        damage_hash: owner_hash("nextengine.body-projection.no-damage.v1"),
        fatigue_revision: 0,
        fatigue_hash: owner_hash("nextengine.body-projection.no-fatigue.v1"),
        attachment_revision: 0,
        attachment_hash: owner_hash("nextengine.body-projection.no-attachments.v1"),
        topology_revision: 1,
    };
    value.validate_against(schema)?;
    Ok(value)
}

pub fn reference_humanoid_body_instance_projection_v1(
    subject_id: PersistentId,
) -> Result<BodyInstanceProjectionV1, next_contracts::body::BodyContractError> {
    neutral_body_instance_projection_v1(&reference_humanoid_body_schema_v1(), subject_id)
}
