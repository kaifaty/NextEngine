use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Barrier, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, content_hash_from_bytes};
use next_contracts::motor::STAGE0_SUBSTEPS;

use crate::{
    CompiledBodySchemaV1, DeterministicHumanoidMotor, HumanoidMotorCheckpoint,
    MotorReplayCodecError, MotorRuntimeError, REFERENCE_HUMANOID_DOF,
    reference_humanoid_body_schema_v1,
};

pub const REFERENCE_HUMANOID_PERFORMANCE_SLOTS: u32 = 16;
pub const REFERENCE_HUMANOID_PERFORMANCE_WARMUP_SUBSTEPS: u64 = 240;
pub const REFERENCE_HUMANOID_PERFORMANCE_MEASURED_SUBSTEPS: u64 = 10_000;
pub const REFERENCE_HUMANOID_PERFORMANCE_WORKERS: [u32; 3] = [1, 4, 8];

const PERFORMANCE_RUN_ROOT: ContentHash = ContentHash::from_bytes([
    0x52, 0x35, 0x50, 0x48, 0x59, 0x53, 0x58, 0x31, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanoidWorkerPerformanceV1 {
    pub worker_count: u32,
    pub elapsed_microseconds: u64,
    pub lockstep_motor_frame_microseconds: Vec<u64>,
    pub aggregate_physics_substeps_per_second: u64,
    pub aggregate_motor_frames_per_second: u64,
    pub scaling_efficiency_basis_points: u64,
    pub authoritative_root: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanoidPerformanceReportV1 {
    pub slot_count: u32,
    pub physics_hz: u32,
    pub motor_hz: u32,
    pub warmup_substeps_per_slot: u64,
    pub measured_substeps_per_slot: u64,
    pub measured_motor_frames_per_slot: u64,
    pub worker_runs: Vec<HumanoidWorkerPerformanceV1>,
    pub checkpoint_bytes_per_slot: Vec<u64>,
    pub restore_microseconds_per_slot: Vec<u64>,
    pub restore_wall_microseconds: u64,
    pub replay_prefix_substeps_per_slot: u64,
    pub authoritative_root: ContentHash,
    pub worker_root_parity: bool,
}

#[derive(Clone, Copy, Debug)]
struct WorkloadConfig {
    slot_count: u32,
    warmup_frames: u64,
    measured_frames: u64,
    worker_counts: &'static [u32],
    restore_worker_count: u32,
}

impl WorkloadConfig {
    const fn production() -> Self {
        Self {
            slot_count: REFERENCE_HUMANOID_PERFORMANCE_SLOTS,
            warmup_frames: REFERENCE_HUMANOID_PERFORMANCE_WARMUP_SUBSTEPS / STAGE0_SUBSTEPS as u64,
            measured_frames: REFERENCE_HUMANOID_PERFORMANCE_MEASURED_SUBSTEPS
                / STAGE0_SUBSTEPS as u64,
            worker_counts: &REFERENCE_HUMANOID_PERFORMANCE_WORKERS,
            restore_worker_count: 8,
        }
    }

    fn validate(self) -> Result<(), HumanoidPerformanceError> {
        if self.slot_count == 0
            || self.measured_frames == 0
            || self.worker_counts.is_empty()
            || self.worker_counts.iter().any(|workers| {
                *workers == 0
                    || *workers > self.slot_count
                    || !self.slot_count.is_multiple_of(*workers)
            })
            || !self.worker_counts.contains(&self.restore_worker_count)
        {
            return Err(HumanoidPerformanceError::InvalidConfiguration);
        }
        let total_frames = self
            .warmup_frames
            .checked_add(self.measured_frames)
            .ok_or(HumanoidPerformanceError::NumericOverflow)?;
        if total_frames > crate::MAX_REPLAY_MOTOR_TICKS as u64 {
            return Err(HumanoidPerformanceError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct SlotEvidence {
    vector_slot: u32,
    checkpoint: HumanoidMotorCheckpoint,
    checkpoint_bytes: u64,
    final_frame_hash: ContentHash,
    restore_microseconds: Option<u64>,
}

#[derive(Debug)]
struct ShardReport {
    elapsed: Duration,
    frame_durations: Vec<Duration>,
    restore_elapsed: Duration,
    slots: Vec<SlotEvidence>,
}

#[derive(Debug)]
struct WorkerRun {
    report: HumanoidWorkerPerformanceV1,
    slots: Vec<SlotEvidence>,
    restore_wall_microseconds: u64,
}

pub fn run_reference_humanoid_performance_v1()
-> Result<HumanoidPerformanceReportV1, HumanoidPerformanceError> {
    run_workload(WorkloadConfig::production())
}

fn run_workload(
    config: WorkloadConfig,
) -> Result<HumanoidPerformanceReportV1, HumanoidPerformanceError> {
    config.validate()?;
    let mut runs = Vec::with_capacity(config.worker_counts.len());
    for worker_count in config.worker_counts {
        runs.push(run_worker_configuration(config, *worker_count)?);
    }

    let expected_root = runs
        .first()
        .map(|run| run.report.authoritative_root)
        .ok_or(HumanoidPerformanceError::InvalidConfiguration)?;
    let worker_root_parity = runs
        .iter()
        .all(|run| run.report.authoritative_root == expected_root);
    if !worker_root_parity {
        return Err(HumanoidPerformanceError::WorkerRootDivergence);
    }

    let baseline_throughput = runs[0].report.aggregate_physics_substeps_per_second.max(1);
    for run in &mut runs {
        let numerator =
            u128::from(run.report.aggregate_physics_substeps_per_second).saturating_mul(10_000);
        let denominator =
            u128::from(baseline_throughput).saturating_mul(u128::from(run.report.worker_count));
        run.report.scaling_efficiency_basis_points = u64::try_from(numerator / denominator)
            .map_err(|_| HumanoidPerformanceError::NumericOverflow)?;
    }

    let restore = runs
        .iter_mut()
        .find(|run| run.report.worker_count == config.restore_worker_count)
        .ok_or(HumanoidPerformanceError::InvalidConfiguration)?;
    restore.slots.sort_by_key(|slot| slot.vector_slot);
    let checkpoint_bytes_per_slot = restore
        .slots
        .iter()
        .map(|slot| slot.checkpoint_bytes)
        .collect::<Vec<_>>();
    let restore_microseconds_per_slot = restore
        .slots
        .iter()
        .map(|slot| {
            slot.restore_microseconds
                .ok_or(HumanoidPerformanceError::RestoreEvidenceMissing)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let restore_wall_microseconds = restore.restore_wall_microseconds;
    let replay_prefix_substeps_per_slot = config
        .warmup_frames
        .checked_add(config.measured_frames)
        .and_then(|frames| frames.checked_mul(STAGE0_SUBSTEPS as u64))
        .ok_or(HumanoidPerformanceError::NumericOverflow)?;
    let worker_runs = runs.into_iter().map(|run| run.report).collect();

    Ok(HumanoidPerformanceReportV1 {
        slot_count: config.slot_count,
        physics_hz: next_physics_physx::PHYSX_CPU_TIMESTEP_HZ,
        motor_hz: next_physics_physx::PHYSX_CPU_TIMESTEP_HZ / STAGE0_SUBSTEPS as u32,
        warmup_substeps_per_slot: config.warmup_frames * STAGE0_SUBSTEPS as u64,
        measured_substeps_per_slot: config.measured_frames * STAGE0_SUBSTEPS as u64,
        measured_motor_frames_per_slot: config.measured_frames,
        worker_runs,
        checkpoint_bytes_per_slot,
        restore_microseconds_per_slot,
        restore_wall_microseconds,
        replay_prefix_substeps_per_slot,
        authoritative_root: expected_root,
        worker_root_parity,
    })
}

fn run_worker_configuration(
    config: WorkloadConfig,
    worker_count: u32,
) -> Result<WorkerRun, HumanoidPerformanceError> {
    let topology = next_cpu_affinity::CpuTopology::detect()
        .map_err(HumanoidPerformanceError::WorkerPlacementFailed)?;
    let placement = topology
        .deterministic_worker_placement(worker_count as usize)
        .map_err(HumanoidPerformanceError::WorkerPlacementFailed)?;
    let start_barrier = Arc::new(Barrier::new(worker_count as usize + 1));
    let frame_barrier = Arc::new(Barrier::new(worker_count as usize));
    let (ready_sender, ready_receiver) = mpsc::sync_channel(worker_count as usize);
    let measure_restore = worker_count == config.restore_worker_count;

    let shards = thread::scope(
        |scope| -> Result<Vec<ShardReport>, HumanoidPerformanceError> {
            let mut handles = Vec::with_capacity(worker_count as usize);
            for worker_index in 0..worker_count {
                let start_barrier = Arc::clone(&start_barrier);
                let frame_barrier = Arc::clone(&frame_barrier);
                let ready_sender = ready_sender.clone();
                let pinned_cpu = placement[worker_index as usize];
                handles.push(scope.spawn(move || {
                    // ADR-093: each worker pins itself before any warm-up so
                    // every measured frame runs in the declared scheduling
                    // state; failures fail closed before timing starts.
                    next_cpu_affinity::pin_current_thread(pinned_cpu)
                        .map_err(HumanoidPerformanceError::WorkerPlacementFailed)?;
                    run_shard(
                        config,
                        worker_count,
                        worker_index,
                        measure_restore,
                        &start_barrier,
                        &frame_barrier,
                        ready_sender,
                    )
                }));
            }
            drop(ready_sender);
            for _ in 0..worker_count {
                ready_receiver
                    .recv()
                    .map_err(|_| HumanoidPerformanceError::WorkerFailed)?;
            }
            start_barrier.wait();

            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| HumanoidPerformanceError::WorkerPanicked)?
                })
                .collect()
        },
    )?;

    let elapsed = shards
        .iter()
        .map(|shard| shard.elapsed)
        .max()
        .ok_or(HumanoidPerformanceError::WorkerFailed)?;
    let mut lockstep_motor_frame_microseconds = Vec::with_capacity(config.measured_frames as usize);
    for frame_index in 0..config.measured_frames as usize {
        let duration = shards
            .iter()
            .map(|shard| shard.frame_durations[frame_index])
            .max()
            .ok_or(HumanoidPerformanceError::WorkerFailed)?;
        lockstep_motor_frame_microseconds.push(duration_microseconds(duration)?);
    }
    let restore_wall_microseconds = duration_microseconds(
        shards
            .iter()
            .map(|shard| shard.restore_elapsed)
            .max()
            .unwrap_or_default(),
    )?;
    let mut slots = shards
        .into_iter()
        .flat_map(|shard| shard.slots)
        .collect::<Vec<_>>();
    slots.sort_by_key(|slot| slot.vector_slot);
    if slots.len() != config.slot_count as usize
        || slots
            .iter()
            .enumerate()
            .any(|(index, slot)| slot.vector_slot != index as u32)
    {
        return Err(HumanoidPerformanceError::SlotEvidenceInvalid);
    }
    let authoritative_root = aggregate_slot_root(&slots);
    if !measure_restore {
        slots.clear();
        slots.shrink_to_fit();
    }
    let elapsed_microseconds = duration_microseconds(elapsed)?.max(1);
    let aggregate_motor_frames =
        u128::from(config.slot_count).saturating_mul(u128::from(config.measured_frames));
    let aggregate_physics_substeps = aggregate_motor_frames.saturating_mul(STAGE0_SUBSTEPS as u128);
    let aggregate_motor_frames_per_second =
        rate_per_second(aggregate_motor_frames, elapsed_microseconds)?;
    let aggregate_physics_substeps_per_second =
        rate_per_second(aggregate_physics_substeps, elapsed_microseconds)?;

    Ok(WorkerRun {
        report: HumanoidWorkerPerformanceV1 {
            worker_count,
            elapsed_microseconds,
            lockstep_motor_frame_microseconds,
            aggregate_physics_substeps_per_second,
            aggregate_motor_frames_per_second,
            scaling_efficiency_basis_points: 0,
            authoritative_root,
        },
        slots,
        restore_wall_microseconds,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_shard(
    config: WorkloadConfig,
    worker_count: u32,
    worker_index: u32,
    measure_restore: bool,
    start_barrier: &Barrier,
    frame_barrier: &Barrier,
    ready_sender: mpsc::SyncSender<()>,
) -> Result<ShardReport, HumanoidPerformanceError> {
    let assigned_slots = (worker_index..config.slot_count)
        .step_by(worker_count as usize)
        .collect::<Vec<_>>();
    let standing_action = vec![0; REFERENCE_HUMANOID_DOF];
    let prepared = (|| {
        let mut runtimes = assigned_slots
            .iter()
            .map(|slot| create_runtime(*slot))
            .collect::<Result<Vec<_>, _>>()?;
        for _ in 0..config.warmup_frames {
            for runtime in &mut runtimes {
                runtime.step_motor_frame(&standing_action, [0; 3])?;
            }
        }
        Ok::<_, HumanoidPerformanceError>(runtimes)
    })();
    ready_sender
        .send(())
        .map_err(|_| HumanoidPerformanceError::WorkerFailed)?;
    start_barrier.wait();
    let mut runtimes = prepared?;

    let execution_started = Instant::now();
    let mut frame_durations = Vec::with_capacity(config.measured_frames as usize);
    let mut final_frame_hashes = vec![ContentHash::from_bytes([0; 32]); runtimes.len()];
    let mut frame_failure = None;
    for _ in 0..config.measured_frames {
        frame_barrier.wait();
        let frame_started = Instant::now();
        if frame_failure.is_none() {
            for (index, runtime) in runtimes.iter_mut().enumerate() {
                let result = match runtime.step_motor_frame(&standing_action, [0; 3]) {
                    Ok(result) => result,
                    Err(error) => {
                        frame_failure = Some(HumanoidPerformanceError::Runtime(error));
                        break;
                    }
                };
                match result.deterministic_hash() {
                    Ok(hash) => final_frame_hashes[index] = hash,
                    Err(error) => {
                        frame_failure = Some(HumanoidPerformanceError::Replay(error));
                        break;
                    }
                }
            }
        }
        frame_durations.push(frame_started.elapsed());
        frame_barrier.wait();
    }
    let elapsed = execution_started.elapsed();
    if let Some(error) = frame_failure {
        return Err(error);
    }

    let mut slots = Vec::with_capacity(runtimes.len());
    let restore_started = Instant::now();
    for ((vector_slot, runtime), final_frame_hash) in assigned_slots
        .into_iter()
        .zip(&mut runtimes)
        .zip(final_frame_hashes)
    {
        let checkpoint = runtime.checkpoint();
        let checkpoint_bytes = u64::try_from(checkpoint.canonical_bytes()?.len())
            .map_err(|_| HumanoidPerformanceError::NumericOverflow)?;
        let restore_microseconds = if measure_restore {
            let restore_started = Instant::now();
            runtime.restore_fresh(&checkpoint)?;
            let elapsed = duration_microseconds(restore_started.elapsed())?;
            if runtime.checkpoint() != checkpoint {
                return Err(HumanoidPerformanceError::RestoreDivergence);
            }
            Some(elapsed)
        } else {
            None
        };
        slots.push(SlotEvidence {
            vector_slot,
            checkpoint,
            checkpoint_bytes,
            final_frame_hash,
            restore_microseconds,
        });
    }
    let restore_elapsed = if measure_restore {
        restore_started.elapsed()
    } else {
        Duration::ZERO
    };
    Ok(ShardReport {
        elapsed,
        frame_durations,
        restore_elapsed,
        slots,
    })
}

fn create_runtime(
    vector_slot: u32,
) -> Result<DeterministicHumanoidMotor, HumanoidPerformanceError> {
    let schema = reference_humanoid_body_schema_v1();
    let compiled = CompiledBodySchemaV1::compile(&schema, subject_id(vector_slot))
        .map_err(|_| HumanoidPerformanceError::CompileFailed)?;
    Ok(DeterministicHumanoidMotor::create(compiled)?)
}

fn subject_id(vector_slot: u32) -> PersistentId {
    let mut preimage = b"nextengine.r5-physics-16.subject.v1\0".to_vec();
    preimage.extend_from_slice(PERFORMANCE_RUN_ROOT.as_bytes());
    preimage.extend_from_slice(&vector_slot.to_le_bytes());
    let digest = sha256(&preimage);
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    PersistentId::from_bytes(bytes)
}

fn aggregate_slot_root(slots: &[SlotEvidence]) -> ContentHash {
    let mut preimage = b"nextengine.r5-physics-16.authoritative-root.v1\0".to_vec();
    for slot in slots {
        preimage.extend_from_slice(&slot.vector_slot.to_le_bytes());
        preimage.extend_from_slice(canonical_motor_root(&slot.checkpoint).as_bytes());
        preimage.extend_from_slice(slot.final_frame_hash.as_bytes());
    }
    content_hash_from_bytes(sha256(&preimage))
}

fn canonical_motor_root(checkpoint: &HumanoidMotorCheckpoint) -> ContentHash {
    let mut preimage = b"nextengine.r5-physics-16.canonical-motor-state.v1\0".to_vec();
    preimage.extend_from_slice(&checkpoint.motor_tick.to_le_bytes());
    for value in &checkpoint.applied_action_microradians {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for value in checkpoint.command_raw {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for value in &checkpoint.previous_efforts_micronewton_metres {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for frame in &checkpoint.replay_frames {
        preimage.extend_from_slice(&frame.motor_tick.to_le_bytes());
        for value in &frame.post_safety_efforts_micronewton_metres {
            preimage.extend_from_slice(&value.to_le_bytes());
        }
        preimage.extend_from_slice(frame.physics_witness_hash.as_bytes());
    }
    content_hash_from_bytes(sha256(&preimage))
}

fn duration_microseconds(duration: Duration) -> Result<u64, HumanoidPerformanceError> {
    u64::try_from(duration.as_micros()).map_err(|_| HumanoidPerformanceError::NumericOverflow)
}

fn rate_per_second(
    count: u128,
    elapsed_microseconds: u64,
) -> Result<u64, HumanoidPerformanceError> {
    let value = count
        .saturating_mul(1_000_000)
        .checked_div(u128::from(elapsed_microseconds.max(1)))
        .ok_or(HumanoidPerformanceError::NumericOverflow)?;
    u64::try_from(value).map_err(|_| HumanoidPerformanceError::NumericOverflow)
}

#[derive(Debug)]
pub enum HumanoidPerformanceError {
    InvalidConfiguration,
    CompileFailed,
    WorkerFailed,
    WorkerPanicked,
    WorkerPlacementFailed(next_cpu_affinity::AffinityError),
    SlotEvidenceInvalid,
    WorkerRootDivergence,
    RestoreEvidenceMissing,
    RestoreDivergence,
    NumericOverflow,
    Runtime(MotorRuntimeError),
    Replay(MotorReplayCodecError),
}

impl HumanoidPerformanceError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::InvalidConfiguration => "MOTOR_PERF_CONFIGURATION_INVALID",
            Self::CompileFailed => "MOTOR_PERF_BODY_COMPILE_FAILED",
            Self::WorkerFailed => "MOTOR_PERF_WORKER_FAILED",
            Self::WorkerPanicked => "MOTOR_PERF_WORKER_PANICKED",
            Self::WorkerPlacementFailed(_) => "MOTOR_PERF_WORKER_PLACEMENT_FAILED",
            Self::SlotEvidenceInvalid => "MOTOR_PERF_SLOT_EVIDENCE_INVALID",
            Self::WorkerRootDivergence => "MOTOR_PERF_WORKER_ROOT_DIVERGENCE",
            Self::RestoreEvidenceMissing => "MOTOR_PERF_RESTORE_EVIDENCE_MISSING",
            Self::RestoreDivergence => "MOTOR_PERF_RESTORE_DIVERGENCE",
            Self::NumericOverflow => "MOTOR_PERF_NUMERIC_OVERFLOW",
            Self::Runtime(error) => error.stable_code(),
            Self::Replay(_) => "MOTOR_PERF_REPLAY_CODEC_FAILED",
        }
    }
}

impl Display for HumanoidPerformanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for HumanoidPerformanceError {}

impl From<MotorRuntimeError> for HumanoidPerformanceError {
    fn from(value: MotorRuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<MotorReplayCodecError> for HumanoidPerformanceError {
    fn from(value: MotorReplayCodecError) -> Self {
        Self::Replay(value)
    }
}

#[cfg(all(test, any(feature = "mock-abi", feature = "physx-sdk")))]
mod tests {
    use super::*;

    #[test]
    fn worker_permutations_preserve_roots_and_restore_exactly() {
        let report = run_workload(WorkloadConfig {
            slot_count: 4,
            warmup_frames: 1,
            measured_frames: 4,
            worker_counts: &[1, 2, 4],
            restore_worker_count: 4,
        })
        .expect("performance workload");
        assert!(report.worker_root_parity);
        assert_eq!(report.worker_runs.len(), 3);
        assert_eq!(report.checkpoint_bytes_per_slot.len(), 4);
        assert_eq!(report.restore_microseconds_per_slot.len(), 4);
        assert!(
            report
                .worker_runs
                .iter()
                .all(|run| run.authoritative_root == report.authoritative_root)
        );
    }

    #[test]
    fn non_restore_worker_run_releases_checkpoint_evidence_after_rooting() {
        let config = WorkloadConfig {
            slot_count: 2,
            warmup_frames: 1,
            measured_frames: 1,
            worker_counts: &[1, 2],
            restore_worker_count: 2,
        };

        let timing_only = run_worker_configuration(config, 1).expect("one-worker run");
        assert!(timing_only.slots.is_empty());

        let restore = run_worker_configuration(config, 2).expect("restore run");
        assert_eq!(restore.slots.len(), 2);
    }
}
