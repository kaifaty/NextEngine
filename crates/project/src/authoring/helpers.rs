use super::*;

pub(super) fn neutral_kind(kind: AuthoringNeutralRecordKindV1) -> NeutralRecordKindV1 {
    match kind {
        AuthoringNeutralRecordKindV1::Scene => NeutralRecordKindV1::Scene,
        AuthoringNeutralRecordKindV1::Collider => NeutralRecordKindV1::Collider,
        AuthoringNeutralRecordKindV1::CharacterDefinition => {
            NeutralRecordKindV1::CharacterDefinition
        }
        AuthoringNeutralRecordKindV1::ItemDefinition => NeutralRecordKindV1::ItemDefinition,
        AuthoringNeutralRecordKindV1::InventoryDefinition => {
            NeutralRecordKindV1::InventoryDefinition
        }
        AuthoringNeutralRecordKindV1::EquipmentDefinition => {
            NeutralRecordKindV1::EquipmentDefinition
        }
        AuthoringNeutralRecordKindV1::DialogueDefinition => NeutralRecordKindV1::DialogueDefinition,
        AuthoringNeutralRecordKindV1::QuestDefinition => NeutralRecordKindV1::QuestDefinition,
        AuthoringNeutralRecordKindV1::RelationshipDefinition => {
            NeutralRecordKindV1::RelationshipDefinition
        }
        AuthoringNeutralRecordKindV1::InteractionDefinition => {
            NeutralRecordKindV1::InteractionDefinition
        }
        AuthoringNeutralRecordKindV1::AbilityDefinition => NeutralRecordKindV1::AbilityDefinition,
        AuthoringNeutralRecordKindV1::WorldChunk => NeutralRecordKindV1::WorldChunk,
    }
}

pub(super) fn texture_color_space(
    value: AuthoringTextureColorSpaceV1,
) -> NeutralTextureColorSpaceV1 {
    match value {
        AuthoringTextureColorSpaceV1::Srgb => NeutralTextureColorSpaceV1::Srgb,
        AuthoringTextureColorSpaceV1::Linear => NeutralTextureColorSpaceV1::Linear,
        AuthoringTextureColorSpaceV1::Data => NeutralTextureColorSpaceV1::Data,
    }
}

pub(super) fn texture_alpha(value: AuthoringTextureAlphaV1) -> NeutralTextureAlphaSemanticsV1 {
    match value {
        AuthoringTextureAlphaV1::Opaque => NeutralTextureAlphaSemanticsV1::Opaque,
        AuthoringTextureAlphaV1::Straight => NeutralTextureAlphaSemanticsV1::Straight,
    }
}

pub(super) fn insert_revision(
    revisions: &mut BTreeMap<AssetId, AssetRevisionRefV1>,
    revision: AssetRevisionRefV1,
) -> Result<(), ProjectAuthoringError> {
    if revisions.insert(revision.asset_id, revision).is_some() {
        return Err(ProjectAuthoringError::DuplicateIdentity);
    }
    Ok(())
}

pub(super) fn revision(
    revisions: &BTreeMap<AssetId, AssetRevisionRefV1>,
    id: &str,
) -> Result<AssetRevisionRefV1, ProjectAuthoringError> {
    revisions
        .get(&asset_id(id)?)
        .copied()
        .ok_or_else(|| ProjectAuthoringError::MissingReference(id.to_owned()))
}

pub(super) fn build_audio_clip(
    asset_id: AssetId,
    revision: u64,
    sample_rate: u32,
    samples: &[i16],
    loop_region_or_none: Option<AudioLoopRegionV1>,
) -> Result<NeutralAudioV1, ProjectAuthoringError> {
    if samples.is_empty() {
        return Err(ProjectAuthoringError::InvalidValue);
    }
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    let mut peak = 0_u32;
    let mut energy = 0_u64;
    for sample in samples {
        pcm.extend_from_slice(&sample.to_le_bytes());
        let magnitude = u32::from(sample.unsigned_abs());
        peak = peak.max(magnitude);
        energy = energy
            .checked_add(u64::from(magnitude))
            .ok_or(ProjectAuthoringError::InvalidValue)?;
    }
    let peak_q16_16 = peak.saturating_mul(65_536) / 32_767;
    let mean =
        energy / u64::try_from(samples.len()).map_err(|_| ProjectAuthoringError::InvalidValue)?;
    let integrated = i32::try_from(mean.saturating_mul(65_536) / 32_767)
        .unwrap_or(i32::MAX)
        .saturating_sub(65_536);
    Ok(NeutralAudioV1::new(
        asset_id,
        revision,
        sample_rate,
        1,
        AudioPcmEncodingV1::PcmS16Le,
        u64::try_from(samples.len()).map_err(|_| ProjectAuthoringError::InvalidValue)?,
        loop_region_or_none,
        Vec::new(),
        AudioLoudnessMetadataV1::new(integrated, peak_q16_16)?,
        pcm,
    )?)
}

pub(super) fn envelope(amplitude: i32, index: u32, total: u32) -> i32 {
    let total = i64::from(total.max(1));
    let remaining = total.saturating_sub(i64::from(index));
    i32::try_from(i64::from(amplitude).saturating_mul(remaining) / total).unwrap_or(
        if amplitude.is_negative() {
            i32::MIN
        } else {
            i32::MAX
        },
    )
}

pub(super) fn synthesize_noise_burst(frames: u32, amplitude: i32, seed: u32) -> Vec<i16> {
    let mut state = seed.max(1);
    (0..frames)
        .map(|index| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = i32::from((state & 0xffff) as u16) - 32_767;
            let sample = i64::from(noise) * i64::from(envelope(amplitude, index, frames)) / 32_767;
            sample.clamp(-32_767, 32_767) as i16
        })
        .collect()
}

/// Plan `continuum-water/34`: flat noise between two linear fades of
/// `NOISE_LOOP_FADE_FRAMES` (5 ms at 48 kHz), looped over the whole clip.
pub(super) const NOISE_LOOP_FADE_FRAMES: u32 = 240;

pub(super) fn synthesize_noise_loop(frames: u32, amplitude: i32, seed: u32) -> Vec<i16> {
    let mut state = seed.max(1);
    let fade = NOISE_LOOP_FADE_FRAMES.min(frames / 2).max(1);
    (0..frames)
        .map(|index| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let noise = i32::from((state & 0xffff) as u16) - 32_767;
            let gain = if index < fade {
                i64::from(index + 1) * i64::from(amplitude) / i64::from(fade)
            } else if index + fade >= frames {
                i64::from(frames - index) * i64::from(amplitude) / i64::from(fade)
            } else {
                i64::from(amplitude)
            };
            (i64::from(noise) * gain / 32_767).clamp(-32_767, 32_767) as i16
        })
        .collect()
}

pub(super) fn synthesize_two_tone(
    frames: u32,
    first_period: u32,
    second_period: u32,
    amplitude: i32,
) -> Vec<i16> {
    (0..frames)
        .map(|index| {
            let period = if index.saturating_mul(2) < frames {
                first_period.max(2)
            } else {
                second_period.max(2)
            };
            let wave = if (index % period).saturating_mul(2) < period {
                1_i64
            } else {
                -1_i64
            };
            (wave * i64::from(envelope(amplitude, index, frames))).clamp(-32_767, 32_767) as i16
        })
        .collect()
}

pub(super) fn synthesize_thud(frames: u32, period: u32, amplitude: i32) -> Vec<i16> {
    (0..frames)
        .map(|index| {
            let period = period.max(2);
            let wave = if (index % period).saturating_mul(2) < period {
                1_i64
            } else {
                -1_i64
            };
            let linear = i64::from(envelope(amplitude, index, frames));
            let denominator = i64::from(amplitude.max(1));
            (wave * linear * linear / denominator).clamp(-32_767, 32_767) as i16
        })
        .collect()
}

pub(super) fn append_string(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProjectAuthoringError> {
    let length = u32::try_from(value.len()).map_err(|_| ProjectAuthoringError::InvalidValue)?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

#[cfg(test)]
mod noise_loop_tests {
    use super::{NOISE_LOOP_FADE_FRAMES, synthesize_noise_loop};

    /// Plan 34 G2: fades at both ends, a flat body, deterministic.
    #[test]
    fn noise_loop_has_fades_and_a_flat_body() {
        let frames = 24_000;
        let samples = synthesize_noise_loop(frames, 6_000, 7);
        assert_eq!(samples.len(), frames as usize);
        assert!(
            samples[0].unsigned_abs() <= 30,
            "first frame {}",
            samples[0]
        );
        let fade = NOISE_LOOP_FADE_FRAMES as usize;
        let head: i64 = samples[..fade / 4]
            .iter()
            .map(|s| i64::from(s.unsigned_abs()))
            .sum();
        let tail: i64 = samples[frames as usize - fade / 4..]
            .iter()
            .map(|s| i64::from(s.unsigned_abs()))
            .sum();
        let body: i64 = samples[fade..fade + fade / 4]
            .iter()
            .map(|s| i64::from(s.unsigned_abs()))
            .sum();
        assert!(
            head < body / 4 && tail < body / 4,
            "head {head} body {body} tail {tail}"
        );
        let peak = samples.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!(peak <= 6_000 && peak > 5_000, "peak {peak}");
        assert_eq!(samples, synthesize_noise_loop(frames, 6_000, 7));
    }
}
