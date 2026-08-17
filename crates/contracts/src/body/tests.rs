use super::*;

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test identifier")
}

fn body(body_id: &str, parent: Option<&str>) -> BodyDefinitionV1 {
    BodyDefinitionV1 {
        body_id: id(body_id),
        parent_body_id: parent.map(id),
        local_bind_pose: PhysicsPoseV1::default(),
        mass_microkilograms: 1_000_000,
        center_of_mass_micrometres: [0; 3],
        inertia_microkilogram_metre_squared: [1; 3],
        colliders: vec![BodyColliderDefinitionV1 {
            collider_id: id(&format!("{body_id}.collider")),
            local_pose: PhysicsPoseV1::default(),
            geometry: PhysicsGeometryV1::Sphere {
                radius_micrometres: 100_000,
            },
            material_id: id("nextengine.material.training"),
        }],
    }
}

fn schema() -> BodySchemaV1 {
    BodySchemaV1 {
        schema_version: BODY_SCHEMA_VERSION_V1,
        schema_id: id("nextengine.body.test"),
        schema_revision: 1,
        family_id: id("nextengine.family.humanoid"),
        bodies: vec![
            body("nextengine.body.root", None),
            body("nextengine.body.child", Some("nextengine.body.root")),
        ],
        joints: vec![BodyJointDefinitionV1 {
            joint_id: id("nextengine.joint.test"),
            parent_body_id: id("nextengine.body.root"),
            child_body_id: id("nextengine.body.child"),
            parent_frame: PhysicsPoseV1::default(),
            child_frame: PhysicsPoseV1::default(),
            axis_q1_30: [1 << 30, 0, 0],
            limit_min_microradians: -1_000_000,
            limit_max_microradians: 1_000_000,
            maximum_velocity_microradians_per_second: 5_000_000,
        }],
        actuators: vec![BodyActuatorDefinitionV1 {
            actuator_id: id("nextengine.actuator.test"),
            joint_id: id("nextengine.joint.test"),
            neutral_position_microradians: 0,
            stiffness_q16: 65_536,
            damping_q16: 65_536,
            maximum_effort_micronewton_metres: 10_000_000,
            maximum_effort_rate_micronewton_metres_per_second: 20_000_000,
        }],
        effectors: vec![BodyEffectorDefinitionV1 {
            effector_id: id("nextengine.effector.test"),
            body_id: id("nextengine.body.child"),
            local_pose: PhysicsPoseV1::default(),
            semantic_role_id: id("nextengine.role.foot"),
        }],
        symmetry_pairs: Vec::new(),
        capability_ids: vec![id("nextengine.capability.stand")],
    }
    .canonicalize()
}

#[test]
fn canonicalization_erases_source_record_order() {
    let expected = schema().schema_hash().expect("valid schema");
    let mut permuted = schema();
    permuted.bodies.reverse();
    assert!(permuted.validate().is_err());
    assert_eq!(
        permuted.canonicalize().schema_hash().expect("canonical"),
        expected
    );
}

#[test]
fn topology_cycle_fails_closed() {
    let mut invalid = schema();
    invalid.bodies[1].parent_body_id = Some(invalid.bodies[0].body_id.clone());
    invalid.bodies[0].parent_body_id = Some(invalid.bodies[1].body_id.clone());
    assert_eq!(invalid.validate(), Err(BodyContractError::InvalidTopology));
}

#[test]
fn body_schema_asset_round_trips_and_rejects_trailing_bytes() {
    let asset = BodySchemaAssetV1 {
        schema_version: BODY_SCHEMA_ASSET_VERSION_V1,
        asset_id: AssetId::from_bytes([0xbd; 16]),
        record_revision: 1,
        compiler_profile_id: id(BODY_PROJECTION_COMPILER_PROFILE_ID_V1),
        body_schema: reference_humanoid_body_schema_v1(),
    };
    let bytes = asset.canonical_bytes().expect("canonical asset");
    assert_eq!(
        BodySchemaAssetV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        asset
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        BodySchemaAssetV1::from_canonical_bytes(&trailing, CanonicalDecodeLimits::default()),
        Err(BodyContractError::MalformedEncoding)
    );
}

#[test]
fn instance_projection_must_bind_the_exact_schema_revision_and_hash() {
    let schema = reference_humanoid_body_schema_v1();
    let owner_hash = content_hash_from_bytes([7; 32]);
    let mut projection = BodyInstanceProjectionV1 {
        schema_version: BODY_INSTANCE_PROJECTION_VERSION_V1,
        subject_id: crate::ids::PersistentId::from_bytes([9; 16]),
        body_schema_id: schema.schema_id.clone(),
        body_schema_revision: schema.schema_revision,
        body_schema_hash: schema.schema_hash().expect("schema hash"),
        morphology_revision: 0,
        morphology_hash: owner_hash,
        equipment_revision: 0,
        equipment_hash: owner_hash,
        stats_revision: 0,
        stats_hash: owner_hash,
        damage_revision: 0,
        damage_hash: owner_hash,
        fatigue_revision: 0,
        fatigue_hash: owner_hash,
        attachment_revision: 0,
        attachment_hash: owner_hash,
        topology_revision: 1,
    };
    projection
        .validate_against(&schema)
        .expect("exact projection");
    projection.body_schema_hash = content_hash_from_bytes([8; 32]);
    assert_eq!(
        projection.validate_against(&schema),
        Err(BodyContractError::ProjectionMismatch)
    );
}

#[test]
fn zero_subject_projection_remains_valid_for_the_frozen_offline_mirror() {
    let schema = reference_humanoid_body_schema_v1();
    let owner_hash = content_hash_from_bytes([7; 32]);
    let projection = BodyInstanceProjectionV1 {
        schema_version: BODY_INSTANCE_PROJECTION_VERSION_V1,
        subject_id: crate::ids::PersistentId::default(),
        body_schema_id: schema.schema_id.clone(),
        body_schema_revision: schema.schema_revision,
        body_schema_hash: schema.schema_hash().expect("schema hash"),
        morphology_revision: 0,
        morphology_hash: owner_hash,
        equipment_revision: 0,
        equipment_hash: owner_hash,
        stats_revision: 0,
        stats_hash: owner_hash,
        damage_revision: 0,
        damage_hash: owner_hash,
        fatigue_revision: 0,
        fatigue_hash: owner_hash,
        attachment_revision: 0,
        attachment_hash: owner_hash,
        topology_revision: 1,
    };

    projection
        .validate_against(&schema)
        .expect("zero is the frozen offline mirror's nominal subject");
}
