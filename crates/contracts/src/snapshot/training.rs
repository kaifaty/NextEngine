use crate::canonical::{CanonicalError, sha256};
use crate::ids::{StateRoot, content_hash_from_bytes};
use crate::motor::{MotorContractError, MotorWorldCheckpointV1};
use crate::physics::{PhysicsContractError, PhysicsWorldCheckpointV2};
use crate::rpg::{RpgContractErrorV1, RpgSnapshotV2};

use super::{RuntimeSnapshotV3, SnapshotDecodeError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCheckpointV5 {
    pub runtime_snapshot: RuntimeSnapshotV3,
    pub rpg_snapshot: RpgSnapshotV2,
    pub physics_checkpoint: PhysicsWorldCheckpointV2,
    pub motor_checkpoint: MotorWorldCheckpointV1,
    pub state_root: StateRoot,
}

impl WorldCheckpointV5 {
    pub fn new(
        runtime_snapshot: RuntimeSnapshotV3,
        rpg_snapshot: RpgSnapshotV2,
        physics_checkpoint: PhysicsWorldCheckpointV2,
        motor_checkpoint: MotorWorldCheckpointV1,
    ) -> Result<Self, WorldCheckpointV5Error> {
        let mut value = Self {
            runtime_snapshot,
            rpg_snapshot,
            physics_checkpoint,
            motor_checkpoint,
            state_root: StateRoot::default(),
        };
        value.validate_components()?;
        value.state_root = value.compute_state_root()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), WorldCheckpointV5Error> {
        self.validate_components()?;
        if self.state_root != self.compute_state_root()? {
            return Err(WorldCheckpointV5Error::ClosureMismatch);
        }
        Ok(())
    }

    fn validate_components(&self) -> Result<(), WorldCheckpointV5Error> {
        self.runtime_snapshot.validate()?;
        self.rpg_snapshot.validate()?;
        self.physics_checkpoint.validate()?;
        self.motor_checkpoint.validate()?;
        if self.runtime_snapshot.authoritative_revision
            != self.physics_checkpoint.snapshot.base.world_revision
        {
            return Err(WorldCheckpointV5Error::RevisionMismatch);
        }
        Ok(())
    }

    fn compute_state_root(&self) -> Result<StateRoot, WorldCheckpointV5Error> {
        let runtime = self.runtime_snapshot.canonical_bytes()?;
        let rpg = self.rpg_snapshot.canonical_bytes()?;
        let physics = self.physics_checkpoint.checkpoint_hash_v2()?;
        let motor = self.motor_checkpoint.checkpoint_hash()?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.world-checkpoint.v5\0");
        for segment in [&runtime[..], &rpg[..], physics.as_bytes(), motor.as_bytes()] {
            bytes.extend_from_slice(
                &u64::try_from(segment.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(segment);
        }
        Ok(StateRoot::from_bytes(
            *content_hash_from_bytes(sha256(&bytes)).as_bytes(),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCheckpointV5Error {
    Runtime(SnapshotDecodeError),
    Rpg(RpgContractErrorV1),
    Physics(PhysicsContractError),
    Motor(MotorContractError),
    Canonical(CanonicalError),
    RevisionMismatch,
    ClosureMismatch,
}

impl From<SnapshotDecodeError> for WorldCheckpointV5Error {
    fn from(value: SnapshotDecodeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<RpgContractErrorV1> for WorldCheckpointV5Error {
    fn from(value: RpgContractErrorV1) -> Self {
        Self::Rpg(value)
    }
}

impl From<PhysicsContractError> for WorldCheckpointV5Error {
    fn from(value: PhysicsContractError) -> Self {
        Self::Physics(value)
    }
}

impl From<MotorContractError> for WorldCheckpointV5Error {
    fn from(value: MotorContractError) -> Self {
        Self::Motor(value)
    }
}

impl From<CanonicalError> for WorldCheckpointV5Error {
    fn from(value: CanonicalError) -> Self {
        Self::Canonical(value)
    }
}
