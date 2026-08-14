use next_physics_physx_ffi::{ContactOutput, ContactOutputV2, JointState, LinkState};

use crate::{
    CanonicalPhysXContact, CanonicalPhysXContactV2, CanonicalPhysXJointState,
    CanonicalPhysXLinkState, CanonicalPhysXSnapshot, CanonicalPhysXSnapshotV2, PhysXAdapterError,
};

pub(crate) fn canonicalize_native_output(
    links: &[LinkState],
    joints: &[JointState],
    contacts: &[ContactOutput],
) -> Result<CanonicalPhysXSnapshot, PhysXAdapterError> {
    let mut canonical_links = links
        .iter()
        .map(|link| {
            Ok(CanonicalPhysXLinkState {
                user_token: link.user_token,
                position_micrometres: quantize_vector(link.position_bits, 1_000_000.0)?,
                rotation_q1_30: [
                    quantize_bits(link.rotation_bits[0], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[1], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[2], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[3], (1_u64 << 30) as f64)?,
                ],
                linear_velocity_micrometres_per_second: quantize_vector(
                    link.linear_velocity_bits,
                    1_000_000.0,
                )?,
                angular_velocity_microradians_per_second: quantize_vector(
                    link.angular_velocity_bits,
                    1_000_000.0,
                )?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_links.sort_unstable_by_key(|link| link.user_token);
    if canonical_links
        .windows(2)
        .any(|pair| pair[0].user_token == pair[1].user_token)
    {
        return Err(PhysXAdapterError::InvalidOutput);
    }

    let canonical_joints = joints
        .iter()
        .enumerate()
        .map(|(ordinal, joint)| {
            Ok(CanonicalPhysXJointState {
                ordinal: u32::try_from(ordinal).map_err(|_| PhysXAdapterError::CapacityExceeded)?,
                position_microradians: quantize_bits(joint.position_bits, 1_000_000.0)?,
                velocity_microradians_per_second: quantize_bits(joint.velocity_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;

    let mut canonical_contacts = contacts
        .iter()
        .map(|contact| {
            let swapped = contact.actor_a_token > contact.actor_b_token;
            let (actor_a_token, actor_b_token) = if swapped {
                (contact.actor_b_token, contact.actor_a_token)
            } else {
                (contact.actor_a_token, contact.actor_b_token)
            };
            let mut normal = quantize_vector(contact.normal_bits, (1_u64 << 30) as f64)?;
            let mut impulse = quantize_vector(contact.impulse_bits, 1_000_000.0)?;
            if swapped {
                for value in &mut normal {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
                for value in &mut impulse {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
            }
            Ok(CanonicalPhysXContact {
                actor_a_token,
                actor_b_token,
                position_micrometres: quantize_vector(contact.position_bits, 1_000_000.0)?,
                normal_q1_30: normal,
                impulse_micronewton_seconds: impulse,
                separation_micrometres: quantize_bits(contact.separation_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_contacts.sort_unstable();
    Ok(CanonicalPhysXSnapshot {
        links: canonical_links,
        joints: canonical_joints,
        contacts: canonical_contacts,
    })
}

pub(crate) fn canonicalize_native_output_v2(
    links: &[LinkState],
    joints: &[JointState],
    contacts: &[ContactOutputV2],
) -> Result<CanonicalPhysXSnapshotV2, PhysXAdapterError> {
    let base = canonicalize_native_output(links, joints, &[])?;
    let mut canonical_contacts = contacts
        .iter()
        .map(|contact| {
            let swapped = (contact.actor_a_token, contact.shape_a_token)
                > (contact.actor_b_token, contact.shape_b_token);
            let (actor_a_token, actor_b_token, shape_a_token, shape_b_token) = if swapped {
                (
                    contact.actor_b_token,
                    contact.actor_a_token,
                    contact.shape_b_token,
                    contact.shape_a_token,
                )
            } else {
                (
                    contact.actor_a_token,
                    contact.actor_b_token,
                    contact.shape_a_token,
                    contact.shape_b_token,
                )
            };
            let mut normal = quantize_vector(contact.normal_bits, (1_u64 << 30) as f64)?;
            let mut impulse = quantize_vector(contact.impulse_bits, 1_000_000.0)?;
            if swapped {
                for value in &mut normal {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
                for value in &mut impulse {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
            }
            Ok(CanonicalPhysXContactV2 {
                actor_a_token,
                actor_b_token,
                shape_a_token,
                shape_b_token,
                position_micrometres: quantize_vector(contact.position_bits, 1_000_000.0)?,
                normal_q1_30: normal,
                impulse_micronewton_seconds: impulse,
                separation_micrometres: quantize_bits(contact.separation_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_contacts.sort_unstable();
    Ok(CanonicalPhysXSnapshotV2 {
        links: base.links,
        joints: base.joints,
        contacts: canonical_contacts,
    })
}

fn quantize_vector(bits: [u32; 3], scale: f64) -> Result<[i64; 3], PhysXAdapterError> {
    Ok([
        quantize_bits(bits[0], scale)?,
        quantize_bits(bits[1], scale)?,
        quantize_bits(bits[2], scale)?,
    ])
}

fn quantize_bits(bits: u32, scale: f64) -> Result<i64, PhysXAdapterError> {
    let value = f64::from(f32::from_bits(bits));
    let scaled = value * scale;
    if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        return Err(PhysXAdapterError::NumericOverflow);
    }
    Ok(scaled.round_ties_even() as i64)
}

pub(crate) fn scaled_f32_bits(value: i64, scale: f64) -> Result<u32, PhysXAdapterError> {
    let value = value as f64 / scale;
    let value = value as f32;
    if value.is_finite() {
        Ok(value.to_bits())
    } else {
        Err(PhysXAdapterError::NumericOverflow)
    }
}
