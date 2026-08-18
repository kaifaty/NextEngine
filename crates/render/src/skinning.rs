use std::collections::BTreeMap;

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::presentation::{
    BaseSkinningProjectionModeV1, CharacterDeformationLodV1, CharacterSkinningPresentationRecordV1,
};
use next_contracts::render_content::{
    NeutralBaseSkinningProfileV1, NeutralMeshV1, PoseCorrectiveLodClassV1,
};

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
        || record.deformation_lod == CharacterDeformationLodV1::Culled
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

    if record.projection_mode == BaseSkinningProjectionModeV1::BindPoseFallback {
        let positions = mesh.positions_micrometres().to_vec();
        let vertex_stream_hash =
            vertex_stream_hash(record.canonical_hash, 0, false, true, &positions)?;
        return Ok(B0SkinnedVertexStreamV1 {
            skinning_record_hash: record.canonical_hash,
            mesh_revision: record.mesh_revision,
            positions_micrometres: positions,
            applied_pose_corrective_count: 0,
            used_pose_corrective_fallback: false,
            used_bind_pose_fallback: true,
            vertex_stream_hash,
        });
    }

    let corrective_attempt = corrected_bind_positions(record, profile, mesh, &local_poses)
        .and_then(|(positions, count)| {
            skin_positions(profile, &positions, &local_poses).map(|positions| (positions, count))
        });
    let corrected_positions_valid = corrective_attempt.as_ref().is_some_and(|(positions, _)| {
        positions
            .iter()
            .all(|position| mesh.bounds().contains(*position))
    });
    let requests_correctives = matches!(
        record.deformation_lod,
        CharacterDeformationLodV1::FullCorrectives | CharacterDeformationLodV1::ReducedCorrectives
    );
    let (
        positions,
        applied_pose_corrective_count,
        used_pose_corrective_fallback,
        used_bind_pose_fallback,
    ) = if corrected_positions_valid {
        let (positions, count) = corrective_attempt.expect("validated corrected positions");
        (positions, count, false, false)
    } else {
        let base_attempt = skin_positions(profile, mesh.positions_micrometres(), &local_poses);
        let base_positions_valid = base_attempt.as_ref().is_some_and(|positions| {
            positions
                .iter()
                .all(|position| mesh.bounds().contains(*position))
        });
        if base_positions_valid {
            (
                base_attempt.expect("validated base skinning positions"),
                0,
                requests_correctives,
                false,
            )
        } else {
            (
                mesh.positions_micrometres().to_vec(),
                0,
                requests_correctives,
                true,
            )
        }
    };
    let vertex_stream_hash = vertex_stream_hash(
        record.canonical_hash,
        applied_pose_corrective_count,
        used_pose_corrective_fallback,
        used_bind_pose_fallback,
        &positions,
    )?;
    Ok(B0SkinnedVertexStreamV1 {
        skinning_record_hash: record.canonical_hash,
        mesh_revision: record.mesh_revision,
        positions_micrometres: positions,
        applied_pose_corrective_count,
        used_pose_corrective_fallback,
        used_bind_pose_fallback,
        vertex_stream_hash,
    })
}

fn corrected_bind_positions(
    record: &CharacterSkinningPresentationRecordV1,
    profile: &NeutralBaseSkinningProfileV1,
    mesh: &NeutralMeshV1,
    local_poses: &BTreeMap<&SchemaId, next_contracts::animation_content::NeutralTransformV1>,
) -> Option<(Vec<[i64; 3]>, u32)> {
    if record.deformation_lod == CharacterDeformationLodV1::BaseSkinningOnly {
        return Some((mesh.positions_micrometres().to_vec(), 0));
    }
    let bind_poses = profile
        .render_joints()
        .iter()
        .map(|joint| (&joint.render_joint_id, joint.bind_transform))
        .collect::<BTreeMap<_, _>>();
    let mut positions = mesh.positions_micrometres().to_vec();
    let mut applied_count = 0_u32;
    for corrective in profile.pose_correctives() {
        if record.deformation_lod == CharacterDeformationLodV1::ReducedCorrectives
            && corrective.lod_class() != PoseCorrectiveLodClassV1::Essential
        {
            continue;
        }
        let current = local_poses.get(corrective.driver_render_joint_id())?;
        let bind = bind_poses.get(corrective.driver_render_joint_id())?;
        let axis = corrective.driver_axis().index();
        let driver_delta = i128::from(current.translation_micrometres[axis])
            .checked_sub(i128::from(bind.translation_micrometres[axis]))?;
        let weight = corrective_weight_unorm16(
            driver_delta,
            corrective.activation_start_delta_micrometres(),
            corrective.activation_full_delta_micrometres(),
        )?;
        if weight == 0 {
            continue;
        }
        applied_count = applied_count.checked_add(1)?;
        for delta in corrective.vertex_deltas() {
            let position = positions.get_mut(usize::try_from(delta.vertex_index).ok()?)?;
            for (position_component, delta_component) in
                position.iter_mut().zip(delta.delta_micrometres.iter())
            {
                let weighted_delta = i128::from(*delta_component)
                    .checked_mul(i128::from(weight))?
                    / i128::from(u16::MAX);
                *position_component =
                    i64::try_from(i128::from(*position_component).checked_add(weighted_delta)?)
                        .ok()?;
            }
        }
    }
    Some((positions, applied_count))
}

fn corrective_weight_unorm16(driver_delta: i128, start: i64, full: i64) -> Option<u16> {
    let start = i128::from(start);
    let full = i128::from(full);
    let (numerator, denominator) = if full > start {
        if driver_delta <= start {
            return Some(0);
        }
        if driver_delta >= full {
            return Some(u16::MAX);
        }
        (driver_delta.checked_sub(start)?, full.checked_sub(start)?)
    } else {
        if driver_delta >= start {
            return Some(0);
        }
        if driver_delta <= full {
            return Some(u16::MAX);
        }
        (start.checked_sub(driver_delta)?, start.checked_sub(full)?)
    };
    u16::try_from(numerator.checked_mul(i128::from(u16::MAX))? / denominator).ok()
}

fn skin_positions(
    profile: &NeutralBaseSkinningProfileV1,
    bind_positions: &[[i64; 3]],
    local_poses: &BTreeMap<&SchemaId, next_contracts::animation_content::NeutralTransformV1>,
) -> Option<Vec<[i64; 3]>> {
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
    bind_positions
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
    applied_pose_corrective_count: u32,
    used_pose_corrective_fallback: bool,
    used_bind_pose_fallback: bool,
    positions: &[[i64; 3]],
) -> Result<ContentHash, RenderDeviceError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.b0-skinned-vertex-stream.v2\0");
    bytes.extend_from_slice(record_hash.as_bytes());
    bytes.extend_from_slice(&applied_pose_corrective_count.to_le_bytes());
    bytes.push(u8::from(used_pose_corrective_fallback));
    bytes.push(u8::from(used_bind_pose_fallback));
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

#[cfg(test)]
mod tests {
    use super::corrective_weight_unorm16;

    #[test]
    fn corrective_weight_is_exact_for_both_signed_activation_directions() {
        assert_eq!(corrective_weight_unorm16(-1, 0, 100), Some(0));
        assert_eq!(corrective_weight_unorm16(0, 0, 100), Some(0));
        assert_eq!(corrective_weight_unorm16(50, 0, 100), Some(32_767));
        assert_eq!(corrective_weight_unorm16(100, 0, 100), Some(u16::MAX));
        assert_eq!(corrective_weight_unorm16(101, 0, 100), Some(u16::MAX));

        assert_eq!(corrective_weight_unorm16(1, 0, -100), Some(0));
        assert_eq!(corrective_weight_unorm16(0, 0, -100), Some(0));
        assert_eq!(corrective_weight_unorm16(-50, 0, -100), Some(32_767));
        assert_eq!(corrective_weight_unorm16(-100, 0, -100), Some(u16::MAX));
        assert_eq!(corrective_weight_unorm16(-101, 0, -100), Some(u16::MAX));
    }
}
