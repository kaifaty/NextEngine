//! Local player preference profile contract (SPEC-18 §Accessibility, SPEC-17).
//!
//! A `PlayerPreferenceProfileV1` is versioned **local** data of the SPEC-17
//! `PresentationOnly` configuration class: it may tune presentation (text
//! scale, requested UI locale) but it is never save/domain state, never
//! enters a gameplay hash and its stored values are not gameplay authority.
//! Corruption follows `PLAYER_PREFERENCE_INVALID`: the profile is quarantined
//! by the loading store and bounded defaults are used; save/gameplay remain
//! untouched. Import/export validates schema and bounds and never loads
//! executable content.

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_U32, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{ContentHash, IdentifierError};
use crate::localization::{TEXT_CATALOG_MAX_LOCALE_TAG_BYTES, TextLocaleTagV1};
use crate::project::domain_hash;

pub const PLAYER_PREFERENCE_SCHEMA_ID: &str = "nextengine.local.player-preference";
pub const PLAYER_PREFERENCE_OWNER_ID: &str = "nextengine.player-experience";
pub const PLAYER_PREFERENCE_SEGMENT_ID: &str = "nextengine.player-preference.v1";
pub const PLAYER_PREFERENCE_SCHEMA_VERSION: u32 = 1;
pub const PLAYER_PREFERENCE_INVALID_CODE: &str = "PLAYER_PREFERENCE_INVALID";
pub const PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN: u32 = 500;
pub const PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX: u32 = 2_000;
pub const PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT: u32 = 1_000;

/// Maps the bounded `text_scale_milli` preference to the integer overlay text
/// scale (`500..=2000` → `1..=4`). Defensive: out-of-contract values clamp to
/// the same range so a corrupt caller can never produce a zero or runaway
/// scale.
#[must_use]
pub const fn text_scale_from_milli(text_scale_milli: u32) -> u32 {
    let scale = text_scale_milli / 500;
    if scale < 1 {
        1
    } else if scale > 4 {
        4
    } else {
        scale
    }
}

/// Versioned local `PresentationOnly` preference profile (minimal scope:
/// text scale + requested UI locale + subtitle visibility).
///
/// The same values must always encode to the same canonical bytes; two
/// profiles with equal fields carry equal content hashes. The profile never
/// participates in project locks, saves, replay compatibility or gameplay
/// hashes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreferenceProfileV1 {
    pub schema_version: u32,
    pub revision: u32,
    pub text_scale_milli: u32,
    pub ui_locale_or_none: Option<TextLocaleTagV1>,
    pub subtitles_enabled: bool,
    pub content_hash: ContentHash,
}

impl PlayerPreferenceProfileV1 {
    pub fn new(
        revision: u32,
        text_scale_milli: u32,
        ui_locale_or_none: Option<TextLocaleTagV1>,
        subtitles_enabled: bool,
    ) -> Result<Self, PlayerPreferenceErrorV1> {
        if revision == 0 {
            return Err(PlayerPreferenceErrorV1::InvalidRevision);
        }
        if !(PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN..=PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX)
            .contains(&text_scale_milli)
        {
            return Err(PlayerPreferenceErrorV1::TextScaleOutOfRange {
                actual: text_scale_milli,
                minimum: PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN,
                maximum: PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX,
            });
        }
        let mut profile = Self {
            schema_version: PLAYER_PREFERENCE_SCHEMA_VERSION,
            revision,
            text_scale_milli,
            ui_locale_or_none,
            subtitles_enabled,
            content_hash: ContentHash::from_bytes([0; 32]),
        };
        profile.content_hash = profile.compute_content_hash()?;
        Ok(profile)
    }

    /// Bounded defaults used after `PLAYER_PREFERENCE_INVALID` quarantine and
    /// when no local profile exists: reference text scale, project-default
    /// locale, subtitles enabled (voice-absent subtitle fallback per SPEC-08).
    /// Construction cannot fail; the constant fields are in range.
    #[must_use]
    pub fn bounded_defaults() -> Self {
        Self::new(1, PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT, None, true)
            .expect("bounded preference defaults are in range")
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PlayerPreferenceErrorV1> {
        let mut fields = self.body_fields()?;
        fields.push(CanonicalField::new(
            6,
            CANONICAL_TYPE_HASH256,
            self.content_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            PLAYER_PREFERENCE_OWNER_ID,
            PLAYER_PREFERENCE_SCHEMA_ID,
            PLAYER_PREFERENCE_SEGMENT_ID,
            fields,
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PlayerPreferenceErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != PLAYER_PREFERENCE_OWNER_ID
            || segment.schema_id != PLAYER_PREFERENCE_SCHEMA_ID
            || segment.segment_id != PLAYER_PREFERENCE_SEGMENT_ID
            || segment.fields.len() != 6
        {
            return Err(PlayerPreferenceErrorV1::EnvelopeMismatch);
        }
        let version = read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?;
        if version != PLAYER_PREFERENCE_SCHEMA_VERSION {
            return Err(PlayerPreferenceErrorV1::UnsupportedVersion(version));
        }
        let revision = read_u32(field(&segment, 2, CANONICAL_TYPE_U32)?)?;
        let text_scale_milli = read_u32(field(&segment, 3, CANONICAL_TYPE_U32)?)?;
        let ui_locale_or_none =
            decode_optional_locale(field(&segment, 4, CANONICAL_TYPE_OPTIONAL)?)?;
        let subtitles_enabled = match read_u32(field(&segment, 5, CANONICAL_TYPE_U32)?)? {
            0 => false,
            1 => true,
            _ => return Err(PlayerPreferenceErrorV1::InvalidPayload),
        };
        let content_hash =
            ContentHash::from_bytes(read_fixed(field(&segment, 6, CANONICAL_TYPE_HASH256)?)?);
        let profile = Self::new(
            revision,
            text_scale_milli,
            ui_locale_or_none,
            subtitles_enabled,
        )?;
        if profile.content_hash != content_hash {
            return Err(PlayerPreferenceErrorV1::HashMismatch);
        }
        if profile.canonical_bytes()? != bytes {
            return Err(PlayerPreferenceErrorV1::NonCanonical);
        }
        Ok(profile)
    }

    fn body_fields(&self) -> Result<Vec<CanonicalField>, PlayerPreferenceErrorV1> {
        Ok(vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_U32, self.revision.to_le_bytes().to_vec()),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_U32,
                self.text_scale_milli.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_OPTIONAL,
                encode_optional_locale(&self.ui_locale_or_none),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U32,
                u32::from(self.subtitles_enabled).to_le_bytes().to_vec(),
            ),
        ])
    }

    fn compute_content_hash(&self) -> Result<ContentHash, PlayerPreferenceErrorV1> {
        let body = encode_canonical_segment(
            PLAYER_PREFERENCE_OWNER_ID,
            PLAYER_PREFERENCE_SCHEMA_ID,
            PLAYER_PREFERENCE_SEGMENT_ID,
            self.body_fields()?,
        )?;
        Ok(domain_hash(PLAYER_PREFERENCE_SEGMENT_ID, &body))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlayerPreferenceErrorV1 {
    Canonical(CanonicalError),
    Decode(crate::canonical::CanonicalDecodeError),
    Identifier(IdentifierError),
    Localization(crate::localization::TextCatalogErrorV1),
    InvalidRevision,
    TextScaleOutOfRange {
        actual: u32,
        minimum: u32,
        maximum: u32,
    },
    EnvelopeMismatch,
    UnsupportedVersion(u32),
    HashMismatch,
    NonCanonical,
    WrongFieldType(u32),
    InvalidPayload,
}

impl PlayerPreferenceErrorV1 {
    /// Stable diagnostic code required by SPEC-18 for every profile failure.
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        PLAYER_PREFERENCE_INVALID_CODE
    }
}

impl Display for PlayerPreferenceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "preference encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "preference decode failed: {error}"),
            Self::Identifier(error) => write!(formatter, "preference ID is invalid: {error}"),
            Self::Localization(error) => {
                write!(formatter, "preference locale is invalid: {error}")
            }
            Self::InvalidRevision => formatter.write_str("preference revision must be positive"),
            Self::TextScaleOutOfRange {
                actual,
                minimum,
                maximum,
            } => {
                write!(
                    formatter,
                    "preference text scale {actual} is outside {minimum}..={maximum}"
                )
            }
            Self::EnvelopeMismatch => formatter.write_str("preference envelope mismatch"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported preference version {version}")
            }
            Self::HashMismatch => formatter.write_str("preference content hash mismatch"),
            Self::NonCanonical => formatter.write_str("preference bytes are not canonical"),
            Self::WrongFieldType(field_id) => {
                write!(formatter, "preference field {field_id} has wrong type")
            }
            Self::InvalidPayload => formatter.write_str("preference payload is invalid"),
        }
    }
}

impl Error for PlayerPreferenceErrorV1 {}

impl From<CanonicalError> for PlayerPreferenceErrorV1 {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<crate::canonical::CanonicalDecodeError> for PlayerPreferenceErrorV1 {
    fn from(error: crate::canonical::CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for PlayerPreferenceErrorV1 {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::localization::TextCatalogErrorV1> for PlayerPreferenceErrorV1 {
    fn from(error: crate::localization::TextCatalogErrorV1) -> Self {
        Self::Localization(error)
    }
}

fn field(
    segment: &crate::canonical::DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&[u8], PlayerPreferenceErrorV1> {
    let field = segment
        .field(field_id)
        .ok_or(PlayerPreferenceErrorV1::InvalidPayload)?;
    if field.type_tag != expected_type {
        return Err(PlayerPreferenceErrorV1::WrongFieldType(field_id));
    }
    Ok(&field.payload)
}

fn read_u32(bytes: &[u8]) -> Result<u32, PlayerPreferenceErrorV1> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| PlayerPreferenceErrorV1::InvalidPayload)?,
    ))
}

fn read_fixed<const LENGTH: usize>(bytes: &[u8]) -> Result<[u8; LENGTH], PlayerPreferenceErrorV1> {
    bytes
        .try_into()
        .map_err(|_| PlayerPreferenceErrorV1::InvalidPayload)
}

fn encode_optional_locale(locale: &Option<TextLocaleTagV1>) -> Vec<u8> {
    match locale {
        None => vec![0],
        Some(tag) => {
            let mut bytes = Vec::with_capacity(1 + tag.as_str().len());
            bytes.push(1);
            bytes.extend_from_slice(tag.as_str().as_bytes());
            bytes
        }
    }
}

fn decode_optional_locale(
    bytes: &[u8],
) -> Result<Option<TextLocaleTagV1>, PlayerPreferenceErrorV1> {
    match bytes.split_first() {
        Some((0, [])) => Ok(None),
        Some((1, rest)) if !rest.is_empty() && rest.len() <= TEXT_CATALOG_MAX_LOCALE_TAG_BYTES => {
            let value =
                std::str::from_utf8(rest).map_err(|_| PlayerPreferenceErrorV1::InvalidPayload)?;
            Ok(Some(TextLocaleTagV1::new(value)?))
        }
        _ => Err(PlayerPreferenceErrorV1::InvalidPayload),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PLAYER_PREFERENCE_INVALID_CODE, PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT,
        PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX, PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN,
        PlayerPreferenceErrorV1, PlayerPreferenceProfileV1,
    };
    use crate::canonical::{CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalField};
    use crate::canonical::{decode_canonical_segment, encode_canonical_segment};
    use crate::localization::TextLocaleTagV1;

    fn sample_profile() -> PlayerPreferenceProfileV1 {
        PlayerPreferenceProfileV1::new(
            3,
            1_250,
            Some(TextLocaleTagV1::new("qps-ploc").expect("locale")),
            false,
        )
        .expect("profile")
    }

    #[test]
    fn diagnostic_code_is_stable() {
        assert_eq!(PLAYER_PREFERENCE_INVALID_CODE, "PLAYER_PREFERENCE_INVALID");
        assert_eq!(
            PlayerPreferenceErrorV1::InvalidPayload.diagnostic_code(),
            PLAYER_PREFERENCE_INVALID_CODE
        );
    }

    #[test]
    fn bounded_defaults_are_in_range_and_deterministic() {
        let defaults = PlayerPreferenceProfileV1::bounded_defaults();
        assert_eq!(defaults.revision, 1);
        assert_eq!(
            defaults.text_scale_milli,
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT
        );
        assert_eq!(defaults.ui_locale_or_none, None);
        assert!(defaults.subtitles_enabled);
        assert_eq!(
            defaults.content_hash,
            PlayerPreferenceProfileV1::bounded_defaults().content_hash
        );
    }

    #[test]
    fn text_scale_bounds_validate_fail_closed() {
        for valid in [
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN,
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT,
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX,
        ] {
            assert!(
                PlayerPreferenceProfileV1::new(1, valid, None, true).is_ok(),
                "valid scale rejected: {valid}"
            );
        }
        for invalid in [
            0,
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MIN - 1,
            PLAYER_PREFERENCE_TEXT_SCALE_MILLI_MAX + 1,
            u32::MAX,
        ] {
            assert!(
                matches!(
                    PlayerPreferenceProfileV1::new(1, invalid, None, true),
                    Err(PlayerPreferenceErrorV1::TextScaleOutOfRange { .. })
                ),
                "invalid scale accepted: {invalid}"
            );
        }
    }

    #[test]
    fn profile_round_trips_canonically() {
        let profile = sample_profile();
        let bytes = profile.canonical_bytes().expect("encode");
        let decoded = PlayerPreferenceProfileV1::from_canonical_bytes(
            &bytes,
            CanonicalDecodeLimits::default(),
        )
        .expect("decode");
        assert_eq!(decoded, profile);
        assert_eq!(decoded.canonical_bytes().expect("re-encode"), bytes);
    }

    #[test]
    fn profile_without_locale_round_trips() {
        let profile = PlayerPreferenceProfileV1::new(7, 800, None, false).expect("profile");
        let bytes = profile.canonical_bytes().expect("encode");
        assert_eq!(
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("decode"),
            profile
        );
    }

    #[test]
    fn subtitle_flag_round_trips_and_rejects_invalid_value() {
        for enabled in [false, true] {
            let profile = PlayerPreferenceProfileV1::new(2, 1_000, None, enabled).expect("profile");
            assert_eq!(profile.subtitles_enabled, enabled);
            let bytes = profile.canonical_bytes().expect("encode");
            assert_eq!(
                PlayerPreferenceProfileV1::from_canonical_bytes(
                    &bytes,
                    CanonicalDecodeLimits::default()
                )
                .expect("decode"),
                profile
            );
        }
        let profile = sample_profile();
        let bytes = profile.canonical_bytes().expect("encode");
        let segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");
        let mut bad_flag = segment.fields.clone();
        bad_flag[4] = CanonicalField::new(5, CANONICAL_TYPE_U32, 2_u32.to_le_bytes().to_vec());
        let bad_flag_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            bad_flag,
        )
        .expect("re-encode");
        assert_eq!(
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &bad_flag_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PlayerPreferenceErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn decoder_rejects_tampered_hash_and_unsupported_version() {
        let profile = sample_profile();
        let bytes = profile.canonical_bytes().expect("encode");
        let mut segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");

        let mut tampered_hash = segment.fields.clone();
        tampered_hash[5].payload[0] ^= 0xff;
        let tampered_hash_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            tampered_hash,
        )
        .expect("re-encode");
        assert_eq!(
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &tampered_hash_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PlayerPreferenceErrorV1::HashMismatch)
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
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &wrong_version_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PlayerPreferenceErrorV1::UnsupportedVersion(2))
        );
    }

    #[test]
    fn decoder_rejects_out_of_range_scale_and_bad_locale() {
        let profile = sample_profile();
        let bytes = profile.canonical_bytes().expect("encode");
        let segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");

        let mut bad_scale = segment.fields.clone();
        bad_scale[2] = CanonicalField::new(3, CANONICAL_TYPE_U32, 2_001_u32.to_le_bytes().to_vec());
        let bad_scale_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            bad_scale,
        )
        .expect("re-encode");
        assert!(matches!(
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &bad_scale_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PlayerPreferenceErrorV1::TextScaleOutOfRange { .. })
        ));

        let mut bad_locale = segment.fields.clone();
        bad_locale[3].payload = vec![1];
        bad_locale[3].payload.extend_from_slice(b"EN");
        let bad_locale_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            bad_locale,
        )
        .expect("re-encode");
        assert_eq!(
            PlayerPreferenceProfileV1::from_canonical_bytes(
                &bad_locale_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(PlayerPreferenceErrorV1::Localization(
                crate::localization::TextCatalogErrorV1::InvalidLocale
            ))
        );
    }
}
