#![allow(
    unsafe_code,
    reason = "ADR-058 confines all PhysX ABI calls and pointer ownership to this crate"
)]

use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

#[cfg(all(feature = "physx-sdk", feature = "mock-abi"))]
compile_error!("features `physx-sdk` and `mock-abi` are mutually exclusive");

pub const NEXTENGINE_PHYSX_ABI_VERSION: u32 = 4;
pub const EXPECTED_PHYSX_VERSION: PhysXVersion = PhysXVersion {
    abi: NEXTENGINE_PHYSX_ABI_VERSION,
    major: 5,
    minor: 9,
    patch: 0,
};

const STATUS_OK: i32 = 0;
const STATUS_INVALID_ARGUMENT: i32 = 1;
const STATUS_OUT_OF_MEMORY: i32 = 2;
const STATUS_CAPACITY_EXCEEDED: i32 = 3;
const STATUS_INTERNAL_FAILURE: i32 = 4;
const STATUS_UNAVAILABLE: i32 = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysXVersion {
    pub abi: u32,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RawSweepOutput {
    hit: u32,
    distance_bits: u32,
    normal_x_bits: u32,
    normal_y_bits: u32,
    normal_z_bits: u32,
    user_token: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneProfileInput {
    pub gravity_bits: [u32; 3],
    pub timestep_bits: u32,
    pub position_iterations: u32,
    pub velocity_iterations: u32,
    pub max_contacts: u32,
    pub max_actors: u32,
    pub max_joints: u32,
}

pub const MATERIAL_COEFFICIENT_ENCODING_F32_BITS: u32 = 1;
pub const MATERIAL_COEFFICIENT_ENCODING_Q16: u32 = 2;
pub const MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN: u32 = 3;
pub const MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaterialProfileInput {
    pub coefficient_encoding: u32,
    pub static_friction: u32,
    pub dynamic_friction: u32,
    pub restitution: u32,
    pub rolling_friction: u32,
    pub spinning_friction: u32,
    pub surface_velocity_micrometres_per_second: [i64; 3],
    pub coefficient_combine_rules: [u32; 5],
    pub surface_velocity_combine_rule: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RigidBodyInput {
    pub user_token: u64,
    pub position_bits: [u32; 3],
    pub rotation_bits: [u32; 4],
    pub half_extents_bits: [u32; 3],
    pub mass_bits: u32,
    pub inertia_bits: [u32; 3],
}

pub const NO_PARENT_LINK: u32 = u32::MAX;
pub const SHAPE_BOX: u32 = 1;
pub const SHAPE_SPHERE: u32 = 2;
pub const SHAPE_CAPSULE: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArticulationLinkInput {
    pub user_token: u64,
    pub parent_link_index: u32,
    pub shape_kind: u32,
    pub position_bits: [u32; 3],
    pub rotation_bits: [u32; 4],
    pub shape_dimensions_bits: [u32; 3],
    pub mass_bits: u32,
    pub inertia_bits: [u32; 3],
    pub linear_damping_bits: u32,
    pub angular_damping_bits: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArticulationJointInput {
    pub child_link_index: u32,
    pub reserved: u32,
    pub parent_position_bits: [u32; 3],
    pub parent_rotation_bits: [u32; 4],
    pub child_position_bits: [u32; 3],
    pub child_rotation_bits: [u32; 4],
    pub lower_limit_bits: u32,
    pub upper_limit_bits: u32,
    pub max_velocity_bits: u32,
}

/// A link record for the current biomechanics articulation path.
///
/// Shapes are supplied separately so a non-colliding carrier may have zero
/// shapes and a physical link may own more than one. `centre_of_mass_*` is the
/// PhysX mass-frame pose; `inertia_bits` is expressed in that frame.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArticulationLinkInputV2 {
    pub user_token: u64,
    pub parent_link_index: u32,
    pub first_shape_index: u32,
    pub shape_count: u32,
    pub reserved: u32,
    pub position_bits: [u32; 3],
    pub rotation_bits: [u32; 4],
    pub centre_of_mass_position_bits: [u32; 3],
    pub centre_of_mass_rotation_bits: [u32; 4],
    pub mass_bits: u32,
    pub inertia_bits: [u32; 3],
    pub linear_damping_bits: u32,
    pub angular_damping_bits: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArticulationShapeInputV2 {
    pub user_token: u64,
    pub link_index: u32,
    pub shape_kind: u32,
    pub position_bits: [u32; 3],
    pub rotation_bits: [u32; 4],
    pub shape_dimensions_bits: [u32; 3],
    pub collision_layer: u32,
    pub collision_mask_low: u32,
    pub collision_mask_high: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArticulationCollisionExclusionV2 {
    pub first_link_index: u32,
    pub second_link_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LinkState {
    pub user_token: u64,
    pub position_bits: [u32; 3],
    pub rotation_bits: [u32; 4],
    pub linear_velocity_bits: [u32; 3],
    pub angular_velocity_bits: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JointState {
    pub position_bits: u32,
    pub velocity_bits: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContactOutput {
    pub actor_a_token: u64,
    pub actor_b_token: u64,
    pub position_bits: [u32; 3],
    pub normal_bits: [u32; 3],
    pub impulse_bits: [u32; 3],
    pub separation_bits: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContactOutputV2 {
    pub actor_a_token: u64,
    pub actor_b_token: u64,
    pub shape_a_token: u64,
    pub shape_b_token: u64,
    pub position_bits: [u32; 3],
    pub normal_bits: [u32; 3],
    pub impulse_bits: [u32; 3],
    pub separation_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticBoxInput {
    pub centre_bits: [u32; 3],
    pub half_extents_bits: [u32; 3],
    pub user_token: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapsuleAxisSweepInput {
    pub centre_bits: [u32; 3],
    pub radius_bits: u32,
    pub half_segment_bits: u32,
    pub axis: u32,
    pub direction_sign: i32,
    pub distance_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapsuleAxisSweepOutput {
    pub hit: bool,
    pub distance_bits: u32,
    pub normal_bits: [u32; 3],
    pub user_token: u64,
}

/// Plan `continuum-water/23` (ADR-106): the PBD probe descriptor.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PbdProbeDescRaw {
    pub particle_count: u32,
    pub frames: u32,
    pub spacing_bits: u32,
    pub box_half_bits: u32,
    pub timestep_bits: u32,
    pub margin_bits: u32,
    pub gpu_library_path: *const std::ffi::c_char,
    pub final_positions: *mut f32,
}

/// Plan 23: the PBD probe report as the bridge fills it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PbdProbeReportRaw {
    pub gpu_available: u32,
    pub reason: u32,
    pub frames_completed: u32,
    pub particles_out_of_bounds_max: u32,
    pub step_max_microseconds: u64,
    pub step_mean_microseconds: u64,
    pub readback_max_microseconds: u64,
    pub readback_mean_microseconds: u64,
    pub device_name: [std::ffi::c_char; 64],
}

/// Plan 23: the probe's inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct PbdProbeDesc {
    pub particle_count: u32,
    pub frames: u32,
    pub spacing_metres: f32,
    pub box_half_metres: f32,
    pub timestep_seconds: f32,
    pub margin_metres: f32,
    /// Absolute path of the GPU library, or `None` for the SDK's default name.
    pub gpu_library_path: Option<String>,
}

/// Plan 23: the probe's outcome. `gpu_available == false` is a valid
/// outcome with a `reason`; the CPU bridge stays usable.
#[derive(Clone, Debug, PartialEq)]
pub struct PbdProbeReport {
    pub gpu_available: bool,
    pub reason: u32,
    pub frames_completed: u32,
    pub particles_out_of_bounds_max: u32,
    pub step_max_microseconds: u64,
    pub step_mean_microseconds: u64,
    pub readback_max_microseconds: u64,
    pub readback_mean_microseconds: u64,
    pub device_name: String,
    /// The final frame's positions, `4` floats per particle (position and
    /// inverse mass).
    pub final_positions: Vec<f32>,
}

impl PbdProbeReport {
    /// The bridge's reason codes as text.
    #[must_use]
    pub fn reason_text(&self) -> &'static str {
        match self.reason {
            0 => "ok",
            1 => "gpu library or cuda device unavailable",
            2 => "cuda context invalid",
            3 => "gpu scene creation failed",
            4 => "particle system or material creation failed",
            5 => "particle buffer creation failed",
            6 => "simulation step failed",
            _ => "unknown",
        }
    }
}

/// Plan 23: runs the PBD probe through the bridge.
pub fn pbd_probe(desc: &PbdProbeDesc) -> Result<PbdProbeReport, PhysXFfiError> {
    let version = version()?;
    validate_version(version)?;
    if desc.particle_count == 0 || desc.frames == 0 {
        return Err(PhysXFfiError::InvalidArgument);
    }
    let path = desc
        .gpu_library_path
        .as_deref()
        .map(std::ffi::CString::new)
        .transpose()
        .map_err(|_| PhysXFfiError::InvalidArgument)?;
    let mut final_positions = vec![0.0_f32; desc.particle_count as usize * 4];
    let raw_desc = PbdProbeDescRaw {
        particle_count: desc.particle_count,
        frames: desc.frames,
        spacing_bits: desc.spacing_metres.to_bits(),
        box_half_bits: desc.box_half_metres.to_bits(),
        timestep_bits: desc.timestep_seconds.to_bits(),
        margin_bits: desc.margin_metres.to_bits(),
        gpu_library_path: path.as_ref().map_or(std::ptr::null(), |path| path.as_ptr()),
        final_positions: final_positions.as_mut_ptr(),
    };
    let mut raw_report = PbdProbeReportRaw {
        gpu_available: 0,
        reason: 0,
        frames_completed: 0,
        particles_out_of_bounds_max: 0,
        step_max_microseconds: 0,
        step_mean_microseconds: 0,
        readback_max_microseconds: 0,
        readback_mean_microseconds: 0,
        device_name: [0; 64],
    };
    // SAFETY: the descriptor points at a live CString and at
    // `4 * particle_count` floats of writable storage that outlive the
    // call; the report is writable local storage; the bridge retains no
    // pointer after returning.
    let status = unsafe { raw::pbd_probe(&raw_desc, &mut raw_report) };
    status_result(status)?;
    let device_name = raw_report
        .device_name
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| *byte as u8 as char)
        .collect();
    Ok(PbdProbeReport {
        gpu_available: raw_report.gpu_available != 0,
        reason: raw_report.reason,
        frames_completed: raw_report.frames_completed,
        particles_out_of_bounds_max: raw_report.particles_out_of_bounds_max,
        step_max_microseconds: raw_report.step_max_microseconds,
        step_mean_microseconds: raw_report.step_mean_microseconds,
        readback_max_microseconds: raw_report.readback_max_microseconds,
        readback_mean_microseconds: raw_report.readback_mean_microseconds,
        device_name,
        final_positions,
    })
}

/// Plan 24: the persistent fluid descriptor as the bridge reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FluidDescRaw {
    pub spacing_bits: u32,
    pub box_min_bits: [u32; 3],
    pub box_max_bits: [u32; 3],
    pub seed_min_bits: [u32; 3],
    pub seed_max_bits: [u32; 3],
    pub timestep_bits: u32,
    pub max_particles: u32,
    pub gpu_library_path: *const std::ffi::c_char,
}

/// Plan 24: a GPU fluid in a box (floor at `box_min.y`, four walls, open
/// top) seeded on the spacing grid inside the seed block.
#[derive(Clone, Debug, PartialEq)]
pub struct FluidDesc {
    pub spacing_metres: f32,
    pub box_min_metres: [f32; 3],
    pub box_max_metres: [f32; 3],
    pub seed_min_metres: [f32; 3],
    pub seed_max_metres: [f32; 3],
    pub timestep_seconds: f32,
    pub max_particles: u32,
    pub gpu_library_path: Option<String>,
}

/// Plan 24: one readback of the fluid, metres and metres per second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FluidSample {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

/// Plan 24: a persistent GPU fluid of the bridge. `create` fails with
/// [`PhysXFfiError::Unavailable`] and a reason when the GPU lane cannot
/// start; the CPU bridge stays usable.
pub struct NativeFluid {
    handle: NonNull<c_void>,
    max_particles: u32,
    _single_threaded: PhantomData<Rc<()>>,
}

impl NativeFluid {
    pub fn create(desc: &FluidDesc) -> Result<Self, PhysXFfiError> {
        let version = version()?;
        validate_version(version)?;
        if desc.max_particles == 0 {
            return Err(PhysXFfiError::InvalidArgument);
        }
        let path = desc
            .gpu_library_path
            .as_deref()
            .map(std::ffi::CString::new)
            .transpose()
            .map_err(|_| PhysXFfiError::InvalidArgument)?;
        let raw = FluidDescRaw {
            spacing_bits: desc.spacing_metres.to_bits(),
            box_min_bits: desc.box_min_metres.map(f32::to_bits),
            box_max_bits: desc.box_max_metres.map(f32::to_bits),
            seed_min_bits: desc.seed_min_metres.map(f32::to_bits),
            seed_max_bits: desc.seed_max_metres.map(f32::to_bits),
            timestep_bits: desc.timestep_seconds.to_bits(),
            max_particles: desc.max_particles,
            gpu_library_path: path.as_ref().map_or(std::ptr::null(), |path| path.as_ptr()),
        };
        let mut handle = std::ptr::null_mut();
        let mut reason = 0_u32;
        // SAFETY: the descriptor points at a live CString; `handle` and
        // `reason` are writable locals the bridge fills before returning
        // and never retains.
        let status = unsafe { raw::fluid_create(&raw, &mut handle, &mut reason) };
        match status_result(status) {
            Ok(()) => {}
            Err(PhysXFfiError::InternalFailure) if reason != 0 => {
                return Err(PhysXFfiError::Unavailable);
            }
            Err(error) => return Err(error),
        }
        let handle = NonNull::new(handle).ok_or(PhysXFfiError::InternalFailure)?;
        Ok(Self {
            handle,
            max_particles: desc.max_particles,
            _single_threaded: PhantomData,
        })
    }

    pub fn step(&mut self) -> Result<(), PhysXFfiError> {
        // SAFETY: the handle is a live fluid owned by this value.
        status_result(unsafe { raw::fluid_step(self.handle.as_ptr()) })
    }

    /// Plan 25: replaces the active particle set (metres and metres per
    /// second); at most `max_particles` entries.
    pub fn set(
        &mut self,
        positions: &[[f32; 3]],
        velocities: &[[f32; 3]],
    ) -> Result<(), PhysXFfiError> {
        if positions.len() != velocities.len() {
            return Err(PhysXFfiError::InvalidArgument);
        }
        if positions.len() > self.max_particles as usize {
            return Err(PhysXFfiError::CapacityExceeded);
        }
        let flat_positions: Vec<f32> = positions.iter().flatten().copied().collect();
        let flat_velocities: Vec<f32> = velocities.iter().flatten().copied().collect();
        let count = u32::try_from(positions.len()).map_err(|_| PhysXFfiError::InvalidArgument)?;
        // SAFETY: both slices hold `3 * count` floats that outlive the call;
        // the bridge copies them and retains no pointer.
        let status = unsafe {
            raw::fluid_set(
                self.handle.as_ptr(),
                flat_positions.as_ptr(),
                flat_velocities.as_ptr(),
                count,
            )
        };
        status_result(status)
    }

    #[must_use]
    pub const fn max_particles(&self) -> u32 {
        self.max_particles
    }

    /// Plan 26: a kinematic box collider in the fluid (metres), created on
    /// first use of `slot` (`< 16`) and moved to `centre` otherwise.
    pub fn set_box(
        &mut self,
        slot: u32,
        centre_metres: [f32; 3],
        half_extents_metres: [f32; 3],
    ) -> Result<(), PhysXFfiError> {
        let centre = centre_metres.map(f32::to_bits);
        let half = half_extents_metres.map(f32::to_bits);
        // SAFETY: the handle is a live fluid owned by this value; both
        // arrays hold three lanes and outlive the call.
        status_result(unsafe {
            raw::fluid_set_box(self.handle.as_ptr(), slot, centre.as_ptr(), half.as_ptr())
        })
    }

    /// Plan 26: removes the collider of `slot`, if any.
    pub fn clear_box(&mut self, slot: u32) -> Result<(), PhysXFfiError> {
        // SAFETY: the handle is a live fluid owned by this value.
        status_result(unsafe { raw::fluid_clear_box(self.handle.as_ptr(), slot) })
    }

    pub fn read(&mut self) -> Result<FluidSample, PhysXFfiError> {
        let capacity = self.max_particles as usize;
        let mut positions = vec![0.0_f32; capacity * 3];
        let mut velocities = vec![0.0_f32; capacity * 3];
        let mut count = 0_u32;
        // SAFETY: both buffers hold `3 * max_particles` floats, the declared
        // capacity; the bridge writes at most `count <= max_particles`.
        let status = unsafe {
            raw::fluid_read(
                self.handle.as_ptr(),
                positions.as_mut_ptr(),
                velocities.as_mut_ptr(),
                self.max_particles,
                &mut count,
            )
        };
        status_result(status)?;
        let count = count as usize;
        Ok(FluidSample {
            positions: positions[..count * 3]
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect(),
            velocities: velocities[..count * 3]
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect(),
        })
    }
}

impl Drop for NativeFluid {
    fn drop(&mut self) {
        // SAFETY: the handle is a live fluid owned by this value and is
        // destroyed exactly once.
        unsafe { raw::fluid_destroy(self.handle.as_ptr()) };
    }
}

/// Plan 24: where the GPU library lives on this host —
/// `NEXTENGINE_PHYSX_GPU_LIBRARY`, else `NEXTENGINE_PHYSX_SDK_DIR/lib`, else
/// the SDK cache locator of `xtask physx setup`. `None` when nothing
/// resolves; the caller then reports the lane unavailable.
#[must_use]
pub fn gpu_library_path() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    const NAME: &str = "libPhysXGpu_64.so";
    if let Some(path) = std::env::var_os("NEXTENGINE_PHYSX_GPU_LIBRARY") {
        return Some(PathBuf::from(path));
    }
    if let Some(sdk) = std::env::var_os("NEXTENGINE_PHYSX_SDK_DIR") {
        return Some(PathBuf::from(sdk).join("lib").join(NAME));
    }
    let cache = if let Some(path) = std::env::var_os("NEXTENGINE_PHYSX_CACHE_DIR") {
        PathBuf::from(path).join("physx")
    } else if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path).join("nextengine").join("physx")
    } else {
        PathBuf::from(std::env::var_os("HOME")?)
            .join(".cache")
            .join("nextengine")
            .join("physx")
    };
    let target = format!(
        "{}-unknown-{}-gnu",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    let locator =
        std::fs::read_to_string(cache.join("active").join(format!("{target}.txt"))).ok()?;
    Some(PathBuf::from(locator.trim()).join("lib").join(NAME))
}

pub struct NativeWorld {
    handle: NonNull<c_void>,
    link_count: u32,
    joint_count: u32,
    max_contacts: u32,
    material_configured: bool,
    scene_configured: bool,
    _single_threaded: PhantomData<Rc<()>>,
}

impl NativeWorld {
    pub fn create() -> Result<Self, PhysXFfiError> {
        let version = version()?;
        validate_version(version)?;
        let mut handle = std::ptr::null_mut();
        // SAFETY: `handle` points to writable local storage and the bridge
        // initializes it on STATUS_OK without retaining the pointer-to-pointer.
        let status = unsafe { raw::world_create(&mut handle) };
        status_result(status)?;
        let handle = NonNull::new(handle).ok_or(PhysXFfiError::InternalFailure)?;
        Ok(Self {
            handle,
            link_count: 0,
            joint_count: 0,
            max_contacts: 0,
            material_configured: false,
            scene_configured: false,
            _single_threaded: PhantomData,
        })
    }

    pub fn configure_material(&mut self, input: MaterialProfileInput) -> Result<(), PhysXFfiError> {
        if self.material_configured || self.scene_configured {
            return Err(PhysXFfiError::InvalidArgument);
        }
        // SAFETY: the handle and fixed-layout input are valid for this
        // synchronous call; the bridge copies every field.
        status_result(unsafe { raw::world_configure_material(self.handle.as_ptr(), &input) })?;
        self.material_configured = true;
        Ok(())
    }

    pub fn configure_scene(&mut self, input: SceneProfileInput) -> Result<(), PhysXFfiError> {
        if !self.material_configured || self.scene_configured {
            return Err(PhysXFfiError::InvalidArgument);
        }
        // SAFETY: the handle and the fixed-layout input are valid for this call;
        // the bridge copies all values and retains no Rust pointer.
        status_result(unsafe { raw::world_configure_scene(self.handle.as_ptr(), &input) })?;
        self.max_contacts = input.max_contacts;
        self.scene_configured = true;
        Ok(())
    }

    pub fn reserve_static_boxes(&mut self, capacity: u32) -> Result<(), PhysXFfiError> {
        // SAFETY: `self.handle` is a live bridge-owned world until Drop and the
        // call does not retain any Rust reference.
        status_result(unsafe { raw::world_reserve(self.handle.as_ptr(), capacity) })
    }

    pub fn add_static_box(&mut self, input: StaticBoxInput) -> Result<(), PhysXFfiError> {
        // SAFETY: both fixed-size arrays remain alive for the duration of the
        // call; the bridge copies their bits and does not retain their pointers.
        status_result(unsafe {
            raw::world_add_static_box(
                self.handle.as_ptr(),
                input.centre_bits.as_ptr(),
                input.half_extents_bits.as_ptr(),
                input.user_token,
            )
        })
    }

    pub fn add_dynamic_box(&mut self, input: RigidBodyInput) -> Result<(), PhysXFfiError> {
        // SAFETY: the bridge reads the fixed-layout value synchronously.
        status_result(unsafe { raw::world_add_dynamic_box(self.handle.as_ptr(), &input) })
    }

    pub fn add_articulation(
        &mut self,
        links: &[ArticulationLinkInput],
        joints: &[ArticulationJointInput],
        position_iterations: u32,
        velocity_iterations: u32,
    ) -> Result<(), PhysXFfiError> {
        if links.is_empty()
            || joints.len().checked_add(1) != Some(links.len())
            || links.len() > u32::MAX as usize
            || joints.len() > u32::MAX as usize
            || links[0].parent_link_index != NO_PARENT_LINK
        {
            return Err(PhysXFfiError::InvalidArgument);
        }
        for (index, link) in links.iter().enumerate() {
            if index != 0 && link.parent_link_index as usize >= index {
                return Err(PhysXFfiError::InvalidArgument);
            }
            if links[..index]
                .iter()
                .any(|previous| previous.user_token == link.user_token)
            {
                return Err(PhysXFfiError::InvalidArgument);
            }
        }
        for (index, joint) in joints.iter().enumerate() {
            if joint.child_link_index as usize != index + 1 || joint.reserved != 0 {
                return Err(PhysXFfiError::InvalidArgument);
            }
        }
        // SAFETY: both slices remain live for the synchronous call. Their
        // elements have C layout and the bridge retains no pointers.
        status_result(unsafe {
            raw::world_add_articulation(
                self.handle.as_ptr(),
                links.as_ptr(),
                links.len() as u32,
                joints.as_ptr(),
                joints.len() as u32,
                position_iterations,
                velocity_iterations,
            )
        })?;
        self.link_count = links.len() as u32;
        self.joint_count = joints.len() as u32;
        Ok(())
    }

    pub fn add_articulation_v2(
        &mut self,
        links: &[ArticulationLinkInputV2],
        shapes: &[ArticulationShapeInputV2],
        joints: &[ArticulationJointInput],
        exclusions: &[ArticulationCollisionExclusionV2],
        position_iterations: u32,
        velocity_iterations: u32,
    ) -> Result<(), PhysXFfiError> {
        if links.is_empty()
            || joints.len().checked_add(1) != Some(links.len())
            || links.len() > u32::MAX as usize
            || shapes.len() > u32::MAX as usize
            || joints.len() > u32::MAX as usize
            || exclusions.len() > u32::MAX as usize
            || links[0].parent_link_index != NO_PARENT_LINK
        {
            return Err(PhysXFfiError::InvalidArgument);
        }
        let mut next_shape = 0_usize;
        for (index, link) in links.iter().enumerate() {
            if link.reserved != 0
                || (index != 0 && link.parent_link_index as usize >= index)
                || link.first_shape_index as usize != next_shape
                || links[..index]
                    .iter()
                    .any(|previous| previous.user_token == link.user_token)
            {
                return Err(PhysXFfiError::InvalidArgument);
            }
            next_shape = next_shape
                .checked_add(link.shape_count as usize)
                .ok_or(PhysXFfiError::InvalidArgument)?;
            if next_shape > shapes.len() {
                return Err(PhysXFfiError::InvalidArgument);
            }
        }
        if next_shape != shapes.len()
            || shapes.iter().enumerate().any(|(shape_index, shape)| {
                shape.link_index as usize >= links.len()
                    || !(links[shape.link_index as usize].first_shape_index as usize
                        ..links[shape.link_index as usize].first_shape_index as usize
                            + links[shape.link_index as usize].shape_count as usize)
                        .contains(&shape_index)
                    || shape.collision_layer >= 64
                    || (shape.collision_mask_low == 0 && shape.collision_mask_high == 0)
            })
            || joints.iter().enumerate().any(|(index, joint)| {
                joint.child_link_index as usize != index + 1 || joint.reserved != 0
            })
            || exclusions.iter().any(|pair| {
                pair.first_link_index >= pair.second_link_index
                    || pair.second_link_index as usize >= links.len()
            })
            || exclusions.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(PhysXFfiError::InvalidArgument);
        }
        // SAFETY: all slices remain live for the synchronous call. The bridge
        // copies fixed-layout values and retains no caller-owned pointers.
        status_result(unsafe {
            raw::world_add_articulation_v2(
                self.handle.as_ptr(),
                links.as_ptr(),
                links.len() as u32,
                shapes.as_ptr(),
                shapes.len() as u32,
                joints.as_ptr(),
                joints.len() as u32,
                exclusions.as_ptr(),
                exclusions.len() as u32,
                position_iterations,
                velocity_iterations,
            )
        })?;
        self.link_count = links.len() as u32;
        self.joint_count = joints.len() as u32;
        Ok(())
    }

    pub fn apply_articulation_efforts(&mut self, effort_bits: &[u32]) -> Result<(), PhysXFfiError> {
        if effort_bits.len() != self.joint_count as usize {
            return Err(PhysXFfiError::InvalidArgument);
        }
        // SAFETY: the slice is sized to the declared DOF count and retained only
        // for this call.
        status_result(unsafe {
            raw::world_apply_articulation_efforts(
                self.handle.as_ptr(),
                effort_bits.as_ptr(),
                effort_bits.len() as u32,
            )
        })
    }

    pub fn import_articulation_state(
        &mut self,
        root_state: LinkState,
        joint_states: &[JointState],
    ) -> Result<(), PhysXFfiError> {
        if joint_states.len() != self.joint_count as usize {
            return Err(PhysXFfiError::InvalidArgument);
        }
        // SAFETY: fixed-layout state buffers remain live for the synchronous call.
        status_result(unsafe {
            raw::world_import_articulation_state(
                self.handle.as_ptr(),
                &root_state,
                joint_states.as_ptr(),
                joint_states.len() as u32,
            )
        })
    }

    pub fn step(&mut self) -> Result<(), PhysXFfiError> {
        // SAFETY: the live world is exclusively borrowed for the complete step.
        status_result(unsafe { raw::world_step(self.handle.as_ptr()) })
    }

    pub fn export_articulation_state(
        &mut self,
    ) -> Result<(Vec<LinkState>, Vec<JointState>), PhysXFfiError> {
        let mut links = vec![LinkState::default(); self.link_count as usize];
        let mut joints = vec![JointState::default(); self.joint_count as usize];
        let mut link_count = 0;
        let mut joint_count = 0;
        // SAFETY: output slices have the exact capacities passed to the bridge,
        // and both count pointers refer to writable locals.
        status_result(unsafe {
            raw::world_export_articulation_state(
                self.handle.as_ptr(),
                links.as_mut_ptr(),
                self.link_count,
                &mut link_count,
                joints.as_mut_ptr(),
                self.joint_count,
                &mut joint_count,
            )
        })?;
        if link_count != self.link_count || joint_count != self.joint_count {
            return Err(PhysXFfiError::InvalidOutput);
        }
        Ok((links, joints))
    }

    pub fn export_contacts(&mut self) -> Result<Vec<ContactOutput>, PhysXFfiError> {
        let mut contacts = vec![ContactOutput::default(); self.max_contacts as usize];
        let mut count = 0;
        // SAFETY: output storage has the declared capacity and the count pointer
        // refers to writable local storage.
        status_result(unsafe {
            raw::world_export_contacts(
                self.handle.as_ptr(),
                contacts.as_mut_ptr(),
                self.max_contacts,
                &mut count,
            )
        })?;
        if count > self.max_contacts {
            return Err(PhysXFfiError::InvalidOutput);
        }
        contacts.truncate(count as usize);
        contacts.sort_unstable_by_key(|contact| {
            (
                contact.actor_a_token.min(contact.actor_b_token),
                contact.actor_a_token.max(contact.actor_b_token),
                contact.position_bits,
                contact.normal_bits,
            )
        });
        Ok(contacts)
    }

    pub fn export_contacts_v2(&mut self) -> Result<Vec<ContactOutputV2>, PhysXFfiError> {
        let mut contacts = vec![ContactOutputV2::default(); self.max_contacts as usize];
        let mut count = 0;
        // SAFETY: output storage has the declared capacity and the count
        // pointer refers to writable local storage.
        status_result(unsafe {
            raw::world_export_contacts_v2(
                self.handle.as_ptr(),
                contacts.as_mut_ptr(),
                self.max_contacts,
                &mut count,
            )
        })?;
        if count > self.max_contacts {
            return Err(PhysXFfiError::InvalidOutput);
        }
        contacts.truncate(count as usize);
        Ok(contacts)
    }

    pub fn sweep_capsule_axis(
        &mut self,
        input: CapsuleAxisSweepInput,
    ) -> Result<CapsuleAxisSweepOutput, PhysXFfiError> {
        let mut output = RawSweepOutput::default();
        // SAFETY: the world handle is live, the centre array and output storage
        // are valid for this call, and the bridge retains neither pointer.
        status_result(unsafe {
            raw::world_sweep_capsule_axis(
                self.handle.as_ptr(),
                input.centre_bits.as_ptr(),
                input.radius_bits,
                input.half_segment_bits,
                input.axis,
                input.direction_sign,
                input.distance_bits,
                &mut output,
            )
        })?;
        let hit = match output.hit {
            0 => false,
            1 => true,
            _ => return Err(PhysXFfiError::InvalidOutput),
        };
        Ok(CapsuleAxisSweepOutput {
            hit,
            distance_bits: output.distance_bits,
            normal_bits: [
                output.normal_x_bits,
                output.normal_y_bits,
                output.normal_z_bits,
            ],
            user_token: output.user_token,
        })
    }
}

fn validate_version(version: PhysXVersion) -> Result<(), PhysXFfiError> {
    if version == EXPECTED_PHYSX_VERSION {
        Ok(())
    } else {
        Err(PhysXFfiError::VersionMismatch)
    }
}

impl Drop for NativeWorld {
    fn drop(&mut self) {
        // SAFETY: this handle was returned by exactly one successful
        // `world_create` call and Drop is its unique destruction point.
        unsafe { raw::world_destroy(self.handle.as_ptr()) };
    }
}

impl std::fmt::Debug for NativeWorld {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeWorld")
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

pub fn version() -> Result<PhysXVersion, PhysXFfiError> {
    // SAFETY: the version function takes no pointers and returns a C-layout
    // value by copy.
    let version = unsafe { raw::version() };
    if version.abi == 0 {
        Err(PhysXFfiError::Unavailable)
    } else {
        Ok(version)
    }
}

fn status_result(status: i32) -> Result<(), PhysXFfiError> {
    match status {
        STATUS_OK => Ok(()),
        STATUS_INVALID_ARGUMENT => Err(PhysXFfiError::InvalidArgument),
        STATUS_OUT_OF_MEMORY => Err(PhysXFfiError::OutOfMemory),
        STATUS_CAPACITY_EXCEEDED => Err(PhysXFfiError::CapacityExceeded),
        STATUS_INTERNAL_FAILURE => Err(PhysXFfiError::InternalFailure),
        STATUS_UNAVAILABLE => Err(PhysXFfiError::Unavailable),
        _ => Err(PhysXFfiError::UnknownStatus(status)),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysXFfiError {
    Unavailable,
    VersionMismatch,
    InvalidArgument,
    OutOfMemory,
    CapacityExceeded,
    InvalidOutput,
    InternalFailure,
    UnknownStatus(i32),
}

impl PhysXFfiError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::Unavailable => "PHYSX_SDK_UNAVAILABLE",
            Self::VersionMismatch => "PHYSX_VERSION_MISMATCH",
            Self::InvalidArgument => "PHYSX_INVALID_ARGUMENT",
            Self::OutOfMemory => "PHYSX_OUT_OF_MEMORY",
            Self::CapacityExceeded => "PHYSX_CAPACITY_EXCEEDED",
            Self::InvalidOutput => "PHYSX_INVALID_OUTPUT",
            Self::InternalFailure | Self::UnknownStatus(_) => "PHYSX_INTERNAL_FAILURE",
        }
    }
}

impl Display for PhysXFfiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysXFfiError {}

#[cfg(feature = "physx-sdk")]
mod raw {
    use super::{
        ArticulationCollisionExclusionV2, ArticulationJointInput, ArticulationLinkInput,
        ArticulationLinkInputV2, ArticulationShapeInputV2, ContactOutput, ContactOutputV2,
        FluidDescRaw, JointState, LinkState, MaterialProfileInput, PbdProbeDescRaw,
        PbdProbeReportRaw, PhysXVersion, RawSweepOutput, RigidBodyInput, SceneProfileInput, c_void,
    };

    unsafe extern "C" {
        #[link_name = "ne_physx_version"]
        pub fn version() -> PhysXVersion;
        #[link_name = "ne_physx_pbd_probe"]
        pub fn pbd_probe(desc: *const PbdProbeDescRaw, report: *mut PbdProbeReportRaw) -> i32;
        #[link_name = "ne_physx_fluid_create"]
        pub fn fluid_create(
            desc: *const FluidDescRaw,
            output: *mut *mut c_void,
            reason: *mut u32,
        ) -> i32;
        #[link_name = "ne_physx_fluid_step"]
        pub fn fluid_step(fluid: *mut c_void) -> i32;
        #[link_name = "ne_physx_fluid_read"]
        pub fn fluid_read(
            fluid: *mut c_void,
            positions: *mut f32,
            velocities: *mut f32,
            capacity: u32,
            count: *mut u32,
        ) -> i32;
        #[link_name = "ne_physx_fluid_set"]
        pub fn fluid_set(
            fluid: *mut c_void,
            positions: *const f32,
            velocities: *const f32,
            count: u32,
        ) -> i32;
        #[link_name = "ne_physx_fluid_set_box"]
        pub fn fluid_set_box(
            fluid: *mut c_void,
            slot: u32,
            centre_bits: *const u32,
            half_extents_bits: *const u32,
        ) -> i32;
        #[link_name = "ne_physx_fluid_clear_box"]
        pub fn fluid_clear_box(fluid: *mut c_void, slot: u32) -> i32;
        #[link_name = "ne_physx_fluid_destroy"]
        pub fn fluid_destroy(fluid: *mut c_void);
        #[link_name = "ne_physx_world_create"]
        pub fn world_create(output: *mut *mut c_void) -> i32;
        #[link_name = "ne_physx_world_destroy"]
        pub fn world_destroy(world: *mut c_void);
        #[link_name = "ne_physx_world_configure_material"]
        pub fn world_configure_material(
            world: *mut c_void,
            input: *const MaterialProfileInput,
        ) -> i32;
        #[link_name = "ne_physx_world_configure_scene"]
        pub fn world_configure_scene(world: *mut c_void, input: *const SceneProfileInput) -> i32;
        #[link_name = "ne_physx_world_reserve"]
        pub fn world_reserve(world: *mut c_void, capacity: u32) -> i32;
        #[link_name = "ne_physx_world_add_static_box"]
        pub fn world_add_static_box(
            world: *mut c_void,
            centre_bits: *const u32,
            half_extents_bits: *const u32,
            user_token: u64,
        ) -> i32;
        #[link_name = "ne_physx_world_add_dynamic_box"]
        pub fn world_add_dynamic_box(world: *mut c_void, input: *const RigidBodyInput) -> i32;
        #[link_name = "ne_physx_world_add_articulation"]
        pub fn world_add_articulation(
            world: *mut c_void,
            links: *const ArticulationLinkInput,
            link_count: u32,
            joints: *const ArticulationJointInput,
            joint_count: u32,
            position_iterations: u32,
            velocity_iterations: u32,
        ) -> i32;
        #[link_name = "ne_physx_world_add_articulation_v2"]
        pub fn world_add_articulation_v2(
            world: *mut c_void,
            links: *const ArticulationLinkInputV2,
            link_count: u32,
            shapes: *const ArticulationShapeInputV2,
            shape_count: u32,
            joints: *const ArticulationJointInput,
            joint_count: u32,
            exclusions: *const ArticulationCollisionExclusionV2,
            exclusion_count: u32,
            position_iterations: u32,
            velocity_iterations: u32,
        ) -> i32;
        #[link_name = "ne_physx_world_apply_articulation_efforts"]
        pub fn world_apply_articulation_efforts(
            world: *mut c_void,
            efforts: *const u32,
            effort_count: u32,
        ) -> i32;
        #[link_name = "ne_physx_world_import_articulation_state"]
        pub fn world_import_articulation_state(
            world: *mut c_void,
            root: *const LinkState,
            joints: *const JointState,
            joint_count: u32,
        ) -> i32;
        #[link_name = "ne_physx_world_step"]
        pub fn world_step(world: *mut c_void) -> i32;
        #[link_name = "ne_physx_world_export_articulation_state"]
        pub fn world_export_articulation_state(
            world: *mut c_void,
            links: *mut LinkState,
            link_capacity: u32,
            link_count: *mut u32,
            joints: *mut JointState,
            joint_capacity: u32,
            joint_count: *mut u32,
        ) -> i32;
        #[link_name = "ne_physx_world_export_contacts"]
        pub fn world_export_contacts(
            world: *mut c_void,
            contacts: *mut ContactOutput,
            capacity: u32,
            count: *mut u32,
        ) -> i32;
        #[link_name = "ne_physx_world_export_contacts_v2"]
        pub fn world_export_contacts_v2(
            world: *mut c_void,
            contacts: *mut ContactOutputV2,
            capacity: u32,
            count: *mut u32,
        ) -> i32;
        #[link_name = "ne_physx_world_sweep_capsule_axis"]
        pub fn world_sweep_capsule_axis(
            world: *mut c_void,
            centre_bits: *const u32,
            radius_bits: u32,
            half_segment_bits: u32,
            axis: u32,
            direction_sign: i32,
            distance_bits: u32,
            output: *mut RawSweepOutput,
        ) -> i32;
    }
}

#[cfg(all(not(feature = "physx-sdk"), not(feature = "mock-abi")))]
mod raw {
    use super::{
        ArticulationCollisionExclusionV2, ArticulationJointInput, ArticulationLinkInput,
        ArticulationLinkInputV2, ArticulationShapeInputV2, ContactOutput, ContactOutputV2,
        FluidDescRaw, JointState, LinkState, MaterialProfileInput, PbdProbeDescRaw,
        PbdProbeReportRaw, PhysXVersion, RawSweepOutput, RigidBodyInput, STATUS_UNAVAILABLE,
        SceneProfileInput, c_void,
    };

    pub unsafe fn pbd_probe(_desc: *const PbdProbeDescRaw, _report: *mut PbdProbeReportRaw) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_create(
        _desc: *const FluidDescRaw,
        _output: *mut *mut c_void,
        _reason: *mut u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_step(_fluid: *mut c_void) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_read(
        _fluid: *mut c_void,
        _positions: *mut f32,
        _velocities: *mut f32,
        _capacity: u32,
        _count: *mut u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_set(
        _fluid: *mut c_void,
        _positions: *const f32,
        _velocities: *const f32,
        _count: u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_set_box(
        _fluid: *mut c_void,
        _slot: u32,
        _centre_bits: *const u32,
        _half_extents_bits: *const u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_clear_box(_fluid: *mut c_void, _slot: u32) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn fluid_destroy(_fluid: *mut c_void) {}

    pub unsafe fn version() -> PhysXVersion {
        PhysXVersion {
            abi: 0,
            major: 0,
            minor: 0,
            patch: 0,
        }
    }

    pub unsafe fn world_create(_output: *mut *mut c_void) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_destroy(_world: *mut c_void) {}

    pub unsafe fn world_configure_material(
        _world: *mut c_void,
        _input: *const MaterialProfileInput,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_configure_scene(
        _world: *mut c_void,
        _input: *const SceneProfileInput,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_reserve(_world: *mut c_void, _capacity: u32) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_add_static_box(
        _world: *mut c_void,
        _centre_bits: *const u32,
        _half_extents_bits: *const u32,
        _user_token: u64,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_add_dynamic_box(_world: *mut c_void, _input: *const RigidBodyInput) -> i32 {
        STATUS_UNAVAILABLE
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn world_add_articulation(
        _world: *mut c_void,
        _links: *const ArticulationLinkInput,
        _link_count: u32,
        _joints: *const ArticulationJointInput,
        _joint_count: u32,
        _position_iterations: u32,
        _velocity_iterations: u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn world_add_articulation_v2(
        _world: *mut c_void,
        _links: *const ArticulationLinkInputV2,
        _link_count: u32,
        _shapes: *const ArticulationShapeInputV2,
        _shape_count: u32,
        _joints: *const ArticulationJointInput,
        _joint_count: u32,
        _exclusions: *const ArticulationCollisionExclusionV2,
        _exclusion_count: u32,
        _position_iterations: u32,
        _velocity_iterations: u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_apply_articulation_efforts(
        _world: *mut c_void,
        _efforts: *const u32,
        _effort_count: u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_import_articulation_state(
        _world: *mut c_void,
        _root: *const LinkState,
        _joints: *const JointState,
        _joint_count: u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_step(_world: *mut c_void) -> i32 {
        STATUS_UNAVAILABLE
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn world_export_articulation_state(
        _world: *mut c_void,
        _links: *mut LinkState,
        _link_capacity: u32,
        _link_count: *mut u32,
        _joints: *mut JointState,
        _joint_capacity: u32,
        _joint_count: *mut u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_export_contacts(
        _world: *mut c_void,
        _contacts: *mut ContactOutput,
        _capacity: u32,
        _count: *mut u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    pub unsafe fn world_export_contacts_v2(
        _world: *mut c_void,
        _contacts: *mut ContactOutputV2,
        _capacity: u32,
        _count: *mut u32,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }

    #[allow(clippy::too_many_arguments)]
    pub unsafe fn world_sweep_capsule_axis(
        _world: *mut c_void,
        _centre_bits: *const u32,
        _radius_bits: u32,
        _half_segment_bits: u32,
        _axis: u32,
        _direction_sign: i32,
        _distance_bits: u32,
        _output: *mut RawSweepOutput,
    ) -> i32 {
        STATUS_UNAVAILABLE
    }
}

#[cfg(feature = "mock-abi")]
mod raw_mock;
#[cfg(feature = "mock-abi")]
use raw_mock as raw;

#[cfg(test)]
mod tests;
