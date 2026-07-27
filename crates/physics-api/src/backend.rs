use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use next_contracts::{
    AuthoritativeNumericProfileV1, PhysicsCanonicalSnapshotV2, PhysicsQuantizationProfileV1,
    PhysicsStepInputV2, PhysicsStepResultV1, PhysicsWorldCheckpointV1, TickRateProfileV1,
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
    fn tick_rate_profile(&self) -> &TickRateProfileV1;
    fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1;
    fn quantization_profile(&self) -> &PhysicsQuantizationProfileV1;
    fn set_checkpoint_revision(&mut self, revision: u64);
    fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, PhysicsBackendError>;
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
    ) -> Result<next_contracts::ContentHash, next_contracts::CanonicalError> {
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
