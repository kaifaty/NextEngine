mod articulated;
mod catalog;
mod codec;
mod contact;
mod descriptors;
mod error;
mod primitives;
mod profiles;
mod query;
mod snapshot;
mod step;

pub use catalog::{
    PhysicsCoordinateProfileV1, PhysicsLimitsProfileV1, PhysicsSolverSemanticsProfileV1,
    PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1, PhysicsWorldDescriptorV1,
};
pub use contact::{
    ClosedPhysicsContactBatchV1, ContactEventV1, ContactPhaseV1, PhysicsContactContinuityStateV1,
    derive_physics_contact_id,
};
pub use descriptors::{
    PhysicsBodyDescriptorV1, PhysicsMaterialDescriptorV1, PhysicsShapeDescriptorV1,
};
pub use error::PhysicsContractError;
pub use primitives::{
    PhysicsBodyIdV1, PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsMotionKindV1,
    PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeIdV1,
};
pub use profiles::{
    AuthoritativeNumericProfileV1, FixedPointDescriptorV1, PhysicsQuantizationProfileV1,
    PhysicsQuantizationRuleV1, PhysicsSourceFormatV1,
};
pub use query::{
    PhysicsQueryBatchV1, PhysicsQueryCardinalityV1, PhysicsQueryFilterV1, PhysicsQueryGeometryV1,
    PhysicsQueryHitV1, PhysicsQueryIdV1, PhysicsQueryKindV1, PhysicsQueryRequestV1,
    PhysicsQueryResultPayloadV1, PhysicsQueryResultV1, PhysicsQueryShapeV1,
    PhysicsSnapshotSelectorV1,
};
pub use snapshot::{PhysicsBodyStateV2, PhysicsCanonicalSnapshotV2, PhysicsWorldCheckpointV1};
pub use step::{
    AcceptedLocomotionIntentV2, AppliedLocomotionResultV1, PhysicalCommandV1, PhysicalEventV1,
    PhysicsStepInputV2, PhysicsStepResultV1,
};

pub const PHYSICAL_COMMAND_SCHEMA_ID: &str = "nextengine.command.physical";
pub const PHYSICAL_COMMAND_CAPABILITY_ID: &str = "nextengine.capability.physical-avatar-intent";
pub const PHYSICAL_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const PHYSICS_SNAPSHOT_SCHEMA_VERSION: u16 = 2;
pub const LEGACY_PHYSICS_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
pub const AUTHORITATIVE_NUMERIC_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_QUANTIZATION_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND: i64 = 3_000_000;
pub const PHYSICS_SWEEP_DISTANCE_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.sweep-distance";
pub const PHYSICS_CONTACT_NORMAL_X_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-x";
pub const PHYSICS_CONTACT_NORMAL_Y_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-y";
pub const PHYSICS_CONTACT_NORMAL_Z_FIELD_ID: &str =
    "nextengine.physics.raw.grounded-capsule.contact-normal-z";
pub const PHYSICS_METRES_UNIT_ID: &str = "nextengine.unit.metre";
pub const PHYSICS_MICROMETRES_UNIT_ID: &str = "nextengine.unit.micrometre";
pub const PHYSICS_SCALAR_UNIT_ID: &str = "nextengine.unit.scalar";
pub const PHYSICS_MICROMETRES_FIXED_POINT_ID: &str = "nextengine.fixed.physics-micrometres-i64";
pub const PHYSICS_Q1_30_FIXED_POINT_ID: &str = "nextengine.fixed.q1-30";

pub const PHYSICS_SNAPSHOT_OWNER_ID: &str = "nextengine.physics";
pub const PHYSICS_SNAPSHOT_SCHEMA_ID: &str = "nextengine.physics-canonical-snapshot";
pub const PHYSICS_SNAPSHOT_SEGMENT_ID: &str = "v2";
pub const LEGACY_PHYSICS_SNAPSHOT_SEGMENT_ID: &str = "v1";
pub const PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID: &str = "nextengine.physics-world-checkpoint";
pub const PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID: &str = "v1";
pub const PHYSICS_STEP_INPUT_SCHEMA_VERSION: u16 = 2;
pub const CLOSED_PHYSICS_CONTACT_BATCH_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_QUERY_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_QUERY_BATCH_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_QUERY_RESULT_SCHEMA_VERSION: u16 = 1;
pub const MAX_PHYSICS_QUERY_REQUESTS_PER_BATCH: usize = 65_536;
pub const MAX_PHYSICS_QUERY_EXCLUSIONS: usize = 4_096;
pub const MAX_PHYSICS_QUERY_CANDIDATE_HITS: usize = 4_194_304;
pub const MAX_PHYSICS_QUERY_PUBLISHED_HITS: u32 = 4_096;
pub const MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES: i64 = 33_554_432_000_000;
pub const REFERENCE_GRAVITY_MICROMETRES_PER_SECOND_SQUARED: i64 = -9_792_000;
const PHYSICS_OWNER_ID: &str = "nextengine.physics";
const NUMERIC_OWNER_ID: &str = "nextengine.runtime";
const NUMERIC_PROFILE_SCHEMA_ID: &str = "nextengine.authoritative-numeric-profile";
const QUANTIZATION_PROFILE_SCHEMA_ID: &str = "nextengine.physics-quantization-profile";
const PHYSICAL_EVENT_SCHEMA_ID: &str = "nextengine.event.capsule-step-applied";
const SEGMENT_V1: &str = "v1";

#[cfg(test)]
mod tests;
pub use articulated::{
    AppliedActuatorEffortV1, PhysicsActuatorDescriptorV1, PhysicsArticulationJointStateV1,
    PhysicsBodyDescriptorV2, PhysicsCanonicalSnapshotV3, PhysicsJointDescriptorV1,
    PhysicsJointKindV1, PhysicsStepInputV3, PhysicsStepResultV2, PhysicsSubstepActuationV1,
    PhysicsWorldCatalogV2, PhysicsWorldCheckpointV2,
};
