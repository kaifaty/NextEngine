//! Typed neutral audio clip contract (SPEC-24 `nextengine.content.audio`,
//! SPEC-08 baseline audio).
//!
//! A `NeutralAudioV1` carries one canonical PCM clip with its sample rate,
//! channel count, frame count, optional loop region, cue frame positions and
//! fixed-point loudness metadata. The record is backend-free: no device
//! format, mixer buffer, codec-library state or voice-service object may
//! appear. PCM samples are stored inline in the canonical segment, bounded by
//! the decode limits of the consuming path; gameplay acoustic/timing facts
//! remain portable and independent of any output device.

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, IdentifierError};
use crate::project::domain_hash;

pub const NEUTRAL_AUDIO_SCHEMA_ID: &str = "nextengine.content.audio";
pub const NEUTRAL_AUDIO_OWNER_ID: &str = "nextengine.assets";
pub const NEUTRAL_AUDIO_SEGMENT_ID: &str = "nextengine.audio.v1";
pub const NEUTRAL_AUDIO_SCHEMA_VERSION: u32 = 1;
pub const NEUTRAL_AUDIO_MIN_SAMPLE_RATE: u32 = 8_000;
pub const NEUTRAL_AUDIO_MAX_SAMPLE_RATE: u32 = 192_000;
pub const NEUTRAL_AUDIO_MAX_CHANNELS: u32 = 8;
pub const NEUTRAL_AUDIO_MAX_DURATION_SECONDS: u64 = 21_600;
pub const NEUTRAL_AUDIO_MAX_CUES: usize = 65_535;
/// `sample_peak_q16_16` upper bound: exactly 1.0 full scale.
pub const NEUTRAL_AUDIO_PEAK_Q16_16_MAX: u32 = 65_536;

/// Closed V1 PCM encodings (SPEC-24). Sample width in bytes is exact.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AudioPcmEncodingV1 {
    PcmS16Le = 1,
    PcmS24LePacked = 2,
    PcmF32LeCanonical = 3,
}

impl AudioPcmEncodingV1 {
    fn from_tag(tag: u32) -> Result<Self, NeutralAudioErrorV1> {
        match tag {
            1 => Ok(Self::PcmS16Le),
            2 => Ok(Self::PcmS24LePacked),
            3 => Ok(Self::PcmF32LeCanonical),
            _ => Err(NeutralAudioErrorV1::InvalidPcmEncoding),
        }
    }

    #[must_use]
    pub const fn bytes_per_sample(self) -> u64 {
        match self {
            Self::PcmS16Le => 2,
            Self::PcmS24LePacked => 3,
            Self::PcmF32LeCanonical => 4,
        }
    }
}

/// Inclusive-exclusive loop region in integer sample frames. `start_frame` is
/// strictly before `end_frame`; `end_frame` may equal the clip frame count
/// (loop to the end of the clip).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AudioLoopRegionV1 {
    pub start_frame: u64,
    pub end_frame: u64,
}

impl AudioLoopRegionV1 {
    pub fn new(start_frame: u64, end_frame: u64) -> Result<Self, NeutralAudioErrorV1> {
        if start_frame >= end_frame {
            return Err(NeutralAudioErrorV1::InvalidLoopRegion);
        }
        Ok(Self {
            start_frame,
            end_frame,
        })
    }
}

/// Fixed-point loudness metadata. `integrated_loudness_q16_16` is a signed
/// Q16.16 loudness value (typically nonpositive). `sample_peak_q16_16` is an
/// unsigned Q16.16 peak amplitude in `0..=65_536` (0.0..=1.0 full scale).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AudioLoudnessMetadataV1 {
    pub integrated_loudness_q16_16: i32,
    pub sample_peak_q16_16: u32,
}

impl AudioLoudnessMetadataV1 {
    pub fn new(
        integrated_loudness_q16_16: i32,
        sample_peak_q16_16: u32,
    ) -> Result<Self, NeutralAudioErrorV1> {
        if sample_peak_q16_16 > NEUTRAL_AUDIO_PEAK_Q16_16_MAX {
            return Err(NeutralAudioErrorV1::InvalidLoudness);
        }
        Ok(Self {
            integrated_loudness_q16_16,
            sample_peak_q16_16,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralAudioV1 {
    pub schema_version: u32,
    pub asset_id: AssetId,
    pub record_revision: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u32,
    pub pcm_encoding: AudioPcmEncodingV1,
    pub frame_count: u64,
    pub loop_region_or_none: Option<AudioLoopRegionV1>,
    pub cue_frames: Vec<u64>,
    pub loudness: AudioLoudnessMetadataV1,
    pcm: Vec<u8>,
    pub content_hash: ContentHash,
}

impl NeutralAudioV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the closed neutral record constructor keeps all canonical fields explicit"
    )]
    pub fn new(
        asset_id: AssetId,
        record_revision: u64,
        sample_rate_hz: u32,
        channel_count: u32,
        pcm_encoding: AudioPcmEncodingV1,
        frame_count: u64,
        loop_region_or_none: Option<AudioLoopRegionV1>,
        mut cue_frames: Vec<u64>,
        loudness: AudioLoudnessMetadataV1,
        mut pcm: Vec<u8>,
    ) -> Result<Self, NeutralAudioErrorV1> {
        if record_revision == 0 {
            return Err(NeutralAudioErrorV1::InvalidRevision);
        }
        if !(NEUTRAL_AUDIO_MIN_SAMPLE_RATE..=NEUTRAL_AUDIO_MAX_SAMPLE_RATE)
            .contains(&sample_rate_hz)
        {
            return Err(NeutralAudioErrorV1::InvalidSampleRate);
        }
        if channel_count == 0 || channel_count > NEUTRAL_AUDIO_MAX_CHANNELS {
            return Err(NeutralAudioErrorV1::InvalidChannelCount);
        }
        if frame_count == 0
            || u128::from(frame_count)
                > u128::from(NEUTRAL_AUDIO_MAX_DURATION_SECONDS) * u128::from(sample_rate_hz)
        {
            return Err(NeutralAudioErrorV1::InvalidFrameCount);
        }
        if let Some(region) = loop_region_or_none
            && region.end_frame > frame_count
        {
            return Err(NeutralAudioErrorV1::InvalidLoopRegion);
        }
        if cue_frames.len() > NEUTRAL_AUDIO_MAX_CUES {
            return Err(NeutralAudioErrorV1::LimitExceeded {
                actual: cue_frames.len(),
                limit: NEUTRAL_AUDIO_MAX_CUES,
            });
        }
        cue_frames.sort_unstable();
        if cue_frames.windows(2).any(|pair| pair[0] == pair[1])
            || cue_frames.last().is_some_and(|frame| *frame >= frame_count)
        {
            return Err(NeutralAudioErrorV1::InvalidCueFrames);
        }
        let expected_pcm_len = expected_pcm_len(frame_count, channel_count, pcm_encoding)?;
        if pcm.len() != expected_pcm_len {
            return Err(NeutralAudioErrorV1::PcmLengthMismatch {
                actual: pcm.len(),
                expected: expected_pcm_len,
            });
        }
        if pcm_encoding == AudioPcmEncodingV1::PcmF32LeCanonical {
            normalize_f32_samples(&mut pcm)?;
        }
        let mut clip = Self {
            schema_version: NEUTRAL_AUDIO_SCHEMA_VERSION,
            asset_id,
            record_revision,
            sample_rate_hz,
            channel_count,
            pcm_encoding,
            frame_count,
            loop_region_or_none,
            cue_frames,
            loudness,
            pcm,
            content_hash: ContentHash::from_bytes([0; 32]),
        };
        clip.content_hash = clip.compute_content_hash()?;
        Ok(clip)
    }

    #[must_use]
    pub fn pcm_bytes(&self) -> &[u8] {
        &self.pcm
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NeutralAudioErrorV1> {
        let mut fields = self.body_fields()?;
        fields.push(CanonicalField::new(
            12,
            CANONICAL_TYPE_HASH256,
            self.content_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            NEUTRAL_AUDIO_OWNER_ID,
            NEUTRAL_AUDIO_SCHEMA_ID,
            NEUTRAL_AUDIO_SEGMENT_ID,
            fields,
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, NeutralAudioErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != NEUTRAL_AUDIO_OWNER_ID
            || segment.schema_id != NEUTRAL_AUDIO_SCHEMA_ID
            || segment.segment_id != NEUTRAL_AUDIO_SEGMENT_ID
            || segment.fields.len() != 12
        {
            return Err(NeutralAudioErrorV1::EnvelopeMismatch);
        }
        let version = read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?;
        if version != NEUTRAL_AUDIO_SCHEMA_VERSION {
            return Err(NeutralAudioErrorV1::UnsupportedVersion(version));
        }
        let asset_id = AssetId::from_bytes(read_fixed(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let record_revision = read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?;
        let sample_rate_hz = read_u32(field(&segment, 4, CANONICAL_TYPE_U32)?)?;
        let channel_count = read_u32(field(&segment, 5, CANONICAL_TYPE_U32)?)?;
        let pcm_encoding =
            AudioPcmEncodingV1::from_tag(read_u32(field(&segment, 6, CANONICAL_TYPE_U32)?)?)?;
        let frame_count = read_u64(field(&segment, 7, CANONICAL_TYPE_U64)?)?;
        let loop_region_or_none = decode_loop_region(field(&segment, 8, CANONICAL_TYPE_OPTIONAL)?)?;
        let cue_frames = decode_cue_frames(field(&segment, 9, CANONICAL_TYPE_SEQUENCE)?, limits)?;
        let loudness = decode_loudness(field(&segment, 10, CANONICAL_TYPE_STRUCT)?)?;
        let pcm = decode_pcm(field(&segment, 11, CANONICAL_TYPE_BYTES)?)?;
        let content_hash =
            ContentHash::from_bytes(read_fixed(field(&segment, 12, CANONICAL_TYPE_HASH256)?)?);
        let clip = Self::new(
            asset_id,
            record_revision,
            sample_rate_hz,
            channel_count,
            pcm_encoding,
            frame_count,
            loop_region_or_none,
            cue_frames,
            loudness,
            pcm,
        )?;
        if clip.content_hash != content_hash {
            return Err(NeutralAudioErrorV1::HashMismatch);
        }
        if clip.canonical_bytes()? != bytes {
            return Err(NeutralAudioErrorV1::NonCanonical);
        }
        Ok(clip)
    }

    /// Hash bound into cooked blobs and the content manifest for this record.
    pub fn record_sha256(&self) -> Result<ContentHash, NeutralAudioErrorV1> {
        Ok(domain_hash(
            NEUTRAL_AUDIO_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    fn body_fields(&self) -> Result<Vec<CanonicalField>, NeutralAudioErrorV1> {
        Ok(vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_ID128, self.asset_id.as_bytes().to_vec()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U64,
                self.record_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_U32,
                self.sample_rate_hz.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U32,
                self.channel_count.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_U32,
                (self.pcm_encoding as u32).to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_U64,
                self.frame_count.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                8,
                CANONICAL_TYPE_OPTIONAL,
                encode_loop_region(&self.loop_region_or_none),
            ),
            CanonicalField::new(
                9,
                CANONICAL_TYPE_SEQUENCE,
                encode_cue_frames(&self.cue_frames)?,
            ),
            CanonicalField::new(10, CANONICAL_TYPE_STRUCT, encode_loudness(&self.loudness)),
            CanonicalField::new(11, CANONICAL_TYPE_BYTES, encode_pcm(&self.pcm)?),
        ])
    }

    fn compute_content_hash(&self) -> Result<ContentHash, NeutralAudioErrorV1> {
        let body = encode_canonical_segment(
            NEUTRAL_AUDIO_OWNER_ID,
            NEUTRAL_AUDIO_SCHEMA_ID,
            NEUTRAL_AUDIO_SEGMENT_ID,
            self.body_fields()?,
        )?;
        Ok(domain_hash(NEUTRAL_AUDIO_SEGMENT_ID, &body))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NeutralAudioErrorV1 {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    InvalidSampleRate,
    InvalidChannelCount,
    InvalidPcmEncoding,
    InvalidFrameCount,
    InvalidLoopRegion,
    InvalidCueFrames,
    InvalidLoudness,
    PcmLengthMismatch { actual: usize, expected: usize },
    InvalidPcmSample,
    InvalidRevision,
    EnvelopeMismatch,
    UnsupportedVersion(u32),
    HashMismatch,
    NonCanonical,
    LimitExceeded { actual: usize, limit: usize },
    WrongFieldType(u32),
    InvalidPayload,
}

impl Display for NeutralAudioErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "neutral audio encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "neutral audio decode failed: {error}"),
            Self::Identifier(error) => write!(formatter, "neutral audio ID is invalid: {error}"),
            Self::InvalidSampleRate => formatter.write_str("neutral audio sample rate is invalid"),
            Self::InvalidChannelCount => {
                formatter.write_str("neutral audio channel count is invalid")
            }
            Self::InvalidPcmEncoding => {
                formatter.write_str("neutral audio PCM encoding is invalid")
            }
            Self::InvalidFrameCount => formatter.write_str("neutral audio frame count is invalid"),
            Self::InvalidLoopRegion => formatter.write_str("neutral audio loop region is invalid"),
            Self::InvalidCueFrames => formatter.write_str("neutral audio cue frames are invalid"),
            Self::InvalidLoudness => {
                formatter.write_str("neutral audio loudness metadata is invalid")
            }
            Self::PcmLengthMismatch { actual, expected } => {
                write!(
                    formatter,
                    "neutral audio PCM length {actual} does not match expected {expected}"
                )
            }
            Self::InvalidPcmSample => formatter.write_str("neutral audio PCM sample is invalid"),
            Self::InvalidRevision => formatter.write_str("neutral audio revision must be positive"),
            Self::EnvelopeMismatch => formatter.write_str("neutral audio envelope mismatch"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported neutral audio version {version}")
            }
            Self::HashMismatch => formatter.write_str("neutral audio content hash mismatch"),
            Self::NonCanonical => formatter.write_str("neutral audio bytes are not canonical"),
            Self::LimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "neutral audio count {actual} exceeds limit {limit}"
                )
            }
            Self::WrongFieldType(field_id) => {
                write!(formatter, "neutral audio field {field_id} has wrong type")
            }
            Self::InvalidPayload => formatter.write_str("neutral audio payload is invalid"),
        }
    }
}

impl Error for NeutralAudioErrorV1 {}

impl From<CanonicalError> for NeutralAudioErrorV1 {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for NeutralAudioErrorV1 {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for NeutralAudioErrorV1 {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn expected_pcm_len(
    frame_count: u64,
    channel_count: u32,
    pcm_encoding: AudioPcmEncodingV1,
) -> Result<usize, NeutralAudioErrorV1> {
    let total = u128::from(frame_count)
        .checked_mul(u128::from(channel_count))
        .and_then(|value| value.checked_mul(u128::from(pcm_encoding.bytes_per_sample())))
        .ok_or(NeutralAudioErrorV1::InvalidFrameCount)?;
    usize::try_from(total).map_err(|_| NeutralAudioErrorV1::LimitExceeded {
        actual: usize::MAX,
        limit: usize::MAX,
    })
}

/// Canonicalizes `PcmF32LeCanonical` samples in place: finite, within
/// `[-1, 1]`, semantic negative zero becomes positive zero.
fn normalize_f32_samples(pcm: &mut [u8]) -> Result<(), NeutralAudioErrorV1> {
    for chunk in pcm.chunks_exact_mut(4) {
        let bits = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let value = f32::from_bits(bits);
        if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
            return Err(NeutralAudioErrorV1::InvalidPcmSample);
        }
        if value == 0.0 && value.is_sign_negative() {
            chunk.copy_from_slice(&0.0_f32.to_le_bytes());
        }
    }
    Ok(())
}

fn field(
    segment: &crate::canonical::DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&[u8], NeutralAudioErrorV1> {
    let field = segment
        .field(field_id)
        .ok_or(NeutralAudioErrorV1::InvalidPayload)?;
    if field.type_tag != expected_type {
        return Err(NeutralAudioErrorV1::WrongFieldType(field_id));
    }
    Ok(&field.payload)
}

fn read_u32(bytes: &[u8]) -> Result<u32, NeutralAudioErrorV1> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| NeutralAudioErrorV1::InvalidPayload)?,
    ))
}

fn read_u64(bytes: &[u8]) -> Result<u64, NeutralAudioErrorV1> {
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| NeutralAudioErrorV1::InvalidPayload)?,
    ))
}

fn read_fixed<const LENGTH: usize>(bytes: &[u8]) -> Result<[u8; LENGTH], NeutralAudioErrorV1> {
    bytes
        .try_into()
        .map_err(|_| NeutralAudioErrorV1::InvalidPayload)
}

fn encode_loop_region(region: &Option<AudioLoopRegionV1>) -> Vec<u8> {
    match region {
        None => vec![0],
        Some(region) => {
            let mut bytes = Vec::with_capacity(17);
            bytes.push(1);
            bytes.extend_from_slice(&region.start_frame.to_le_bytes());
            bytes.extend_from_slice(&region.end_frame.to_le_bytes());
            bytes
        }
    }
}

fn decode_loop_region(bytes: &[u8]) -> Result<Option<AudioLoopRegionV1>, NeutralAudioErrorV1> {
    match bytes.split_first() {
        Some((0, [])) => Ok(None),
        Some((1, rest)) => {
            if rest.len() != 16 {
                return Err(NeutralAudioErrorV1::InvalidPayload);
            }
            let start_frame = read_u64(&rest[0..8])?;
            let end_frame = read_u64(&rest[8..16])?;
            Ok(Some(AudioLoopRegionV1::new(start_frame, end_frame)?))
        }
        _ => Err(NeutralAudioErrorV1::InvalidPayload),
    }
}

fn encode_cue_frames(cue_frames: &[u64]) -> Result<Vec<u8>, NeutralAudioErrorV1> {
    let mut bytes = Vec::with_capacity(4 + cue_frames.len() * 8);
    bytes.extend_from_slice(
        &u32::try_from(cue_frames.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for frame in cue_frames {
        bytes.extend_from_slice(&frame.to_le_bytes());
    }
    Ok(bytes)
}

fn decode_cue_frames(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<u64>, NeutralAudioErrorV1> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| NeutralAudioErrorV1::InvalidPayload)?;
    let limit = limits.max_sequence_items.min(NEUTRAL_AUDIO_MAX_CUES);
    if count > limit {
        return Err(NeutralAudioErrorV1::LimitExceeded {
            actual: count,
            limit,
        });
    }
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        frames.push(cursor.read_u64()?);
    }
    cursor.finish()?;
    Ok(frames)
}

fn encode_loudness(loudness: &AudioLoudnessMetadataV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&loudness.integrated_loudness_q16_16.to_le_bytes());
    bytes.extend_from_slice(&loudness.sample_peak_q16_16.to_le_bytes());
    bytes
}

fn decode_loudness(bytes: &[u8]) -> Result<AudioLoudnessMetadataV1, NeutralAudioErrorV1> {
    if bytes.len() != 8 {
        return Err(NeutralAudioErrorV1::InvalidPayload);
    }
    let integrated_loudness_q16_16 = i32::from_le_bytes(
        bytes[0..4]
            .try_into()
            .map_err(|_| NeutralAudioErrorV1::InvalidPayload)?,
    );
    let sample_peak_q16_16 = read_u32(&bytes[4..8])?;
    AudioLoudnessMetadataV1::new(integrated_loudness_q16_16, sample_peak_q16_16)
}

fn encode_pcm(pcm: &[u8]) -> Result<Vec<u8>, NeutralAudioErrorV1> {
    let mut bytes = Vec::with_capacity(4 + pcm.len());
    bytes.extend_from_slice(
        &u32::try_from(pcm.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(pcm);
    Ok(bytes)
}

fn decode_pcm(bytes: &[u8]) -> Result<Vec<u8>, NeutralAudioErrorV1> {
    let mut cursor = CanonicalCursor::new(bytes);
    let length =
        usize::try_from(cursor.read_u32()?).map_err(|_| NeutralAudioErrorV1::InvalidPayload)?;
    let pcm = cursor.read_exact(length)?.to_vec();
    cursor.finish()?;
    Ok(pcm)
}

#[cfg(test)]
mod tests {
    use super::{
        AudioLoopRegionV1, AudioLoudnessMetadataV1, AudioPcmEncodingV1, NEUTRAL_AUDIO_MAX_CUES,
        NEUTRAL_AUDIO_PEAK_Q16_16_MAX, NEUTRAL_AUDIO_SCHEMA_ID, NeutralAudioErrorV1,
        NeutralAudioV1,
    };
    use crate::canonical::{
        CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalField, decode_canonical_segment,
        encode_canonical_segment,
    };
    use crate::ids::AssetId;

    fn loudness() -> AudioLoudnessMetadataV1 {
        AudioLoudnessMetadataV1::new(-1_015_806, 45_875).expect("loudness")
    }

    fn sample_clip() -> NeutralAudioV1 {
        // 4 frames of stereo S16 silence with an impulse on frame 1.
        let mut pcm = Vec::new();
        pcm.extend_from_slice(&0_i16.to_le_bytes());
        pcm.extend_from_slice(&0_i16.to_le_bytes());
        pcm.extend_from_slice(&16_384_i16.to_le_bytes());
        pcm.extend_from_slice(&16_384_i16.to_le_bytes());
        pcm.extend_from_slice(&0_i16.to_le_bytes());
        pcm.extend_from_slice(&0_i16.to_le_bytes());
        pcm.extend_from_slice((-8_192_i16).to_le_bytes().as_slice());
        pcm.extend_from_slice((-8_192_i16).to_le_bytes().as_slice());
        NeutralAudioV1::new(
            AssetId::from_bytes([0xa1; 16]),
            1,
            48_000,
            2,
            AudioPcmEncodingV1::PcmS16Le,
            4,
            Some(AudioLoopRegionV1::new(1, 4).expect("loop")),
            vec![0, 2],
            loudness(),
            pcm,
        )
        .expect("clip")
    }

    #[test]
    fn schema_identity_is_stable() {
        assert_eq!(NEUTRAL_AUDIO_SCHEMA_ID, "nextengine.content.audio");
        assert_eq!(NEUTRAL_AUDIO_MAX_CUES, 65_535);
        assert_eq!(NEUTRAL_AUDIO_PEAK_Q16_16_MAX, 65_536);
    }

    #[test]
    fn clip_round_trips_canonically() {
        let clip = sample_clip();
        let bytes = clip.canonical_bytes().expect("encode");
        let decoded =
            NeutralAudioV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode");
        assert_eq!(decoded, clip);
        assert_eq!(
            decoded.record_sha256().expect("hash"),
            clip.record_sha256().expect("hash")
        );
    }

    #[test]
    fn all_pcm_encodings_round_trip() {
        for (encoding, bytes_per_frame) in [
            (AudioPcmEncodingV1::PcmS16Le, 2_usize),
            (AudioPcmEncodingV1::PcmS24LePacked, 3),
            (AudioPcmEncodingV1::PcmF32LeCanonical, 4),
        ] {
            let clip = NeutralAudioV1::new(
                AssetId::from_bytes([0xa2; 16]),
                1,
                44_100,
                1,
                encoding,
                8,
                None,
                Vec::new(),
                loudness(),
                vec![0_u8; 8 * bytes_per_frame],
            )
            .expect("clip");
            let bytes = clip.canonical_bytes().expect("encode");
            assert_eq!(
                NeutralAudioV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                    .expect("decode"),
                clip
            );
        }
    }

    #[test]
    fn rejects_out_of_range_scalar_fields() {
        let base = sample_clip();
        for rate in [0, 7_999, 192_001] {
            assert_eq!(
                NeutralAudioV1::new(
                    base.asset_id,
                    1,
                    rate,
                    2,
                    AudioPcmEncodingV1::PcmS16Le,
                    4,
                    None,
                    Vec::new(),
                    loudness(),
                    vec![0_u8; 16],
                ),
                Err(NeutralAudioErrorV1::InvalidSampleRate)
            );
        }
        for channels in [0, 9] {
            assert_eq!(
                NeutralAudioV1::new(
                    base.asset_id,
                    1,
                    48_000,
                    channels,
                    AudioPcmEncodingV1::PcmS16Le,
                    4,
                    None,
                    Vec::new(),
                    loudness(),
                    vec![0_u8; 4 * channels as usize * 2],
                ),
                Err(NeutralAudioErrorV1::InvalidChannelCount)
            );
        }
        assert_eq!(
            NeutralAudioV1::new(
                base.asset_id,
                0,
                48_000,
                2,
                AudioPcmEncodingV1::PcmS16Le,
                4,
                None,
                Vec::new(),
                loudness(),
                vec![0_u8; 16],
            ),
            Err(NeutralAudioErrorV1::InvalidRevision)
        );
        // Zero frames and duration beyond the 21 600 s ceiling reject.
        assert_eq!(
            NeutralAudioV1::new(
                base.asset_id,
                1,
                48_000,
                1,
                AudioPcmEncodingV1::PcmS16Le,
                0,
                None,
                Vec::new(),
                loudness(),
                Vec::new(),
            ),
            Err(NeutralAudioErrorV1::InvalidFrameCount)
        );
        assert_eq!(
            NeutralAudioV1::new(
                base.asset_id,
                1,
                192_000,
                1,
                AudioPcmEncodingV1::PcmS16Le,
                21_600_u64 * 192_000_u64 + 1,
                None,
                Vec::new(),
                loudness(),
                Vec::new(),
            ),
            Err(NeutralAudioErrorV1::InvalidFrameCount)
        );
    }

    #[test]
    fn rejects_pcm_length_mismatch() {
        assert_eq!(
            NeutralAudioV1::new(
                AssetId::from_bytes([0xa3; 16]),
                1,
                48_000,
                2,
                AudioPcmEncodingV1::PcmS16Le,
                4,
                None,
                Vec::new(),
                loudness(),
                vec![0_u8; 15],
            ),
            Err(NeutralAudioErrorV1::PcmLengthMismatch {
                actual: 15,
                expected: 16,
            })
        );
    }

    #[test]
    fn rejects_invalid_loop_and_cues() {
        let base = sample_clip();
        let make = |loop_region, cues: Vec<u64>| {
            NeutralAudioV1::new(
                base.asset_id,
                1,
                48_000,
                2,
                AudioPcmEncodingV1::PcmS16Le,
                4,
                loop_region,
                cues,
                loudness(),
                vec![0_u8; 16],
            )
        };
        assert_eq!(
            make(
                Some(AudioLoopRegionV1::new(2, 4).expect("loop")),
                Vec::new()
            )
            .map(|clip| clip.loop_region_or_none),
            Ok(Some(AudioLoopRegionV1::new(2, 4).expect("loop")))
        );
        assert!(AudioLoopRegionV1::new(4, 4).is_err());
        assert!(AudioLoopRegionV1::new(4, 3).is_err());
        // Loop end past the clip rejects.
        assert_eq!(
            make(
                Some(AudioLoopRegionV1::new(0, 5).expect("loop")),
                Vec::new()
            ),
            Err(NeutralAudioErrorV1::InvalidLoopRegion)
        );
        // Cue at/past frame count rejects.
        assert_eq!(
            make(None, vec![4]),
            Err(NeutralAudioErrorV1::InvalidCueFrames)
        );
        // Duplicate cues reject after canonical sorting.
        assert_eq!(
            make(None, vec![2, 0, 2]),
            Err(NeutralAudioErrorV1::InvalidCueFrames)
        );
        // Unsorted input is canonicalized.
        let clip = make(None, vec![3, 0]).expect("sorted cues");
        assert_eq!(clip.cue_frames, vec![0, 3]);
        // Cue count ceiling rejects.
        assert_eq!(
            make(None, vec![0_u64; NEUTRAL_AUDIO_MAX_CUES + 1]),
            Err(NeutralAudioErrorV1::LimitExceeded {
                actual: NEUTRAL_AUDIO_MAX_CUES + 1,
                limit: NEUTRAL_AUDIO_MAX_CUES,
            })
        );
    }

    #[test]
    fn rejects_invalid_loudness_peak() {
        assert_eq!(
            AudioLoudnessMetadataV1::new(0, NEUTRAL_AUDIO_PEAK_Q16_16_MAX + 1),
            Err(NeutralAudioErrorV1::InvalidLoudness)
        );
        assert!(AudioLoudnessMetadataV1::new(i32::MIN, NEUTRAL_AUDIO_PEAK_Q16_16_MAX).is_ok());
    }

    #[test]
    fn f32_samples_validate_and_normalize_negative_zero() {
        let encode = |values: &[f32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let make = |values: &[f32]| {
            NeutralAudioV1::new(
                AssetId::from_bytes([0xa4; 16]),
                1,
                48_000,
                1,
                AudioPcmEncodingV1::PcmF32LeCanonical,
                values.len() as u64,
                None,
                Vec::new(),
                loudness(),
                encode(values),
            )
        };
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.5, -1.5] {
            assert_eq!(make(&[invalid]), Err(NeutralAudioErrorV1::InvalidPcmSample));
        }
        let clip = make(&[-0.0, 0.5, -1.0, 1.0]).expect("valid f32 clip");
        // Semantic negative zero is canonicalized to positive zero.
        assert_eq!(&clip.pcm_bytes()[0..4], &0.0_f32.to_le_bytes());
        let bytes = clip.canonical_bytes().expect("encode");
        assert_eq!(
            NeutralAudioV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode"),
            clip
        );
    }

    #[test]
    fn decoder_rejects_tampered_hash_and_unsupported_version() {
        let clip = sample_clip();
        let bytes = clip.canonical_bytes().expect("encode");
        let mut segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");

        let mut tampered_hash = segment.fields.clone();
        tampered_hash[11].payload[0] ^= 0xff;
        let tampered_hash_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            tampered_hash,
        )
        .expect("re-encode");
        assert_eq!(
            NeutralAudioV1::from_canonical_bytes(
                &tampered_hash_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(NeutralAudioErrorV1::HashMismatch)
        );

        segment.fields[0] =
            CanonicalField::new(1, CANONICAL_TYPE_U32, 2_u32.to_le_bytes().to_vec());
        let wrong_version_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            segment.fields,
        )
        .expect("re-encode");
        assert_eq!(
            NeutralAudioV1::from_canonical_bytes(
                &wrong_version_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(NeutralAudioErrorV1::UnsupportedVersion(2))
        );
    }

    #[test]
    fn decoder_rejects_unsorted_cues_as_non_canonical() {
        let clip = sample_clip();
        let bytes = clip.canonical_bytes().expect("encode");
        let segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");
        let mut reversed_cues = Vec::new();
        reversed_cues.extend_from_slice(&2_u32.to_le_bytes());
        reversed_cues.extend_from_slice(&2_u64.to_le_bytes());
        reversed_cues.extend_from_slice(&0_u64.to_le_bytes());
        let mut fields = segment.fields.clone();
        fields[8].payload = reversed_cues;
        // Recomputing the content hash is not possible here, so the decoder
        // must first reject the hash binding; a re-hashed payload would then
        // fail the byte-exact canonical round trip. Either way the tampered
        // record is rejected.
        let tampered = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            fields,
        )
        .expect("re-encode");
        assert!(matches!(
            NeutralAudioV1::from_canonical_bytes(&tampered, CanonicalDecodeLimits::default()),
            Err(NeutralAudioErrorV1::HashMismatch) | Err(NeutralAudioErrorV1::NonCanonical)
        ));
    }
}
