use ash::vk;

use crate::{DesktopAdapterError, DesktopFrameTimingSample};

/// start, particle-surface start, particle-surface end, end.
const TIMESTAMPS_PER_FRAME: u32 = 4;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CpuFramePhaseTimings {
    pub(super) event_and_frame_source_update_microseconds: u64,
    pub(super) frame_slot_wait_microseconds: u64,
    pub(super) image_acquire_wait_microseconds: u64,
    pub(super) swapchain_image_wait_microseconds: u64,
    pub(super) frame_plan_microseconds: u64,
    pub(super) dynamic_surface_upload_microseconds: u64,
    pub(super) dynamic_surface_uploads: u64,
    pub(super) command_record_microseconds: u64,
    pub(super) queue_submit_microseconds: u64,
}

#[derive(Clone, Copy, Debug)]
struct PendingFrame {
    sequence: u64,
    cpu_extract_and_submit_microseconds: u64,
    phases: CpuFramePhaseTimings,
    present_wait_microseconds: Option<u64>,
}

#[derive(Debug, Default)]
pub(crate) struct FrameProfilingReport {
    pub(crate) samples: Vec<DesktopFrameTimingSample>,
    pub(crate) timestamp_query_count: u64,
    pub(crate) dropped_samples: u64,
}

pub(super) struct VulkanFrameProfiler {
    device: ash::Device,
    query_pool: vk::QueryPool,
    timestamp_period_nanoseconds: f32,
    timestamp_valid_bits: u32,
    completed_samples: Vec<(u64, DesktopFrameTimingSample)>,
    sample_capacity: usize,
    pending_frames: Vec<Option<PendingFrame>>,
    next_sequence: u64,
    timestamp_query_count: u64,
    dropped_samples: u64,
}

impl VulkanFrameProfiler {
    pub(super) fn new(
        device: &ash::Device,
        sample_capacity: u32,
        timestamp_period_nanoseconds: f32,
        timestamp_valid_bits: u32,
        frame_slot_count: usize,
    ) -> Result<Self, DesktopAdapterError> {
        if sample_capacity == 0
            || frame_slot_count == 0
            || timestamp_valid_bits == 0
            || timestamp_valid_bits > u64::BITS
            || !timestamp_period_nanoseconds.is_finite()
            || timestamp_period_nanoseconds <= 0.0
        {
            return Err(DesktopAdapterError::GpuTimestampsUnsupported);
        }
        let frame_slot_count_u32 =
            u32::try_from(frame_slot_count).map_err(|_| DesktopAdapterError::CounterOverflow)?;
        let query_count = frame_slot_count_u32
            .checked_mul(TIMESTAMPS_PER_FRAME)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        let query_pool_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(query_count);
        // SAFETY: the device is live, no host pointers are retained, and this
        // profiler destroys the query pool before the device is destroyed.
        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None) }?;
        Ok(Self {
            device: device.clone(),
            query_pool,
            timestamp_period_nanoseconds,
            timestamp_valid_bits,
            completed_samples: Vec::with_capacity(
                usize::try_from(sample_capacity)
                    .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            ),
            sample_capacity: usize::try_from(sample_capacity)
                .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            pending_frames: vec![None; frame_slot_count],
            next_sequence: 0,
            timestamp_query_count: 0,
            dropped_samples: 0,
        })
    }

    pub(super) fn collect_pending(
        &mut self,
        frame_slot_index: usize,
    ) -> Result<(), DesktopAdapterError> {
        let pending = self
            .pending_frames
            .get_mut(frame_slot_index)
            .ok_or(DesktopAdapterError::GpuTimestampStateInvalid)?;
        let Some(pending) = pending.take() else {
            return Ok(());
        };
        let present_wait_microseconds = pending
            .present_wait_microseconds
            .ok_or(DesktopAdapterError::GpuTimestampStateInvalid)?;
        let query_start = timestamp_query_start(frame_slot_index)?;
        let mut timestamps = [0_u64; TIMESTAMPS_PER_FRAME as usize];
        // SAFETY: completion of the frame fence or device idle is established
        // by the caller before this read. The query pool contains exactly two
        // 64-bit timestamp results written by that submitted command buffer.
        unsafe {
            self.device.get_query_pool_results(
                self.query_pool,
                query_start,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )?;
        }
        let ticks = timestamp_delta(timestamps[0], timestamps[3], self.timestamp_valid_bits);
        let gpu_duration_microseconds =
            ticks_to_microseconds(ticks, self.timestamp_period_nanoseconds)?;
        let particle_ticks =
            timestamp_delta(timestamps[1], timestamps[2], self.timestamp_valid_bits);
        let particle_surface_gpu_microseconds =
            ticks_to_microseconds(particle_ticks, self.timestamp_period_nanoseconds)?;
        if self.completed_samples.len() < self.sample_capacity {
            self.completed_samples.push((
                pending.sequence,
                DesktopFrameTimingSample {
                    cpu_extract_and_submit_microseconds: pending
                        .cpu_extract_and_submit_microseconds,
                    gpu_duration_microseconds,
                    event_and_frame_source_update_microseconds: pending
                        .phases
                        .event_and_frame_source_update_microseconds,
                    frame_slot_wait_microseconds: pending.phases.frame_slot_wait_microseconds,
                    image_acquire_wait_microseconds: pending.phases.image_acquire_wait_microseconds,
                    swapchain_image_wait_microseconds: pending
                        .phases
                        .swapchain_image_wait_microseconds,
                    frame_plan_microseconds: pending.phases.frame_plan_microseconds,
                    dynamic_surface_upload_microseconds: pending
                        .phases
                        .dynamic_surface_upload_microseconds,
                    dynamic_surface_uploads: pending.phases.dynamic_surface_uploads,
                    command_record_microseconds: pending.phases.command_record_microseconds,
                    queue_submit_microseconds: pending.phases.queue_submit_microseconds,
                    present_wait_microseconds,
                    particle_surface_gpu_microseconds,
                },
            ));
        } else {
            self.dropped_samples = self
                .dropped_samples
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        Ok(())
    }

    pub(super) fn collect_all_pending(&mut self) -> Result<(), DesktopAdapterError> {
        for frame_slot_index in 0..self.pending_frames.len() {
            self.collect_pending(frame_slot_index)?;
        }
        Ok(())
    }

    pub(super) fn write_start(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
    ) -> Result<(), DesktopAdapterError> {
        let query_start = timestamp_query_start(frame_slot_index)?;
        // SAFETY: the command buffer is recording, this query pool is not in
        // use for this slot after its frame fence completed, and both queries
        // are reset before the first timestamp write.
        unsafe {
            self.device.cmd_reset_query_pool(
                command_buffer,
                self.query_pool,
                query_start,
                TIMESTAMPS_PER_FRAME,
            );
            self.device.cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::TOP_OF_PIPE,
                self.query_pool,
                query_start,
            );
        }
        Ok(())
    }

    /// Writes the particle-surface boundary timestamps (`1` start, `2` end);
    /// a frame without the pass writes both back to back.
    pub(super) fn write_particle_surface(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
        end: bool,
    ) -> Result<(), DesktopAdapterError> {
        let query_start = timestamp_query_start(frame_slot_index)?;
        // SAFETY: the command buffer is recording and the four queries of
        // this slot were reset by `write_start`.
        unsafe {
            self.device.cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                self.query_pool,
                query_start + if end { 2 } else { 1 },
            );
        }
        Ok(())
    }

    pub(super) fn write_end(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
    ) -> Result<(), DesktopAdapterError> {
        let query_start = timestamp_query_start(frame_slot_index)?;
        // SAFETY: the same command buffer is still recording and the queries
        // were reset before any timestamp was written.
        unsafe {
            self.device.cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                self.query_pool,
                query_start + 3,
            );
        }
        Ok(())
    }

    pub(super) fn mark_submitted(
        &mut self,
        frame_slot_index: usize,
        cpu_extract_and_submit_microseconds: u64,
        phases: CpuFramePhaseTimings,
    ) -> Result<(), DesktopAdapterError> {
        let pending = self
            .pending_frames
            .get_mut(frame_slot_index)
            .ok_or(DesktopAdapterError::GpuTimestampStateInvalid)?;
        if pending.is_some() {
            return Err(DesktopAdapterError::GpuTimestampStateInvalid);
        }
        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        *pending = Some(PendingFrame {
            sequence,
            cpu_extract_and_submit_microseconds,
            phases,
            present_wait_microseconds: None,
        });
        self.timestamp_query_count = self
            .timestamp_query_count
            .checked_add(u64::from(TIMESTAMPS_PER_FRAME))
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        Ok(())
    }

    pub(super) fn mark_presented(
        &mut self,
        frame_slot_index: usize,
        present_wait_microseconds: u64,
    ) -> Result<(), DesktopAdapterError> {
        let pending = self
            .pending_frames
            .get_mut(frame_slot_index)
            .and_then(Option::as_mut)
            .ok_or(DesktopAdapterError::GpuTimestampStateInvalid)?;
        if pending.present_wait_microseconds.is_some() {
            return Err(DesktopAdapterError::GpuTimestampStateInvalid);
        }
        pending.present_wait_microseconds = Some(present_wait_microseconds);
        Ok(())
    }

    pub(super) fn take_report(&mut self) -> FrameProfilingReport {
        for pending in &mut self.pending_frames {
            if pending.take().is_some() {
                self.dropped_samples = self.dropped_samples.saturating_add(1);
            }
        }
        self.completed_samples
            .sort_unstable_by_key(|(sequence, _)| *sequence);
        FrameProfilingReport {
            samples: std::mem::take(&mut self.completed_samples)
                .into_iter()
                .map(|(_, sample)| sample)
                .collect(),
            timestamp_query_count: self.timestamp_query_count,
            dropped_samples: self.dropped_samples,
        }
    }
}

fn timestamp_query_start(frame_slot_index: usize) -> Result<u32, DesktopAdapterError> {
    u32::try_from(frame_slot_index)
        .map_err(|_| DesktopAdapterError::CounterOverflow)?
        .checked_mul(TIMESTAMPS_PER_FRAME)
        .ok_or(DesktopAdapterError::CounterOverflow)
}

impl Drop for VulkanFrameProfiler {
    fn drop(&mut self) {
        // SAFETY: the owning GraphicsContext waits for device idle before this
        // profiler is dropped, and the pool belongs to the still-live device.
        unsafe {
            self.device.destroy_query_pool(self.query_pool, None);
        }
    }
}

fn timestamp_delta(start: u64, end: u64, valid_bits: u32) -> u64 {
    let mask = if valid_bits == u64::BITS {
        u64::MAX
    } else {
        (1_u64 << valid_bits) - 1
    };
    end.wrapping_sub(start) & mask
}

fn ticks_to_microseconds(
    ticks: u64,
    timestamp_period_nanoseconds: f32,
) -> Result<u64, DesktopAdapterError> {
    let microseconds = ((ticks as f64) * f64::from(timestamp_period_nanoseconds) / 1_000.0).ceil();
    if !microseconds.is_finite() || microseconds < 0.0 || microseconds > u64::MAX as f64 {
        return Err(DesktopAdapterError::CounterOverflow);
    }
    Ok(microseconds as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_delta_honors_queue_valid_bits_and_wraparound() {
        assert_eq!(timestamp_delta(100, 125, 64), 25);
        assert_eq!(timestamp_delta(250, 5, 8), 11);
        assert_eq!(timestamp_delta(0x00ff_ffff_ffff_ff00, 0x25, 56), 0x125);
    }

    #[test]
    fn timestamp_ticks_round_up_to_budget_microseconds() {
        assert_eq!(ticks_to_microseconds(1, 1.0).expect("one tick"), 1);
        assert_eq!(
            ticks_to_microseconds(1_000, 1.0).expect("one microsecond"),
            1
        );
        assert_eq!(ticks_to_microseconds(1_001, 1.0).expect("round up"), 2);
    }

    #[test]
    fn timestamp_queries_are_disjoint_for_each_frame_slot() {
        assert_eq!(timestamp_query_start(0).expect("slot zero"), 0);
        assert_eq!(timestamp_query_start(1).expect("slot one"), 4);
        assert_eq!(timestamp_query_start(2).expect("slot two"), 8);
    }
}
