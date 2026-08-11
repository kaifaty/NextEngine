use next_contracts::body::{
    BODY_SCHEMA_VERSION_V1, BodyActuatorDefinitionV1, BodyColliderDefinitionV1, BodyDefinitionV1,
    BodyEffectorDefinitionV1, BodyJointDefinitionV1, BodySchemaV1, BodySymmetryPairV1,
};
use next_contracts::ids::SchemaId;
use next_contracts::physics::{PhysicsGeometryV1, PhysicsPoseV1};

pub const REFERENCE_HUMANOID_DOF: usize = 23;
pub const REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES: i64 = 1_095_000;

#[derive(Clone, Copy)]
struct LinkSpec {
    name: &'static str,
    parent: &'static str,
    translation: [i64; 3],
    mass_microkilograms: u64,
    radius_micrometres: i64,
}

const LINKS: [LinkSpec; REFERENCE_HUMANOID_DOF] = [
    LinkSpec {
        name: "torso-yaw",
        parent: "pelvis",
        translation: [0, 180_000, 0],
        mass_microkilograms: 6_000_000,
        radius_micrometres: 130_000,
    },
    LinkSpec {
        name: "torso-pitch",
        parent: "torso-yaw",
        translation: [0, 120_000, 0],
        mass_microkilograms: 6_000_000,
        radius_micrometres: 140_000,
    },
    LinkSpec {
        name: "torso-roll",
        parent: "torso-pitch",
        translation: [0, 120_000, 0],
        mass_microkilograms: 6_000_000,
        radius_micrometres: 150_000,
    },
    LinkSpec {
        name: "left-hip-yaw",
        parent: "pelvis",
        translation: [-100_000, -120_000, 0],
        mass_microkilograms: 2_000_000,
        radius_micrometres: 80_000,
    },
    LinkSpec {
        name: "left-hip-roll",
        parent: "left-hip-yaw",
        translation: [0, -50_000, 0],
        mass_microkilograms: 2_000_000,
        radius_micrometres: 75_000,
    },
    LinkSpec {
        name: "left-hip-pitch",
        parent: "left-hip-roll",
        translation: [0, -50_000, 0],
        mass_microkilograms: 4_000_000,
        radius_micrometres: 90_000,
    },
    LinkSpec {
        name: "left-knee",
        parent: "left-hip-pitch",
        translation: [0, -360_000, 0],
        mass_microkilograms: 4_000_000,
        radius_micrometres: 85_000,
    },
    LinkSpec {
        name: "left-ankle-pitch",
        parent: "left-knee",
        translation: [0, -360_000, 0],
        mass_microkilograms: 1_500_000,
        radius_micrometres: 70_000,
    },
    LinkSpec {
        name: "left-ankle-roll",
        parent: "left-ankle-pitch",
        translation: [0, -80_000, 60_000],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 75_000,
    },
    LinkSpec {
        name: "right-hip-yaw",
        parent: "pelvis",
        translation: [100_000, -120_000, 0],
        mass_microkilograms: 2_000_000,
        radius_micrometres: 80_000,
    },
    LinkSpec {
        name: "right-hip-roll",
        parent: "right-hip-yaw",
        translation: [0, -50_000, 0],
        mass_microkilograms: 2_000_000,
        radius_micrometres: 75_000,
    },
    LinkSpec {
        name: "right-hip-pitch",
        parent: "right-hip-roll",
        translation: [0, -50_000, 0],
        mass_microkilograms: 4_000_000,
        radius_micrometres: 90_000,
    },
    LinkSpec {
        name: "right-knee",
        parent: "right-hip-pitch",
        translation: [0, -360_000, 0],
        mass_microkilograms: 4_000_000,
        radius_micrometres: 85_000,
    },
    LinkSpec {
        name: "right-ankle-pitch",
        parent: "right-knee",
        translation: [0, -360_000, 0],
        mass_microkilograms: 1_500_000,
        radius_micrometres: 70_000,
    },
    LinkSpec {
        name: "right-ankle-roll",
        parent: "right-ankle-pitch",
        translation: [0, -80_000, 60_000],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 75_000,
    },
    LinkSpec {
        name: "left-shoulder-pitch",
        parent: "torso-roll",
        translation: [-250_000, 100_000, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 70_000,
    },
    LinkSpec {
        name: "left-shoulder-roll",
        parent: "left-shoulder-pitch",
        translation: [-60_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 65_000,
    },
    LinkSpec {
        name: "left-shoulder-yaw",
        parent: "left-shoulder-roll",
        translation: [-60_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 60_000,
    },
    LinkSpec {
        name: "left-elbow",
        parent: "left-shoulder-yaw",
        translation: [-280_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 60_000,
    },
    LinkSpec {
        name: "right-shoulder-pitch",
        parent: "torso-roll",
        translation: [250_000, 100_000, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 70_000,
    },
    LinkSpec {
        name: "right-shoulder-roll",
        parent: "right-shoulder-pitch",
        translation: [60_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 65_000,
    },
    LinkSpec {
        name: "right-shoulder-yaw",
        parent: "right-shoulder-roll",
        translation: [60_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 60_000,
    },
    LinkSpec {
        name: "right-elbow",
        parent: "right-shoulder-yaw",
        translation: [280_000, 0, 0],
        mass_microkilograms: 1_000_000,
        radius_micrometres: 60_000,
    },
];

#[must_use]
pub fn reference_humanoid_body_schema_v1() -> BodySchemaV1 {
    let mut bodies = Vec::with_capacity(REFERENCE_HUMANOID_DOF + 1);
    bodies.push(body(
        "pelvis",
        None,
        [0, REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES, 0],
        14_000_000,
        160_000,
    ));
    let mut joints = Vec::with_capacity(REFERENCE_HUMANOID_DOF);
    let mut actuators = Vec::with_capacity(REFERENCE_HUMANOID_DOF);
    for link in LINKS {
        bodies.push(body(
            link.name,
            Some(link.parent),
            link.translation,
            link.mass_microkilograms,
            link.radius_micrometres,
        ));
        let joint_id = id(&format!("joint.{}", link.name));
        joints.push(BodyJointDefinitionV1 {
            joint_id: joint_id.clone(),
            parent_body_id: body_id(link.parent),
            child_body_id: body_id(link.name),
            parent_frame: PhysicsPoseV1 {
                translation_micrometres: link.translation,
                ..PhysicsPoseV1::default()
            },
            child_frame: PhysicsPoseV1::default(),
            axis_q1_30: [1 << 30, 0, 0],
            limit_min_microradians: -1_500_000,
            limit_max_microradians: 1_500_000,
            maximum_velocity_microradians_per_second: 20_000_000,
        });
        actuators.push(BodyActuatorDefinitionV1 {
            actuator_id: id(&format!("actuator.{}", link.name)),
            joint_id,
            neutral_position_microradians: 0,
            stiffness_q16: 80 * 65_536,
            damping_q16: 4 * 65_536,
            maximum_effort_micronewton_metres: 150_000_000,
            maximum_effort_rate_micronewton_metres_per_second: 6_000_000_000,
        });
    }
    let schema = BodySchemaV1 {
        schema_version: BODY_SCHEMA_VERSION_V1,
        schema_id: id("nextengine.body.humanoid-stage0.v1"),
        schema_revision: 2,
        family_id: id("policy-family.humanoid"),
        bodies,
        joints,
        actuators,
        effectors: vec![
            BodyEffectorDefinitionV1 {
                effector_id: id("effector.left-foot"),
                body_id: body_id("left-ankle-roll"),
                local_pose: PhysicsPoseV1::default(),
                semantic_role_id: id("contact-role.foot"),
            },
            BodyEffectorDefinitionV1 {
                effector_id: id("effector.right-foot"),
                body_id: body_id("right-ankle-roll"),
                local_pose: PhysicsPoseV1::default(),
                semantic_role_id: id("contact-role.foot"),
            },
        ],
        symmetry_pairs: symmetry_pairs(),
        capability_ids: vec![id("capability.stand"), id("capability.velocity-command")],
    }
    .canonicalize();
    schema
        .validate()
        .expect("the engine-owned reference humanoid must remain valid");
    schema
}

fn body(
    name: &str,
    parent: Option<&str>,
    translation: [i64; 3],
    mass_microkilograms: u64,
    radius_micrometres: i64,
) -> BodyDefinitionV1 {
    let body_id = body_id(name);
    BodyDefinitionV1 {
        body_id: body_id.clone(),
        parent_body_id: parent.map(body_id_from_name),
        local_bind_pose: PhysicsPoseV1 {
            translation_micrometres: translation,
            ..PhysicsPoseV1::default()
        },
        mass_microkilograms,
        center_of_mass_micrometres: [0; 3],
        inertia_microkilogram_metre_squared: [100_000; 3],
        colliders: vec![BodyColliderDefinitionV1 {
            collider_id: id(&format!("collider.{name}")),
            local_pose: PhysicsPoseV1::default(),
            geometry: PhysicsGeometryV1::Sphere { radius_micrometres },
            material_id: id("physics-material.humanoid"),
        }],
    }
}

fn symmetry_pairs() -> Vec<BodySymmetryPairV1> {
    let names = [
        "ankle-pitch",
        "ankle-roll",
        "elbow",
        "hip-pitch",
        "hip-roll",
        "hip-yaw",
        "knee",
        "shoulder-pitch",
        "shoulder-roll",
        "shoulder-yaw",
    ];
    let mut pairs = names
        .into_iter()
        .map(|name| BodySymmetryPairV1 {
            left_id: body_id(&format!("left-{name}")),
            right_id: body_id(&format!("right-{name}")),
        })
        .collect::<Vec<_>>();
    pairs.sort();
    pairs
}

fn body_id(name: &str) -> SchemaId {
    body_id_from_name(name)
}

fn body_id_from_name(name: &str) -> SchemaId {
    id(&format!("body.{name}"))
}

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned schema identifiers are valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn reference_humanoid_is_one_free_root_plus_twenty_three_actuators() {
        let schema = reference_humanoid_body_schema_v1();
        assert_eq!(schema.bodies.len(), REFERENCE_HUMANOID_DOF + 1);
        assert_eq!(schema.joints.len(), REFERENCE_HUMANOID_DOF);
        assert_eq!(schema.actuators.len(), REFERENCE_HUMANOID_DOF);
        assert_eq!(
            schema
                .bodies
                .iter()
                .filter(|body| body.parent_body_id.is_none())
                .count(),
            1
        );
    }

    #[test]
    fn authored_standing_pose_is_tangent_to_the_ground() {
        let schema = reference_humanoid_body_schema_v1();
        let bodies = schema
            .bodies
            .iter()
            .map(|body| (body.body_id.clone(), body))
            .collect::<BTreeMap<_, _>>();
        let mut world_heights = BTreeMap::new();
        while world_heights.len() < bodies.len() {
            let before = world_heights.len();
            for (body_id, body) in &bodies {
                if world_heights.contains_key(body_id) {
                    continue;
                }
                let parent_height = match &body.parent_body_id {
                    Some(parent) => match world_heights.get(parent) {
                        Some(height) => *height,
                        None => continue,
                    },
                    None => 0,
                };
                world_heights.insert(
                    body_id.clone(),
                    parent_height + body.local_bind_pose.translation_micrometres[1],
                );
            }
            assert!(world_heights.len() > before, "body hierarchy must resolve");
        }

        let minimum = schema
            .bodies
            .iter()
            .flat_map(|body| {
                let body_height = world_heights[&body.body_id];
                body.colliders.iter().map(move |collider| {
                    let extent = match collider.geometry {
                        PhysicsGeometryV1::Sphere { radius_micrometres } => radius_micrometres,
                        _ => panic!("reference humanoid colliders must remain spheres"),
                    };
                    body_height + collider.local_pose.translation_micrometres[1] - extent
                })
            })
            .min()
            .expect("reference humanoid has colliders");
        assert_eq!(minimum, 0);
    }
}
