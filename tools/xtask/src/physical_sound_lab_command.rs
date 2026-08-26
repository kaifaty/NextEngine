use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use next_presentation::physical_sound_lab::{
    ExperimentalPhysicalSoundMixer, GlassBodyVariant, PhysicalSoundExcitation,
    PhysicalSoundImpactPoint, PhysicalSoundMaterial, render_glass_body_variant_impact,
    render_physical_sound_impact, render_physical_sound_lab_sequence,
    render_selected_glass_q30_impact,
};
use serde::Serialize;

use crate::physical_sound_eval_command::{Q0Candidate, write_q0_manifest};

pub(super) struct Request {
    output: PathBuf,
    benchmark_runs: Option<usize>,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut output = None;
    let mut benchmark_runs = None;
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--output" if output.is_none() => {
                output =
                    Some(arguments.next().ok_or_else(|| {
                        "--output requires an external empty directory".to_owned()
                    })?);
            }
            "--benchmark-runs" if benchmark_runs.is_none() => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--benchmark-runs requires an integer".to_owned())?;
                let value = value
                    .parse::<usize>()
                    .map_err(|_| "--benchmark-runs requires an integer".to_owned())?;
                if !(20..=10_000).contains(&value) {
                    return Err("--benchmark-runs must be between 20 and 10000".to_owned());
                }
                benchmark_runs = Some(value);
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    let output = output.ok_or_else(|| {
        "physical-sound-lab requires --output <external-empty-directory>".to_owned()
    })?;
    Ok(Request {
        output: PathBuf::from(output),
        benchmark_runs,
    })
}

#[derive(Serialize)]
struct LabOutputReport {
    file: String,
    material: String,
    impact_point: String,
    energy_q16: u32,
    stereo_frame_count: usize,
    peak_sample: u16,
    wav_sha256: String,
}

#[derive(Serialize)]
struct GlassBodyOutputReport {
    file: String,
    object: String,
    energy_q16: u32,
    stereo_frame_count: usize,
    peak_sample: u16,
    wav_sha256: String,
}

#[derive(Serialize)]
struct TimingDistribution {
    measured_runs: usize,
    minimum_nanoseconds: u128,
    p50_nanoseconds: u128,
    p95_nanoseconds: u128,
    p99_nanoseconds: u128,
    maximum_nanoseconds: u128,
}

#[derive(Serialize)]
struct GlassCostBenchmark {
    claim: &'static str,
    warmup_runs_per_profile: usize,
    glass_h_mode_count: usize,
    selected_q30_mode_count: usize,
    glass_h_cooked_payload_bytes: usize,
    selected_q30_cooked_payload_bytes: usize,
    glass_h_stereo_frame_count: usize,
    selected_q30_stereo_frame_count: usize,
    glass_h_full_render: TimingDistribution,
    selected_q30_full_render: TimingDistribution,
    glass_h_p50_nanoseconds_per_frame: f64,
    selected_q30_p50_nanoseconds_per_frame: f64,
    selected_to_glass_h_p50_per_frame_ratio: f64,
    saturated_voice_count: usize,
    glass_h_saturated_tick: TimingDistribution,
    selected_q30_saturated_tick: TimingDistribution,
    glass_h_saturated_p99_realtime_fraction: f64,
    selected_q30_saturated_p99_realtime_fraction: f64,
}

#[derive(Serialize)]
struct LabReport {
    schema: &'static str,
    status: &'static str,
    claim: &'static str,
    model: &'static str,
    fallback: &'static str,
    sample_rate_hz: u32,
    channel_count: u32,
    repeated_render_identical: bool,
    demo_sequence_file: &'static str,
    demo_sequence_wav_sha256: String,
    quality_manifest_file: &'static str,
    outputs: Vec<LabOutputReport>,
    glass_body_outputs: Vec<GlassBodyOutputReport>,
    selected_glass_output: GlassBodyOutputReport,
    cost_benchmark: Option<GlassCostBenchmark>,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let output = if request.output.is_absolute() {
        request.output.clone()
    } else {
        root.join(&request.output)
    };
    if output.starts_with(root) {
        return Err(format!(
            "physical-sound-lab output must stay outside the repository: {}",
            output.display()
        ));
    }
    if output.exists()
        && fs::read_dir(&output)
            .map_err(|error| format!("read {}: {error}", output.display()))?
            .next()
            .is_some()
    {
        return Err(format!(
            "physical-sound-lab output directory must be empty: {}",
            output.display()
        ));
    }
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;

    let profile = AudioMixProfileV1::stereo_baseline_v1().map_err(|error| error.to_string())?;
    let mut reports = Vec::new();
    let mut repeated_render_identical = true;
    for material in [
        PhysicalSoundMaterial::Steel,
        PhysicalSoundMaterial::Wood,
        PhysicalSoundMaterial::Glass,
    ] {
        for impact_point in [
            PhysicalSoundImpactPoint::Center,
            PhysicalSoundImpactPoint::Edge,
            PhysicalSoundImpactPoint::Corner,
        ] {
            let excitation = PhysicalSoundExcitation::new(
                material,
                impact_point,
                49_152,
                0,
                lab_seed(material, impact_point),
            );
            let first = render_physical_sound_impact(excitation);
            let second = render_physical_sound_impact(excitation);
            repeated_render_identical &= first == second;
            let wav = encode_canonical_wav(&profile, &first);
            let file = format!("{}-{}.wav", material.label(), impact_point.label());
            fs::write(output.join(&file), &wav)
                .map_err(|error| format!("write {file}: {error}"))?;
            reports.push(LabOutputReport {
                file,
                material: material.label().to_owned(),
                impact_point: impact_point.label().to_owned(),
                energy_q16: excitation.energy_q16,
                stereo_frame_count: first.len() / 2,
                peak_sample: peak_sample(&first),
                wav_sha256: ContentHash::from_bytes(sha256(&wav)).to_hex(),
            });
        }
    }

    let glass_body_energy_q16 = 49_152;
    let glass_body_seed = 0x61a5_b0d1;
    let mut glass_body_reports = Vec::new();
    for variant in [
        GlassBodyVariant::ThinGoblet,
        GlassBodyVariant::Bottle,
        GlassBodyVariant::ThickJar,
    ] {
        let first =
            render_glass_body_variant_impact(variant, glass_body_energy_q16, glass_body_seed);
        let second =
            render_glass_body_variant_impact(variant, glass_body_energy_q16, glass_body_seed);
        repeated_render_identical &= first == second;
        let wav = encode_canonical_wav(&profile, &first);
        let file = format!("glass-{}.wav", variant.label());
        fs::write(output.join(&file), &wav).map_err(|error| format!("write {file}: {error}"))?;
        glass_body_reports.push(GlassBodyOutputReport {
            file,
            object: variant.label().to_owned(),
            energy_q16: glass_body_energy_q16,
            stereo_frame_count: first.len() / 2,
            peak_sample: peak_sample(&first),
            wav_sha256: ContentHash::from_bytes(sha256(&wav)).to_hex(),
        });
    }

    let selected_energy_q16 = 65_536;
    let selected_first = render_selected_glass_q30_impact(selected_energy_q16);
    let selected_second = render_selected_glass_q30_impact(selected_energy_q16);
    repeated_render_identical &= selected_first == selected_second;
    let selected_wav = encode_canonical_wav(&profile, &selected_first);
    let selected_file = "glass-selected-thin-container-q30.wav";
    fs::write(output.join(selected_file), &selected_wav)
        .map_err(|error| format!("write {selected_file}: {error}"))?;
    let selected_glass_output = GlassBodyOutputReport {
        file: selected_file.to_owned(),
        object: "selected-thin-container-q30".to_owned(),
        energy_q16: selected_energy_q16,
        stereo_frame_count: selected_first.len() / 2,
        peak_sample: peak_sample(&selected_first),
        wav_sha256: ContentHash::from_bytes(sha256(&selected_wav)).to_hex(),
    };

    let demo_sequence = render_physical_sound_lab_sequence();
    let second_demo_sequence = render_physical_sound_lab_sequence();
    repeated_render_identical &= demo_sequence == second_demo_sequence;
    let demo_wav = encode_canonical_wav(&profile, &demo_sequence);
    fs::write(output.join("demo-sequence.wav"), &demo_wav)
        .map_err(|error| format!("write demo-sequence.wav: {error}"))?;
    let quality_candidates = reports
        .iter()
        .map(|entry| Q0Candidate {
            id: format!("{}-{}", entry.material, entry.impact_point),
            object_id: format!("p0-{}", entry.material),
            material: entry.material.clone(),
            impact_position: entry.impact_point.clone(),
            force_band: "medium".to_owned(),
            file: entry.file.clone(),
            sha256: entry.wav_sha256.clone(),
        })
        .chain(glass_body_reports.iter().map(|entry| Q0Candidate {
            id: format!("glass-{}", entry.object),
            object_id: format!("p0-glass-{}", entry.object),
            material: "glass".to_owned(),
            impact_position: "center".to_owned(),
            force_band: "medium".to_owned(),
            file: entry.file.clone(),
            sha256: entry.wav_sha256.clone(),
        }))
        .chain(std::iter::once(Q0Candidate {
            id: "glass-selected-thin-container-q30".to_owned(),
            object_id: "p0-glass-selected-thin-container-q30".to_owned(),
            material: "glass".to_owned(),
            impact_position: "selected-calibration".to_owned(),
            force_band: "full-calibration".to_owned(),
            file: selected_glass_output.file.clone(),
            sha256: selected_glass_output.wav_sha256.clone(),
        }))
        .collect::<Vec<_>>();
    write_q0_manifest(&output, &quality_candidates)?;
    let cost_benchmark = request.benchmark_runs.map(benchmark_glass_cost);
    let report = LabReport {
        schema: "nextengine.experimental-physical-sound-lab.report.v1",
        status: "PASS",
        claim: "EXPERIMENT_ONLY / NOT_A_SHIPPED_AUDIO_CONTRACT",
        model: "bounded fixed-point 12-mode steel/wood banks, current 4-mode Glass-H, three separated glass-object profiles and one explicit 16-mode selected-glass Q30 candidate",
        fallback: "ordinary AudioMixerV1 output with the physical-sound-lab feature disabled",
        sample_rate_hz: ExperimentalPhysicalSoundMixer::sample_rate_hz(),
        channel_count: 2,
        repeated_render_identical,
        demo_sequence_file: "demo-sequence.wav",
        demo_sequence_wav_sha256: ContentHash::from_bytes(sha256(&demo_wav)).to_hex(),
        quality_manifest_file: "quality-manifest.json",
        outputs: reports,
        glass_body_outputs: glass_body_reports,
        selected_glass_output,
        cost_benchmark,
    };
    if !report.repeated_render_identical
        || report.outputs.iter().any(|entry| entry.peak_sample == 0)
        || report
            .glass_body_outputs
            .iter()
            .any(|entry| entry.peak_sample == 0)
        || report.selected_glass_output.peak_sample == 0
    {
        return Err("physical-sound-lab deterministic/non-silent acceptance failed".to_owned());
    }
    let json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(output.join("report.json"), &json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn benchmark_glass_cost(measured_runs: usize) -> GlassCostBenchmark {
    const WARMUP_RUNS: usize = 16;
    let glass_h = || {
        render_physical_sound_impact(PhysicalSoundExcitation::new(
            PhysicalSoundMaterial::Glass,
            PhysicalSoundImpactPoint::Center,
            65_536,
            0,
            0x61a5_0101,
        ))
    };
    let selected = || render_selected_glass_q30_impact(65_536);
    for _ in 0..WARMUP_RUNS {
        black_box(glass_h());
        black_box(selected());
    }

    let mut glass_h_timings = Vec::with_capacity(measured_runs);
    let mut selected_timings = Vec::with_capacity(measured_runs);
    let mut glass_h_frames = 0;
    let mut selected_frames = 0;
    for index in 0..measured_runs {
        let measure = |render: &dyn Fn() -> Vec<i16>| {
            let started = Instant::now();
            let pcm = render();
            let elapsed = started.elapsed().as_nanos();
            let frame_count = pcm.len() / 2;
            black_box(pcm);
            (elapsed, frame_count)
        };
        if index.is_multiple_of(2) {
            let (elapsed, frames) = measure(&glass_h);
            glass_h_timings.push(elapsed);
            glass_h_frames = frames;
            let (elapsed, frames) = measure(&selected);
            selected_timings.push(elapsed);
            selected_frames = frames;
        } else {
            let (elapsed, frames) = measure(&selected);
            selected_timings.push(elapsed);
            selected_frames = frames;
            let (elapsed, frames) = measure(&glass_h);
            glass_h_timings.push(elapsed);
            glass_h_frames = frames;
        }
    }

    let glass_h_full_render = summarize_timings(glass_h_timings);
    let selected_q30_full_render = summarize_timings(selected_timings);
    let glass_h_per_frame = glass_h_full_render.p50_nanoseconds as f64 / glass_h_frames as f64;
    let selected_per_frame =
        selected_q30_full_render.p50_nanoseconds as f64 / selected_frames as f64;

    let saturated_voice_count = ExperimentalPhysicalSoundMixer::max_voice_count();
    let saturated_excitations = vec![
        PhysicalSoundExcitation::new(
            PhysicalSoundMaterial::Glass,
            PhysicalSoundImpactPoint::Center,
            49_152,
            0,
            0x61a5_0101,
        );
        saturated_voice_count
    ];
    let mut glass_h_template = ExperimentalPhysicalSoundMixer::default();
    black_box(glass_h_template.mix_tick(&saturated_excitations));
    let mut selected_template = ExperimentalPhysicalSoundMixer::with_glass_profile(
        next_presentation::physical_sound_lab::ExperimentalGlassProfile::SelectedThinContainerQ30,
    );
    black_box(selected_template.mix_tick(&saturated_excitations));
    let mut glass_h_tick_timings = Vec::with_capacity(measured_runs);
    let mut selected_tick_timings = Vec::with_capacity(measured_runs);
    for index in 0..measured_runs {
        let measure_tick = |template: &ExperimentalPhysicalSoundMixer| {
            let mut mixer = template.clone();
            let started = Instant::now();
            black_box(mixer.mix_tick(&[]));
            started.elapsed().as_nanos()
        };
        if index.is_multiple_of(2) {
            glass_h_tick_timings.push(measure_tick(&glass_h_template));
            selected_tick_timings.push(measure_tick(&selected_template));
        } else {
            selected_tick_timings.push(measure_tick(&selected_template));
            glass_h_tick_timings.push(measure_tick(&glass_h_template));
        }
    }
    let glass_h_saturated_tick = summarize_timings(glass_h_tick_timings);
    let selected_q30_saturated_tick = summarize_timings(selected_tick_timings);
    let tick_budget_nanoseconds =
        f64::from(ExperimentalPhysicalSoundMixer::frames_per_tick() as u32) * 1_000_000_000.0
            / f64::from(ExperimentalPhysicalSoundMixer::sample_rate_hz());
    GlassCostBenchmark {
        claim: "LOCAL_NON_GATING_WALL_TIME_DIAGNOSTIC / NOT_A_PRODUCT_BUDGET",
        warmup_runs_per_profile: WARMUP_RUNS,
        glass_h_mode_count: 4,
        selected_q30_mode_count: 16,
        glass_h_cooked_payload_bytes: 176,
        selected_q30_cooked_payload_bytes: 1_536,
        glass_h_stereo_frame_count: glass_h_frames,
        selected_q30_stereo_frame_count: selected_frames,
        glass_h_full_render,
        selected_q30_full_render,
        glass_h_p50_nanoseconds_per_frame: glass_h_per_frame,
        selected_q30_p50_nanoseconds_per_frame: selected_per_frame,
        selected_to_glass_h_p50_per_frame_ratio: selected_per_frame / glass_h_per_frame,
        saturated_voice_count,
        glass_h_saturated_p99_realtime_fraction: glass_h_saturated_tick.p99_nanoseconds as f64
            / tick_budget_nanoseconds,
        selected_q30_saturated_p99_realtime_fraction: selected_q30_saturated_tick.p99_nanoseconds
            as f64
            / tick_budget_nanoseconds,
        glass_h_saturated_tick,
        selected_q30_saturated_tick,
    }
}

fn summarize_timings(mut timings: Vec<u128>) -> TimingDistribution {
    timings.sort_unstable();
    let percentile = |percent: usize| timings[(timings.len() - 1) * percent / 100];
    TimingDistribution {
        measured_runs: timings.len(),
        minimum_nanoseconds: timings[0],
        p50_nanoseconds: percentile(50),
        p95_nanoseconds: percentile(95),
        p99_nanoseconds: percentile(99),
        maximum_nanoseconds: timings[timings.len() - 1],
    }
}

const fn lab_seed(material: PhysicalSoundMaterial, point: PhysicalSoundImpactPoint) -> u32 {
    let material = match material {
        PhysicalSoundMaterial::Steel => 0x51ee_0000,
        PhysicalSoundMaterial::Wood => 0x700d_0000,
        PhysicalSoundMaterial::Glass => 0x61a5_0000,
    };
    let point = match point {
        PhysicalSoundImpactPoint::Center => 0x0101,
        PhysicalSoundImpactPoint::Edge => 0x0202,
        PhysicalSoundImpactPoint::Corner => 0x0303,
    };
    material | point
}

fn peak_sample(samples: &[i16]) -> u16 {
    samples
        .iter()
        .map(|sample| sample.unsigned_abs())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_require_one_external_output() {
        assert!(
            parse_arguments(["--output".to_owned(), "/tmp/lab".to_owned()].into_iter()).is_ok()
        );
        assert!(
            parse_arguments(
                [
                    "--benchmark-runs".to_owned(),
                    "100".to_owned(),
                    "--output".to_owned(),
                    "/tmp/lab".to_owned(),
                ]
                .into_iter()
            )
            .is_ok()
        );
        assert!(
            parse_arguments(
                [
                    "--output".to_owned(),
                    "/tmp/lab".to_owned(),
                    "--benchmark-runs".to_owned(),
                    "1".to_owned(),
                ]
                .into_iter()
            )
            .is_err()
        );
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["--else".to_owned()].into_iter()).is_err());
    }
}
