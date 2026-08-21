//! Baseline-audio read API of the interactive simulation worker (A4).

use super::*;

impl InteractiveSimulationWorkerV1 {
    /// Latest published baseline-audio frame, if at least one fixed step has
    /// committed. Absent before the first audio publication; a poisoned lock
    /// is a typed failure (presentation-only, never gameplay evidence).
    pub fn read_latest_audio(
        &self,
    ) -> Result<Option<crate::ApplicationAudioFrameV1>, InteractiveWorkerFailureV1> {
        let latest = self.latest_audio.read().map_err(|_| {
            InteractiveWorkerFailureV1::runtime(
                "PLATFORM_PRESENTATION_STATE_POISONED",
                "latest audio frame lock was poisoned",
            )
        })?;
        Ok(latest.as_ref().cloned())
    }
}
