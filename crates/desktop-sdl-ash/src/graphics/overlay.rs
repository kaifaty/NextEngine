//! Per-frame semantic UI overlay orchestration for the graphics context.
//!
//! The inherent `GraphicsContext` implementation lives in this child module
//! so the parent file stays within the repository's per-file size budget;
//! child modules can access the parent's private fields.

use super::*;

impl GraphicsContext {
    /// Re-rasterizes and re-uploads the optional overlay for the current
    /// publication. Called after the frame-slot fence proves the slot's prior
    /// overlay sample completed; overlay failures never fail the frame.
    pub(super) fn update_ui_overlay(&mut self, snapshot: &PresentationSnapshotV2) {
        let Some(extent) = self
            .swapchain
            .as_ref()
            .map(|swapchain| swapchain.extent)
        else {
            return;
        };
        self.ui_overlay.update(
            snapshot,
            extent,
            &self.instance,
            self.physical_device,
            self.queue,
            self.queue_family_index,
        );
    }

    /// Returns `(frames drawn, successful uploads, bounded failures)`.
    pub(crate) const fn ui_overlay_counters(&self) -> (u64, u64, u64) {
        self.ui_overlay.counters()
    }
}
