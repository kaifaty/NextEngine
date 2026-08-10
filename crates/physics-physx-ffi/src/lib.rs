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

pub const NEXTENGINE_PHYSX_ABI_VERSION: u32 = 2;
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

pub struct NativeWorld {
    handle: NonNull<c_void>,
    link_count: u32,
    joint_count: u32,
    max_contacts: u32,
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
            scene_configured: false,
            _single_threaded: PhantomData,
        })
    }

    pub fn configure_scene(&mut self, input: SceneProfileInput) -> Result<(), PhysXFfiError> {
        if self.scene_configured {
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
        ArticulationJointInput, ArticulationLinkInput, ContactOutput, JointState, LinkState,
        PhysXVersion, RawSweepOutput, RigidBodyInput, SceneProfileInput, c_void,
    };

    unsafe extern "C" {
        #[link_name = "ne_physx_version"]
        pub fn version() -> PhysXVersion;
        #[link_name = "ne_physx_world_create"]
        pub fn world_create(output: *mut *mut c_void) -> i32;
        #[link_name = "ne_physx_world_destroy"]
        pub fn world_destroy(world: *mut c_void);
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
        ArticulationJointInput, ArticulationLinkInput, ContactOutput, JointState, LinkState,
        PhysXVersion, RawSweepOutput, RigidBodyInput, STATUS_UNAVAILABLE, SceneProfileInput,
        c_void,
    };

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
mod tests {
    use super::*;

    #[test]
    fn sdk_and_abi_version_must_match_exactly() {
        assert_eq!(std::mem::size_of::<PhysXVersion>(), 16);
        assert_eq!(std::mem::size_of::<RawSweepOutput>(), 32);
        assert_eq!(std::mem::size_of::<SceneProfileInput>(), 36);
        assert_eq!(std::mem::size_of::<RigidBodyInput>(), 64);
        assert_eq!(std::mem::size_of::<ArticulationLinkInput>(), 80);
        assert_eq!(std::mem::size_of::<ArticulationJointInput>(), 76);
        assert_eq!(std::mem::size_of::<LinkState>(), 64);
        assert_eq!(std::mem::size_of::<JointState>(), 8);
        assert_eq!(std::mem::size_of::<ContactOutput>(), 56);
        assert_eq!(validate_version(EXPECTED_PHYSX_VERSION), Ok(()));
        let mut wrong_patch = EXPECTED_PHYSX_VERSION;
        wrong_patch.patch = 1;
        assert_eq!(
            validate_version(wrong_patch),
            Err(PhysXFfiError::VersionMismatch)
        );
        let mut wrong_abi = EXPECTED_PHYSX_VERSION;
        wrong_abi.abi += 1;
        assert_eq!(
            validate_version(wrong_abi),
            Err(PhysXFfiError::VersionMismatch)
        );
    }

    #[test]
    #[cfg(not(any(feature = "physx-sdk", feature = "mock-abi")))]
    fn disabled_sdk_fails_before_allocating_a_world() {
        assert!(matches!(
            NativeWorld::create(),
            Err(PhysXFfiError::Unavailable)
        ));
    }

    #[test]
    #[cfg(feature = "mock-abi")]
    fn mock_abi_owns_and_destroys_world() {
        for _ in 0..1_000 {
            let mut world = NativeWorld::create().expect("mock world");
            world.reserve_static_boxes(1).expect("reserve");
            world
                .add_static_box(StaticBoxInput {
                    centre_bits: [0.0_f32.to_bits(); 3],
                    half_extents_bits: [1.0_f32.to_bits(); 3],
                    user_token: 7,
                })
                .expect("box");
        }
    }

    #[test]
    #[cfg(feature = "mock-abi")]
    fn mock_abi_reports_capacity_and_native_validation_failures() {
        let mut world = NativeWorld::create().expect("mock world");
        world.reserve_static_boxes(0).expect("zero capacity");
        assert_eq!(
            world.add_static_box(StaticBoxInput {
                centre_bits: [0.0_f32.to_bits(); 3],
                half_extents_bits: [1.0_f32.to_bits(); 3],
                user_token: 7,
            }),
            Err(PhysXFfiError::CapacityExceeded)
        );
        assert_eq!(
            world.sweep_capsule_axis(CapsuleAxisSweepInput {
                centre_bits: [f32::NAN.to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()],
                radius_bits: 1.0_f32.to_bits(),
                half_segment_bits: 1.0_f32.to_bits(),
                axis: 0,
                direction_sign: 1,
                distance_bits: 1.0_f32.to_bits(),
            }),
            Err(PhysXFfiError::InvalidArgument)
        );
    }

    #[test]
    #[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
    fn scene_articulation_effort_step_and_state_round_trip() {
        let zero = 0.0_f32.to_bits();
        let one = 1.0_f32.to_bits();
        let mut world = NativeWorld::create().expect("world");
        world
            .configure_scene(SceneProfileInput {
                gravity_bits: [zero, (-9.81_f32).to_bits(), zero],
                timestep_bits: (1.0_f32 / 240.0).to_bits(),
                position_iterations: 8,
                velocity_iterations: 2,
                max_contacts: 128,
                max_actors: 8,
                max_joints: 4,
            })
            .expect("scene");
        world.reserve_static_boxes(1).expect("reserve ground");
        world
            .add_static_box(StaticBoxInput {
                centre_bits: [zero, (-0.5_f32).to_bits(), zero],
                half_extents_bits: [5.0_f32.to_bits(), 0.5_f32.to_bits(), 5.0_f32.to_bits()],
                user_token: 1,
            })
            .expect("ground");
        let identity = [zero, zero, zero, one];
        let links = [
            ArticulationLinkInput {
                user_token: 10,
                parent_link_index: NO_PARENT_LINK,
                shape_kind: SHAPE_CAPSULE,
                position_bits: [zero, 2.0_f32.to_bits(), zero],
                rotation_bits: identity,
                shape_dimensions_bits: [0.2_f32.to_bits(), 0.25_f32.to_bits(), zero],
                mass_bits: 5.0_f32.to_bits(),
                inertia_bits: [0.2_f32.to_bits(); 3],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            },
            ArticulationLinkInput {
                user_token: 11,
                parent_link_index: 0,
                shape_kind: SHAPE_CAPSULE,
                position_bits: [zero, 1.5_f32.to_bits(), zero],
                rotation_bits: identity,
                shape_dimensions_bits: [0.15_f32.to_bits(), 0.2_f32.to_bits(), zero],
                mass_bits: 2.0_f32.to_bits(),
                inertia_bits: [0.1_f32.to_bits(); 3],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            },
        ];
        let joints = [ArticulationJointInput {
            child_link_index: 1,
            reserved: 0,
            parent_position_bits: [zero, (-0.25_f32).to_bits(), zero],
            parent_rotation_bits: identity,
            child_position_bits: [zero, 0.2_f32.to_bits(), zero],
            child_rotation_bits: identity,
            lower_limit_bits: (-1.0_f32).to_bits(),
            upper_limit_bits: 1.0_f32.to_bits(),
            max_velocity_bits: 20.0_f32.to_bits(),
        }];
        world
            .add_articulation(&links, &joints, 8, 2)
            .expect("articulation");
        let initial = world.export_articulation_state().expect("initial state");
        world
            .apply_articulation_efforts(&[10.0_f32.to_bits()])
            .expect("effort");
        for _ in 0..4 {
            world.step().expect("step");
        }
        let stepped = world.export_articulation_state().expect("stepped state");
        assert_ne!(stepped.1[0].velocity_bits, initial.1[0].velocity_bits);
        world
            .import_articulation_state(initial.0[0], &initial.1)
            .expect("restore");
        let restored = world.export_articulation_state().expect("restored state");
        assert_eq!(restored.0[0], initial.0[0]);
        assert_eq!(restored.1, initial.1);
    }
}
