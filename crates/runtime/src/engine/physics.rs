use std::collections::BTreeMap;

use next_contracts::ids::PhysicsWorldId;
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsCanonicalSnapshotV2, PhysicsContractError,
    PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1, PhysicsQuantizationProfileV1,
    PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
    PhysicsWorldCheckpointV1,
};
#[cfg(not(any(feature = "physx", feature = "physx-mock")))]
use next_physics_api::ReferencePhysicsError;
use next_physics_api::{
    PhysicsBackendError, PhysicsBackendKind, PhysicsBackendPolicy, PhysicsWorldHost,
    ReferencePhysicsFactory,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicsLaunchOptions {
    pub backend_policy: PhysicsBackendPolicy,
}

impl PhysicsLaunchOptions {
    #[must_use]
    pub const fn new(backend_policy: PhysicsBackendPolicy) -> Self {
        Self { backend_policy }
    }

    pub(super) const fn require(kind: PhysicsBackendKind) -> Self {
        Self {
            backend_policy: match kind {
                PhysicsBackendKind::Reference => PhysicsBackendPolicy::ReferenceOnly,
                PhysicsBackendKind::PhysX => PhysicsBackendPolicy::RequirePhysX,
            },
        }
    }
}

pub(super) fn activate_physics(
    options: PhysicsLaunchOptions,
    checkpoint: PhysicsWorldCheckpointV1,
    tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    match options.backend_policy {
        PhysicsBackendPolicy::ReferenceOnly => PhysicsWorldHost::activate(
            &ReferencePhysicsFactory,
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        ),
        PhysicsBackendPolicy::RequirePhysX => activate_physx(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        ),
        PhysicsBackendPolicy::PreferPhysXThenReference => activate_physx(
            checkpoint.clone(),
            tick_rate_profile,
            numeric_profile.clone(),
            quantization_profile.clone(),
        )
        .or_else(|_| {
            PhysicsWorldHost::activate(
                &ReferencePhysicsFactory,
                checkpoint,
                tick_rate_profile,
                numeric_profile,
                quantization_profile,
            )
        }),
    }
}

#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn activate_physx(
    checkpoint: PhysicsWorldCheckpointV1,
    tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    PhysicsWorldHost::activate(
        &next_physics_physx::PhysXPhysicsFactory,
        checkpoint,
        tick_rate_profile,
        numeric_profile,
        quantization_profile,
    )
}

#[cfg(not(any(feature = "physx", feature = "physx-mock")))]
fn activate_physx(
    _checkpoint: PhysicsWorldCheckpointV1,
    _tick_rate_profile: TickRateProfileV1,
    _numeric_profile: AuthoritativeNumericProfileV1,
    _quantization_profile: PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldHost, PhysicsBackendError> {
    Err(PhysicsBackendError::from(
        ReferencePhysicsError::BackendUnavailable,
    ))
}

pub(super) fn empty_physics_checkpoint(
    world_id: PhysicsWorldId,
    tick_rate_profile: &TickRateProfileV1,
    numeric_profile: &AuthoritativeNumericProfileV1,
    quantization_profile: &PhysicsQuantizationProfileV1,
) -> Result<PhysicsWorldCheckpointV1, PhysicsContractError> {
    let catalog = PhysicsWorldCatalogV1::new(
        world_id,
        PhysicsWorldCatalogProfilesV1 {
            coordinate: PhysicsCoordinateProfileV1::reference_v1()
                .expect("built-in coordinate profile identifier is valid"),
            limits: PhysicsLimitsProfileV1::reference_v1()
                .expect("built-in limits profile identifier is valid"),
            solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1()
                .expect("built-in solver profile identifier is valid"),
            tick_rate_hash: tick_rate_profile.profile_hash()?,
            authoritative_numeric_hash: numeric_profile.profile_hash()?,
            quantization_hash: quantization_profile.profile_hash()?,
        },
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )?;
    let snapshot = PhysicsCanonicalSnapshotV2::genesis(
        &catalog,
        tick_rate_profile,
        numeric_profile,
        quantization_profile,
    )?;
    PhysicsWorldCheckpointV1::new(catalog, snapshot)
}
