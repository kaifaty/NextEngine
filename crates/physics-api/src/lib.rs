#![forbid(unsafe_code)]

mod backend;
mod quantization;
mod reference_world;

pub use backend::{
    PhysicsBackendError, PhysicsBackendFactory, PhysicsBackendKind, PhysicsBackendPolicy,
    PhysicsWorldBackend, PhysicsWorldHost, ReferencePhysicsFactory,
};
pub use quantization::{PhysicsQuantizationError, PhysicsQuantizer};
pub use reference_world::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepHit,
    GroundedCapsuleSweepRequest, GroundedCapsuleSweepResult, GroundedCapsuleWorld,
    ReferenceGroundedCapsuleQuery, ReferencePhysicsError, ReferencePhysicsWorld,
    grounded_capsule_axis_hit, grounded_capsule_collision_filter, reference_grounded_capsule_sweep,
};
