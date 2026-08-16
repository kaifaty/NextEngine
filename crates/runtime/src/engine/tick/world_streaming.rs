use super::*;

/// Runtime and World candidates prepared by the same ingress pipeline.
pub struct PreparedRuntimeWorldTick {
    runtime: PreparedRuntimeTick,
    world: next_world::PreparedWorldStreamingPublicationV1,
}

/// A paired generation for which both live bases were checked together.
pub struct ValidatedRuntimeWorldTick {
    runtime: ValidatedRuntimeTick,
    world: next_world::ValidatedWorldStreamingPublicationV1,
}

impl RuntimeTickPreparation<'_> {
    pub fn prepare_with_world_streaming(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        world: &next_world::WorldStreamerV1,
        publication: next_world::PreparedWorldStreamingPublicationV1,
    ) -> Result<PreparedRuntimeWorldTick, RuntimeFatalError> {
        let runtime_tick = self.runtime.prepare_tick_internal(
            self.base_generation,
            self.ingress_checkpoint,
            commands,
            &mut NoOutcomes,
            None,
            Some(WorldStreamingStageContext {
                world,
                publication: &publication,
            }),
            None,
            None,
            None,
        )?;
        Ok(PreparedRuntimeWorldTick {
            runtime: runtime_tick,
            world: publication,
        })
    }
}

impl RuntimeState {
    pub fn validate_prepared_world_tick(
        &self,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldTick,
    ) -> Result<ValidatedRuntimeWorldTick, RuntimeFatalError> {
        let runtime = self.validate_prepared_tick(prepared.runtime)?;
        let world = world.validate_prepared_publication(prepared.world, runtime.report().tick)?;
        Ok(ValidatedRuntimeWorldTick { runtime, world })
    }

    /// Publishes both jointly validated generations. No fallible work occurs
    /// after either live state begins to change.
    #[must_use]
    pub fn commit_validated_world_tick(
        &mut self,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldTick,
    ) -> (TickReport, Option<next_world::WorldTransitionCommitV1>) {
        let report = self.commit_validated_tick(validated.runtime);
        let world_receipt = world.commit_validated_publication(validated.world);
        (report, world_receipt)
    }
}
