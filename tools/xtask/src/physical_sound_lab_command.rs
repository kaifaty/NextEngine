use std::fs;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use next_presentation::physical_sound_lab::{
    ExperimentalPhysicalSoundMixer, PhysicalSoundExcitation, PhysicalSoundImpactPoint,
    PhysicalSoundMaterial, render_physical_sound_impact, render_physical_sound_lab_sequence,
};
use serde::Serialize;

use crate::physical_sound_eval_command::{Q0Candidate, write_q0_manifest};

pub(super) struct Request {
    output: PathBuf,
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let Some(flag) = arguments.next() else {
        return Err("physical-sound-lab requires --output <external-empty-directory>".to_owned());
    };
    if flag != "--output" {
        return Err(format!("unexpected argument: {flag}"));
    }
    let output = arguments
        .next()
        .ok_or_else(|| "--output requires an external empty directory".to_owned())?;
    if let Some(extra) = arguments.next() {
        return Err(format!("unexpected argument: {extra}"));
    }
    Ok(Request {
        output: PathBuf::from(output),
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
        .collect::<Vec<_>>();
    write_q0_manifest(&output, &quality_candidates)?;
    let report = LabReport {
        schema: "nextengine.experimental-physical-sound-lab.report.v0",
        status: "PASS",
        claim: "EXPERIMENT_ONLY / NOT_A_SHIPPED_AUDIO_CONTRACT",
        model: "fixed-point damped modal resonators plus bounded deterministic strike noise",
        fallback: "ordinary AudioMixerV1 output with the physical-sound-lab feature disabled",
        sample_rate_hz: ExperimentalPhysicalSoundMixer::sample_rate_hz(),
        channel_count: 2,
        repeated_render_identical,
        demo_sequence_file: "demo-sequence.wav",
        demo_sequence_wav_sha256: ContentHash::from_bytes(sha256(&demo_wav)).to_hex(),
        quality_manifest_file: "quality-manifest.json",
        outputs: reports,
    };
    if !report.repeated_render_identical
        || report.outputs.iter().any(|entry| entry.peak_sample == 0)
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
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["--else".to_owned()].into_iter()).is_err());
    }
}
