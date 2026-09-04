use next_contracts::body::{
    BODY_SCHEMA_VERSION_V2, BodyActuatorDefinitionV2, BodyColliderDefinitionV2,
    BodyCollisionExclusionV2, BodyContactRoleV2, BodyDefinitionV2, BodyEffectorDefinitionV2,
    BodyJointDefinitionV2, BodyMassProjectionGroupV2, BodyMassProjectionMemberV2,
    BodyMirrorValueRuleV2, BodyPoseV2, BodySchemaV2, BodySemanticRoleV2, BodySymmetryPairV2,
};
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::physics::{
    PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsParticipationV1,
};

pub const BIOMECHANICS_HUMANOID_DOF: usize = 23;
pub const BIOMECHANICS_HUMANOID_BODY_COUNT: usize = 24;
pub const BIOMECHANICS_HUMANOID_COLLIDER_COUNT: usize = 19;
pub const BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS: u64 = 75_337_000;
pub const BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES: i64 = 943_500;
pub const BIOMECHANICS_HUMANOID_V2_SHOULDER_HALF_WIDTH_MICROMETRES: i64 = 170_000;
pub const BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES: i64 = 215_000;
pub const BIOMECHANICS_HUMANOID_V3_CARRIER_MASS_MICROKILOGRAMS: u64 = 250_000;
pub const BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED: i64 = 1_000;
pub const BIOMECHANICS_HUMANOID_V3_THIGH_HALF_WIDTH_MICROMETRES: i64 = 75_000;
pub const BIOMECHANICS_HUMANOID_V4_THIGH_HALF_WIDTH_MICROMETRES: i64 = 55_000;
pub const BIOMECHANICS_HUMANOID_V3_SHANK_HALF_WIDTH_MICROMETRES: i64 = 55_000;
pub const BIOMECHANICS_HUMANOID_V4_SHANK_HALF_WIDTH_MICROMETRES: i64 = 45_000;
pub const BIOMECHANICS_HUMANOID_V3_KNEE_RADIUS_MICROMETRES: i64 = 65_000;
pub const BIOMECHANICS_HUMANOID_V4_KNEE_RADIUS_MICROMETRES: i64 = 50_000;

mod profile;

use profile::*;

#[derive(Clone, Copy)]
struct LegCollisionProjection {
    thigh_half_width_micrometres: i64,
    shank_half_extent_micrometres: i64,
    knee_radius_micrometres: i64,
}

const LEG_COLLISIONS_V3: LegCollisionProjection = LegCollisionProjection {
    thigh_half_width_micrometres: BIOMECHANICS_HUMANOID_V3_THIGH_HALF_WIDTH_MICROMETRES,
    shank_half_extent_micrometres: BIOMECHANICS_HUMANOID_V3_SHANK_HALF_WIDTH_MICROMETRES,
    knee_radius_micrometres: BIOMECHANICS_HUMANOID_V3_KNEE_RADIUS_MICROMETRES,
};

const LEG_COLLISIONS_V4: LegCollisionProjection = LegCollisionProjection {
    thigh_half_width_micrometres: BIOMECHANICS_HUMANOID_V4_THIGH_HALF_WIDTH_MICROMETRES,
    shank_half_extent_micrometres: BIOMECHANICS_HUMANOID_V4_SHANK_HALF_WIDTH_MICROMETRES,
    knee_radius_micrometres: BIOMECHANICS_HUMANOID_V4_KNEE_RADIUS_MICROMETRES,
};

#[must_use]
pub fn biomechanics_humanoid_body_schema_v2() -> BodySchemaV2 {
    biomechanics_humanoid_body_schema(
        "nextengine.body.humanoid-biomechanics-raja-1700.v2",
        2,
        content_hash_from_bytes([
            0xe6, 0xc5, 0x4e, 0x43, 0xd3, 0x71, 0x3e, 0x4e, 0x9c, 0xc6, 0xa3, 0xb2, 0x73, 0x5a,
            0x61, 0x33, 0x28, 0xc3, 0xb5, 0x1d, 0x01, 0x00, 0x03, 0x7c, 0xd2, 0xf6, 0x7b, 0x9a,
            0x0c, 0xca, 0xe1, 0xe4,
        ]),
        BIOMECHANICS_HUMANOID_V2_SHOULDER_HALF_WIDTH_MICROMETRES,
        1_000,
        1,
        LEG_COLLISIONS_V3,
    )
}

#[must_use]
pub fn biomechanics_humanoid_body_schema_v3() -> BodySchemaV2 {
    biomechanics_humanoid_body_schema(
        "nextengine.body.humanoid-biomechanics-raja-1700.v3",
        3,
        domain_hash(b"nextengine.source.raja-1700.self-clearance-solver-projection.v3"),
        BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES,
        BIOMECHANICS_HUMANOID_V3_CARRIER_MASS_MICROKILOGRAMS,
        BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
        LEG_COLLISIONS_V3,
    )
}

#[must_use]
pub fn biomechanics_humanoid_body_schema_v4() -> BodySchemaV2 {
    biomechanics_humanoid_body_schema(
        "nextengine.body.humanoid-biomechanics-raja-1700.v4",
        4,
        domain_hash(b"nextengine.source.raja-1700.gait-clearance-collider-projection.v4"),
        BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES,
        BIOMECHANICS_HUMANOID_V3_CARRIER_MASS_MICROKILOGRAMS,
        BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
        LEG_COLLISIONS_V4,
    )
}

fn biomechanics_humanoid_body_schema(
    schema_id: &str,
    schema_revision: u32,
    source_provenance_hash: ContentHash,
    shoulder_half_width_micrometres: i64,
    carrier_mass_microkilograms: u64,
    carrier_inertia_microkilogram_metre_squared: i64,
    leg_collisions: LegCollisionProjection,
) -> BodySchemaV2 {
    let bodies = BODY_SPECS
        .into_iter()
        .map(|spec| {
            body_from_spec(
                spec,
                shoulder_half_width_micrometres,
                carrier_mass_microkilograms,
                carrier_inertia_microkilogram_metre_squared,
                leg_collisions.thigh_half_width_micrometres,
                leg_collisions.shank_half_extent_micrometres,
                leg_collisions.knee_radius_micrometres,
            )
        })
        .collect();
    let joints = JOINT_SPECS
        .into_iter()
        .map(|spec| joint_from_spec(spec, shoulder_half_width_micrometres))
        .collect::<Vec<_>>();
    let actuators = JOINT_SPECS.into_iter().map(actuator_from_spec).collect();
    let mut collision_exclusions = JOINT_SPECS
        .iter()
        .map(|joint| BodyCollisionExclusionV2 {
            first_body_id: body_id(joint.parent),
            second_body_id: body_id(joint.child),
        })
        .collect::<Vec<_>>();
    for (first, second) in [
        ("pelvis", "left-hip-yaw"),
        ("pelvis", "right-hip-yaw"),
        ("left-knee", "left-ankle-roll"),
        ("right-knee", "right-ankle-roll"),
        ("pelvis", "torso-yaw"),
        ("torso-yaw", "left-shoulder-yaw"),
        ("torso-yaw", "right-shoulder-yaw"),
    ] {
        collision_exclusions.push(BodyCollisionExclusionV2 {
            first_body_id: body_id(first),
            second_body_id: body_id(second),
        });
    }
    let schema = BodySchemaV2 {
        schema_version: BODY_SCHEMA_VERSION_V2,
        schema_id: id(schema_id),
        schema_revision,
        family_id: id("policy-family.humanoid"),
        coordinate_profile_hash: domain_hash(b"nextengine.coordinate.y-up-x-right-z-forward.v1"),
        source_provenance_hash,
        solver_projection_profile_hash: domain_hash(if schema_revision >= 3 {
            b"nextengine.solver-projection.humanoid-biomechanics-raja-1700.v2"
        } else {
            b"nextengine.solver-projection.humanoid-biomechanics-raja-1700.v1"
        }),
        rotation_norm_tolerance_q2_60: 1,
        bodies,
        mass_projection_groups: mass_projection_groups(),
        joints,
        actuators,
        effectors: effectors(),
        symmetry_pairs: symmetry_pairs(),
        collision_exclusions,
        capability_ids: vec![
            id("capability.brace"),
            id("capability.get-up"),
            id("capability.reference-motion-tracking"),
            id("capability.velocity-command"),
        ],
    }
    .canonicalize();
    schema
        .validate()
        .expect("the frozen TRAIN-1 biomechanics profile must remain valid");
    schema
}

fn body_from_spec(
    spec: BodySpec,
    shoulder_half_width_micrometres: i64,
    carrier_mass_microkilograms: u64,
    carrier_inertia_microkilogram_metre_squared: i64,
    thigh_half_width_micrometres: i64,
    shank_half_width_micrometres: i64,
    knee_radius_micrometres: i64,
) -> BodyDefinitionV2 {
    let spec = projected_body_spec(
        spec,
        carrier_mass_microkilograms,
        carrier_inertia_microkilogram_metre_squared,
    );
    BodyDefinitionV2 {
        body_id: body_id(spec.name),
        parent_body_id: spec.parent.map(body_id),
        local_bind_pose: pose(body_bind(spec, shoulder_half_width_micrometres)),
        semantic_role: spec.role,
        mapping_group_id: map_id(spec.mapping),
        mass_microkilograms: spec.mass,
        center_of_mass_micrometres: spec.com,
        inertia_tensor_microkilogram_metre_squared: spec.inertia,
        solver_mass_microkilograms: spec.mass,
        solver_center_of_mass_micrometres: spec.com,
        solver_principal_inertia_microkilogram_metre_squared: [
            spec.inertia[0] as u64,
            spec.inertia[3] as u64,
            spec.inertia[5] as u64,
        ],
        solver_principal_frame: BodyPoseV2::default(),
        solver_tensor_error_max_microkilogram_metre_squared: spec.solver_error,
        colliders: colliders_for(
            spec.name,
            thigh_half_width_micrometres,
            shank_half_width_micrometres,
            knee_radius_micrometres,
        ),
    }
}

fn projected_body_spec(
    mut spec: BodySpec,
    carrier_mass_microkilograms: u64,
    carrier_inertia_microkilogram_metre_squared: i64,
) -> BodySpec {
    const V2_CARRIER_MASS: u64 = 1_000;
    const V2_CARRIER_INERTIA: i64 = 1;
    if spec.role == BodySemanticRoleV2::NonCollidingCarrier {
        spec.mass = carrier_mass_microkilograms;
        spec.inertia = [
            carrier_inertia_microkilogram_metre_squared,
            0,
            0,
            carrier_inertia_microkilogram_metre_squared,
            0,
            carrier_inertia_microkilogram_metre_squared,
        ];
        return spec;
    }
    if matches!(
        spec.name,
        "torso-yaw" | "right-hip-yaw" | "left-hip-yaw" | "right-shoulder-yaw" | "left-shoulder-yaw"
    ) {
        let mass_delta = carrier_mass_microkilograms - V2_CARRIER_MASS;
        spec.mass -= 2 * mass_delta;
        let inertia_delta = carrier_inertia_microkilogram_metre_squared - V2_CARRIER_INERTIA;
        for index in [0, 3, 5] {
            spec.inertia[index] -= 2 * inertia_delta;
        }
    }
    spec
}

fn colliders_for(
    body: &str,
    thigh_half_width_micrometres: i64,
    shank_half_width_micrometres: i64,
    knee_radius_micrometres: i64,
) -> Vec<BodyColliderDefinitionV2> {
    let rows: &[(&str, PhysicsGeometryV1, [i64; 3], BodyContactRoleV2)] = match body {
        "pelvis" => &[(
            "pelvis",
            box_geometry([145000, 100000, 110000]),
            [0, -20000, -30000],
            BodyContactRoleV2::PelvisGround,
        )],
        "torso-yaw" => &[
            (
                "torso",
                box_geometry([160000, 220000, 120000]),
                [0, 300000, -20000],
                BodyContactRoleV2::TorsoGround,
            ),
            (
                "head",
                PhysicsGeometryV1::Sphere {
                    radius_micrometres: 105000,
                },
                [0, 570000, 0],
                BodyContactRoleV2::HeadGround,
            ),
        ],
        "right-hip-yaw" => &[(
            "right-thigh",
            box_geometry([thigh_half_width_micrometres, 190000, 75000]),
            [0, -195000, 0],
            BodyContactRoleV2::ThighGround,
        )],
        "left-hip-yaw" => &[(
            "left-thigh",
            box_geometry([thigh_half_width_micrometres, 190000, 75000]),
            [0, -195000, 0],
            BodyContactRoleV2::ThighGround,
        )],
        "right-knee" => &[
            (
                "right-shank",
                box_geometry([
                    shank_half_width_micrometres,
                    185000,
                    shank_half_width_micrometres,
                ]),
                [0, -190000, 0],
                BodyContactRoleV2::ShankGround,
            ),
            (
                "right-knee",
                PhysicsGeometryV1::Sphere {
                    radius_micrometres: knee_radius_micrometres,
                },
                [0, 0, 20000],
                BodyContactRoleV2::KneeGround,
            ),
        ],
        "left-knee" => &[
            (
                "left-shank",
                box_geometry([
                    shank_half_width_micrometres,
                    185000,
                    shank_half_width_micrometres,
                ]),
                [0, -190000, 0],
                BodyContactRoleV2::ShankGround,
            ),
            (
                "left-knee",
                PhysicsGeometryV1::Sphere {
                    radius_micrometres: knee_radius_micrometres,
                },
                [0, 0, 20000],
                BodyContactRoleV2::KneeGround,
            ),
        ],
        "right-ankle-pitch" => &[(
            "right-talus",
            PhysicsGeometryV1::Sphere {
                radius_micrometres: 40000,
            },
            [0, 0, 0],
            BodyContactRoleV2::AnkleGround,
        )],
        "left-ankle-pitch" => &[(
            "left-talus",
            PhysicsGeometryV1::Sphere {
                radius_micrometres: 40000,
            },
            [0, 0, 0],
            BodyContactRoleV2::AnkleGround,
        )],
        "right-ankle-roll" => &[(
            "right-foot",
            box_geometry([55000, 30000, 130000]),
            [0, 11365, 80000],
            BodyContactRoleV2::FootWithSoleFeature,
        )],
        "left-ankle-roll" => &[(
            "left-foot",
            box_geometry([55000, 30000, 130000]),
            [0, 11365, 80000],
            BodyContactRoleV2::FootWithSoleFeature,
        )],
        "right-shoulder-yaw" => &[(
            "right-upper-arm",
            box_geometry([55000, 135000, 55000]),
            [0, -140000, 0],
            BodyContactRoleV2::UpperArmGround,
        )],
        "left-shoulder-yaw" => &[(
            "left-upper-arm",
            box_geometry([55000, 135000, 55000]),
            [0, -140000, 0],
            BodyContactRoleV2::UpperArmGround,
        )],
        "right-elbow" => &[
            (
                "right-forearm",
                box_geometry([35000, 110000, 40000]),
                [20000, -120000, 0],
                BodyContactRoleV2::ForearmGround,
            ),
            (
                "right-hand",
                box_geometry([45000, 65000, 30000]),
                [40000, -305000, -15000],
                BodyContactRoleV2::HandGround,
            ),
        ],
        "left-elbow" => &[
            (
                "left-forearm",
                box_geometry([35000, 110000, 40000]),
                [-20000, -120000, 0],
                BodyContactRoleV2::ForearmGround,
            ),
            (
                "left-hand",
                box_geometry([45000, 65000, 30000]),
                [-40000, -305000, -15000],
                BodyContactRoleV2::HandGround,
            ),
        ],
        _ => &[],
    };
    rows.iter()
        .map(|(name, geometry, centre, role)| BodyColliderDefinitionV2 {
            collider_id: id(&format!("collider.{name}")),
            local_pose: pose(*centre),
            geometry: geometry.clone(),
            material_id: id(if *role == BodyContactRoleV2::FootWithSoleFeature {
                "physics-material.humanoid-sole.v1"
            } else {
                "physics-material.humanoid-body.v1"
            }),
            collision_layer: 10,
            collision_mask: HUMANOID_MASK,
            participation: PhysicsParticipationV1::Solid,
            contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
            contact_role: *role,
        })
        .collect()
}

fn joint_from_spec(spec: JointSpec, shoulder_half_width_micrometres: i64) -> BodyJointDefinitionV2 {
    let bind = BODY_SPECS
        .iter()
        .find(|body| body.name == spec.child)
        .expect("joint child row")
        .to_owned();
    BodyJointDefinitionV2 {
        joint_id: joint_id(spec.name),
        parent_body_id: body_id(spec.parent),
        child_body_id: body_id(spec.child),
        anatomical_semantic_id: id(&format!("anatomical-joint.{}", spec.semantic)),
        parent_frame: pose(body_bind(bind, shoulder_half_width_micrometres)),
        child_frame: BodyPoseV2::default(),
        axis_q1_30: spec.axis,
        hard_minimum_microradians: spec.hard[0],
        hard_maximum_microradians: spec.hard[1],
        soft_minimum_microradians: spec.soft[0],
        soft_maximum_microradians: spec.soft[1],
        neutral_position_microradians: 0,
        maximum_velocity_microradians_per_second: spec.maximum_velocity,
    }
}

fn body_bind(spec: BodySpec, shoulder_half_width_micrometres: i64) -> [i64; 3] {
    match spec.name {
        "right-shoulder-pitch" => [shoulder_half_width_micrometres, spec.bind[1], spec.bind[2]],
        "left-shoulder-pitch" => [-shoulder_half_width_micrometres, spec.bind[1], spec.bind[2]],
        _ => spec.bind,
    }
}

fn actuator_from_spec(spec: JointSpec) -> BodyActuatorDefinitionV2 {
    let profile = actuator_profile(spec.actuator_profile);
    BodyActuatorDefinitionV2 {
        actuator_id: id(&format!("actuator.{}", spec.name)),
        joint_id: joint_id(spec.name),
        stiffness_q16: profile.kp,
        damping_q16: profile.kd,
        minimum_effort_micronewton_metres: -profile.effort,
        maximum_effort_micronewton_metres: profile.effort,
        maximum_effort_rate_micronewton_metres_per_second: profile.rate,
        maximum_power_microwatts: profile.power,
        maximum_positive_work_microjoules_per_motor_tick: profile.work,
        residual_scale_microradians: profile.residual,
        minimum_target_delta_microradians_per_motor_tick: -profile.delta,
        maximum_target_delta_microradians_per_motor_tick: profile.delta,
    }
}

fn actuator_profile(name: &str) -> ActuatorProfile {
    match name {
        "spine" => actuator(
            19660800, 1966080, 180000000, 1800000000, 600000000, 10000000, 150000, 80000,
        ),
        "hip-pitch" => actuator(
            29491200, 2949120, 320000000, 3200000000, 1200000000, 20000000, 200000, 100000,
        ),
        "hip-roll" => actuator(
            22937600, 2293760, 220000000, 2200000000, 800000000, 13333333, 150000, 80000,
        ),
        "hip-yaw" => actuator(
            19660800, 1966080, 180000000, 1800000000, 700000000, 11666667, 150000, 80000,
        ),
        "knee" => actuator(
            32768000, 3276800, 350000000, 3500000000, 1200000000, 20000000, 220000, 100000,
        ),
        "ankle-pitch" => actuator(
            26214400, 2621440, 220000000, 2200000000, 800000000, 13333333, 150000, 80000,
        ),
        "ankle-roll" => actuator(
            19660800, 1966080, 140000000, 1400000000, 500000000, 8333333, 120000, 60000,
        ),
        "shoulder-pitch" => actuator(
            11796480, 1179648, 120000000, 1200000000, 400000000, 6666667, 250000, 120000,
        ),
        "shoulder-roll" => actuator(
            10485760, 1048576, 100000000, 1000000000, 350000000, 5833333, 200000, 100000,
        ),
        "shoulder-yaw" => actuator(
            9175040, 917504, 80000000, 800000000, 300000000, 5000000, 200000, 100000,
        ),
        "elbow" => actuator(
            9175040, 917504, 90000000, 900000000, 300000000, 5000000, 250000, 120000,
        ),
        _ => unreachable!("fixed profile references only declared actuator groups"),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "fixed actuator rows stay auditable as one table"
)]
const fn actuator(
    kp: u64,
    kd: u64,
    effort: i64,
    rate: u64,
    power: u64,
    work: u64,
    residual: u64,
    delta: i64,
) -> ActuatorProfile {
    ActuatorProfile {
        kp,
        kd,
        effort,
        rate,
        power,
        work,
        residual,
        delta,
    }
}

fn mass_projection_groups() -> Vec<BodyMassProjectionGroupV2> {
    vec![
        mapping(
            "pelvis",
            11777000,
            [0, 0, -70700],
            [57900, 0, 0, 87100, 0, 102800],
            &[("pelvis", [0, 0, 0])],
        ),
        mapping(
            "torso",
            26826600,
            [0, 320000, -30000],
            [1431400, 0, 0, 755500, 0, 1474500],
            &[
                ("torso-pitch", [0, 0, 0]),
                ("torso-roll", [0, 0, 0]),
                ("torso-yaw", [0, 0, 0]),
            ],
        ),
        mapping(
            "right-thigh",
            9301400,
            [0, -170000, 0],
            [141200, 0, 0, 35100, 0, 133900],
            &[
                ("right-hip-pitch", [0, 0, 0]),
                ("right-hip-roll", [0, 0, 0]),
                ("right-hip-yaw", [0, 0, 0]),
            ],
        ),
        mapping(
            "left-thigh",
            9301400,
            [0, -170000, 0],
            [141200, 0, 0, 35100, 0, 133900],
            &[
                ("left-hip-pitch", [0, 0, 0]),
                ("left-hip-roll", [0, 0, 0]),
                ("left-hip-yaw", [0, 0, 0]),
            ],
        ),
        mapping(
            "right-knee",
            3793700,
            [1451, -178403, 7947],
            [54816, 26, -1, 5117, 111, 54103],
            &[("right-knee", [0, 0, 0])],
        ),
        mapping(
            "left-knee",
            3793700,
            [-1451, -178403, 7947],
            [54816, -26, 1, 5117, 111, 54103],
            &[("left-knee", [0, 0, 0])],
        ),
        mapping(
            "right-ankle",
            1566600,
            [5144, -14782, 63639],
            [9055, -71, 309, 7959, 644, 2701],
            &[
                ("right-ankle-pitch", [0, 0, 0]),
                ("right-ankle-roll", [7920, -41950, -48770]),
            ],
        ),
        mapping(
            "left-ankle",
            1566600,
            [-5144, -14782, 63639],
            [9055, 71, -309, 7959, 644, 2701],
            &[
                ("left-ankle-pitch", [0, 0, 0]),
                ("left-ankle-roll", [-7920, -41950, -48770]),
            ],
        ),
        mapping(
            "right-arm",
            3705000,
            [4847, -300265, 2913],
            [116303, 4748, 97, 6554, 987, 114337],
            &[
                ("right-shoulder-pitch", [0, 0, 0]),
                ("right-shoulder-roll", [0, 0, 0]),
                ("right-shoulder-yaw", [0, 0, 0]),
                ("right-elbow", [-9595, -286273, 13144]),
            ],
        ),
        mapping(
            "left-arm",
            3705000,
            [-4847, -300265, 2913],
            [116303, -4748, -97, 6554, 987, 114337],
            &[
                ("left-shoulder-pitch", [0, 0, 0]),
                ("left-shoulder-roll", [0, 0, 0]),
                ("left-shoulder-yaw", [0, 0, 0]),
                ("left-elbow", [9595, -286273, 13144]),
            ],
        ),
    ]
}

fn mapping(
    name: &str,
    mass: u64,
    com: [i64; 3],
    inertia: [i64; 6],
    members: &[(&str, [i64; 3])],
) -> BodyMassProjectionGroupV2 {
    BodyMassProjectionGroupV2 {
        mapping_group_id: map_id(name),
        source_mass_microkilograms: mass,
        source_center_of_mass_micrometres: com,
        source_inertia_tensor_microkilogram_metre_squared: inertia,
        maximum_first_moment_error_microkilogram_micrometres: 4_000_000,
        maximum_inertia_error_microkilogram_metre_squared: 6,
        members: members
            .iter()
            .map(|(name, origin)| BodyMassProjectionMemberV2 {
                body_id: body_id(name),
                body_origin_in_group_micrometres: *origin,
            })
            .collect(),
    }
}

fn effectors() -> Vec<BodyEffectorDefinitionV2> {
    [
        (
            "left-heel",
            "left-ankle-roll",
            [-0, -18635, -35000],
            "locomotion-sole-support",
        ),
        (
            "left-forefoot",
            "left-ankle-roll",
            [0, -18635, 180000],
            "locomotion-sole-support",
        ),
        (
            "right-heel",
            "right-ankle-roll",
            [0, -18635, -35000],
            "locomotion-sole-support",
        ),
        (
            "right-forefoot",
            "right-ankle-roll",
            [0, -18635, 180000],
            "locomotion-sole-support",
        ),
        (
            "left-palm",
            "left-elbow",
            [-40000, -305000, -15000],
            "recovery-hand-support",
        ),
        (
            "right-palm",
            "right-elbow",
            [40000, -305000, -15000],
            "recovery-hand-support",
        ),
    ]
    .into_iter()
    .map(|(name, body, translation, role)| BodyEffectorDefinitionV2 {
        effector_id: id(&format!("effector.{name}")),
        body_id: body_id(body),
        local_pose: pose(translation),
        semantic_role_id: id(&format!("contact-role.{role}")),
    })
    .collect()
}

fn symmetry_pairs() -> Vec<BodySymmetryPairV2> {
    let bilateral = [
        "hip-pitch",
        "hip-roll",
        "hip-yaw",
        "knee",
        "ankle-pitch",
        "ankle-roll",
        "shoulder-pitch",
        "shoulder-roll",
        "shoulder-yaw",
        "elbow",
    ];
    let mut pairs = Vec::new();
    for name in bilateral {
        for prefix in ["body", "joint", "actuator"] {
            pairs.push(BodySymmetryPairV2 {
                left_id: id(&format!("{prefix}.left-{name}")),
                right_id: id(&format!("{prefix}.right-{name}")),
                value_rule: BodyMirrorValueRuleV2::Preserved,
            });
        }
    }
    for name in ["heel", "forefoot", "palm"] {
        pairs.push(BodySymmetryPairV2 {
            left_id: id(&format!("effector.left-{name}")),
            right_id: id(&format!("effector.right-{name}")),
            value_rule: BodyMirrorValueRuleV2::Preserved,
        });
    }
    pairs
}

const fn box_geometry(half_extents_micrometres: [i64; 3]) -> PhysicsGeometryV1 {
    PhysicsGeometryV1::Box {
        half_extents_micrometres,
    }
}

fn pose(translation_micrometres: [i64; 3]) -> BodyPoseV2 {
    BodyPoseV2 {
        translation_micrometres,
        ..BodyPoseV2::default()
    }
}

fn body_id(name: &str) -> SchemaId {
    id(&format!("body.{name}"))
}
fn joint_id(name: &str) -> SchemaId {
    id(&format!("joint.{name}"))
}
fn map_id(name: &str) -> SchemaId {
    id(&format!("map.{name}"))
}
fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned biomechanics identifier")
}
fn domain_hash(value: &[u8]) -> ContentHash {
    content_hash_from_bytes(sha256(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn frozen_profile_has_exact_topology_mass_and_geometry_counts() {
        let schema = biomechanics_humanoid_body_schema_v2();
        assert_eq!(schema.bodies.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
        assert_eq!(schema.joints.len(), BIOMECHANICS_HUMANOID_DOF);
        assert_eq!(schema.actuators.len(), BIOMECHANICS_HUMANOID_DOF);
        assert_eq!(
            schema
                .bodies
                .iter()
                .map(|body| body.mass_microkilograms)
                .sum::<u64>(),
            BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS
        );
        assert_eq!(
            schema
                .bodies
                .iter()
                .map(|body| body.colliders.len())
                .sum::<usize>(),
            BIOMECHANICS_HUMANOID_COLLIDER_COUNT
        );
        assert_eq!(
            schema
                .bodies
                .iter()
                .map(|body| body.solver_mass_microkilograms)
                .sum::<u64>(),
            BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS
        );
        assert!(schema.bodies.iter().all(|body| {
            (body.semantic_role == BodySemanticRoleV2::NonCollidingCarrier)
                == body.colliders.is_empty()
        }));
    }

    #[test]
    fn v3_preserves_v2_and_adds_robust_neutral_forearm_clearance() {
        let v2 = biomechanics_humanoid_body_schema_v2();
        let v3 = biomechanics_humanoid_body_schema_v3();
        assert_eq!(v2.schema_revision, 2);
        assert_eq!(v3.schema_revision, 3);
        let v2_hash = crate::CompiledBodySchemaV2::compile(
            &v2,
            next_contracts::ids::PersistentId::from_bytes([0; 16]),
        )
        .expect("compile v2")
        .body_schema_hash;
        let v3_hash = crate::CompiledBodySchemaV2::compile(
            &v3,
            next_contracts::ids::PersistentId::from_bytes([0; 16]),
        )
        .expect("compile v3")
        .body_schema_hash;
        assert_ne!(v2_hash, v3_hash);
        assert_eq!(
            v3.bodies
                .iter()
                .map(|body| body.mass_microkilograms)
                .sum::<u64>(),
            BIOMECHANICS_HUMANOID_TOTAL_MASS_MICROKILOGRAMS
        );
        assert!(
            v3.bodies
                .iter()
                .filter(|body| body.semantic_role == BodySemanticRoleV2::NonCollidingCarrier)
                .all(|body| {
                    body.mass_microkilograms
                        == BIOMECHANICS_HUMANOID_V3_CARRIER_MASS_MICROKILOGRAMS
                        && body.inertia_tensor_microkilogram_metre_squared
                            == [
                                BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
                                0,
                                0,
                                BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
                                0,
                                BIOMECHANICS_HUMANOID_V3_CARRIER_INERTIA_MICROKILOGRAM_METRE_SQUARED,
                            ]
                })
        );

        for (side, sign) in [("right", 1_i64), ("left", -1_i64)] {
            let shoulder_id = format!("body.{side}-shoulder-pitch");
            let old_shoulder = v2
                .bodies
                .iter()
                .find(|body| body.body_id.as_str() == shoulder_id)
                .expect("v2 shoulder");
            let new_shoulder = v3
                .bodies
                .iter()
                .find(|body| body.body_id.as_str() == shoulder_id)
                .expect("v3 shoulder");
            assert_eq!(
                old_shoulder.local_bind_pose.translation_micrometres[0],
                sign * BIOMECHANICS_HUMANOID_V2_SHOULDER_HALF_WIDTH_MICROMETRES
            );
            assert_eq!(
                new_shoulder.local_bind_pose.translation_micrometres[0],
                sign * BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES
            );
        }

        let pelvis_half_width = 145_000_i64;
        let forearm_half_width = 35_000_i64;
        let elbow_bind_inward = 9_595_i64;
        let forearm_local_outward = 20_000_i64;
        let clearance = BIOMECHANICS_HUMANOID_V3_SHOULDER_HALF_WIDTH_MICROMETRES
            + forearm_local_outward
            - elbow_bind_inward
            - forearm_half_width
            - pelvis_half_width;
        assert_eq!(clearance, 45_405);
        assert!(clearance > 40_000);
    }

    #[test]
    fn v4_preserves_v3_dynamics_and_adds_leg_collision_proxy_clearance() {
        let v3 = biomechanics_humanoid_body_schema_v3();
        let v4 = biomechanics_humanoid_body_schema_v4();
        assert_eq!(v3.schema_revision, 3);
        assert_eq!(v4.schema_revision, 4);
        assert_eq!(v3.joints, v4.joints);
        assert_eq!(v3.actuators, v4.actuators);
        assert_eq!(v3.collision_exclusions, v4.collision_exclusions);
        assert_eq!(v3.mass_projection_groups, v4.mass_projection_groups);
        assert_eq!(v3.effectors, v4.effectors);
        assert_eq!(v3.symmetry_pairs, v4.symmetry_pairs);
        assert_eq!(v3.capability_ids, v4.capability_ids);
        for (old, new) in v3.bodies.iter().zip(&v4.bodies) {
            assert_eq!(old.body_id, new.body_id);
            assert_eq!(old.local_bind_pose, new.local_bind_pose);
            assert_eq!(old.mass_microkilograms, new.mass_microkilograms);
            assert_eq!(
                old.center_of_mass_micrometres,
                new.center_of_mass_micrometres
            );
            assert_eq!(
                old.inertia_tensor_microkilogram_metre_squared,
                new.inertia_tensor_microkilogram_metre_squared
            );
            if old.body_id.as_str().ends_with("hip-yaw") || old.body_id.as_str().ends_with("knee") {
                assert_ne!(old.colliders, new.colliders);
            } else {
                assert_eq!(old.colliders, new.colliders);
            }
            let mut restored_colliders = new.clone();
            restored_colliders.colliders.clone_from(&old.colliders);
            assert_eq!(old, &restored_colliders);
        }

        let hip_half_spacing = 77_260_i64;
        let clearance =
            2 * (hip_half_spacing - BIOMECHANICS_HUMANOID_V4_THIGH_HALF_WIDTH_MICROMETRES);
        assert_eq!(clearance, 44_520);
        assert!(clearance > 40_000);
        let knee_half_spacing = 77_260_i64 - 2_750;
        let shank_clearance =
            2 * (knee_half_spacing - BIOMECHANICS_HUMANOID_V4_SHANK_HALF_WIDTH_MICROMETRES);
        let knee_clearance =
            2 * (knee_half_spacing - BIOMECHANICS_HUMANOID_V4_KNEE_RADIUS_MICROMETRES);
        assert_eq!(shank_clearance, 59_020);
        assert_eq!(knee_clearance, 49_020);
        assert!(shank_clearance > 40_000);
        assert!(knee_clearance > 40_000);
    }

    #[test]
    fn compound_joint_source_adjacencies_are_collision_excluded() {
        let schema = biomechanics_humanoid_body_schema_v2();
        assert_eq!(schema.schema_revision, 2);
        let exclusions = schema
            .collision_exclusions
            .iter()
            .map(|pair| (pair.first_body_id.clone(), pair.second_body_id.clone()))
            .collect::<BTreeSet<_>>();
        for (first, second) in [
            ("left-knee", "left-ankle-roll"),
            ("right-knee", "right-ankle-roll"),
            ("pelvis", "torso-yaw"),
        ] {
            let pair = if body_id(first) < body_id(second) {
                (body_id(first), body_id(second))
            } else {
                (body_id(second), body_id(first))
            };
            assert!(
                exclusions.contains(&pair),
                "missing source adjacency {pair:?}"
            );
        }
    }

    #[test]
    fn bilateral_joint_rows_preserve_limits_and_reflect_roll_yaw_axes() {
        let schema = biomechanics_humanoid_body_schema_v2();
        let joints = schema
            .joints
            .iter()
            .map(|joint| (joint.joint_id.as_str(), joint))
            .collect::<BTreeMap<_, _>>();
        for name in [
            "hip-pitch",
            "hip-roll",
            "hip-yaw",
            "knee",
            "ankle-pitch",
            "ankle-roll",
            "shoulder-pitch",
            "shoulder-roll",
            "shoulder-yaw",
            "elbow",
        ] {
            let left = joints[format!("joint.left-{name}").as_str()];
            let right = joints[format!("joint.right-{name}").as_str()];
            assert_eq!(
                [
                    left.hard_minimum_microradians,
                    left.hard_maximum_microradians
                ],
                [
                    right.hard_minimum_microradians,
                    right.hard_maximum_microradians
                ]
            );
            assert_eq!(
                [
                    left.soft_minimum_microradians,
                    left.soft_maximum_microradians
                ],
                [
                    right.soft_minimum_microradians,
                    right.soft_maximum_microradians
                ]
            );
            let expected_left_axis = if name.ends_with("roll") || name.ends_with("yaw") {
                right.axis_q1_30.map(|value| -value)
            } else {
                right.axis_q1_30
            };
            assert_eq!(left.axis_q1_30, expected_left_axis);
        }
        for name in ["left-knee", "right-knee", "left-elbow", "right-elbow"] {
            assert_eq!(
                joints[format!("joint.{name}").as_str()].hard_minimum_microradians,
                0
            );
        }
    }

    #[test]
    fn neutral_geometry_has_exact_stature_soles_and_no_nonexcluded_overlap() {
        let schema = biomechanics_humanoid_body_schema_v2();
        let bodies = schema
            .bodies
            .iter()
            .map(|body| (body.body_id.clone(), body))
            .collect::<BTreeMap<_, _>>();
        let mut global = BTreeMap::new();
        fn resolve(
            id: &SchemaId,
            bodies: &BTreeMap<SchemaId, &BodyDefinitionV2>,
            global: &mut BTreeMap<SchemaId, [i64; 3]>,
        ) -> [i64; 3] {
            if let Some(value) = global.get(id) {
                return *value;
            }
            let body = bodies[id];
            let parent = body
                .parent_body_id
                .as_ref()
                .map_or([0; 3], |parent| resolve(parent, bodies, global));
            let value = std::array::from_fn(|axis| {
                parent[axis] + body.local_bind_pose.translation_micrometres[axis]
            });
            global.insert(id.clone(), value);
            value
        }
        for id in bodies.keys() {
            resolve(id, &bodies, &mut global);
        }
        let torso = bodies[&body_id("torso-yaw")];
        let head = torso
            .colliders
            .iter()
            .find(|collider| collider.collider_id == id("collider.head"))
            .expect("head");
        let PhysicsGeometryV1::Sphere { radius_micrometres } = head.geometry else {
            panic!("head sphere")
        };
        assert_eq!(
            global[&torso.body_id][1]
                + head.local_pose.translation_micrometres[1]
                + radius_micrometres,
            1_700_000
        );
        for side in ["left", "right"] {
            let foot = bodies[&body_id(&format!("{side}-ankle-roll"))];
            let collider = &foot.colliders[0];
            let PhysicsGeometryV1::Box {
                half_extents_micrometres,
            } = collider.geometry
            else {
                panic!("foot box")
            };
            assert_eq!(
                global[&foot.body_id][1] + collider.local_pose.translation_micrometres[1]
                    - half_extents_micrometres[1],
                0
            );
        }

        let exclusions = schema
            .collision_exclusions
            .iter()
            .map(|pair| (pair.first_body_id.clone(), pair.second_body_id.clone()))
            .collect::<BTreeSet<_>>();
        let mut aabbs = Vec::new();
        for body in &schema.bodies {
            for collider in &body.colliders {
                let centre: [i64; 3] = std::array::from_fn(|axis| {
                    global[&body.body_id][axis] + collider.local_pose.translation_micrometres[axis]
                });
                let half = match collider.geometry {
                    PhysicsGeometryV1::Box {
                        half_extents_micrometres,
                    } => half_extents_micrometres,
                    PhysicsGeometryV1::Sphere { radius_micrometres } => [radius_micrometres; 3],
                    _ => panic!("frozen profile geometry"),
                };
                aabbs.push((body.body_id.clone(), centre, half));
            }
        }
        for left in 0..aabbs.len() {
            for right in left + 1..aabbs.len() {
                if aabbs[left].0 == aabbs[right].0 {
                    continue;
                }
                let pair = if aabbs[left].0 < aabbs[right].0 {
                    (aabbs[left].0.clone(), aabbs[right].0.clone())
                } else {
                    (aabbs[right].0.clone(), aabbs[left].0.clone())
                };
                if exclusions.contains(&pair) {
                    continue;
                }
                let overlaps = (0..3).all(|axis| {
                    (aabbs[left].1[axis] - aabbs[right].1[axis]).abs()
                        < aabbs[left].2[axis] + aabbs[right].2[axis]
                });
                assert!(!overlaps, "unexpected neutral AABB overlap: {pair:?}");
            }
        }
    }
}
