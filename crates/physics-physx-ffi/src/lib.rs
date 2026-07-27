#![allow(
    unsafe_code,
    reason = "ADR-033 confines all PhysX ABI calls and pointer ownership to this crate"
)]

use std::error::Error;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

#[cfg(all(feature = "physx-sdk", feature = "mock-abi"))]
compile_error!("features `physx-sdk` and `mock-abi` are mutually exclusive");

pub const NEXTENGINE_PHYSX_ABI_VERSION: u32 = 1;
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
            _single_threaded: PhantomData,
        })
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
    use super::{PhysXVersion, RawSweepOutput, c_void};

    unsafe extern "C" {
        #[link_name = "ne_physx_version"]
        pub fn version() -> PhysXVersion;
        #[link_name = "ne_physx_world_create"]
        pub fn world_create(output: *mut *mut c_void) -> i32;
        #[link_name = "ne_physx_world_destroy"]
        pub fn world_destroy(world: *mut c_void);
        #[link_name = "ne_physx_world_reserve"]
        pub fn world_reserve(world: *mut c_void, capacity: u32) -> i32;
        #[link_name = "ne_physx_world_add_static_box"]
        pub fn world_add_static_box(
            world: *mut c_void,
            centre_bits: *const u32,
            half_extents_bits: *const u32,
            user_token: u64,
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
    use super::{PhysXVersion, RawSweepOutput, STATUS_UNAVAILABLE, c_void};

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
mod raw {
    use super::{
        EXPECTED_PHYSX_VERSION, PhysXVersion, RawSweepOutput, STATUS_CAPACITY_EXCEEDED,
        STATUS_INVALID_ARGUMENT, STATUS_OK, c_void,
    };

    struct MockBox {
        centre: [f32; 3],
        half_extents: [f32; 3],
        token: u64,
    }

    struct MockWorld {
        capacity: usize,
        boxes: Vec<MockBox>,
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
        });
        // SAFETY: `output` was checked non-null and points to caller-owned
        // writable storage. Ownership of the Box moves to the opaque handle.
        unsafe { output.write(Box::into_raw(world).cast()) };
        STATUS_OK
    }

    pub unsafe fn world_destroy(world: *mut c_void) {
        if !world.is_null() {
            // SAFETY: the wrapper passes a handle created by `world_create`
            // exactly once at Drop.
            unsafe { drop(Box::from_raw(world.cast::<MockWorld>())) };
        }
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
        if centre_bits.is_null()
            || output.is_null()
            || axis > 2
            || !matches!(direction_sign, -1 | 1)
        {
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
            let candidate = if signed_delta > 0.0 && start <= lower && start + signed_delta > lower
            {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_and_abi_version_must_match_exactly() {
        assert_eq!(std::mem::size_of::<PhysXVersion>(), 16);
        assert_eq!(std::mem::size_of::<RawSweepOutput>(), 32);
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
}
