#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use next_contracts::{
    AuthoritativeNumericProfileV1, PHYSICS_CONTACT_NORMAL_X_FIELD_ID,
    PHYSICS_CONTACT_NORMAL_Y_FIELD_ID, PHYSICS_CONTACT_NORMAL_Z_FIELD_ID,
    PHYSICS_SWEEP_DISTANCE_FIELD_ID, PhysicsQuantizationProfileV1, PhysicsShapeIdV1,
    PhysicsWorldCheckpointV1, TickRateProfileV1,
};
use next_physics_api::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepRequest,
    GroundedCapsuleSweepResult, GroundedCapsuleWorld, PhysicsBackendError, PhysicsBackendFactory,
    PhysicsBackendKind, PhysicsQuantizationError, PhysicsQuantizer, PhysicsWorldBackend,
    ReferencePhysicsError, grounded_capsule_collision_filter, reference_grounded_capsule_sweep,
};
use next_physics_physx_ffi::{CapsuleAxisSweepInput, NativeWorld, PhysXFfiError, StaticBoxInput};

pub type PhysXPhysicsWorld = GroundedCapsuleWorld<PhysXGroundedCapsuleQuery>;

pub struct PhysXGroundedCapsuleQuery {
    native: NativeWorld,
    quantizer: PhysicsQuantizer,
    loaded_shape_ids: BTreeMap<u64, PhysicsShapeIdV1>,
    loaded_scene: Option<Vec<GroundedCapsuleStaticBox>>,
    registration_seed: u64,
}

impl PhysXGroundedCapsuleQuery {
    pub fn new(
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, PhysXAdapterError> {
        Self::with_registration_seed(numeric_profile, quantization_profile, 0)
    }

    pub fn with_registration_seed(
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
        registration_seed: u64,
    ) -> Result<Self, PhysXAdapterError> {
        let expected_quantization = PhysicsQuantizationProfileV1::grounded_capsule_v2()
            .expect("built-in PhysX quantization identifiers are valid");
        let expected_numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&expected_quantization)
                .expect("built-in PhysX numeric profile is canonical");
        if quantization_profile != expected_quantization || numeric_profile != expected_numeric {
            return Err(PhysXAdapterError::UnsupportedProfile);
        }
        let quantizer = PhysicsQuantizer::new(&quantization_profile, &numeric_profile)?;
        for field in [
            PHYSICS_SWEEP_DISTANCE_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_X_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_Y_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_Z_FIELD_ID,
        ] {
            quantizer.quantize_f32_bits(field, 0.0_f32.to_bits())?;
        }
        Ok(Self {
            native: NativeWorld::create()?,
            quantizer,
            loaded_shape_ids: BTreeMap::new(),
            loaded_scene: None,
            registration_seed,
        })
    }

    fn ensure_loaded(
        &mut self,
        request: &GroundedCapsuleSweepRequest<'_>,
    ) -> Result<(), PhysXAdapterError> {
        let mut filtered = request
            .static_boxes
            .iter()
            .filter(|shape| {
                grounded_capsule_collision_filter(
                    request.capsule_collision_layer,
                    request.capsule_collision_mask,
                    shape,
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        filtered.sort_by_key(|shape| shape.shape_id);
        if let Some(loaded) = &self.loaded_scene {
            return if loaded == &filtered {
                Ok(())
            } else {
                Err(PhysXAdapterError::SceneChangedAfterActivation)
            };
        }
        let mut shapes = filtered
            .iter()
            .enumerate()
            .map(|(index, shape)| {
                let token = u64::try_from(index)
                    .ok()
                    .and_then(|index| index.checked_add(1))
                    .ok_or(PhysXAdapterError::CapacityExceeded)?;
                Ok((token, shape))
            })
            .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
        shapes.sort_by_key(|(token, _)| permutation_key(self.registration_seed, *token));
        self.native.reserve_static_boxes(
            u32::try_from(shapes.len()).map_err(|_| PhysXAdapterError::CapacityExceeded)?,
        )?;
        for (user_token, shape) in shapes {
            let centre = checked_midpoint(shape.minimum, shape.maximum)?;
            let half_extents = checked_half_extents(shape.minimum, shape.maximum)?;
            self.native.add_static_box(StaticBoxInput {
                centre_bits: metres_bits(centre)?,
                half_extents_bits: metres_bits(half_extents)?,
                user_token,
            })?;
            self.loaded_shape_ids.insert(user_token, shape.shape_id);
        }
        self.loaded_scene = Some(filtered);
        Ok(())
    }

    fn shape_for_token(&self, token: u64) -> Result<PhysicsShapeIdV1, PhysXAdapterError> {
        self.loaded_shape_ids
            .get(&token)
            .copied()
            .ok_or(PhysXAdapterError::InvalidOutput)
    }
}

impl Debug for PhysXGroundedCapsuleQuery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysXGroundedCapsuleQuery")
            .field("native", &self.native)
            .field("loaded_shape_ids", &self.loaded_shape_ids)
            .field(
                "loaded_scene_shape_count",
                &self.loaded_scene.as_ref().map(Vec::len),
            )
            .field("registration_seed", &self.registration_seed)
            .finish_non_exhaustive()
    }
}

impl GroundedCapsuleQuery for PhysXGroundedCapsuleQuery {
    fn backend_kind(&self) -> PhysicsBackendKind {
        PhysicsBackendKind::PhysX
    }

    fn recreate(&self) -> Result<Self, ReferencePhysicsError> {
        Self::with_registration_seed(
            self.quantizer.numeric_profile().clone(),
            self.quantizer.profile().clone(),
            self.registration_seed,
        )
        .map_err(ReferencePhysicsError::from)
    }

    fn sweep_axis(
        &mut self,
        request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
        let canonical = reference_grounded_capsule_sweep(GroundedCapsuleSweepRequest {
            centre_micrometres: request.centre_micrometres,
            axis: request.axis,
            delta_micrometres: request.delta_micrometres,
            capsule_radius_micrometres: request.capsule_radius_micrometres,
            capsule_half_segment_micrometres: request.capsule_half_segment_micrometres,
            capsule_collision_layer: request.capsule_collision_layer,
            capsule_collision_mask: request.capsule_collision_mask,
            static_boxes: request.static_boxes,
        })?;
        self.ensure_loaded(&request)?;
        if request.delta_micrometres == 0 {
            return Ok(canonical);
        }
        if canonical.applied_delta_micrometres == 0 && canonical.hit.is_some() {
            return Ok(canonical);
        }
        let distance_micrometres = request.delta_micrometres.unsigned_abs();
        let distance_micrometres =
            i64::try_from(distance_micrometres).map_err(|_| PhysXAdapterError::NumericOverflow)?;
        let output = self
            .native
            .sweep_capsule_axis(CapsuleAxisSweepInput {
                centre_bits: metres_bits(request.centre_micrometres)?,
                radius_bits: metres(request.capsule_radius_micrometres)?.to_bits(),
                half_segment_bits: metres(request.capsule_half_segment_micrometres)?.to_bits(),
                axis: u32::from(request.axis),
                direction_sign: request.delta_micrometres.signum() as i32,
                distance_bits: metres(distance_micrometres)?.to_bits(),
            })
            .map_err(PhysXAdapterError::from)?;
        if !output.hit {
            if canonical.hit.is_some() {
                return Err(ReferencePhysicsError::BackendHitMismatch);
            }
            return Ok(canonical);
        }
        let quantizer = &self.quantizer;
        let distance = quantizer
            .quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, output.distance_bits)
            .map_err(PhysXAdapterError::from)?;
        if canonical.hit.is_none() {
            if u64::try_from(distance).ok() == u64::try_from(distance_micrometres).ok() {
                return Ok(canonical);
            }
            return Err(ReferencePhysicsError::BackendHitMismatch);
        }
        let canonical_distance = canonical.applied_delta_micrometres.unsigned_abs();
        if distance < 0
            || distance > distance_micrometres
            || u64::try_from(distance).ok() != Some(canonical_distance)
        {
            return Err(ReferencePhysicsError::BackendDistanceMismatch);
        }
        let shape_id = self.shape_for_token(output.user_token)?;
        let hit = canonical.hit.ok_or(PhysXAdapterError::InvalidOutput)?;
        if hit.shape_id != shape_id {
            return Err(ReferencePhysicsError::BackendFeatureMismatch);
        }
        let quantized_normal = [
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_X_FIELD_ID, output.normal_bits[0])
                .map_err(PhysXAdapterError::from)?,
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_Y_FIELD_ID, output.normal_bits[1])
                .map_err(PhysXAdapterError::from)?,
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_Z_FIELD_ID, output.normal_bits[2])
                .map_err(PhysXAdapterError::from)?,
        ];
        let quantized_normal = [
            i32::try_from(quantized_normal[0]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
            i32::try_from(quantized_normal[1]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
            i32::try_from(quantized_normal[2]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
        ];
        if quantized_normal != hit.normal_box_to_capsule {
            return Err(ReferencePhysicsError::BackendNormalMismatch);
        }
        Ok(canonical)
    }
}

fn permutation_key(seed: u64, token: u64) -> u64 {
    let mut value = seed ^ token.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysXPhysicsFactory;

impl PhysicsBackendFactory for PhysXPhysicsFactory {
    fn backend_kind(&self) -> PhysicsBackendKind {
        PhysicsBackendKind::PhysX
    }

    fn create(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        let query =
            PhysXGroundedCapsuleQuery::new(numeric_profile.clone(), quantization_profile.clone())
                .map_err(ReferencePhysicsError::from)?;
        let world = PhysXPhysicsWorld::with_query(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
            query,
        )?;
        Ok(Box::new(world))
    }
}

fn checked_midpoint(minimum: [i64; 3], maximum: [i64; 3]) -> Result<[i64; 3], PhysXAdapterError> {
    let mut output = [0; 3];
    for axis in 0..3 {
        output[axis] = minimum[axis]
            .checked_add(maximum[axis])
            .and_then(|value| value.checked_div(2))
            .ok_or(PhysXAdapterError::NumericOverflow)?;
    }
    Ok(output)
}

fn checked_half_extents(
    minimum: [i64; 3],
    maximum: [i64; 3],
) -> Result<[i64; 3], PhysXAdapterError> {
    let mut output = [0; 3];
    for axis in 0..3 {
        output[axis] = maximum[axis]
            .checked_sub(minimum[axis])
            .and_then(|value| value.checked_div(2))
            .filter(|value| *value > 0)
            .ok_or(PhysXAdapterError::NumericOverflow)?;
    }
    Ok(output)
}

fn metres_bits(values: [i64; 3]) -> Result<[u32; 3], PhysXAdapterError> {
    Ok([
        metres(values[0])?.to_bits(),
        metres(values[1])?.to_bits(),
        metres(values[2])?.to_bits(),
    ])
}

fn metres(micrometres: i64) -> Result<f32, PhysXAdapterError> {
    let value = micrometres as f32 / 1_000_000.0;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PhysXAdapterError::NumericOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysXAdapterError {
    Ffi(PhysXFfiError),
    Quantization(PhysicsQuantizationError),
    UnsupportedProfile,
    CapacityExceeded,
    NumericOverflow,
    InvalidOutput,
    SceneChangedAfterActivation,
}

impl PhysXAdapterError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::Ffi(error) => error.stable_code(),
            Self::Quantization(error) => error.stable_code(),
            Self::UnsupportedProfile => "PHYSX_PROFILE_UNSUPPORTED",
            Self::CapacityExceeded => "PHYSX_CAPACITY_EXCEEDED",
            Self::NumericOverflow => "PHYSICS_NUMERIC_OVERFLOW",
            Self::InvalidOutput => "PHYSX_INVALID_OUTPUT",
            Self::SceneChangedAfterActivation => "PHYSX_SCENE_CHANGED_AFTER_ACTIVATION",
        }
    }
}

impl Display for PhysXAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysXAdapterError {}

impl From<PhysXFfiError> for PhysXAdapterError {
    fn from(error: PhysXFfiError) -> Self {
        Self::Ffi(error)
    }
}

impl From<PhysicsQuantizationError> for PhysXAdapterError {
    fn from(error: PhysicsQuantizationError) -> Self {
        Self::Quantization(error)
    }
}

impl From<PhysXAdapterError> for ReferencePhysicsError {
    fn from(error: PhysXAdapterError) -> Self {
        match error {
            PhysXAdapterError::Ffi(PhysXFfiError::Unavailable) => Self::BackendUnavailable,
            PhysXAdapterError::Ffi(PhysXFfiError::VersionMismatch) => Self::BackendVersionMismatch,
            PhysXAdapterError::Ffi(PhysXFfiError::CapacityExceeded)
            | PhysXAdapterError::CapacityExceeded => Self::BackendCapacityExceeded,
            PhysXAdapterError::Quantization(
                PhysicsQuantizationError::NumericOverflow | PhysicsQuantizationError::OutOfBounds,
            )
            | PhysXAdapterError::NumericOverflow => Self::NumericOverflow,
            PhysXAdapterError::UnsupportedProfile => Self::UnsupportedProfile,
            PhysXAdapterError::Ffi(_)
            | PhysXAdapterError::Quantization(_)
            | PhysXAdapterError::InvalidOutput
            | PhysXAdapterError::SceneChangedAfterActivation => Self::BackendFailure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(any(feature = "physx-sdk", feature = "mock-abi")))]
    fn disabled_sdk_is_reported_before_world_activation() {
        let quantization =
            PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
        let error =
            PhysXGroundedCapsuleQuery::new(numeric, quantization).expect_err("SDK unavailable");
        assert_eq!(error.stable_code(), "PHYSX_SDK_UNAVAILABLE");
    }

    #[test]
    fn legacy_reference_profile_is_not_accepted_by_physx() {
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        let error = PhysXGroundedCapsuleQuery::new(numeric, quantization)
            .expect_err("legacy profile remains reference-only");
        assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
    }

    #[test]
    fn altered_v2_recipe_is_not_accepted_under_the_builtin_profile_id() {
        let mut quantization =
            PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
        quantization
            .rules
            .get_mut(
                &next_contracts::SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID)
                    .expect("distance field ID"),
            )
            .expect("distance rule")
            .scale_numerator += 1;
        let numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
        let error = PhysXGroundedCapsuleQuery::new(numeric, quantization)
            .expect_err("altered built-in recipe");
        assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
    }
}
