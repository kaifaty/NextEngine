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

const Q30: i32 = 1 << 30;
const HUMANOID_MASK: u64 = 0x0000_0000_0000_1c01;

#[derive(Clone, Copy)]
struct BodySpec {
    name: &'static str,
    parent: Option<&'static str>,
    role: BodySemanticRoleV2,
    mapping: &'static str,
    bind: [i64; 3],
    mass: u64,
    com: [i64; 3],
    inertia: [i64; 6],
    solver_error: u64,
}

const BODY_SPECS: [BodySpec; BIOMECHANICS_HUMANOID_BODY_COUNT] = [
    BodySpec {
        name: "pelvis",
        parent: None,
        role: BodySemanticRoleV2::PhysicalRoot,
        mapping: "pelvis",
        bind: [0, 943500, 0],
        mass: 11777000,
        com: [0, 0, -70700],
        inertia: [57900, 0, 0, 87100, 0, 102800],
        solver_error: 0,
    },
    BodySpec {
        name: "torso-pitch",
        parent: Some("pelvis"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "torso",
        bind: [0, 81500, -100700],
        mass: 1000,
        com: [0, 320000, -30000],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "torso-roll",
        parent: Some("torso-pitch"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "torso",
        bind: [0, 0, 0],
        mass: 1000,
        com: [0, 320000, -30000],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "torso-yaw",
        parent: Some("torso-roll"),
        role: BodySemanticRoleV2::PhysicalTorsoHead,
        mapping: "torso",
        bind: [0, 0, 0],
        mass: 26824600,
        com: [0, 320000, -30000],
        inertia: [1431398, 0, 0, 755498, 0, 1474498],
        solver_error: 0,
    },
    BodySpec {
        name: "right-hip-pitch",
        parent: Some("pelvis"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "right-thigh",
        bind: [77260, -78490, -56276],
        mass: 1000,
        com: [0, -170000, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "right-hip-roll",
        parent: Some("right-hip-pitch"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "right-thigh",
        bind: [0, 0, 0],
        mass: 1000,
        com: [0, -170000, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "right-hip-yaw",
        parent: Some("right-hip-roll"),
        role: BodySemanticRoleV2::PhysicalThigh,
        mapping: "right-thigh",
        bind: [0, 0, 0],
        mass: 9299400,
        com: [0, -170000, 0],
        inertia: [141198, 0, 0, 35098, 0, 133898],
        solver_error: 0,
    },
    BodySpec {
        name: "right-knee",
        parent: Some("right-hip-yaw"),
        role: BodySemanticRoleV2::PhysicalShankKnee,
        mapping: "right-knee",
        bind: [-2750, -407960, -8090],
        mass: 3793700,
        com: [1451, -178403, 7947],
        inertia: [54816, 26, -1, 5117, 111, 54103],
        solver_error: 111,
    },
    BodySpec {
        name: "right-ankle-pitch",
        parent: Some("right-knee"),
        role: BodySemanticRoleV2::PhysicalTalus,
        mapping: "right-ankle",
        bind: [1485, -396465, -1910],
        mass: 100000,
        com: [0, 0, 0],
        inertia: [1000, 0, 0, 1000, 0, 1000],
        solver_error: 0,
    },
    BodySpec {
        name: "right-ankle-roll",
        parent: Some("right-ankle-pitch"),
        role: BodySemanticRoleV2::PhysicalFoot,
        mapping: "right-ankle",
        bind: [7920, -41950, -48770],
        mass: 1466600,
        com: [-2425, 26160, 116748],
        inertia: [7599, -79, 344, 6524, 544, 1675],
        solver_error: 544,
    },
    BodySpec {
        name: "left-hip-pitch",
        parent: Some("pelvis"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "left-thigh",
        bind: [-77260, -78490, -56276],
        mass: 1000,
        com: [0, -170000, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "left-hip-roll",
        parent: Some("left-hip-pitch"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "left-thigh",
        bind: [0, 0, 0],
        mass: 1000,
        com: [0, -170000, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "left-hip-yaw",
        parent: Some("left-hip-roll"),
        role: BodySemanticRoleV2::PhysicalThigh,
        mapping: "left-thigh",
        bind: [0, 0, 0],
        mass: 9299400,
        com: [0, -170000, 0],
        inertia: [141198, 0, 0, 35098, 0, 133898],
        solver_error: 0,
    },
    BodySpec {
        name: "left-knee",
        parent: Some("left-hip-yaw"),
        role: BodySemanticRoleV2::PhysicalShankKnee,
        mapping: "left-knee",
        bind: [2750, -407960, -8090],
        mass: 3793700,
        com: [-1451, -178403, 7947],
        inertia: [54816, -26, 1, 5117, 111, 54103],
        solver_error: 111,
    },
    BodySpec {
        name: "left-ankle-pitch",
        parent: Some("left-knee"),
        role: BodySemanticRoleV2::PhysicalTalus,
        mapping: "left-ankle",
        bind: [-1485, -396465, -1910],
        mass: 100000,
        com: [0, 0, 0],
        inertia: [1000, 0, 0, 1000, 0, 1000],
        solver_error: 0,
    },
    BodySpec {
        name: "left-ankle-roll",
        parent: Some("left-ankle-pitch"),
        role: BodySemanticRoleV2::PhysicalFoot,
        mapping: "left-ankle",
        bind: [-7920, -41950, -48770],
        mass: 1466600,
        com: [2425, 26160, 116748],
        inertia: [7599, 79, -344, 6524, 544, 1675],
        solver_error: 544,
    },
    BodySpec {
        name: "right-shoulder-pitch",
        parent: Some("torso-yaw"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "right-arm",
        bind: [170000, 371500, 3155],
        mass: 1000,
        com: [0, -164502, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "right-shoulder-roll",
        parent: Some("right-shoulder-pitch"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "right-arm",
        bind: [0, 0, 0],
        mass: 1000,
        com: [0, -164502, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "right-shoulder-yaw",
        parent: Some("right-shoulder-roll"),
        role: BodySemanticRoleV2::PhysicalUpperArm,
        mapping: "right-arm",
        bind: [0, 0, 0],
        mass: 2030500,
        com: [0, -164502, 0],
        inertia: [13407, 0, 0, 4119, 0, 11944],
        solver_error: 0,
    },
    BodySpec {
        name: "right-elbow",
        parent: Some("right-shoulder-yaw"),
        role: BodySemanticRoleV2::PhysicalForearmHand,
        mapping: "right-arm",
        bind: [-9595, -286273, 13144],
        mass: 1672500,
        com: [20332, -178978, -6690],
        inertia: [19867, 1785, 161, 2289, -794, 19297],
        solver_error: 1785,
    },
    BodySpec {
        name: "left-shoulder-pitch",
        parent: Some("torso-yaw"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "left-arm",
        bind: [-170000, 371500, 3155],
        mass: 1000,
        com: [0, -164502, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "left-shoulder-roll",
        parent: Some("left-shoulder-pitch"),
        role: BodySemanticRoleV2::NonCollidingCarrier,
        mapping: "left-arm",
        bind: [0, 0, 0],
        mass: 1000,
        com: [0, -164502, 0],
        inertia: [1, 0, 0, 1, 0, 1],
        solver_error: 0,
    },
    BodySpec {
        name: "left-shoulder-yaw",
        parent: Some("left-shoulder-roll"),
        role: BodySemanticRoleV2::PhysicalUpperArm,
        mapping: "left-arm",
        bind: [0, 0, 0],
        mass: 2030500,
        com: [0, -164502, 0],
        inertia: [13407, 0, 0, 4119, 0, 11944],
        solver_error: 0,
    },
    BodySpec {
        name: "left-elbow",
        parent: Some("left-shoulder-yaw"),
        role: BodySemanticRoleV2::PhysicalForearmHand,
        mapping: "left-arm",
        bind: [9595, -286273, 13144],
        mass: 1672500,
        com: [-20332, -178978, -6690],
        inertia: [19867, -1785, -161, 2289, -794, 19297],
        solver_error: 1785,
    },
];

#[derive(Clone, Copy)]
struct JointSpec {
    name: &'static str,
    parent: &'static str,
    child: &'static str,
    semantic: &'static str,
    axis: [i32; 3],
    hard: [i64; 2],
    soft: [i64; 2],
    maximum_velocity: u64,
    actuator_profile: &'static str,
}

const JOINT_SPECS: [JointSpec; BIOMECHANICS_HUMANOID_DOF] = [
    joint(
        "torso-pitch",
        "pelvis",
        "torso-pitch",
        "lumbar-flexion",
        [Q30, 0, 0],
        [-523599, 785398],
        [-349066, 610865],
        4000000,
        "spine",
    ),
    joint(
        "torso-roll",
        "torso-pitch",
        "torso-roll",
        "lumbar-side-bend",
        [0, 0, Q30],
        [-523599, 523599],
        [-349066, 349066],
        4000000,
        "spine",
    ),
    joint(
        "torso-yaw",
        "torso-roll",
        "torso-yaw",
        "lumbar-axial-rotation",
        [0, Q30, 0],
        [-785398, 785398],
        [-523599, 523599],
        4000000,
        "spine",
    ),
    joint(
        "right-hip-pitch",
        "pelvis",
        "right-hip-pitch",
        "hip-flexion",
        [Q30, 0, 0],
        [-523599, 2094395],
        [-349066, 1919862],
        8000000,
        "hip-pitch",
    ),
    joint(
        "right-hip-roll",
        "right-hip-pitch",
        "right-hip-roll",
        "hip-adduction",
        [0, 0, Q30],
        [-872665, 523599],
        [-698132, 436332],
        8000000,
        "hip-roll",
    ),
    joint(
        "right-hip-yaw",
        "right-hip-roll",
        "right-hip-yaw",
        "hip-rotation",
        [0, Q30, 0],
        [-698132, 698132],
        [-523599, 523599],
        8000000,
        "hip-yaw",
    ),
    joint(
        "right-knee",
        "right-hip-yaw",
        "right-knee",
        "knee-flexion",
        [Q30, 0, 0],
        [0, 2443461],
        [0, 2356194],
        10000000,
        "knee",
    ),
    joint(
        "right-ankle-pitch",
        "right-knee",
        "right-ankle-pitch",
        "ankle-pitch",
        [Q30, 0, 0],
        [-698132, 523599],
        [-523599, 436332],
        8000000,
        "ankle-pitch",
    ),
    joint(
        "right-ankle-roll",
        "right-ankle-pitch",
        "right-ankle-roll",
        "ankle-roll",
        [0, 0, Q30],
        [-349066, 349066],
        [-261799, 261799],
        8000000,
        "ankle-roll",
    ),
    joint(
        "left-hip-pitch",
        "pelvis",
        "left-hip-pitch",
        "hip-flexion",
        [Q30, 0, 0],
        [-523599, 2094395],
        [-349066, 1919862],
        8000000,
        "hip-pitch",
    ),
    joint(
        "left-hip-roll",
        "left-hip-pitch",
        "left-hip-roll",
        "hip-adduction",
        [0, 0, -Q30],
        [-872665, 523599],
        [-698132, 436332],
        8000000,
        "hip-roll",
    ),
    joint(
        "left-hip-yaw",
        "left-hip-roll",
        "left-hip-yaw",
        "hip-rotation",
        [0, -Q30, 0],
        [-698132, 698132],
        [-523599, 523599],
        8000000,
        "hip-yaw",
    ),
    joint(
        "left-knee",
        "left-hip-yaw",
        "left-knee",
        "knee-flexion",
        [Q30, 0, 0],
        [0, 2443461],
        [0, 2356194],
        10000000,
        "knee",
    ),
    joint(
        "left-ankle-pitch",
        "left-knee",
        "left-ankle-pitch",
        "ankle-pitch",
        [Q30, 0, 0],
        [-698132, 523599],
        [-523599, 436332],
        8000000,
        "ankle-pitch",
    ),
    joint(
        "left-ankle-roll",
        "left-ankle-pitch",
        "left-ankle-roll",
        "ankle-roll",
        [0, 0, -Q30],
        [-349066, 349066],
        [-261799, 261799],
        8000000,
        "ankle-roll",
    ),
    joint(
        "right-shoulder-pitch",
        "torso-yaw",
        "right-shoulder-pitch",
        "shoulder-flexion",
        [Q30, 0, 0],
        [-1047198, 2967060],
        [-872665, 2792527],
        10000000,
        "shoulder-pitch",
    ),
    joint(
        "right-shoulder-roll",
        "right-shoulder-pitch",
        "right-shoulder-roll",
        "shoulder-abduction",
        [0, 0, Q30],
        [-1745329, 1570796],
        [-1570796, 1396263],
        10000000,
        "shoulder-roll",
    ),
    joint(
        "right-shoulder-yaw",
        "right-shoulder-roll",
        "right-shoulder-yaw",
        "shoulder-rotation",
        [0, Q30, 0],
        [-1570796, 1570796],
        [-1308997, 1308997],
        10000000,
        "shoulder-yaw",
    ),
    joint(
        "right-elbow",
        "right-shoulder-yaw",
        "right-elbow",
        "elbow-flexion",
        [Q30, 0, 0],
        [0, 2617994],
        [0, 2530727],
        12000000,
        "elbow",
    ),
    joint(
        "left-shoulder-pitch",
        "torso-yaw",
        "left-shoulder-pitch",
        "shoulder-flexion",
        [Q30, 0, 0],
        [-1047198, 2967060],
        [-872665, 2792527],
        10000000,
        "shoulder-pitch",
    ),
    joint(
        "left-shoulder-roll",
        "left-shoulder-pitch",
        "left-shoulder-roll",
        "shoulder-abduction",
        [0, 0, -Q30],
        [-1745329, 1570796],
        [-1570796, 1396263],
        10000000,
        "shoulder-roll",
    ),
    joint(
        "left-shoulder-yaw",
        "left-shoulder-roll",
        "left-shoulder-yaw",
        "shoulder-rotation",
        [0, -Q30, 0],
        [-1570796, 1570796],
        [-1308997, 1308997],
        10000000,
        "shoulder-yaw",
    ),
    joint(
        "left-elbow",
        "left-shoulder-yaw",
        "left-elbow",
        "elbow-flexion",
        [Q30, 0, 0],
        [0, 2617994],
        [0, 2530727],
        12000000,
        "elbow",
    ),
];

#[allow(
    clippy::too_many_arguments,
    reason = "fixed profile rows stay auditable as one table"
)]
const fn joint(
    name: &'static str,
    parent: &'static str,
    child: &'static str,
    semantic: &'static str,
    axis: [i32; 3],
    hard: [i64; 2],
    soft: [i64; 2],
    maximum_velocity: u64,
    actuator_profile: &'static str,
) -> JointSpec {
    JointSpec {
        name,
        parent,
        child,
        semantic,
        axis,
        hard,
        soft,
        maximum_velocity,
        actuator_profile,
    }
}

#[derive(Clone, Copy)]
struct ActuatorProfile {
    kp: u64,
    kd: u64,
    effort: i64,
    rate: u64,
    power: u64,
    work: u64,
    residual: u64,
    delta: i64,
}

#[must_use]
pub fn biomechanics_humanoid_body_schema_v2() -> BodySchemaV2 {
    let bodies = BODY_SPECS.into_iter().map(body_from_spec).collect();
    let joints = JOINT_SPECS
        .into_iter()
        .map(joint_from_spec)
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
        schema_id: id("nextengine.body.humanoid-biomechanics-raja-1700.v2"),
        schema_revision: 1,
        family_id: id("policy-family.humanoid"),
        coordinate_profile_hash: domain_hash(b"nextengine.coordinate.y-up-x-right-z-forward.v1"),
        source_provenance_hash: content_hash_from_bytes([
            0xe6, 0xc5, 0x4e, 0x43, 0xd3, 0x71, 0x3e, 0x4e, 0x9c, 0xc6, 0xa3, 0xb2, 0x73, 0x5a,
            0x61, 0x33, 0x28, 0xc3, 0xb5, 0x1d, 0x01, 0x00, 0x03, 0x7c, 0xd2, 0xf6, 0x7b, 0x9a,
            0x0c, 0xca, 0xe1, 0xe4,
        ]),
        solver_projection_profile_hash: domain_hash(
            b"nextengine.solver-projection.humanoid-biomechanics-raja-1700.v1",
        ),
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

fn body_from_spec(spec: BodySpec) -> BodyDefinitionV2 {
    BodyDefinitionV2 {
        body_id: body_id(spec.name),
        parent_body_id: spec.parent.map(body_id),
        local_bind_pose: pose(spec.bind),
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
        colliders: colliders_for(spec.name),
    }
}

fn colliders_for(body: &str) -> Vec<BodyColliderDefinitionV2> {
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
            box_geometry([75000, 190000, 75000]),
            [0, -195000, 0],
            BodyContactRoleV2::ThighGround,
        )],
        "left-hip-yaw" => &[(
            "left-thigh",
            box_geometry([75000, 190000, 75000]),
            [0, -195000, 0],
            BodyContactRoleV2::ThighGround,
        )],
        "right-knee" => &[
            (
                "right-shank",
                box_geometry([55000, 185000, 55000]),
                [0, -190000, 0],
                BodyContactRoleV2::ShankGround,
            ),
            (
                "right-knee",
                PhysicsGeometryV1::Sphere {
                    radius_micrometres: 65000,
                },
                [0, 0, 20000],
                BodyContactRoleV2::KneeGround,
            ),
        ],
        "left-knee" => &[
            (
                "left-shank",
                box_geometry([55000, 185000, 55000]),
                [0, -190000, 0],
                BodyContactRoleV2::ShankGround,
            ),
            (
                "left-knee",
                PhysicsGeometryV1::Sphere {
                    radius_micrometres: 65000,
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

fn joint_from_spec(spec: JointSpec) -> BodyJointDefinitionV2 {
    let bind = BODY_SPECS
        .iter()
        .find(|body| body.name == spec.child)
        .expect("joint child row")
        .bind;
    BodyJointDefinitionV2 {
        joint_id: joint_id(spec.name),
        parent_body_id: body_id(spec.parent),
        child_body_id: body_id(spec.child),
        anatomical_semantic_id: id(&format!("anatomical-joint.{}", spec.semantic)),
        parent_frame: pose(bind),
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
