use super::*;

impl PreparedRuntimeTick {
    /// Immutable committed-contact candidate for presentation probes bound to
    /// this prepared tick. This does not materialize the full tick report.
    #[must_use]
    pub fn contact_batch(&self) -> &next_contracts::physics::ClosedPhysicsContactBatchV1 {
        &self.report_parts.contact_batch
    }
}
