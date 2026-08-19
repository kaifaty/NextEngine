#![forbid(unsafe_code)]

use std::time::Instant;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(crate) struct StepStageTimings {
    pub(crate) decode_nanoseconds: u64,
    pub(crate) reconstruction_nanoseconds: u64,
    pub(crate) initial_diagnostics_nanoseconds: u64,
    pub(crate) divergence_nanoseconds: u64,
    pub(crate) gravity_nanoseconds: u64,
    pub(crate) density_nanoseconds: u64,
    pub(crate) contact_nanoseconds: u64,
    pub(crate) integration_nanoseconds: u64,
    pub(crate) publication_nanoseconds: u64,
    pub(crate) total_nanoseconds: u64,
}

impl StepStageTimings {
    pub(crate) fn saturating_add_assign(&mut self, other: Self) {
        self.decode_nanoseconds = self
            .decode_nanoseconds
            .saturating_add(other.decode_nanoseconds);
        self.reconstruction_nanoseconds = self
            .reconstruction_nanoseconds
            .saturating_add(other.reconstruction_nanoseconds);
        self.initial_diagnostics_nanoseconds = self
            .initial_diagnostics_nanoseconds
            .saturating_add(other.initial_diagnostics_nanoseconds);
        self.divergence_nanoseconds = self
            .divergence_nanoseconds
            .saturating_add(other.divergence_nanoseconds);
        self.gravity_nanoseconds = self
            .gravity_nanoseconds
            .saturating_add(other.gravity_nanoseconds);
        self.density_nanoseconds = self
            .density_nanoseconds
            .saturating_add(other.density_nanoseconds);
        self.contact_nanoseconds = self
            .contact_nanoseconds
            .saturating_add(other.contact_nanoseconds);
        self.integration_nanoseconds = self
            .integration_nanoseconds
            .saturating_add(other.integration_nanoseconds);
        self.publication_nanoseconds = self
            .publication_nanoseconds
            .saturating_add(other.publication_nanoseconds);
        self.total_nanoseconds = self
            .total_nanoseconds
            .saturating_add(other.total_nanoseconds);
    }
}

#[derive(Clone, Copy)]
pub(super) enum StepStage {
    Decode,
    Reconstruction,
    InitialDiagnostics,
    Divergence,
    Gravity,
    Density,
    Contact,
    Integration,
    Publication,
}

pub(super) struct StepTimer<'a> {
    output: Option<&'a mut StepStageTimings>,
    started: Option<Instant>,
}

impl<'a> StepTimer<'a> {
    pub(super) fn new(output: Option<&'a mut StepStageTimings>) -> Self {
        let started = output.as_ref().map(|_| Instant::now());
        Self { output, started }
    }

    pub(super) fn finish(&mut self, stage: StepStage) {
        let Some(started) = self.started else {
            return;
        };
        let now = Instant::now();
        let elapsed = u64::try_from(now.duration_since(started).as_nanos()).unwrap_or(u64::MAX);
        let Some(output) = self.output.as_deref_mut() else {
            return;
        };
        match stage {
            StepStage::Decode => output.decode_nanoseconds = elapsed,
            StepStage::Reconstruction => output.reconstruction_nanoseconds = elapsed,
            StepStage::InitialDiagnostics => output.initial_diagnostics_nanoseconds = elapsed,
            StepStage::Divergence => output.divergence_nanoseconds = elapsed,
            StepStage::Gravity => output.gravity_nanoseconds = elapsed,
            StepStage::Density => output.density_nanoseconds = elapsed,
            StepStage::Contact => output.contact_nanoseconds = elapsed,
            StepStage::Integration => output.integration_nanoseconds = elapsed,
            StepStage::Publication => output.publication_nanoseconds = elapsed,
        }
        output.total_nanoseconds = output.total_nanoseconds.saturating_add(elapsed);
        self.started = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_timer_leaves_the_canonical_path_unmeasured() {
        let mut timer = StepTimer::new(None);
        timer.finish(StepStage::Decode);
    }

    #[test]
    fn aggregation_is_saturating() {
        let mut total = StepStageTimings {
            decode_nanoseconds: u64::MAX,
            total_nanoseconds: u64::MAX,
            ..StepStageTimings::default()
        };
        total.saturating_add_assign(StepStageTimings {
            decode_nanoseconds: 1,
            density_nanoseconds: 7,
            total_nanoseconds: 1,
            ..StepStageTimings::default()
        });
        assert_eq!(total.decode_nanoseconds, u64::MAX);
        assert_eq!(total.density_nanoseconds, 7);
        assert_eq!(total.total_nanoseconds, u64::MAX);
    }
}
