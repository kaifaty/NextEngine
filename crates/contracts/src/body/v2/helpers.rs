use super::*;

pub(super) fn abs_diff_i128(left: i128, right: i128) -> u128 {
    left.abs_diff(right)
}

pub(super) fn validate_geometry(value: &PhysicsGeometryV1) -> Result<(), BodyV2ContractError> {
    match value {
        PhysicsGeometryV1::Box { .. }
        | PhysicsGeometryV1::Sphere { .. }
        | PhysicsGeometryV1::Capsule { .. } => value.validate().map_err(Into::into),
        _ => Err(BodyV2ContractError::UnsupportedGeometry),
    }
}

pub(super) fn require_strict_order<'a>(
    values: impl Iterator<Item = &'a SchemaId>,
) -> Result<(), BodyV2ContractError> {
    let mut previous: Option<&SchemaId> = None;
    for value in values {
        if previous.is_some_and(|previous| previous >= value) {
            return Err(BodyV2ContractError::NonCanonicalOrder);
        }
        previous = Some(value);
    }
    Ok(())
}

pub(super) fn require_strict_struct_order<T: Ord>(values: &[T]) -> Result<(), BodyV2ContractError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(BodyV2ContractError::NonCanonicalOrder);
    }
    Ok(())
}

pub(super) fn encode_body(
    value: &BodyDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.body_id)?;
    encode_optional_id(out, value.parent_body_id.as_ref())?;
    encode_pose(out, value.local_bind_pose);
    out.push(value.semantic_role as u8);
    push_id(out, &value.mapping_group_id)?;
    push_u64(out, value.mass_microkilograms);
    encode_i64_3(out, value.center_of_mass_micrometres);
    encode_i64_6(out, value.inertia_tensor_microkilogram_metre_squared);
    push_u64(out, value.solver_mass_microkilograms);
    encode_i64_3(out, value.solver_center_of_mass_micrometres);
    for item in value.solver_principal_inertia_microkilogram_metre_squared {
        push_u64(out, item);
    }
    encode_pose(out, value.solver_principal_frame);
    push_u64(
        out,
        value.solver_tensor_error_max_microkilogram_metre_squared,
    );
    push_sequence(out, &value.colliders, |collider, bytes| {
        push_id(bytes, &collider.collider_id)?;
        encode_pose(bytes, collider.local_pose);
        encode_geometry(bytes, &collider.geometry)?;
        push_id(bytes, &collider.material_id)?;
        bytes.push(collider.collision_layer);
        push_u64(bytes, collider.collision_mask);
        bytes.push(collider.participation as u8);
        bytes.push(collider.contact_reporting as u8);
        bytes.push(collider.contact_role as u8);
        Ok(())
    })
}

pub(super) fn encode_mapping_group(
    value: &BodyMassProjectionGroupV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.mapping_group_id)?;
    push_u64(out, value.source_mass_microkilograms);
    encode_i64_3(out, value.source_center_of_mass_micrometres);
    encode_i64_6(out, value.source_inertia_tensor_microkilogram_metre_squared);
    push_u64(
        out,
        value.maximum_first_moment_error_microkilogram_micrometres,
    );
    push_u64(out, value.maximum_inertia_error_microkilogram_metre_squared);
    push_sequence(out, &value.members, |member, bytes| {
        push_id(bytes, &member.body_id)?;
        encode_i64_3(bytes, member.body_origin_in_group_micrometres);
        Ok(())
    })
}

pub(super) fn encode_joint(
    value: &BodyJointDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.joint_id)?;
    push_id(out, &value.parent_body_id)?;
    push_id(out, &value.child_body_id)?;
    push_id(out, &value.anatomical_semantic_id)?;
    encode_pose(out, value.parent_frame);
    encode_pose(out, value.child_frame);
    for axis in value.axis_q1_30 {
        out.extend_from_slice(&axis.to_le_bytes());
    }
    for angle in [
        value.hard_minimum_microradians,
        value.hard_maximum_microradians,
        value.soft_minimum_microradians,
        value.soft_maximum_microradians,
        value.neutral_position_microradians,
    ] {
        push_i64(out, angle);
    }
    push_u64(out, value.maximum_velocity_microradians_per_second);
    Ok(())
}

pub(super) fn encode_actuator(
    value: &BodyActuatorDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.actuator_id)?;
    push_id(out, &value.joint_id)?;
    for item in [value.stiffness_q16, value.damping_q16] {
        push_u64(out, item);
    }
    push_i64(out, value.minimum_effort_micronewton_metres);
    push_i64(out, value.maximum_effort_micronewton_metres);
    for item in [
        value.maximum_effort_rate_micronewton_metres_per_second,
        value.maximum_power_microwatts,
        value.maximum_positive_work_microjoules_per_motor_tick,
        value.residual_scale_microradians,
    ] {
        push_u64(out, item);
    }
    push_i64(out, value.minimum_target_delta_microradians_per_motor_tick);
    push_i64(out, value.maximum_target_delta_microradians_per_motor_tick);
    Ok(())
}

pub(super) fn encode_effector(
    value: &BodyEffectorDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.effector_id)?;
    push_id(out, &value.body_id)?;
    encode_pose(out, value.local_pose);
    push_id(out, &value.semantic_role_id)
}

pub(super) fn encode_pose(out: &mut Vec<u8>, value: BodyPoseV2) {
    encode_i64_3(out, value.translation_micrometres);
    for item in value.rotation_q1_30 {
        out.extend_from_slice(&item.to_le_bytes());
    }
}

pub(super) fn encode_geometry(
    out: &mut Vec<u8>,
    value: &PhysicsGeometryV1,
) -> Result<(), BodyV2ContractError> {
    match value {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => {
            out.push(1);
            encode_i64_3(out, *half_extents_micrometres);
        }
        PhysicsGeometryV1::Sphere { radius_micrometres } => {
            out.push(2);
            push_i64(out, *radius_micrometres);
        }
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => {
            out.push(3);
            push_i64(out, *radius_micrometres);
            push_i64(out, *half_segment_micrometres);
        }
        _ => return Err(BodyV2ContractError::UnsupportedGeometry),
    }
    Ok(())
}

pub(super) fn encode_optional_id(
    out: &mut Vec<u8>,
    value: Option<&SchemaId>,
) -> Result<(), BodyV2ContractError> {
    if let Some(value) = value {
        out.push(1);
        push_id(out, value)
    } else {
        out.push(0);
        Ok(())
    }
}

pub(super) fn encode_i64_3(out: &mut Vec<u8>, values: [i64; 3]) {
    for value in values {
        push_i64(out, value);
    }
}

pub(super) fn encode_i64_6(out: &mut Vec<u8>, values: [i64; 6]) {
    for value in values {
        push_i64(out, value);
    }
}

pub(super) fn push_sequence<T>(
    out: &mut Vec<u8>,
    values: &[T],
    mut encode: impl FnMut(&T, &mut Vec<u8>) -> Result<(), BodyV2ContractError>,
) -> Result<(), BodyV2ContractError> {
    push_u32(
        out,
        u32::try_from(values.len()).map_err(|_| BodyV2ContractError::LengthOverflow)?,
    );
    for value in values {
        encode(value, out)?;
    }
    Ok(())
}

pub(super) fn push_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<(), BodyV2ContractError> {
    push_u32(
        out,
        u32::try_from(value.len()).map_err(|_| BodyV2ContractError::LengthOverflow)?,
    );
    out.extend_from_slice(value);
    Ok(())
}

pub(super) fn push_id(out: &mut Vec<u8>, value: &SchemaId) -> Result<(), BodyV2ContractError> {
    push_bytes(out, value.as_str().as_bytes())
}

pub(super) fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
pub(super) fn push_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}
