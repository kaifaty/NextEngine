use std::collections::BTreeMap;

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::presentation::CharacterSkinningPresentationRecordV1;
use next_contracts::render_content::{NeutralBaseSkinningProfileV1, NeutralMeshV1};

use super::{B0SkinnedVertexStreamV1, RenderDeviceError};

#[derive(Clone, Copy)]
struct RigidTransformQ30 {
    translation: [i64; 3],
    rotation: [i32; 4],
}

pub(super) fn build_skinning_stream(
    record: &CharacterSkinningPresentationRecordV1,
    profile: &NeutralBaseSkinningProfileV1,
    mesh: &NeutralMeshV1,
) -> Result<B0SkinnedVertexStreamV1, RenderDeviceError> {
    if record.mesh_revision != profile.mesh_revision()
        || record.skinning_profile_revision != profile.asset_revision()?
        || record.source_skeleton_revision != profile.skeleton_revision()
        || record.source_body_schema_revision != profile.body_schema_revision()
        || mesh.asset_revision()? != profile.mesh_revision()
        || mesh.positions_micrometres().len() != profile.vertices().len()
    {
        return Err(RenderDeviceError::SkinningBindingInvalid);
    }
    let local_poses = record
        .ordered_local_joint_poses
        .iter()
        .map(|pose| (&pose.render_joint_id, pose.local_transform))
        .collect::<BTreeMap<_, _>>();
    if local_poses.len() != profile.render_joints().len()
        || profile
            .render_joints()
            .iter()
            .any(|joint| !local_poses.contains_key(&joint.render_joint_id))
    {
        return Err(RenderDeviceError::SkinningBindingInvalid);
    }

    let attempted = skin_positions(record, profile, mesh, &local_poses);
    let sampled_positions_valid = attempted.as_ref().is_some_and(|positions| {
        positions
            .iter()
            .all(|position| mesh.bounds().contains(*position))
    });
    let used_bind_pose_fallback = record.projection_mode
        == next_contracts::presentation::BaseSkinningProjectionModeV1::BindPoseFallback
        || !sampled_positions_valid;
    let positions = if sampled_positions_valid {
        attempted.expect("validated sampled positions")
    } else {
        mesh.positions_micrometres().to_vec()
    };
    let vertex_stream_hash = vertex_stream_hash(record.canonical_hash, &positions)?;
    Ok(B0SkinnedVertexStreamV1 {
        skinning_record_hash: record.canonical_hash,
        mesh_revision: record.mesh_revision,
        positions_micrometres: positions,
        used_bind_pose_fallback,
        vertex_stream_hash,
    })
}

fn skin_positions(
    record: &CharacterSkinningPresentationRecordV1,
    profile: &NeutralBaseSkinningProfileV1,
    mesh: &NeutralMeshV1,
    local_poses: &BTreeMap<&SchemaId, next_contracts::animation_content::NeutralTransformV1>,
) -> Option<Vec<[i64; 3]>> {
    if record.projection_mode
        == next_contracts::presentation::BaseSkinningProjectionModeV1::BindPoseFallback
    {
        return Some(mesh.positions_micrometres().to_vec());
    }
    let mut bind_globals = BTreeMap::new();
    let mut current_globals = BTreeMap::new();
    for joint in profile.render_joints() {
        let bind_local = RigidTransformQ30 {
            translation: joint.bind_transform.translation_micrometres,
            rotation: joint.bind_transform.rotation_q1_30,
        };
        let current = local_poses.get(&joint.render_joint_id)?;
        let current_local = RigidTransformQ30 {
            translation: current.translation_micrometres,
            rotation: current.rotation_q1_30,
        };
        let (bind_global, current_global) = match &joint.parent_render_joint_id {
            Some(parent) => (
                compose(*bind_globals.get(parent)?, bind_local)?,
                compose(*current_globals.get(parent)?, current_local)?,
            ),
            None => (bind_local, current_local),
        };
        bind_globals.insert(joint.render_joint_id.clone(), bind_global);
        current_globals.insert(joint.render_joint_id.clone(), current_global);
    }
    let origin = profile.mesh_origin_in_skeleton_micrometres();
    mesh.positions_micrometres()
        .iter()
        .zip(profile.vertices())
        .map(|(position, vertex)| {
            let skeleton_position = checked_add3(*position, origin)?;
            let mut weighted = [0_i128; 3];
            for influence in vertex.influences() {
                let bind = *bind_globals.get(&influence.render_joint_id)?;
                let current = *current_globals.get(&influence.render_joint_id)?;
                let bind_local = rotate(
                    conjugate(bind.rotation),
                    checked_sub3(skeleton_position, bind.translation)?,
                )?;
                let current_skeleton =
                    checked_add3(rotate(current.rotation, bind_local)?, current.translation)?;
                let current_mesh = checked_sub3(current_skeleton, origin)?;
                for axis in 0..3 {
                    weighted[axis] = weighted[axis].checked_add(
                        i128::from(current_mesh[axis])
                            .checked_mul(i128::from(influence.weight_unorm16))?,
                    )?;
                }
            }
            let mut result = [0_i64; 3];
            for axis in 0..3 {
                result[axis] = i64::try_from(weighted[axis] / i128::from(u16::MAX)).ok()?;
            }
            Some(result)
        })
        .collect()
}

fn compose(parent: RigidTransformQ30, local: RigidTransformQ30) -> Option<RigidTransformQ30> {
    Some(RigidTransformQ30 {
        translation: checked_add3(
            parent.translation,
            rotate(parent.rotation, local.translation)?,
        )?,
        rotation: multiply_quaternion(parent.rotation, local.rotation)?,
    })
}

fn multiply_quaternion(left: [i32; 4], right: [i32; 4]) -> Option<[i32; 4]> {
    let [lx, ly, lz, lw] = left.map(i128::from);
    let [rx, ry, rz, rw] = right.map(i128::from);
    let scale = 1_i128 << 30;
    [
        lw.checked_mul(rx)?
            .checked_add(lx.checked_mul(rw)?)?
            .checked_add(ly.checked_mul(rz)?)?
            .checked_sub(lz.checked_mul(ry)?)?,
        lw.checked_mul(ry)?
            .checked_sub(lx.checked_mul(rz)?)?
            .checked_add(ly.checked_mul(rw)?)?
            .checked_add(lz.checked_mul(rx)?)?,
        lw.checked_mul(rz)?
            .checked_add(lx.checked_mul(ry)?)?
            .checked_sub(ly.checked_mul(rx)?)?
            .checked_add(lz.checked_mul(rw)?)?,
        lw.checked_mul(rw)?
            .checked_sub(lx.checked_mul(rx)?)?
            .checked_sub(ly.checked_mul(ry)?)?
            .checked_sub(lz.checked_mul(rz)?)?,
    ]
    .map(|value| i32::try_from(value / scale).ok())
    .into_iter()
    .collect::<Option<Vec<_>>>()?
    .try_into()
    .ok()
}

fn rotate(rotation: [i32; 4], vector: [i64; 3]) -> Option<[i64; 3]> {
    let [x, y, z] = vector.map(i128::from);
    let [qx, qy, qz, qw] = rotation.map(i128::from);
    let two = 2_i128;
    let scale_squared = 1_i128 << 60;
    let xx = qx.checked_mul(qx)?;
    let yy = qy.checked_mul(qy)?;
    let zz = qz.checked_mul(qz)?;
    let ww = qw.checked_mul(qw)?;
    let xy = qx.checked_mul(qy)?;
    let xz = qx.checked_mul(qz)?;
    let yz = qy.checked_mul(qz)?;
    let xw = qx.checked_mul(qw)?;
    let yw = qy.checked_mul(qw)?;
    let zw = qz.checked_mul(qw)?;
    let result = [
        x.checked_mul(xx.checked_sub(yy)?.checked_sub(zz)?.checked_add(ww)?)?
            .checked_add(two.checked_mul(y)?.checked_mul(xy.checked_sub(zw)?)?)?
            .checked_add(two.checked_mul(z)?.checked_mul(xz.checked_add(yw)?)?)?,
        two.checked_mul(x)?
            .checked_mul(xy.checked_add(zw)?)?
            .checked_add(y.checked_mul(yy.checked_sub(xx)?.checked_sub(zz)?.checked_add(ww)?)?)?
            .checked_add(two.checked_mul(z)?.checked_mul(yz.checked_sub(xw)?)?)?,
        two.checked_mul(x)?
            .checked_mul(xz.checked_sub(yw)?)?
            .checked_add(two.checked_mul(y)?.checked_mul(yz.checked_add(xw)?)?)?
            .checked_add(z.checked_mul(zz.checked_sub(xx)?.checked_sub(yy)?.checked_add(ww)?)?)?,
    ];
    result
        .map(|value| i64::try_from(value / scale_squared).ok())
        .into_iter()
        .collect::<Option<Vec<_>>>()?
        .try_into()
        .ok()
}

fn conjugate(rotation: [i32; 4]) -> [i32; 4] {
    [
        rotation[0].saturating_neg(),
        rotation[1].saturating_neg(),
        rotation[2].saturating_neg(),
        rotation[3],
    ]
}

fn checked_add3(left: [i64; 3], right: [i64; 3]) -> Option<[i64; 3]> {
    Some([
        left[0].checked_add(right[0])?,
        left[1].checked_add(right[1])?,
        left[2].checked_add(right[2])?,
    ])
}

fn checked_sub3(left: [i64; 3], right: [i64; 3]) -> Option<[i64; 3]> {
    Some([
        left[0].checked_sub(right[0])?,
        left[1].checked_sub(right[1])?,
        left[2].checked_sub(right[2])?,
    ])
}

fn vertex_stream_hash(
    record_hash: ContentHash,
    positions: &[[i64; 3]],
) -> Result<ContentHash, RenderDeviceError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.b0-skinned-vertex-stream.v1\0");
    bytes.extend_from_slice(record_hash.as_bytes());
    bytes.extend_from_slice(
        &u32::try_from(positions.len())
            .map_err(|_| RenderDeviceError::CountOverflow)?
            .to_le_bytes(),
    );
    for position in positions {
        for value in position {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(content_hash_from_bytes(sha256(&bytes)))
}
