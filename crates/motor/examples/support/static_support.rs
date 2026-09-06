//! ADR-125 diagnostic only: modeled normal support, not measured/native forces.
use next_physics_physx::CanonicalPhysXSnapshotV2;
use serde::{Deserialize, Serialize};

type V = [f64; 3];
type M = [[f64; 3]; 3];

#[derive(Deserialize)]
struct Pose {
    translation_micrometres: [i64; 3],
    rotation_q1_30: [i64; 4],
}
#[derive(Deserialize)]
struct Geometry {
    kind: String,
    half_extents_micrometres: Option<[i64; 3]>,
}
#[derive(Deserialize)]
struct Collider {
    local_translation_micrometres: [i64; 3],
    local_rotation_q1_30: [i64; 4],
    geometry: Geometry,
}
#[derive(Deserialize)]
struct Body {
    body_slot: usize,
    body_token: u64,
    body_id: String,
    parent_body_slot: Option<usize>,
    mass_microkilograms: u64,
    center_of_mass_micrometres: [i64; 3],
    colliders: Vec<Collider>,
}
#[derive(Deserialize)]
struct Joint {
    dof_ordinal: usize,
    parent_body_slot: usize,
    child_body_slot: usize,
    parent_frame: Pose,
    axis_q1_30: [i32; 3],
}
#[derive(Deserialize)]
struct Actuator {
    dof_ordinal: usize,
}
#[derive(Deserialize)]
pub struct SupportModel {
    bodies: Vec<Body>,
    joints: Vec<Joint>,
    actuators: Vec<Actuator>,
}

#[derive(Serialize)]
pub struct SupportInput {
    pub efforts_unm: Vec<i64>,
    pub com_m: V,
    pub points: Vec<SupportPoint>,
    pub enclosing_triangles: usize,
}
#[derive(Serialize)]
pub struct SupportPoint {
    pub body_slot: usize,
    pub position_m: V,
    pub weight: f64,
}

impl SupportModel {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let model: Self =
            serde_json::from_str(&next_motor::biomechanics_body_diagnostic_descriptor_json_v11()?)?;
        if model.bodies.len() != 26 || model.joints.len() != 25 || model.actuators.len() != 25 {
            return Err("support descriptor dimensions".into());
        }
        for (slot, body) in model.bodies.iter().enumerate() {
            if body.body_slot != slot
                || (slot == 0 && body.parent_body_slot.is_some())
                || (slot > 0 && body.parent_body_slot.is_none_or(|parent| parent >= slot))
            {
                return Err("support descriptor topology".into());
            }
        }
        Ok(model)
    }

    pub fn input(&self, snapshot: &CanonicalPhysXSnapshotV2) -> Result<SupportInput, &'static str> {
        if snapshot.links.len() != self.bodies.len() {
            return Err("support snapshot body count");
        }
        let mut positions = Vec::new();
        let mut rotations = Vec::new();
        let mut centres = Vec::new();
        let mut com = [0.; 3];
        let mut total_mass = 0.;
        let mut points = Vec::new();
        for body in &self.bodies {
            let mut matched = snapshot
                .links
                .iter()
                .filter(|link| link.user_token == body.body_token);
            let link = matched.next().ok_or("support missing body")?;
            if matched.next().is_some() {
                return Err("support duplicate body");
            }
            let position = metres(link.position_micrometres);
            let rotation = rotation(link.rotation_q1_30)?;
            let centre = add(
                position,
                transform(rotation, metres(body.center_of_mass_micrometres)),
            );
            let mass = body.mass_microkilograms as f64 / 1e6;
            com = add(com, scale(centre, mass));
            total_mass += mass;
            positions.push(position);
            rotations.push(rotation);
            centres.push(centre);
            if matches!(
                body.body_id.as_str(),
                "body.left-ankle-roll"
                    | "body.right-ankle-roll"
                    | "body.left-mtp"
                    | "body.right-mtp"
            ) {
                for collider in &body.colliders {
                    if collider.geometry.kind != "box" {
                        return Err("support expected foot box");
                    }
                    let half = metres(
                        collider
                            .geometry
                            .half_extents_micrometres
                            .ok_or("support box dimensions")?,
                    );
                    let local_rotation = rotation_matrix(collider.local_rotation_q1_30)?;
                    for x in [-1., 1.] {
                        for y in [-1., 1.] {
                            for z in [-1., 1.] {
                                let local = add(
                                    metres(collider.local_translation_micrometres),
                                    transform(
                                        local_rotation,
                                        [x * half[0], y * half[1], z * half[2]],
                                    ),
                                );
                                let point = add(position, transform(rotation, local));
                                if point[1].abs() <= 0.002 {
                                    points.push(SupportPoint {
                                        body_slot: body.body_slot,
                                        position_m: point,
                                        weight: 0.,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if points.len() > 32 {
            return Err("support point bound");
        }
        com = scale(com, 1. / total_mass);
        let enclosing_triangles = allocate(&mut points, com);
        let mut by_dof = vec![0_i64; self.joints.len()];
        if enclosing_triangles > 0 {
            for joint in &self.joints {
                let parent = joint.parent_body_slot;
                let anchor = add(
                    positions[parent],
                    transform(
                        rotations[parent],
                        metres(joint.parent_frame.translation_micrometres),
                    ),
                );
                let axis = transform(
                    rotations[parent],
                    transform(
                        rotation_matrix(joint.parent_frame.rotation_q1_30)?,
                        joint
                            .axis_q1_30
                            .map(|x| f64::from(x) / (1_u64 << 30) as f64),
                    ),
                );
                let mut moment = [0.; 3];
                for (slot, centre) in centres.iter().enumerate() {
                    if self.descends(slot, joint.child_body_slot) {
                        moment = add(
                            moment,
                            cross(
                                sub(*centre, anchor),
                                [
                                    0.,
                                    self.bodies[slot].mass_microkilograms as f64 / 1e6 * 9.81,
                                    0.,
                                ],
                            ),
                        );
                    }
                }
                for point in &points {
                    if self.descends(point.body_slot, joint.child_body_slot) {
                        moment = sub(
                            moment,
                            cross(
                                sub(point.position_m, anchor),
                                [0., total_mass * 9.81 * point.weight, 0.],
                            ),
                        );
                    }
                }
                let effort = (dot(axis, moment) * 1e6).round_ties_even();
                if !effort.is_finite()
                    || !(-((1_u64 << 63) as f64)..(1_u64 << 63) as f64).contains(&effort)
                {
                    return Err("support effort overflow");
                }
                by_dof[joint.dof_ordinal] = effort as i64;
            }
        }
        Ok(SupportInput {
            efforts_unm: self
                .actuators
                .iter()
                .map(|a| by_dof[a.dof_ordinal])
                .collect(),
            com_m: com,
            points,
            enclosing_triangles,
        })
    }

    fn descends(&self, mut slot: usize, ancestor: usize) -> bool {
        loop {
            if slot == ancestor {
                return true;
            }
            match self.bodies[slot].parent_body_slot {
                Some(parent) => slot = parent,
                None => return false,
            }
        }
    }
}

fn allocate(points: &mut [SupportPoint], com: V) -> usize {
    let mut count = 0;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            for k in j + 1..points.len() {
                let a = points[i].position_m;
                let b = points[j].position_m;
                let c = points[k].position_m;
                let det = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
                if det.abs() <= 1e-10 {
                    continue;
                }
                let u = ((b[2] - c[2]) * (com[0] - c[0]) + (c[0] - b[0]) * (com[2] - c[2])) / det;
                let v = ((c[2] - a[2]) * (com[0] - c[0]) + (a[0] - c[0]) * (com[2] - c[2])) / det;
                let mut weights = [u, v, 1. - u - v];
                if weights.iter().any(|w| *w < -1e-9) {
                    continue;
                }
                for w in &mut weights {
                    *w = w.max(0.);
                }
                let sum: f64 = weights.iter().sum();
                for (index, weight) in [i, j, k].into_iter().zip(weights) {
                    points[index].weight += weight / sum;
                }
                count += 1;
            }
        }
    }
    if count > 0 {
        for point in points {
            point.weight /= count as f64;
        }
    }
    count
}

fn rotation(q: [i64; 4]) -> Result<M, &'static str> {
    rotation_matrix(q)
}
fn rotation_matrix(q: [i64; 4]) -> Result<M, &'static str> {
    let q = q.map(|v| v as f64 / (1_u64 << 30) as f64);
    let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    if !(0.999..=1.001).contains(&norm) {
        return Err("support invalid rotation");
    }
    let [x, y, z, w] = q.map(|v| v / norm);
    Ok([
        [
            1. - 2. * (y * y + z * z),
            2. * (x * y - z * w),
            2. * (x * z + y * w),
        ],
        [
            2. * (x * y + z * w),
            1. - 2. * (x * x + z * z),
            2. * (y * z - x * w),
        ],
        [
            2. * (x * z - y * w),
            2. * (y * z + x * w),
            1. - 2. * (x * x + y * y),
        ],
    ])
}
fn metres(v: [i64; 3]) -> V {
    v.map(|x| x as f64 / 1e6)
}
fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: V, s: f64) -> V {
    a.map(|x| x * s)
}
fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn transform(m: M, v: V) -> V {
    m.map(|row| dot(row, v))
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_descriptor_has_four_feet_and_all_dofs() {
        let model = SupportModel::new().unwrap();
        assert_eq!(
            model
                .bodies
                .iter()
                .filter(
                    |body| body.body_id.ends_with("-ankle-roll") || body.body_id.ends_with("-mtp")
                )
                .count(),
            4
        );
        let mut dofs = model
            .actuators
            .iter()
            .map(|a| a.dof_ordinal)
            .collect::<Vec<_>>();
        dofs.sort_unstable();
        assert_eq!(dofs, (0..25).collect::<Vec<_>>());
    }
    #[test]
    fn moment_and_rotation_have_engine_signs() {
        assert_eq!(dot([1., 0., 0.], cross([0., 0., 1.], [0., 10., 0.])), -10.);
        let q = [759250125, 0, 0, 759250125];
        let rotated = transform(rotation(q).unwrap(), [0., 1., 0.]);
        assert!((rotated[2] - 1.).abs() < 1e-14 && rotated[1].abs() < 1e-14);
        assert!(rotation([0; 4]).is_err());
    }
    #[test]
    fn allocation_preserves_symmetric_force_and_moment() {
        let mut points = [-1., 1.]
            .into_iter()
            .flat_map(|x| {
                [-1., 1.].map(|z| SupportPoint {
                    body_slot: 0,
                    position_m: [x, 0., z],
                    weight: 0.,
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(allocate(&mut points, [0., 1., 0.]), 4);
        for p in &points {
            assert!((p.weight - 0.25).abs() < 1e-14);
        }
        for p in &mut points {
            p.weight = 0.;
        }
        assert_eq!(allocate(&mut points, [2., 1., 0.]), 0);
        assert!(points.iter().all(|p| p.weight == 0.));
        assert_eq!(allocate(&mut points[..2], [0.; 3]), 0);
    }
}
