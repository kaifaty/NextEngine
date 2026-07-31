use ash::vk;

use crate::{DesktopAdapterError, DesktopFrameTimingSample};

const TIMESTAMPS_PER_FRAME: u32 = 2;

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
    samples: Vec<DesktopFrameTimingSample>,
    sample_capacity: usize,
    pending_cpu_microseconds: Option<u64>,
    timestamp_query_count: u64,
    dropped_samples: u64,
}

impl VulkanFrameProfiler {
    pub(super) fn new(
        device: &ash::Device,
        sample_capacity: u32,
        timestamp_period_nanoseconds: f32,
        timestamp_valid_bits: u32,
    ) -> Result<Self, DesktopAdapterError> {
        if sample_capacity == 0
            || timestamp_valid_bits == 0
            || timestamp_valid_bits > u64::BITS
            || !timestamp_period_nanoseconds.is_finite()
            || timestamp_period_nanoseconds <= 0.0
        {
            return Err(DesktopAdapterError::GpuTimestampsUnsupported);
        }
        let query_pool_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(TIMESTAMPS_PER_FRAME);
        // SAFETY: the device is live, no host pointers are retained, and this
        // profiler destroys the query pool before the device is destroyed.
        let query_pool = unsafe { device.create_query_pool(&query_pool_info, None) }?;
        Ok(Self {
            device: device.clone(),
            query_pool,
            timestamp_period_nanoseconds,
            timestamp_valid_bits,
            samples: Vec::with_capacity(
                usize::try_from(sample_capacity)
                    .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            ),
            sample_capacity: usize::try_from(sample_capacity)
                .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            pending_cpu_microseconds: None,
            timestamp_query_count: 0,
            dropped_samples: 0,
        })
    }

    pub(super) fn collect_pending(&mut self) -> Result<(), DesktopAdapterError> {
        let Some(cpu_extract_and_submit_microseconds) = self.pending_cpu_microseconds.take() else {
            return Ok(());
        };
        let mut timestamps = [0_u64; TIMESTAMPS_PER_FRAME as usize];
        // SAFETY: completion of the frame fence or device idle is established
        // by the caller before this read. The query pool contains exactly two
        // 64-bit timestamp results written by that submitted command buffer.
        unsafe {
            self.device.get_query_pool_results(
                self.query_pool,
                0,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )?;
        }
        let ticks = timestamp_delta(timestamps[0], timestamps[1], self.timestamp_valid_bits);
        let gpu_duration_microseconds =
            ticks_to_microseconds(ticks, self.timestamp_period_nanoseconds)?;
        if self.samples.len() < self.sample_capacity {
            self.samples.push(DesktopFrameTimingSample {
                cpu_extract_and_submit_microseconds,
                gpu_duration_microseconds,
            });
        } else {
            self.dropped_samples = self
                .dropped_samples
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        Ok(())
    }

    pub(super) fn write_start(&self, command_buffer: vk::CommandBuffer) {
        // SAFETY: the command buffer is recording, this query pool is not in
        // use after the prior frame fence completed, and both queries are reset
        // before the first timestamp write.
        unsafe {
            self.device.cmd_reset_query_pool(
                command_buffer,
                self.query_pool,
                0,
                TIMESTAMPS_PER_FRAME,
            );
            self.device.cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::TOP_OF_PIPE,
                self.query_pool,
                0,
            );
        }
    }

    pub(super) fn write_end(&self, command_buffer: vk::CommandBuffer) {
        // SAFETY: the same command buffer is still recording and query one was
        // reset with query zero before either timestamp was written.
        unsafe {
            self.device.cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                self.query_pool,
                1,
            );
        }
    }

    pub(super) fn mark_submitted(
        &mut self,
        cpu_extract_and_submit_microseconds: u64,
    ) -> Result<(), DesktopAdapterError> {
        if self.pending_cpu_microseconds.is_some() {
            return Err(DesktopAdapterError::GpuTimestampStateInvalid);
        }
        self.pending_cpu_microseconds = Some(cpu_extract_and_submit_microseconds);
        self.timestamp_query_count = self
            .timestamp_query_count
            .checked_add(u64::from(TIMESTAMPS_PER_FRAME))
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        Ok(())
    }

    pub(super) fn take_report(&mut self) -> FrameProfilingReport {
        if self.pending_cpu_microseconds.take().is_some() {
            self.dropped_samples = self.dropped_samples.saturating_add(1);
        }
        FrameProfilingReport {
            samples: std::mem::take(&mut self.samples),
            timestamp_query_count: self.timestamp_query_count,
            dropped_samples: self.dropped_samples,
        }
    }
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
}
