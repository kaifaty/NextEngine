use std::fs;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use next_presentation::audio_mix::{AudioMixProfileV1, encode_canonical_wav};
use next_presentation::physical_sound_lab::{
    ExperimentalPhysicalSoundMixer, ExperimentalSteelSearchProfile, PhysicalSoundImpactPoint,
    render_experimental_steel_search_impact,
};
use serde::Serialize;

const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-steel-search.report.v1";
pub(super) struct Request {
    output: PathBuf,
    profile_set: ProfileSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProfileSet {
    SparseGridV1,
    RoughnessGridV2,
    RoughnessRefinementV3,
    StochasticResidualV4,
}

impl ProfileSet {
    const fn label(self) -> &'static str {
        match self {
            Self::SparseGridV1 => "sparse-grid-v1",
            Self::RoughnessGridV2 => "roughness-grid-v2",
            Self::RoughnessRefinementV3 => "roughness-refinement-v3",
            Self::StochasticResidualV4 => "stochastic-residual-v4",
        }
    }

    const fn generator_revision(self) -> &'static str {
        match self {
            Self::SparseGridV1 => "nextengine-steel-search-grid.v1",
            Self::RoughnessGridV2 => "nextengine-steel-roughness-grid.v2",
            Self::RoughnessRefinementV3 => "nextengine-steel-roughness-refinement.v3",
            Self::StochasticResidualV4 => "nextengine-steel-stochastic-residual.v4",
        }
    }

    const fn development_objective(self) -> &'static str {
        match self {
            Self::SparseGridV1 | Self::RoughnessGridV2 | Self::RoughnessRefinementV3 => {
                "maximize expected-label margin to real development metal anchors across center/edge/corner"
            }
            Self::StochasticResidualV4 => {
                "test one frozen decaying-white residual grid against real-metal spectral-dynamics gaps; require both learned heads across center/edge/corner"
            }
        }
    }
}

pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut output = None;
    let mut profile_set = None;
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--output" if output.is_none() => {
                output = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    "--output requires an external empty directory".to_owned()
                })?));
            }
            "--profile-set" if profile_set.is_none() => {
                profile_set = Some(
                    match arguments
                        .next()
                        .ok_or_else(|| "--profile-set requires a value".to_owned())?
                        .as_str()
                    {
                        "sparse-grid-v1" => ProfileSet::SparseGridV1,
                        "roughness-grid-v2" => ProfileSet::RoughnessGridV2,
                        "roughness-refinement-v3" => ProfileSet::RoughnessRefinementV3,
                        "stochastic-residual-v4" => ProfileSet::StochasticResidualV4,
                        value => {
                            return Err(format!("unsupported steel search profile set: {value}"));
                        }
                    },
                );
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        output: output.ok_or_else(|| {
            "physical-sound-steel-search requires --output <external-empty-directory>".to_owned()
        })?,
        profile_set: profile_set.unwrap_or(ProfileSet::SparseGridV1),
    })
}

#[derive(Clone, Serialize)]
struct ProfileReport {
    id: String,
    frequency_scale_permille: u16,
    decay_scale_permille: u16,
    high_mode_gain_permille: u16,
    transient: &'static str,
    transient_gain_q15: u16,
    transient_frames: u16,
    roughness_sideband_fraction_permille: u16,
    roughness_index_permille: u16,
    #[serde(skip_serializing_if = "residual_is_none")]
    stochastic_residual: &'static str,
    #[serde(skip_serializing_if = "is_zero_u16")]
    stochastic_residual_gain_q15: u16,
    #[serde(skip_serializing_if = "is_zero_u16")]
    stochastic_residual_t20_ms: u16,
}

fn residual_is_none(value: &&'static str) -> bool {
    *value == "none"
}

const fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

impl ProfileReport {
    fn search_profile(&self) -> ExperimentalSteelSearchProfile {
        ExperimentalSteelSearchProfile {
            frequency_scale_permille: self.frequency_scale_permille,
            decay_scale_permille: self.decay_scale_permille,
            high_mode_gain_permille: self.high_mode_gain_permille,
            transient_gain_q15: self.transient_gain_q15,
            transient_frames: self.transient_frames,
            roughness_sideband_fraction_permille: self.roughness_sideband_fraction_permille,
            roughness_index_permille: self.roughness_index_permille,
            stochastic_residual_gain_q15: self.stochastic_residual_gain_q15,
            stochastic_residual_t20_ms: self.stochastic_residual_t20_ms,
        }
    }
}

#[derive(Serialize)]
struct OutputReport {
    id: String,
    profile_id: String,
    material: &'static str,
    impact_point: &'static str,
    force_band: &'static str,
    file: String,
    stereo_frame_count: usize,
    peak_sample: u16,
    wav_sha256: String,
}

#[derive(Serialize)]
struct SearchReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    profile_set: &'static str,
    generator_revision: &'static str,
    generator_profile_sha256: String,
    sample_rate_hz: u32,
    channel_count: u32,
    development_objective: &'static str,
    selection_partition: &'static str,
    forbidden_tuning_partitions: [&'static str; 2],
    unchanged_controls: [&'static str; 2],
    repeated_render_identical: bool,
    profiles: Vec<ProfileReport>,
    outputs: Vec<OutputReport>,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let output = resolve_output(root, &request.output)?;
    require_empty_directory(&output)?;
    fs::create_dir_all(output.join("wav"))
        .map_err(|error| format!("create {}: {error}", output.display()))?;

    let profiles = frozen_search_profiles(request.profile_set);
    let profile_bytes = serde_json::to_vec(&profiles).map_err(|error| error.to_string())?;
    let generator_profile_sha256 = ContentHash::from_bytes(sha256(&profile_bytes)).to_hex();
    let wav_profile = AudioMixProfileV1::stereo_baseline_v1().map_err(|error| error.to_string())?;
    let mut outputs = Vec::with_capacity(profiles.len() * 3);
    let mut repeated_render_identical = true;

    for profile in &profiles {
        for impact_point in [
            PhysicalSoundImpactPoint::Center,
            PhysicalSoundImpactPoint::Edge,
            PhysicalSoundImpactPoint::Corner,
        ] {
            let seed = search_seed(impact_point);
            let first = render_experimental_steel_search_impact(
                profile.search_profile(),
                impact_point,
                49_152,
                seed,
            )
            .map_err(|error| format!("render {}: {error}", profile.id))?;
            let second = render_experimental_steel_search_impact(
                profile.search_profile(),
                impact_point,
                49_152,
                seed,
            )
            .map_err(|error| format!("repeat render {}: {error}", profile.id))?;
            repeated_render_identical &= first == second;
            let peak = peak_sample(&first);
            if peak == 0 || peak >= i16::MAX as u16 {
                return Err(format!(
                    "steel search candidate {}-{} is silent or clipped (peak {peak})",
                    profile.id,
                    impact_point.label()
                ));
            }
            let wav = encode_canonical_wav(&wav_profile, &first);
            let id = format!("{}-{}", profile.id, impact_point.label());
            let file = format!("wav/{id}.wav");
            fs::write(output.join(&file), &wav)
                .map_err(|error| format!("write {file}: {error}"))?;
            outputs.push(OutputReport {
                id,
                profile_id: profile.id.clone(),
                material: "metal",
                impact_point: impact_point.label(),
                force_band: "medium",
                file,
                stereo_frame_count: first.len() / 2,
                peak_sample: peak,
                wav_sha256: ContentHash::from_bytes(sha256(&wav)).to_hex(),
            });
        }
    }

    if !repeated_render_identical {
        return Err("steel search repeated-render identity failed".to_owned());
    }
    let report = SearchReport {
        schema: REPORT_SCHEMA,
        status: "PASS",
        decision: "CandidateGenerationOnly",
        claim: "BOUNDED_OFFLINE_SEARCH / NO_PERCEPTUAL_PASS_OR_RUNTIME_PROMOTION",
        profile_set: request.profile_set.label(),
        generator_revision: request.profile_set.generator_revision(),
        generator_profile_sha256,
        sample_rate_hz: ExperimentalPhysicalSoundMixer::sample_rate_hz(),
        channel_count: 2,
        development_objective: request.profile_set.development_objective(),
        selection_partition: "calibration",
        forbidden_tuning_partitions: ["holdout", "shadow"],
        unchanged_controls: [
            "accepted wood profile",
            "selected thin-container Q30 glass profile",
        ],
        repeated_render_identical,
        profiles,
        outputs,
    };
    let json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(output.join("report.json"), &json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn frozen_search_profiles(profile_set: ProfileSet) -> Vec<ProfileReport> {
    match profile_set {
        ProfileSet::SparseGridV1 => sparse_grid_v1(),
        ProfileSet::RoughnessGridV2 => roughness_grid_v2(),
        ProfileSet::RoughnessRefinementV3 => roughness_refinement_v3(),
        ProfileSet::StochasticResidualV4 => stochastic_residual_v4(),
    }
}

fn sparse_grid_v1() -> Vec<ProfileReport> {
    let mut profiles = vec![ProfileReport {
        id: "steel-search-control".to_owned(),
        frequency_scale_permille: 1_000,
        decay_scale_permille: 1_000,
        high_mode_gain_permille: 1_000,
        transient: "current-control",
        transient_gain_q15: 8_192,
        transient_frames: 960,
        roughness_sideband_fraction_permille: 0,
        roughness_index_permille: 0,
        stochastic_residual: "none",
        stochastic_residual_gain_q15: 0,
        stochastic_residual_t20_ms: 0,
    }];
    let transients = [
        ("short", 4_096, 240),
        ("balanced", 8_192, 480),
        ("broad", 12_288, 960),
    ];
    for frequency in [750, 1_000, 1_250] {
        for decay in [1_000, 1_800, 2_600] {
            for high_gain in [600, 1_200] {
                for (transient, transient_gain_q15, transient_frames) in transients {
                    profiles.push(ProfileReport {
                        id: format!(
                            "steel-search-f{frequency:04}-d{decay:04}-h{high_gain:04}-t{transient}"
                        ),
                        frequency_scale_permille: frequency,
                        decay_scale_permille: decay,
                        high_mode_gain_permille: high_gain,
                        transient,
                        transient_gain_q15,
                        transient_frames,
                        roughness_sideband_fraction_permille: 0,
                        roughness_index_permille: 0,
                        stochastic_residual: "none",
                        stochastic_residual_gain_q15: 0,
                        stochastic_residual_t20_ms: 0,
                    });
                }
            }
        }
    }
    profiles
}

fn roughness_grid_v2() -> Vec<ProfileReport> {
    let mut profiles = vec![ProfileReport {
        id: "steel-roughness-control-f0750-d2600".to_owned(),
        frequency_scale_permille: 750,
        decay_scale_permille: 2_600,
        high_mode_gain_permille: 1_200,
        transient: "short",
        transient_gain_q15: 4_096,
        transient_frames: 240,
        roughness_sideband_fraction_permille: 0,
        roughness_index_permille: 0,
        stochastic_residual: "none",
        stochastic_residual_gain_q15: 0,
        stochastic_residual_t20_ms: 0,
    }];
    for frequency in [750, 1_000] {
        for decay in [2_600, 4_000] {
            for sideband_fraction in [125, 250, 375] {
                for roughness_index in [300, 600, 900] {
                    profiles.push(ProfileReport {
                        id: format!(
                            "steel-roughness-f{frequency:04}-d{decay:04}-r{sideband_fraction:04}-i{roughness_index:04}"
                        ),
                        frequency_scale_permille: frequency,
                        decay_scale_permille: decay,
                        high_mode_gain_permille: 1_200,
                        transient: "short",
                        transient_gain_q15: 4_096,
                        transient_frames: 240,
                        roughness_sideband_fraction_permille: sideband_fraction,
                        roughness_index_permille: roughness_index,
                        stochastic_residual: "none",
                        stochastic_residual_gain_q15: 0,
                        stochastic_residual_t20_ms: 0,
                    });
                }
            }
        }
    }
    profiles
}

fn roughness_refinement_v3() -> Vec<ProfileReport> {
    let mut profiles = Vec::with_capacity(24);
    for frequency in [650, 700, 750] {
        for sideband_fraction in [350, 375, 400, 450] {
            for roughness_index in [900, 1_000] {
                profiles.push(ProfileReport {
                    id: format!(
                        "steel-refine-f{frequency:04}-d4000-r{sideband_fraction:04}-i{roughness_index:04}"
                    ),
                    frequency_scale_permille: frequency,
                    decay_scale_permille: 4_000,
                    high_mode_gain_permille: 1_200,
                    transient: "short",
                    transient_gain_q15: 4_096,
                    transient_frames: 240,
                    roughness_sideband_fraction_permille: sideband_fraction,
                    roughness_index_permille: roughness_index,
                    stochastic_residual: "none",
                    stochastic_residual_gain_q15: 0,
                    stochastic_residual_t20_ms: 0,
                });
            }
        }
    }
    profiles
}

fn stochastic_residual_v4() -> Vec<ProfileReport> {
    let mut profiles = vec![ProfileReport {
        id: "steel-residual-v3-control".to_owned(),
        frequency_scale_permille: 650,
        decay_scale_permille: 4_000,
        high_mode_gain_permille: 1_200,
        transient: "short",
        transient_gain_q15: 4_096,
        transient_frames: 240,
        roughness_sideband_fraction_permille: 350,
        roughness_index_permille: 900,
        stochastic_residual: "none",
        stochastic_residual_gain_q15: 0,
        stochastic_residual_t20_ms: 0,
    }];
    for gain_q15 in [128, 256, 512, 1_024] {
        for t20_ms in [300, 600, 1_200] {
            profiles.push(ProfileReport {
                id: format!("steel-residual-g{gain_q15:04}-t{t20_ms:04}"),
                frequency_scale_permille: 650,
                decay_scale_permille: 4_000,
                high_mode_gain_permille: 1_200,
                transient: "short",
                transient_gain_q15: 4_096,
                transient_frames: 240,
                roughness_sideband_fraction_permille: 350,
                roughness_index_permille: 900,
                stochastic_residual: "decaying-white",
                stochastic_residual_gain_q15: gain_q15,
                stochastic_residual_t20_ms: t20_ms,
            });
        }
    }
    profiles
}

fn search_seed(impact_point: PhysicalSoundImpactPoint) -> u32 {
    let point = match impact_point {
        PhysicalSoundImpactPoint::Center => 0x0101,
        PhysicalSoundImpactPoint::Edge => 0x0202,
        PhysicalSoundImpactPoint::Corner => 0x0303,
    };
    0x51ee_0000 ^ point
}

fn resolve_output(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let output = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let parent = output
        .parent()
        .ok_or_else(|| "steel search output has no parent directory".to_owned())?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
    let name = output
        .file_name()
        .ok_or_else(|| "steel search output has no directory name".to_owned())?;
    let output = canonical_parent.join(name);
    let canonical_root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    if output.starts_with(canonical_root) {
        return Err(format!(
            "physical-sound-steel-search output must stay outside the repository: {}",
            output.display()
        ));
    }
    Ok(output)
}

fn require_empty_directory(output: &Path) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(format!("output is not a directory: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|error| format!("read {}: {error}", output.display()))?
        .next()
        .is_some()
    {
        return Err(format!(
            "physical-sound-steel-search output directory must be empty: {}",
            output.display()
        ));
    }
    Ok(())
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
            parse_arguments(["--output".to_owned(), "/tmp/search".to_owned()].into_iter()).is_ok()
        );
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["--else".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn frozen_grid_is_unique_bounded_and_keeps_one_control() {
        let profiles = frozen_search_profiles(ProfileSet::SparseGridV1);
        let ids = profiles
            .iter()
            .map(|profile| profile.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(profiles.len(), 55);
        assert_eq!(ids.len(), profiles.len());
        assert_eq!(
            profiles
                .iter()
                .filter(|profile| profile.transient == "current-control")
                .count(),
            1
        );
    }

    #[test]
    fn roughness_grid_is_unique_and_keeps_one_sparse_control() {
        let profiles = frozen_search_profiles(ProfileSet::RoughnessGridV2);
        let ids = profiles
            .iter()
            .map(|profile| profile.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(profiles.len(), 37);
        assert_eq!(ids.len(), profiles.len());
        assert_eq!(
            profiles
                .iter()
                .filter(|profile| profile.roughness_index_permille == 0)
                .count(),
            1
        );
    }

    #[test]
    fn roughness_refinement_is_the_frozen_local_neighborhood() {
        let profiles = frozen_search_profiles(ProfileSet::RoughnessRefinementV3);
        let ids = profiles
            .iter()
            .map(|profile| profile.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(profiles.len(), 24);
        assert_eq!(ids.len(), profiles.len());
        assert!(profiles.iter().all(|profile| {
            profile.decay_scale_permille == 4_000 && profile.roughness_index_permille >= 900
        }));
        let serialized = serde_json::to_value(&profiles).expect("serialize v3 profiles");
        assert!(
            serialized
                .as_array()
                .is_some_and(|values| values.iter().all(|profile| profile
                    .get("stochastic_residual")
                    .is_none()
                    && profile.get("stochastic_residual_gain_q15").is_none()
                    && profile.get("stochastic_residual_t20_ms").is_none()))
        );
    }

    #[test]
    fn stochastic_residual_v4_is_one_frozen_counterfactual() {
        let profiles = frozen_search_profiles(ProfileSet::StochasticResidualV4);
        let ids = profiles
            .iter()
            .map(|profile| profile.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(profiles.len(), 13);
        assert_eq!(ids.len(), profiles.len());
        assert_eq!(
            profiles
                .iter()
                .filter(|profile| profile.stochastic_residual == "none")
                .count(),
            1
        );
        assert!(profiles.iter().skip(1).all(|profile| {
            profile.frequency_scale_permille == 650
                && profile.decay_scale_permille == 4_000
                && profile.roughness_sideband_fraction_permille == 350
                && profile.roughness_index_permille == 900
                && profile.stochastic_residual_gain_q15 > 0
                && profile.stochastic_residual_t20_ms > 0
        }));
        let serialized = serde_json::to_value(&profiles).expect("serialize v4 profiles");
        let values = serialized.as_array().expect("v4 profile array");
        assert!(values[0].get("stochastic_residual").is_none());
        assert!(values.iter().skip(1).all(|profile| {
            profile.get("stochastic_residual").is_some()
                && profile.get("stochastic_residual_gain_q15").is_some()
                && profile.get("stochastic_residual_t20_ms").is_some()
        }));
    }
}
