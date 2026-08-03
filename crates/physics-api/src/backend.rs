use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use next_contracts::canonical::CanonicalError;
use next_contracts::ids::ContentHash;
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PhysicsCanonicalSnapshotV2, PhysicsQuantizationProfileV1,
    PhysicsStepInputV2, PhysicsStepResultV1, PhysicsWorldCheckpointV1,
};

use crate::{GroundedCapsuleQuery, GroundedCapsuleWorld, ReferencePhysicsError};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsBackendPolicy {
    #[default]
    ReferenceOnly,
    PreferPhysXThenReference,
    RequirePhysX,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsBackendKind {
    Reference,
    PhysX,
}

pub trait PhysicsBackendFactory {
    fn backend_kind(&self) -> PhysicsBackendKind;

    fn create(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError>;
}

pub trait PhysicsWorldBackend: Debug {
    fn backend_kind(&self) -> PhysicsBackendKind;
    fn checkpoint(&self) -> &PhysicsWorldCheckpointV1;
    fn snapshot(&self) -> &PhysicsCanonicalSnapshotV2;
    /// Exact hash of the current canonical snapshot. Backends MAY serve it
    /// from a derived private cache; the value always equals
    /// `self.snapshot().snapshot_hash()`.
    fn snapshot_hash(&self) -> Result<ContentHash, CanonicalError> {
        self.snapshot().snapshot_hash()
    }
    /// Exact hash of the immutable session catalog. Backends MAY serve it
    /// from a derived private cache; the value always equals
    /// `self.checkpoint().catalog.catalog_hash()`.
    fn catalog_hash(&self) -> Result<ContentHash, CanonicalError> {
        self.checkpoint().catalog.catalog_hash()
    }
    fn tick_rate_profile(&self) -> &TickRateProfileV1;
    fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1;
    fn quantization_profile(&self) -> &PhysicsQuantizationProfileV1;
    fn set_checkpoint_revision(&mut self, revision: u64);
    fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, PhysicsBackendError>;
    /// Forks an independent private world for an uncommitted live tick.
    ///
    /// The default deliberately reconstructs from the canonical checkpoint.
    /// A backend may override it only when it can copy every future-affecting
    /// private field. Durable recovery and replay continue to use
    /// `fork_from_checkpoint` regardless of this optimization.
    fn fork_for_staging(&self) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        self.fork_from_checkpoint(self.checkpoint().clone())
    }
    fn fork_from_checkpoint(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError>;
}

impl<Q> PhysicsWorldBackend for GroundedCapsuleWorld<Q>
where
    Q: GroundedCapsuleQuery + 'static,
{
    fn backend_kind(&self) -> PhysicsBackendKind {
        GroundedCapsuleWorld::backend_kind(self)
    }

    fn checkpoint(&self) -> &PhysicsWorldCheckpointV1 {
        GroundedCapsuleWorld::checkpoint(self)
    }

    fn snapshot(&self) -> &PhysicsCanonicalSnapshotV2 {
        GroundedCapsuleWorld::snapshot(self)
    }

    fn snapshot_hash(&self) -> Result<ContentHash, CanonicalError> {
        GroundedCapsuleWorld::snapshot_hash(self)
    }

    fn catalog_hash(&self) -> Result<ContentHash, CanonicalError> {
        GroundedCapsuleWorld::catalog_hash(self)
    }

    fn tick_rate_profile(&self) -> &TickRateProfileV1 {
        GroundedCapsuleWorld::tick_rate_profile(self)
    }

    fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1 {
        GroundedCapsuleWorld::numeric_profile(self)
    }

    fn quantization_profile(&self) -> &PhysicsQuantizationProfileV1 {
        GroundedCapsuleWorld::quantization_profile(self)
    }

    fn set_checkpoint_revision(&mut self, revision: u64) {
        GroundedCapsuleWorld::set_checkpoint_revision(self, revision);
    }

    fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, PhysicsBackendError> {
        GroundedCapsuleWorld::step(self, input).map_err(PhysicsBackendError::from)
    }

    fn fork_for_staging(&self) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        if let Some(staged) = GroundedCapsuleWorld::try_fork_for_staging(self)? {
            return Ok(Box::new(staged));
        }
        self.fork_from_checkpoint(self.checkpoint().clone())
    }

    fn fork_from_checkpoint(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        Ok(Box::new(
            GroundedCapsuleWorld::recreate_from_checkpoint(self, checkpoint)
                .map_err(PhysicsBackendError::from)?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ReferencePhysicsFactory;

impl PhysicsBackendFactory for ReferencePhysicsFactory {
    fn backend_kind(&self) -> PhysicsBackendKind {
        PhysicsBackendKind::Reference
    }

    fn create(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        Ok(Box::new(crate::ReferencePhysicsWorld::new(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        )?))
    }
}

pub struct PhysicsWorldHost {
    world: Box<dyn PhysicsWorldBackend>,
}

impl PhysicsWorldHost {
    pub fn activate(
        factory: &dyn PhysicsBackendFactory,
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, PhysicsBackendError> {
        let world = factory.create(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        )?;
        if world.backend_kind() != factory.backend_kind() {
            return Err(PhysicsBackendError::BackendIdentityMismatch);
        }
        Ok(Self { world })
    }

    pub fn reference(
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, PhysicsBackendError> {
        Self::activate(
            &ReferencePhysicsFactory,
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
        )
    }

    pub fn fork_from_checkpoint(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Self, PhysicsBackendError> {
        Ok(Self {
            world: self.world.fork_from_checkpoint(checkpoint)?,
        })
    }

    /// Creates an independent uncommitted live-tick generation.
    ///
    /// Backends without a proven complete private-state copy automatically use
    /// the canonical checkpoint reconstruction path.
    pub fn fork_for_staging(&self) -> Result<Self, PhysicsBackendError> {
        let world = self.world.fork_for_staging()?;
        if world.backend_kind() != self.world.backend_kind() {
            return Err(PhysicsBackendError::BackendIdentityMismatch);
        }
        Ok(Self { world })
    }

    #[must_use]
    pub fn backend_kind(&self) -> PhysicsBackendKind {
        self.world.backend_kind()
    }

    #[must_use]
    pub fn checkpoint(&self) -> &PhysicsWorldCheckpointV1 {
        self.world.checkpoint()
    }

    #[must_use]
    pub fn snapshot(&self) -> &PhysicsCanonicalSnapshotV2 {
        self.world.snapshot()
    }

    /// Exact hash of the current canonical snapshot, served from the
    /// backend's derived cache when available.
    pub fn snapshot_hash(&self) -> Result<ContentHash, CanonicalError> {
        self.world.snapshot_hash()
    }

    /// Exact hash of the immutable session catalog, served from the
    /// backend's derived cache when available.
    pub fn catalog_hash(&self) -> Result<ContentHash, CanonicalError> {
        self.world.catalog_hash()
    }

    #[must_use]
    pub fn tick_rate_profile(&self) -> &TickRateProfileV1 {
        self.world.tick_rate_profile()
    }

    #[must_use]
    pub fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1 {
        self.world.numeric_profile()
    }

    #[must_use]
    pub fn quantization_profile(&self) -> &PhysicsQuantizationProfileV1 {
        self.world.quantization_profile()
    }

    pub fn set_checkpoint_revision(&mut self, revision: u64) {
        self.world.set_checkpoint_revision(revision);
    }

    pub fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, PhysicsBackendError> {
        self.world.step(input)
    }

    pub fn checkpoint_hash(
        &self,
    ) -> Result<next_contracts::ids::ContentHash, next_contracts::canonical::CanonicalError> {
        self.checkpoint().checkpoint_hash()
    }
}

impl Debug for PhysicsWorldHost {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysicsWorldHost")
            .field("backend_kind", &self.backend_kind())
            .field("checkpoint", self.checkpoint())
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicsBackendError {
    World(ReferencePhysicsError),
    BackendIdentityMismatch,
}

impl PhysicsBackendError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::World(error) => error.stable_code(),
            Self::BackendIdentityMismatch => "PHYS_BACKEND_IDENTITY_MISMATCH",
        }
    }
}

impl Display for PhysicsBackendError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysicsBackendError {}

impl From<ReferencePhysicsError> for PhysicsBackendError {
    fn from(error: ReferencePhysicsError) -> Self {
        Self::World(error)
    }
}
