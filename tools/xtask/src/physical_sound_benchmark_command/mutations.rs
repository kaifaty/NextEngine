use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::manifest::{
    BenchmarkManifest, CorpusEntry, EntryOrigin, FileRef, MutationExpectedValidatorOutcome,
    Partition, validate_manifest,
};
use super::{
    MANIFEST_SCHEMA, MAX_LICENSE_RECORD_BYTES, MAX_MANIFEST_BYTES, MAX_WAV_BYTES,
    canonical_external_file, read_bounded_file, read_file_ref, require_empty_output,
    resolve_cli_path, resolve_output_path, sha256_hex,
};
use crate::physical_sound_eval_command::audio_analysis::{WavAudio, parse_wav};

const PACK_SCHEMA: &str = "nextengine.experimental-physical-sound-temporal-mutation-pack.report.v1";
const PACK_PROFILE: &str = "nextengine.experimental-physical-sound-temporal-mutations.av-p0c.v1";

#[derive(Clone, Copy)]
enum MutationFamily {
    StationaryWhiteTail,
    StationaryColoredTail,
    FrozenSpectralEnvelope,
    ShuffledTemporalEnvelope,
}

impl MutationFamily {
    const ALL: [Self; 4] = [
        Self::StationaryWhiteTail,
        Self::StationaryColoredTail,
        Self::FrozenSpectralEnvelope,
        Self::ShuffledTemporalEnvelope,
    ];

    const fn id(self) -> &'static str {
        match self {
            Self::StationaryWhiteTail => "av-p0c-stationary-white-tail",
            Self::StationaryColoredTail => "av-p0c-stationary-colored-tail",
            Self::FrozenSpectralEnvelope => "av-p0c-frozen-spectral-envelope",
            Self::ShuffledTemporalEnvelope => "av-p0c-shuffled-temporal-envelope",
        }
    }

    const fn suffix(self) -> &'static str {
        match self {
            Self::StationaryWhiteTail => "stationary-white",
            Self::StationaryColoredTail => "stationary-colored",
            Self::FrozenSpectralEnvelope => "frozen-spectrum",
            Self::ShuffledTemporalEnvelope => "shuffled-envelope",
        }
    }

    const fn seed_tag(self) -> u64 {
        match self {
            Self::StationaryWhiteTail => 0x2c4f_8b1d_96a3_e705,
            Self::StationaryColoredTail => 0xb763_0fa2_519d_c84e,
            Self::FrozenSpectralEnvelope => 0x83de_4a16_f290_7bc1,
            Self::ShuffledTemporalEnvelope => 0x1a95_ec73_40bf_62d8,
        }
    }
}

struct Request {
    manifest: PathBuf,
    output: PathBuf,
}

#[derive(Serialize)]
struct MutationPackReport {
    schema: &'static str,
    decision: &'static str,
    profile: &'static str,
    claim: &'static str,
    source_manifest_sha256: String,
    derived_manifest_path: &'static str,
    derived_manifest_sha256: String,
    copied_real_entry_count: usize,
    controlled_mutation_entry_count: usize,
    mutation_families: Vec<&'static str>,
}

pub(super) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let request = parse_arguments(&mut arguments)?;
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "manifest")?;
    let manifest: BenchmarkManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_owned())?;

    fs::create_dir_all(output.join("audio"))
        .map_err(|error| format!("create mutation pack output: {error}"))?;
    fs::create_dir_all(output.join("licenses"))
        .map_err(|error| format!("create mutation pack licenses: {error}"))?;

    let source_ids = manifest
        .entries
        .iter()
        .filter(|entry| matches!(&entry.origin, EntryOrigin::Real))
        .map(|entry| entry.source_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut sources = manifest
        .corpus_sources
        .iter()
        .filter(|source| source_ids.contains(source.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    for (index, source) in sources.iter_mut().enumerate() {
        let bytes = read_file_ref(
            &root,
            manifest_directory,
            &source.license.review_record,
            MAX_LICENSE_RECORD_BYTES,
            "license review record",
        )?;
        let relative_path = format!("licenses/source-{index:04}-review.txt");
        fs::write(output.join(&relative_path), &bytes)
            .map_err(|error| format!("write copied license record: {error}"))?;
        source.license.review_record = FileRef {
            path: relative_path,
            sha256: sha256_hex(&bytes),
        };
    }

    let mut entries = Vec::new();
    let mut mutation_index = 0_usize;
    for (real_index, source_entry) in manifest
        .entries
        .iter()
        .filter(|entry| matches!(&entry.origin, EntryOrigin::Real))
        .enumerate()
    {
        let bytes = read_file_ref(
            &root,
            manifest_directory,
            &source_entry.audio,
            MAX_WAV_BYTES,
            "corpus WAV",
        )?;
        let relative_path = format!("audio/real-{real_index:05}.wav");
        fs::write(output.join(&relative_path), &bytes)
            .map_err(|error| format!("write copied real WAV: {error}"))?;
        let mut real_entry = source_entry.clone();
        real_entry.audio = FileRef {
            path: relative_path,
            sha256: sha256_hex(&bytes),
        };
        entries.push(real_entry);

        if source_entry.partition == Partition::Development {
            continue;
        }
        let audio = parse_wav(&bytes)
            .map_err(|error| format!("parse corpus WAV {}: {error}", source_entry.audio.path))?;
        for family in MutationFamily::ALL {
            let mutated = mutate_audio(&audio, &source_entry.audio.sha256, family)?;
            let wav = encode_float32_mono_wav(audio.sample_rate_hz, &mutated)?;
            let mutation_path = format!("audio/mutation-{mutation_index:05}.wav");
            fs::write(output.join(&mutation_path), &wav)
                .map_err(|error| format!("write controlled mutation WAV: {error}"))?;
            entries.push(mutation_entry(
                source_entry,
                family,
                mutation_path,
                sha256_hex(&wav),
            ));
            mutation_index += 1;
        }
    }
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    let source_manifest_sha256 = sha256_hex(&manifest_bytes);
    let derived_benchmark_id =
        derived_benchmark_id(&manifest.benchmark_id, &source_manifest_sha256);
    let derived = BenchmarkManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        benchmark_id: derived_benchmark_id,
        corpus_sources: sources,
        external_feature_sets: Vec::new(),
        entries,
    };
    validate_manifest(&derived)?;
    let derived_bytes = serde_json::to_vec_pretty(&derived).map_err(|error| error.to_string())?;
    fs::write(output.join("manifest.json"), &derived_bytes)
        .map_err(|error| format!("write derived mutation manifest: {error}"))?;
    let report = MutationPackReport {
        schema: PACK_SCHEMA,
        decision: "NoAcceptanceAuthority",
        profile: PACK_PROFILE,
        claim: "CONTROLLED_TEMPORAL_NEGATIVES_ONLY / EXTERNAL_RESEARCH_ARTIFACT",
        source_manifest_sha256,
        derived_manifest_path: "manifest.json",
        derived_manifest_sha256: sha256_hex(&derived_bytes),
        copied_real_entry_count: derived
            .entries
            .iter()
            .filter(|entry| matches!(&entry.origin, EntryOrigin::Real))
            .count(),
        controlled_mutation_entry_count: mutation_index,
        mutation_families: MutationFamily::ALL
            .into_iter()
            .map(MutationFamily::id)
            .collect(),
    };
    let report_bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(output.join("mutation-pack-report.json"), &report_bytes)
        .map_err(|error| format!("write mutation pack report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_bytes).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn parse_arguments(arguments: &mut impl Iterator<Item = String>) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-mutations requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-mutations requires --output <external-empty-directory>".to_owned()
        })?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

fn mutation_entry(
    parent: &CorpusEntry,
    family: MutationFamily,
    path: String,
    sha256: String,
) -> CorpusEntry {
    let mut entry = parent.clone();
    entry.id = format!("{}-{}", parent.id, family.suffix());
    entry.origin = EntryOrigin::Mutation {
        mutation_family: family.id().to_owned(),
        parent_entry_id: parent.id.clone(),
        expected_validator_outcome: MutationExpectedValidatorOutcome::Reject,
    };
    entry.audio = FileRef { path, sha256 };
    entry
}

fn derived_benchmark_id(source: &str, source_hash: &str) -> String {
    let candidate = format!("{source}-av-p0c-temporal-mutations-v1");
    if candidate.len() <= 160 {
        candidate
    } else {
        format!("av-p0c-temporal-mutations-{}", &source_hash[..16])
    }
}

fn mutate_audio(
    audio: &WavAudio,
    parent_sha256: &str,
    family: MutationFamily,
) -> Result<Vec<f32>, String> {
    let samples = &audio.mono_samples;
    if samples.is_empty() {
        return Err("cannot mutate empty audio".to_owned());
    }
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f64, f64::max);
    let onset_threshold = (peak * 0.05).max(1.0e-5);
    let onset = samples
        .iter()
        .position(|sample| sample.abs() >= onset_threshold)
        .unwrap_or(0);
    let transient_samples = usize::try_from(audio.sample_rate_hz / 80)
        .map_err(|_| "sample rate does not fit usize".to_owned())?;
    let tail_start = onset
        .saturating_add(transient_samples)
        .min(samples.len().saturating_sub(1));
    let envelope = rms_envelope(samples, audio.sample_rate_hz);
    let seed = seed_from_hash(parent_sha256)? ^ family.seed_tag();
    let values = match family {
        MutationFamily::StationaryWhiteTail => {
            stationary_noise_tail(samples, &envelope, tail_start, seed, false)
        }
        MutationFamily::StationaryColoredTail => {
            stationary_noise_tail(samples, &envelope, tail_start, seed, true)
        }
        MutationFamily::FrozenSpectralEnvelope => {
            frozen_spectral_envelope(samples, &envelope, tail_start, audio.sample_rate_hz)
        }
        MutationFamily::ShuffledTemporalEnvelope => {
            shuffled_temporal_envelope(samples, tail_start, audio.sample_rate_hz, seed)
        }
    };
    Ok(values
        .into_iter()
        .map(|sample| sample.clamp(-0.98, 0.98) as f32)
        .collect())
}

fn rms_envelope(samples: &[f64], sample_rate_hz: u32) -> Vec<f64> {
    let smoothing = (-1.0 / (f64::from(sample_rate_hz) * 0.008)).exp();
    let mut power = 0.0_f64;
    samples
        .iter()
        .map(|sample| {
            power = smoothing.mul_add(power, (1.0 - smoothing) * sample * sample);
            power.sqrt()
        })
        .collect()
}

fn stationary_noise_tail(
    samples: &[f64],
    envelope: &[f64],
    tail_start: usize,
    seed: u64,
    colored: bool,
) -> Vec<f64> {
    let mut output = samples.to_vec();
    let mut random = XorShift64::new(seed);
    let pole = 0.94_f64;
    let colored_normalization = ((1.0 + pole) / (1.0 - pole)).sqrt();
    let mut state = 0.0_f64;
    for index in tail_start..samples.len() {
        let white = random.bipolar();
        let carrier = if colored {
            state = pole.mul_add(state, (1.0 - pole) * white);
            state * colored_normalization
        } else {
            white
        };
        output[index] = carrier * envelope[index] * 3.0_f64.sqrt();
    }
    output
}

fn frozen_spectral_envelope(
    samples: &[f64],
    envelope: &[f64],
    tail_start: usize,
    sample_rate_hz: u32,
) -> Vec<f64> {
    let mut output = samples.to_vec();
    let desired = usize::try_from(sample_rate_hz / 40).unwrap_or(1_200);
    let template_len = desired.min(samples.len() - tail_start).max(1);
    let template = &samples[tail_start..tail_start + template_len];
    let mean = template.iter().sum::<f64>() / template.len() as f64;
    let rms = (template
        .iter()
        .map(|sample| (sample - mean).powi(2))
        .sum::<f64>()
        / template.len() as f64)
        .sqrt()
        .max(1.0e-9);
    for index in tail_start..samples.len() {
        let carrier = (template[(index - tail_start) % template_len] - mean) / rms;
        output[index] = carrier * envelope[index];
    }
    output
}

fn shuffled_temporal_envelope(
    samples: &[f64],
    tail_start: usize,
    sample_rate_hz: u32,
    seed: u64,
) -> Vec<f64> {
    let mut output = samples.to_vec();
    let block = usize::try_from(sample_rate_hz / 50).unwrap_or(960).max(64);
    let tail_len = samples.len() - tail_start;
    let block_count = tail_len.div_ceil(block);
    if block_count < 2 {
        return output;
    }
    let mut rms = (0..block_count)
        .map(|block_index| {
            let start = tail_start + block_index * block;
            let end = (start + block).min(samples.len());
            (samples[start..end]
                .iter()
                .map(|sample| sample * sample)
                .sum::<f64>()
                / (end - start) as f64)
                .sqrt()
        })
        .collect::<Vec<_>>();
    let mut random = XorShift64::new(seed);
    for index in (1..rms.len()).rev() {
        let other = random.index(index + 1);
        rms.swap(index, other);
    }
    let original_rms = (0..block_count)
        .map(|block_index| {
            let start = tail_start + block_index * block;
            let end = (start + block).min(samples.len());
            (samples[start..end]
                .iter()
                .map(|sample| sample * sample)
                .sum::<f64>()
                / (end - start) as f64)
                .sqrt()
                .max(1.0e-9)
        })
        .collect::<Vec<_>>();
    let gains = rms
        .iter()
        .zip(&original_rms)
        .map(|(target, source)| (target / source).clamp(0.125, 8.0))
        .collect::<Vec<_>>();
    for index in tail_start..samples.len() {
        let relative = index - tail_start;
        let current = (relative / block).min(block_count - 1);
        let next = (current + 1).min(block_count - 1);
        let phase = (relative % block) as f64 / block as f64;
        let gain = gains[current] * (1.0 - phase) + gains[next] * phase;
        output[index] = samples[index] * gain;
    }
    output
}

fn encode_float32_mono_wav(sample_rate_hz: u32, samples: &[f32]) -> Result<Vec<u8>, String> {
    let data_bytes = u32::try_from(
        samples
            .len()
            .checked_mul(4)
            .ok_or_else(|| "float WAV byte length overflow".to_owned())?,
    )
    .map_err(|_| "float WAV exceeds RIFF size limit".to_owned())?;
    let byte_rate = sample_rate_hz
        .checked_mul(4)
        .ok_or_else(|| "float WAV byte rate overflow".to_owned())?;
    let riff_size = data_bytes
        .checked_add(36)
        .ok_or_else(|| "float WAV RIFF size overflow".to_owned())?;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&3_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    Ok(bytes)
}

fn seed_from_hash(hash: &str) -> Result<u64, String> {
    if hash.len() < 16 {
        return Err("parent sha256 is too short for deterministic seed".to_owned());
    }
    u64::from_str_radix(&hash[..16], 16)
        .map_err(|error| format!("invalid parent sha256 seed: {error}"))
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn bipolar(&mut self) -> f64 {
        let fraction = (self.next() >> 11) as f64 / ((1_u64 << 53) - 1) as f64;
        fraction * 2.0 - 1.0
    }

    fn index(&mut self, upper_exclusive: usize) -> usize {
        (self.next() % upper_exclusive as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controlled_mutations_are_deterministic_and_distinct() {
        let samples = (0..4_800)
            .map(|index| {
                let time = index as f64 / 48_000.0;
                (time * 2_400.0 * std::f64::consts::TAU).sin() * (-18.0 * time).exp() * 0.5
            })
            .collect::<Vec<_>>();
        let audio = WavAudio {
            sample_format: "test".to_owned(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            mono_samples: samples,
        };
        let hash = "1".repeat(64);
        let first = mutate_audio(&audio, &hash, MutationFamily::StationaryWhiteTail)
            .expect("first mutation");
        let repeated = mutate_audio(&audio, &hash, MutationFamily::StationaryWhiteTail)
            .expect("repeated mutation");
        let colored = mutate_audio(&audio, &hash, MutationFamily::StationaryColoredTail)
            .expect("colored mutation");
        assert_eq!(first, repeated);
        assert_ne!(first, colored);
        assert!(first.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn arguments_require_manifest_and_output() {
        assert!(
            parse_arguments(
                &mut [
                    "--manifest".to_owned(),
                    "/tmp/manifest.json".to_owned(),
                    "--output".to_owned(),
                    "/tmp/mutations".to_owned(),
                ]
                .into_iter()
            )
            .is_ok()
        );
        assert!(parse_arguments(&mut std::iter::empty()).is_err());
    }
}
