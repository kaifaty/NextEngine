use super::{
    ArticulationCollisionExclusionV2, ArticulationJointInput, ArticulationLinkInput,
    ArticulationLinkInputV2, ArticulationShapeInputV2, ContactOutput, ContactOutputV2,
    EXPECTED_PHYSX_VERSION, JointState, LinkState, MATERIAL_COEFFICIENT_ENCODING_F32_BITS,
    MATERIAL_COEFFICIENT_ENCODING_Q16, MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN,
    MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER, MaterialProfileInput, PhysXVersion,
    RawSweepOutput, RigidBodyInput, STATUS_CAPACITY_EXCEEDED, STATUS_INVALID_ARGUMENT, STATUS_OK,
    SceneProfileInput, c_void,
};

struct MockBox {
    centre: [f32; 3],
    half_extents: [f32; 3],
    token: u64,
}

struct MockWorld {
    capacity: usize,
    boxes: Vec<MockBox>,
    actor_capacity: usize,
    material_configured: bool,
    configured: bool,
    timestep: f32,
    links: Vec<LinkState>,
    joints: Vec<JointState>,
    efforts: Vec<f32>,
}

pub unsafe fn version() -> PhysXVersion {
    EXPECTED_PHYSX_VERSION
}

pub unsafe fn world_create(output: *mut *mut c_void) -> i32 {
    if output.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    let world = Box::new(MockWorld {
        capacity: 0,
        boxes: Vec::new(),
        actor_capacity: 0,
        material_configured: false,
        configured: false,
        timestep: 0.0,
        links: Vec::new(),
        joints: Vec::new(),
        efforts: Vec::new(),
    });
    // SAFETY: `output` was checked non-null and points to caller-owned
    // writable storage. Ownership of the Box moves to the opaque handle.
    unsafe { output.write(Box::into_raw(world).cast()) };
    STATUS_OK
}

pub unsafe fn world_configure_material(
    world: *mut c_void,
    input: *const MaterialProfileInput,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    // SAFETY: wrapper passes a valid fixed-layout input pointer.
    let Some(input) = (unsafe { input.as_ref() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    let coefficients = match input.coefficient_encoding {
        MATERIAL_COEFFICIENT_ENCODING_F32_BITS => [
            f32::from_bits(input.static_friction),
            f32::from_bits(input.dynamic_friction),
            f32::from_bits(input.restitution),
        ],
        MATERIAL_COEFFICIENT_ENCODING_Q16 => [
            input.static_friction as f32 / 65_536.0,
            input.dynamic_friction as f32 / 65_536.0,
            input.restitution as f32 / 65_536.0,
        ],
        _ => return STATUS_INVALID_ARGUMENT,
    };
    if world.material_configured
        || world.configured
        || coefficients.iter().any(|value| !value.is_finite())
        || coefficients.iter().any(|value| *value < 0.0)
        || coefficients[1] > coefficients[0]
        || coefficients[2] > 1.0
        || input.rolling_friction != 0
        || input.spinning_friction != 0
        || input.surface_velocity_micrometres_per_second != [0; 3]
        || input.coefficient_combine_rules != [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5]
        || input.surface_velocity_combine_rule
            != MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER
    {
        return STATUS_INVALID_ARGUMENT;
    }
    world.material_configured = true;
    STATUS_OK
}

pub unsafe fn world_destroy(world: *mut c_void) {
    if !world.is_null() {
        // SAFETY: the wrapper passes a handle created by `world_create`
        // exactly once at Drop.
        unsafe { drop(Box::from_raw(world.cast::<MockWorld>())) };
    }
}

pub unsafe fn world_configure_scene(world: *mut c_void, input: *const SceneProfileInput) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    // SAFETY: wrapper passes a valid fixed-layout input pointer.
    let Some(input) = (unsafe { input.as_ref() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    let timestep = f32::from_bits(input.timestep_bits);
    if !world.material_configured
        || world.configured
        || !timestep.is_finite()
        || timestep <= 0.0
        || input.max_actors == 0
        || input.max_joints == 0
    {
        return STATUS_INVALID_ARGUMENT;
    }
    world.configured = true;
    world.timestep = timestep;
    world.actor_capacity = input.max_actors as usize;
    STATUS_OK
}

pub unsafe fn world_configure_scene_with_force_schedule_v1(
    world: *mut c_void,
    input: *const SceneProfileInput,
    schedule: u32,
) -> i32 {
    if schedule > 1 {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: same live handle and input preconditions as the legacy entry.
    unsafe { world_configure_scene(world, input) }
}

pub unsafe fn world_reserve(world: *mut c_void, capacity: u32) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if (capacity as usize) < world.boxes.len() {
        return STATUS_INVALID_ARGUMENT;
    }
    world.capacity = capacity as usize;
    world
        .boxes
        .reserve(world.capacity.saturating_sub(world.boxes.len()));
    STATUS_OK
}

pub unsafe fn world_add_static_box(
    world: *mut c_void,
    centre_bits: *const u32,
    half_extents_bits: *const u32,
    user_token: u64,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if centre_bits.is_null() || half_extents_bits.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    if world.boxes.len() >= world.capacity {
        return STATUS_CAPACITY_EXCEEDED;
    }
    // SAFETY: the wrapper passes pointers to arrays containing three u32s.
    let centre = unsafe { std::slice::from_raw_parts(centre_bits, 3) };
    // SAFETY: the wrapper passes pointers to arrays containing three u32s.
    let half_extents = unsafe { std::slice::from_raw_parts(half_extents_bits, 3) };
    world.boxes.push(MockBox {
        centre: [
            f32::from_bits(centre[0]),
            f32::from_bits(centre[1]),
            f32::from_bits(centre[2]),
        ],
        half_extents: [
            f32::from_bits(half_extents[0]),
            f32::from_bits(half_extents[1]),
            f32::from_bits(half_extents[2]),
        ],
        token: user_token,
    });
    STATUS_OK
}

pub unsafe fn world_add_dynamic_box(world: *mut c_void, input: *const RigidBodyInput) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_ref() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !world.configured || input.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    STATUS_OK
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn world_add_articulation(
    world: *mut c_void,
    links: *const ArticulationLinkInput,
    link_count: u32,
    joints: *const ArticulationJointInput,
    joint_count: u32,
    _position_iterations: u32,
    _velocity_iterations: u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !world.configured
        || links.is_null()
        || link_count == 0
        || joint_count + 1 != link_count
        || (joint_count != 0 && joints.is_null())
        || link_count as usize > world.actor_capacity
        || !world.links.is_empty()
    {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: wrapper passes arrays with the declared element counts.
    let links = unsafe { std::slice::from_raw_parts(links, link_count as usize) };
    world.links = links
        .iter()
        .map(|link| LinkState {
            user_token: link.user_token,
            position_bits: link.position_bits,
            rotation_bits: link.rotation_bits,
            linear_velocity_bits: [0.0_f32.to_bits(); 3],
            angular_velocity_bits: [0.0_f32.to_bits(); 3],
        })
        .collect();
    world.joints = vec![JointState::default(); joint_count as usize];
    world.efforts = vec![0.0; joint_count as usize];
    STATUS_OK
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn world_add_articulation_v2(
    world: *mut c_void,
    links: *const ArticulationLinkInputV2,
    link_count: u32,
    shapes: *const ArticulationShapeInputV2,
    shape_count: u32,
    joints: *const ArticulationJointInput,
    joint_count: u32,
    exclusions: *const ArticulationCollisionExclusionV2,
    exclusion_count: u32,
    _position_iterations: u32,
    _velocity_iterations: u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !world.configured
        || links.is_null()
        || link_count == 0
        || joint_count + 1 != link_count
        || (shape_count != 0 && shapes.is_null())
        || (joint_count != 0 && joints.is_null())
        || (exclusion_count != 0 && exclusions.is_null())
        || link_count as usize > world.actor_capacity
        || !world.links.is_empty()
    {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: wrapper passes arrays with the declared element counts.
    let links = unsafe { std::slice::from_raw_parts(links, link_count as usize) };
    world.links = links
        .iter()
        .map(|link| LinkState {
            user_token: link.user_token,
            position_bits: link.position_bits,
            rotation_bits: link.rotation_bits,
            linear_velocity_bits: [0.0_f32.to_bits(); 3],
            angular_velocity_bits: [0.0_f32.to_bits(); 3],
        })
        .collect();
    world.joints = vec![JointState::default(); joint_count as usize];
    world.efforts = vec![0.0; joint_count as usize];
    STATUS_OK
}

pub unsafe fn world_apply_articulation_efforts(
    world: *mut c_void,
    efforts: *const u32,
    effort_count: u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if efforts.is_null() || effort_count as usize != world.efforts.len() {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: wrapper passes exactly effort_count values.
    let efforts = unsafe { std::slice::from_raw_parts(efforts, effort_count as usize) };
    for (output, bits) in world.efforts.iter_mut().zip(efforts) {
        *output = f32::from_bits(*bits);
        if !output.is_finite() {
            return STATUS_INVALID_ARGUMENT;
        }
    }
    STATUS_OK
}

pub unsafe fn world_import_articulation_state(
    world: *mut c_void,
    root: *const LinkState,
    joints: *const JointState,
    joint_count: u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if root.is_null()
        || joint_count as usize != world.joints.len()
        || (joint_count != 0 && joints.is_null())
    {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: wrapper passes one root and exactly joint_count joint records.
    let root = unsafe { root.read() };
    if world
        .links
        .first()
        .is_none_or(|link| link.user_token != root.user_token)
    {
        return STATUS_INVALID_ARGUMENT;
    }
    world.links[0] = root;
    // SAFETY: pointer is non-null for a non-empty declared array.
    let joints = unsafe { std::slice::from_raw_parts(joints, joint_count as usize) };
    world.joints.copy_from_slice(joints);
    STATUS_OK
}

pub unsafe fn world_step(world: *mut c_void) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !world.configured {
        return STATUS_INVALID_ARGUMENT;
    }
    for (joint, effort) in world.joints.iter_mut().zip(&world.efforts) {
        let velocity = f32::from_bits(joint.velocity_bits) + effort * world.timestep;
        let position = f32::from_bits(joint.position_bits) + velocity * world.timestep;
        joint.velocity_bits = velocity.to_bits();
        joint.position_bits = position.to_bits();
    }
    STATUS_OK
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn world_export_articulation_state(
    world: *mut c_void,
    links: *mut LinkState,
    link_capacity: u32,
    link_count: *mut u32,
    joints: *mut JointState,
    joint_capacity: u32,
    joint_count: *mut u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_ref() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if link_count.is_null() || joint_count.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: count outputs point to writable locals in the wrapper.
    unsafe {
        link_count.write(world.links.len() as u32);
        joint_count.write(world.joints.len() as u32);
    }
    if link_capacity < world.links.len() as u32 || joint_capacity < world.joints.len() as u32 {
        return STATUS_CAPACITY_EXCEEDED;
    }
    if (!world.links.is_empty() && links.is_null())
        || (!world.joints.is_empty() && joints.is_null())
    {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: caller buffers have at least the capacities checked above.
    unsafe {
        std::ptr::copy_nonoverlapping(world.links.as_ptr(), links, world.links.len());
        std::ptr::copy_nonoverlapping(world.joints.as_ptr(), joints, world.joints.len());
    }
    STATUS_OK
}

pub unsafe fn world_export_contacts(
    world: *mut c_void,
    _contacts: *mut ContactOutput,
    _capacity: u32,
    count: *mut u32,
) -> i32 {
    if world.is_null() || count.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: count points to wrapper-owned writable storage.
    unsafe { count.write(0) };
    STATUS_OK
}

pub unsafe fn world_export_contacts_v2(
    world: *mut c_void,
    _contacts: *mut ContactOutputV2,
    _capacity: u32,
    count: *mut u32,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(_world) = (unsafe { world.cast::<MockWorld>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if count.is_null() {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: wrapper passes a valid writable count pointer.
    unsafe { count.write(0) };
    STATUS_OK
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn world_sweep_capsule_axis(
    world: *mut c_void,
    centre_bits: *const u32,
    radius_bits: u32,
    half_segment_bits: u32,
    axis: u32,
    direction_sign: i32,
    distance_bits: u32,
    output: *mut RawSweepOutput,
) -> i32 {
    // SAFETY: private raw API is called only with a live MockWorld handle.
    let Some(world) = (unsafe { world.cast::<MockWorld>().as_ref() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if centre_bits.is_null() || output.is_null() || axis > 2 || !matches!(direction_sign, -1 | 1) {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: the wrapper passes a pointer to an array containing three u32s.
    let centre_bits = unsafe { std::slice::from_raw_parts(centre_bits, 3) };
    let centre = [
        f32::from_bits(centre_bits[0]),
        f32::from_bits(centre_bits[1]),
        f32::from_bits(centre_bits[2]),
    ];
    let radius = f32::from_bits(radius_bits);
    let half_segment = f32::from_bits(half_segment_bits);
    let distance = f32::from_bits(distance_bits);
    if !centre.iter().all(|value| value.is_finite())
        || !radius.is_finite()
        || radius <= 0.0
        || !half_segment.is_finite()
        || half_segment < 0.0
        || !distance.is_finite()
        || distance < 0.0
    {
        return STATUS_INVALID_ARGUMENT;
    }
    let axis = axis as usize;
    let signed_delta = distance * direction_sign as f32;
    let mut selected: Option<(f32, u64)> = None;
    for shape in &world.boxes {
        let minimum = [
            shape.centre[0] - shape.half_extents[0],
            shape.centre[1] - shape.half_extents[1],
            shape.centre[2] - shape.half_extents[2],
        ];
        let maximum = [
            shape.centre[0] + shape.half_extents[0],
            shape.centre[1] + shape.half_extents[1],
            shape.centre[2] + shape.half_extents[2],
        ];
        let mut perpendicular_squared = 0.0_f32;
        for other in 0..3 {
            if other == axis {
                continue;
            }
            let value = if other == 1 {
                interval_interval_distance(
                    centre[1] - half_segment,
                    centre[1] + half_segment,
                    minimum[1],
                    maximum[1],
                )
            } else {
                interval_distance(centre[other], minimum[other], maximum[other])
            };
            perpendicular_squared += value * value;
        }
        if perpendicular_squared > radius * radius {
            continue;
        }
        let radial = (radius * radius - perpendicular_squared).sqrt();
        let reach = if axis == 1 {
            half_segment + radial
        } else {
            radial
        };
        let lower = minimum[axis] - reach;
        let upper = maximum[axis] + reach;
        let start = centre[axis];
        let candidate = if signed_delta > 0.0 && start <= lower && start + signed_delta > lower {
            Some(lower - start)
        } else if signed_delta < 0.0 && start >= upper && start + signed_delta < upper {
            Some(upper - start)
        } else {
            None
        };
        if let Some(candidate) = candidate {
            let magnitude = candidate.abs();
            if selected.is_none_or(|(best, token)| {
                magnitude < best || (magnitude == best && shape.token < token)
            }) {
                selected = Some((magnitude, shape.token));
            }
        }
    }
    let (hit, hit_distance, token) =
        selected.map_or((0, distance, 0), |(distance, token)| (1, distance, token));
    let mut normal = [0.0_f32; 3];
    if hit == 1 {
        normal[axis] = -(direction_sign as f32);
    }
    // SAFETY: output was checked non-null and points to writable caller storage.
    unsafe {
        output.write(RawSweepOutput {
            hit,
            distance_bits: hit_distance.to_bits(),
            normal_x_bits: normal[0].to_bits(),
            normal_y_bits: normal[1].to_bits(),
            normal_z_bits: normal[2].to_bits(),
            user_token: token,
        });
    }
    STATUS_OK
}

fn interval_distance(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value < minimum {
        minimum - value
    } else if value > maximum {
        value - maximum
    } else {
        0.0
    }
}

fn interval_interval_distance(
    first_minimum: f32,
    first_maximum: f32,
    second_minimum: f32,
    second_maximum: f32,
) -> f32 {
    if first_maximum < second_minimum {
        second_minimum - first_maximum
    } else if first_minimum > second_maximum {
        first_minimum - second_maximum
    } else {
        0.0
    }
}
