use sha2::{Digest, Sha256};

use super::*;

pub fn methodology_for(scenario: PerformanceScenarioV1) -> PerformanceMethodologyV1 {
    let mut methodology = PerformanceMethodologyV1 {
        methodology_version: PERFORMANCE_METHODOLOGY_VERSION.to_owned(),
        warmup_samples: 0,
        measured_samples: 0,
        percentile_method: "nearest-rank".to_owned(),
        outlier_policy: "retain-all-samples".to_owned(),
        frame_critical_path: None,
        notes: Vec::new(),
    };
    match scenario {
        PerformanceScenarioV1::Smoke => {
            methodology.measured_samples = 4;
            methodology.notes = vec![
                "two-chunk streaming, five-object render planning, one-agent planning and live movement are smoke fixtures only".to_owned(),
                "the measured resource-observation window covers the four declared smoke workload bodies; live project preparation, validation/report assembly and scratch cleanup remain outside it".to_owned(),
                "aggregate smoke timings are report-only and cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::LongSessionSoak => {
            methodology.measured_samples = 3;
            methodology.notes = vec![
                "3,600 live ticks in three 1,200-tick windows with held movement and periodic camera input run through both the live driver and interactive application scheduler".to_owned(),
                "identity-index and command-body archive roots are recomputed at each window boundary after that window timer is sampled; the probes remain inside the declared measured window".to_owned(),
                "every 30 ticks the driver materializes an in-memory WorldCheckpoint for state-root and serialization-cost observation; this never publishes the ApplicationSession store or a save".to_owned(),
                "paired application samples split every 30th fixed step from the remaining fixed steps without adding persistence work; granular samples also separate driver prepare and infallible driver commit".to_owned(),
                "application input is staged before timing and each measured host pump advances exactly one 30 Hz fixed step".to_owned(),
                "project cook/publish/activation, driver/application launch, input and sample-buffer preparation, validation/report assembly and scratch cleanup are outside the measured resource-observation window".to_owned(),
                "the soak is report-only and diagnoses history-dependent degradation; it cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::InteractiveFrameSoak => {
            methodology.measured_samples = 240;
            methodology.frame_critical_path =
                Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)".to_owned());
            methodology.notes = vec![
                "240 FIFO-presented frames use the production Vulkan frame path at requested 1920x1080 and immutable reference-game render inputs".to_owned(),
                "phase timings separate event polling plus immutable frame-source update, frame-slot/acquire/image waits, frame-plan, command recording, submit, present and GPU execution".to_owned(),
                "the measured resource-observation window contains only the prepared desktop adapter run; project/scratch preparation, validation/report conversion and cleanup are outside it".to_owned(),
                "the static render-input fixture does not time the game composition root's main-to-simulation-worker handoff; production-worker-soak measures that boundary separately".to_owned(),
                "Vulkan timestamp count and the engine-owned device allocation ceiling are typed resource-counter evidence; swapchain storage remains driver-owned".to_owned(),
                "the workload is report-only and diagnostic; it is not the representative R2 alpha project and cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::ProductionWorkerSoak => {
            methodology.measured_samples = 240;
            methodology.notes = vec![
                "the inner production-worker-soak.v1 diagnostic contract submits 240 FIFO main-callback batches at 60 Hz through the game composition root's bounded queue and next-simulation worker; its containing performance scenario is hash-bound as production-worker-soak.v3".to_owned(),
                "the worker advances the production fixed-step application path, publishes shared immutable presentation snapshots, and the main-side callback reads the latest generation".to_owned(),
                "bounded raw samples separate queue send wait, dequeue age, ordinary fixed steps, lifecycle-boundary fixed steps when present, snapshot publication/read lock waits, and rendered sequence freshness".to_owned(),
                "diagnostic send and dequeue observations are linearized around the same bounded sync channel, so queue high-water is exact channel occupancy rather than an outstanding-work estimate".to_owned(),
                "scratch creation, sample-buffer reservation, application launch, initial publication and worker readiness precede the measured resource-observation window; 240 callbacks and bounded save-on-close/join are measured".to_owned(),
                "wall time and diagnostic sequence counters are operational metadata only and never select simulation work, ordering, or authoritative outcomes".to_owned(),
                "the workload is report-only and diagnostic; it is not a representative R2 workload and cannot close B-12".to_owned(),
            ];
        }
        PerformanceScenarioV1::R2AlphaRender => {
            methodology.warmup_samples = 600 * 3 * 2;
            methodology.measured_samples = 3_600 * 3 * 2;
            methodology.frame_critical_path =
                Some("max(cpu_extract_and_submit_us,gpu_timestamp_duration_us)".to_owned());
            methodology.notes = vec![
                "the data-first reference-alpha project supplies three semantically distinct 60-second windows: exploration, combat and UI/dialogue".to_owned(),
                "primary 1920x1080 and b0-safe-720p30 1280x720 are separate launch profiles; each profile/window pair uses 600 warm-up and 3,600 measured frames".to_owned(),
                "CPU/GPU frame samples exclude VSync wait, retain every outlier and count primary/fallback deadline misses separately".to_owned(),
                "project activation and deterministic scenario/presentation extraction precede the observation window; six production Vulkan adapters are prepared, measured and released sequentially so device residency is a true per-run ceiling".to_owned(),
                "logical charges use the hash-bound r2-alpha-render-v1 accounting profile; physical evidence consists of peak working set, process I/O, device allocation ceilings and Vulkan timestamps".to_owned(),
            ];
        }
        PerformanceScenarioV1::R3MultiregionStreaming => {
            methodology.measured_samples = 1_000;
            methodology.notes = vec![
                "1,000 transitions cycle over the canonical manifest-driven four-region/64-chunk route through production packaged I/O and the paired Runtime/World fixed-stage commit".to_owned(),
                "the measured resource-observation window contains only the prepared streaming workload; project cook/publish/activation, validation/report assembly and scratch cleanup remain outside it".to_owned(),
                "the production default is two bounded workers; focused streaming checks establish root parity for worker counts 1/2/4".to_owned(),
                "the workload is report-only while B-12 and the clean ten-run THOTH hard gate remain open".to_owned(),
            ];
        }
        PerformanceScenarioV1::R4_100Npc => {
            methodology.warmup_samples = 1_000;
            methodology.measured_samples = 10_000;
            methodology.notes = vec![
                "ADR-016 integrated 100-NPC workload at 30 Hz".to_owned(),
                "all stage rows are exclusive; unowned time invalidates the run".to_owned(),
            ];
        }
        PerformanceScenarioV1::R5Physics16 => {
            methodology.warmup_samples = 240;
            methodology.measured_samples = 10_000;
            methodology.notes = vec![
                "16 independent engine-owned 23-DoF humanoids use the production PhysX CPU articulation path, fixed standing residual targets, 240 Hz physics and 60 Hz motor cadence".to_owned(),
                "each 1/4/8-worker configuration receives the same semantic slot identities, 240 warm-up physics substeps per slot and 10,000 measured physics substeps per slot".to_owned(),
                "worker shards advance in lockstep; publication and root aggregation are ordered by vector slot rather than worker completion".to_owned(),
                "the eight-worker run records per-slot canonical checkpoint size and fresh-scene replay-prefix restore latency; restore replays post-safety efforts without PD re-evaluation".to_owned(),
                "throughput and scaling efficiency are descriptive higher-is-better details; hard metrics remain lower-is-better latency, memory and restore costs".to_owned(),
                "wall-clock samples never select action, simulation work, ordering, reset or outcome; exact authoritative root parity is required across all worker counts and profiler control".to_owned(),
            ];
        }
    }
    methodology
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

pub(super) fn relative_change_basis_points(current: u64, baseline: u64) -> i64 {
    if baseline == 0 {
        return if current == 0 { 0 } else { i64::MAX };
    }
    let current = i128::from(current);
    let baseline = i128::from(baseline);
    let value = (current - baseline)
        .saturating_mul(10_000)
        .checked_div(baseline)
        .unwrap_or(i128::from(i64::MAX));
    i64::try_from(value).unwrap_or_else(|_| {
        if value.is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

pub(super) fn relative_verdict(
    absolute: PerformanceVerdict,
    change_basis_points: i64,
    confidence_interval: [i64; 2],
) -> PerformanceVerdict {
    if absolute == PerformanceVerdict::Fail
        || (change_basis_points >= 500 && confidence_interval[0] >= 500)
    {
        PerformanceVerdict::Fail
    } else if change_basis_points >= 200 {
        PerformanceVerdict::Warning
    } else if absolute == PerformanceVerdict::Pass {
        PerformanceVerdict::Pass
    } else {
        PerformanceVerdict::ReportOnly
    }
}

pub(super) fn bootstrap_change_interval(
    current: &[u64],
    baseline: &[u64],
    iterations: usize,
) -> Result<[i64; 2], String> {
    if current.is_empty() || baseline.is_empty() || iterations == 0 {
        return Err("bootstrap requires non-empty samples and iterations".to_owned());
    }
    let mut rng = XorShift64::new(0x4e45_5854_5045_5246);
    let mut current_resample = vec![0_u64; current.len()];
    let mut baseline_resample = vec![0_u64; baseline.len()];
    let mut changes = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        for sample in &mut current_resample {
            *sample = current[rng.index(current.len())];
        }
        for sample in &mut baseline_resample {
            *sample = baseline[rng.index(baseline.len())];
        }
        let current_p95 = nearest_rank_percentile(&current_resample, 95)?;
        let baseline_p95 = nearest_rank_percentile(&baseline_resample, 95)?;
        changes.push(relative_change_basis_points(current_p95, baseline_p95));
    }
    changes.sort_unstable();
    Ok([
        nearest_rank_fraction_i64(&changes, 25, 1_000)?,
        nearest_rank_fraction_i64(&changes, 975, 1_000)?,
    ])
}

fn nearest_rank_fraction_i64(
    samples: &[i64],
    numerator: usize,
    denominator: usize,
) -> Result<i64, String> {
    if samples.is_empty() || numerator == 0 || numerator > denominator {
        return Err("invalid signed nearest-rank fraction input".to_owned());
    }
    let rank_numerator = numerator
        .checked_mul(samples.len())
        .ok_or_else(|| "signed nearest-rank index overflow".to_owned())?;
    let rank = rank_numerator.div_ceil(denominator);
    Ok(samples[rank.saturating_sub(1)])
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn index(&mut self, length: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(length).unwrap_or(u64::MAX)).unwrap_or(0)
    }
}
