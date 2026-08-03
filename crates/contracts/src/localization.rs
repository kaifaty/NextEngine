//! Typed neutral text catalog contract for UI localization (ADR-044, SPEC-24).
//!
//! A `TextCatalogV1` carries text templates for exactly one locale. Catalogs
//! declare at most one fallback locale; the project-level closure (exactly one
//! source-locale root, unique locales, acyclic chains) is validated by the
//! cook. Resolution is deterministic: first hit along the fallback chain wins
//! and a full miss yields the stable `LOCALIZATION_RESOURCE_MISSING` diagnostic
//! plus a `[{text_id}]` placeholder instead of failing presentation.

use std::error::Error;
use std::fmt::{Display, Formatter};

use unicode_normalization::UnicodeNormalization;

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, decode_canonical_segment,
    encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, IdentifierError, SchemaId};
use crate::project::domain_hash;

pub const TEXT_CATALOG_SCHEMA_ID: &str = "nextengine.content.text-catalog";
pub const TEXT_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const TEXT_CATALOG_SEGMENT_ID: &str = "nextengine.text-catalog.v1";
pub const TEXT_CATALOG_SCHEMA_VERSION: u32 = 1;
pub const TEXT_CATALOG_MAX_ENTRIES: usize = 4_096;
pub const TEXT_CATALOG_MAX_TEMPLATE_BYTES: usize = 1_024;
pub const TEXT_CATALOG_MAX_LOCALE_TAG_BYTES: usize = 35;
pub const TEXT_CATALOG_MAX_PLACEHOLDER_INDEX: u8 = 15;
pub const LOCALIZATION_RESOURCE_MISSING_CODE: &str = "LOCALIZATION_RESOURCE_MISSING";

/// BCP-47-shaped lowercase ASCII locale tag: an alpha primary subtag of 2-8
/// characters followed by alnum subtags of 1-8 characters (`en`, `qps-ploc`).
/// Validation is fail-closed: wrong case or shape is rejected, never
/// normalized.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TextLocaleTagV1(String);

impl TextLocaleTagV1 {
    pub fn new(value: &str) -> Result<Self, TextCatalogErrorV1> {
        if value.is_empty() || value.len() > TEXT_CATALOG_MAX_LOCALE_TAG_BYTES || !value.is_ascii()
        {
            return Err(TextCatalogErrorV1::InvalidLocale);
        }
        let mut subtags = value.split('-');
        let primary = subtags.next().ok_or(TextCatalogErrorV1::InvalidLocale)?;
        if primary.len() < 2
            || primary.len() > 8
            || !primary.bytes().all(|byte| byte.is_ascii_lowercase())
        {
            return Err(TextCatalogErrorV1::InvalidLocale);
        }
        for subtag in subtags {
            if subtag.is_empty()
                || subtag.len() > 8
                || !subtag
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            {
                return Err(TextCatalogErrorV1::InvalidLocale);
            }
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One text entry: a stable text ID bound to an NFC template with positional
/// placeholders `{0}`..`{15}`. Braces have no escape form in v1.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TextCatalogEntryV1 {
    pub text_id: SchemaId,
    pub template: String,
}

impl TextCatalogEntryV1 {
    pub fn new(text_id: SchemaId, template: String) -> Result<Self, TextCatalogErrorV1> {
        validate_template(&template)?;
        Ok(Self { text_id, template })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextCatalogV1 {
    pub schema_version: u32,
    pub catalog_asset_id: AssetId,
    pub revision: u32,
    pub locale: TextLocaleTagV1,
    pub fallback_locale_or_none: Option<TextLocaleTagV1>,
    pub entries: Vec<TextCatalogEntryV1>,
    pub content_hash: ContentHash,
}

impl TextCatalogV1 {
    pub fn new(
        catalog_asset_id: AssetId,
        revision: u32,
        locale: TextLocaleTagV1,
        fallback_locale_or_none: Option<TextLocaleTagV1>,
        mut entries: Vec<TextCatalogEntryV1>,
    ) -> Result<Self, TextCatalogErrorV1> {
        if revision == 0 {
            return Err(TextCatalogErrorV1::InvalidRevision);
        }
        if fallback_locale_or_none.as_ref() == Some(&locale) {
            return Err(TextCatalogErrorV1::InvalidLocale);
        }
        if entries.is_empty() {
            return Err(TextCatalogErrorV1::EmptyEntries);
        }
        if entries.len() > TEXT_CATALOG_MAX_ENTRIES {
            return Err(TextCatalogErrorV1::LimitExceeded {
                actual: entries.len(),
                limit: TEXT_CATALOG_MAX_ENTRIES,
            });
        }
        entries.sort();
        if entries
            .windows(2)
            .any(|pair| pair[0].text_id == pair[1].text_id)
        {
            return Err(TextCatalogErrorV1::DuplicateKey);
        }
        let mut catalog = Self {
            schema_version: TEXT_CATALOG_SCHEMA_VERSION,
            catalog_asset_id,
            revision,
            locale,
            fallback_locale_or_none,
            entries,
            content_hash: ContentHash::from_bytes([0; 32]),
        };
        catalog.content_hash = catalog.compute_content_hash()?;
        Ok(catalog)
    }

    /// Looks up an entry by text ID. Entries are sorted by text ID.
    #[must_use]
    pub fn entry(&self, text_id: &SchemaId) -> Option<&TextCatalogEntryV1> {
        self.entries.iter().find(|entry| &entry.text_id == text_id)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, TextCatalogErrorV1> {
        let mut fields = self.body_fields()?;
        fields.push(CanonicalField::new(
            7,
            CANONICAL_TYPE_HASH256,
            self.content_hash.as_bytes().to_vec(),
        ));
        Ok(encode_canonical_segment(
            TEXT_CATALOG_OWNER_ID,
            TEXT_CATALOG_SCHEMA_ID,
            TEXT_CATALOG_SEGMENT_ID,
            fields,
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, TextCatalogErrorV1> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != TEXT_CATALOG_OWNER_ID
            || segment.schema_id != TEXT_CATALOG_SCHEMA_ID
            || segment.segment_id != TEXT_CATALOG_SEGMENT_ID
            || segment.fields.len() != 7
        {
            return Err(TextCatalogErrorV1::EnvelopeMismatch);
        }
        let version = read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?;
        if version != TEXT_CATALOG_SCHEMA_VERSION {
            return Err(TextCatalogErrorV1::UnsupportedVersion(version));
        }
        let catalog_asset_id =
            AssetId::from_bytes(read_fixed(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let revision = read_u32(field(&segment, 3, CANONICAL_TYPE_U32)?)?;
        let locale = TextLocaleTagV1::new(read_text(
            field(&segment, 4, CANONICAL_TYPE_UTF8_NFC)?,
            TEXT_CATALOG_MAX_LOCALE_TAG_BYTES,
        )?)?;
        let fallback_locale_or_none =
            decode_optional_locale(field(&segment, 5, CANONICAL_TYPE_OPTIONAL)?)?;
        let entries = decode_entries(field(&segment, 6, CANONICAL_TYPE_SEQUENCE)?, limits)?;
        let content_hash =
            ContentHash::from_bytes(read_fixed(field(&segment, 7, CANONICAL_TYPE_HASH256)?)?);
        let catalog = Self::new(
            catalog_asset_id,
            revision,
            locale,
            fallback_locale_or_none,
            entries,
        )?;
        if catalog.content_hash != content_hash {
            return Err(TextCatalogErrorV1::HashMismatch);
        }
        if catalog.canonical_bytes()? != bytes {
            return Err(TextCatalogErrorV1::NonCanonical);
        }
        Ok(catalog)
    }

    /// Hash bound into cooked blobs and the project manifest for this record.
    pub fn record_sha256(&self) -> Result<ContentHash, TextCatalogErrorV1> {
        Ok(domain_hash(
            TEXT_CATALOG_SEGMENT_ID,
            &self.canonical_bytes()?,
        ))
    }

    fn body_fields(&self) -> Result<Vec<CanonicalField>, TextCatalogErrorV1> {
        Ok(vec![
            CanonicalField::new(
                1,
                CANONICAL_TYPE_U32,
                self.schema_version.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_ID128,
                self.catalog_asset_id.as_bytes().to_vec(),
            ),
            CanonicalField::new(3, CANONICAL_TYPE_U32, self.revision.to_le_bytes().to_vec()),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_UTF8_NFC,
                self.locale.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_OPTIONAL,
                encode_optional_locale(&self.fallback_locale_or_none),
            ),
            CanonicalField::new(6, CANONICAL_TYPE_SEQUENCE, encode_entries(&self.entries)?),
        ])
    }

    fn compute_content_hash(&self) -> Result<ContentHash, TextCatalogErrorV1> {
        let body = encode_canonical_segment(
            TEXT_CATALOG_OWNER_ID,
            TEXT_CATALOG_SCHEMA_ID,
            TEXT_CATALOG_SEGMENT_ID,
            self.body_fields()?,
        )?;
        Ok(domain_hash(TEXT_CATALOG_SEGMENT_ID, &body))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TextCatalogErrorV1 {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    InvalidLocale,
    InvalidTemplate,
    InvalidRevision,
    EmptyEntries,
    DuplicateKey,
    EnvelopeMismatch,
    UnsupportedVersion(u32),
    HashMismatch,
    NonCanonical,
    LimitExceeded { actual: usize, limit: usize },
    WrongFieldType(u32),
    InvalidPayload,
}

impl Display for TextCatalogErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "text catalog encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "text catalog decode failed: {error}"),
            Self::Identifier(error) => write!(formatter, "text catalog ID is invalid: {error}"),
            Self::InvalidLocale => formatter.write_str("text catalog locale tag is invalid"),
            Self::InvalidTemplate => formatter.write_str("text catalog template is invalid"),
            Self::InvalidRevision => formatter.write_str("text catalog revision must be positive"),
            Self::EmptyEntries => formatter.write_str("text catalog must contain entries"),
            Self::DuplicateKey => formatter.write_str("duplicate text ID in text catalog"),
            Self::EnvelopeMismatch => formatter.write_str("text catalog envelope mismatch"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported text catalog version {version}")
            }
            Self::HashMismatch => formatter.write_str("text catalog content hash mismatch"),
            Self::NonCanonical => formatter.write_str("text catalog bytes are not canonical"),
            Self::LimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "text catalog count {actual} exceeds limit {limit}"
                )
            }
            Self::WrongFieldType(field_id) => {
                write!(formatter, "text catalog field {field_id} has wrong type")
            }
            Self::InvalidPayload => formatter.write_str("text catalog payload is invalid"),
        }
    }
}

impl Error for TextCatalogErrorV1 {}

impl From<CanonicalError> for TextCatalogErrorV1 {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for TextCatalogErrorV1 {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for TextCatalogErrorV1 {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn validate_template(template: &str) -> Result<(), TextCatalogErrorV1> {
    if template.is_empty() || template.len() > TEXT_CATALOG_MAX_TEMPLATE_BYTES {
        return Err(TextCatalogErrorV1::InvalidTemplate);
    }
    if !template.nfc().eq(template.chars()) {
        return Err(TextCatalogErrorV1::InvalidTemplate);
    }
    let bytes = template.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                index += 1;
                let mut digits = 0_usize;
                let mut value = 0_u32;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    value = value * 10 + u32::from(bytes[index] - b'0');
                    digits += 1;
                    index += 1;
                }
                if digits == 0
                    || digits > 2
                    || index >= bytes.len()
                    || bytes[index] != b'}'
                    || value > u32::from(TEXT_CATALOG_MAX_PLACEHOLDER_INDEX)
                {
                    return Err(TextCatalogErrorV1::InvalidTemplate);
                }
                index += 1;
            }
            b'}' => return Err(TextCatalogErrorV1::InvalidTemplate),
            _ => index += 1,
        }
    }
    Ok(())
}

fn field(
    segment: &crate::canonical::DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&[u8], TextCatalogErrorV1> {
    let field = segment
        .field(field_id)
        .ok_or(TextCatalogErrorV1::InvalidPayload)?;
    if field.type_tag != expected_type {
        return Err(TextCatalogErrorV1::WrongFieldType(field_id));
    }
    Ok(&field.payload)
}

fn read_u32(bytes: &[u8]) -> Result<u32, TextCatalogErrorV1> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| TextCatalogErrorV1::InvalidPayload)?,
    ))
}

fn read_fixed<const LENGTH: usize>(bytes: &[u8]) -> Result<[u8; LENGTH], TextCatalogErrorV1> {
    bytes
        .try_into()
        .map_err(|_| TextCatalogErrorV1::InvalidPayload)
}

fn read_text(bytes: &[u8], limit: usize) -> Result<&str, TextCatalogErrorV1> {
    if bytes.len() > limit {
        return Err(TextCatalogErrorV1::LimitExceeded {
            actual: bytes.len(),
            limit,
        });
    }
    std::str::from_utf8(bytes).map_err(|_| TextCatalogErrorV1::InvalidPayload)
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

fn decode_optional_locale(bytes: &[u8]) -> Result<Option<TextLocaleTagV1>, TextCatalogErrorV1> {
    match bytes.split_first() {
        Some((0, [])) => Ok(None),
        Some((1, rest)) if !rest.is_empty() => {
            let value =
                std::str::from_utf8(rest).map_err(|_| TextCatalogErrorV1::InvalidPayload)?;
            Ok(Some(TextLocaleTagV1::new(value)?))
        }
        _ => Err(TextCatalogErrorV1::InvalidPayload),
    }
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), TextCatalogErrorV1> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_entries(entries: &[TextCatalogEntryV1]) -> Result<Vec<u8>, TextCatalogErrorV1> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for entry in entries {
        extend_text(&mut bytes, entry.text_id.as_str())?;
        extend_text(&mut bytes, &entry.template)?;
    }
    Ok(bytes)
}

fn decode_entries(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<TextCatalogEntryV1>, TextCatalogErrorV1> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| TextCatalogErrorV1::InvalidPayload)?;
    if count > limits.max_sequence_items {
        return Err(TextCatalogErrorV1::LimitExceeded {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let text_id = read_cursor_text(&mut cursor, limits.max_identifier_bytes)?;
        let template = read_cursor_text(&mut cursor, TEXT_CATALOG_MAX_TEMPLATE_BYTES)?;
        entries.push(TextCatalogEntryV1::new(SchemaId::new(text_id)?, template)?);
    }
    cursor.finish()?;
    Ok(entries)
}

fn read_cursor_text(
    cursor: &mut CanonicalCursor<'_>,
    limit: usize,
) -> Result<String, TextCatalogErrorV1> {
    let length =
        usize::try_from(cursor.read_u32()?).map_err(|_| TextCatalogErrorV1::InvalidPayload)?;
    if length > limit {
        return Err(TextCatalogErrorV1::LimitExceeded {
            actual: length,
            limit,
        });
    }
    let bytes = cursor.read_exact(length)?;
    Ok(std::str::from_utf8(bytes)
        .map_err(|_| TextCatalogErrorV1::InvalidPayload)?
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        LOCALIZATION_RESOURCE_MISSING_CODE, TEXT_CATALOG_MAX_LOCALE_TAG_BYTES,
        TEXT_CATALOG_MAX_PLACEHOLDER_INDEX, TextCatalogEntryV1, TextCatalogErrorV1, TextCatalogV1,
        TextLocaleTagV1,
    };
    use crate::canonical::{
        CANONICAL_TYPE_U32, CanonicalDecodeLimits, CanonicalField, decode_canonical_segment,
        encode_canonical_segment,
    };
    use crate::ids::{AssetId, SchemaId};

    fn text_id(name: &str) -> SchemaId {
        SchemaId::new(format!("nextengine.ui.text.{name}")).expect("text ID")
    }

    fn entry(name: &str, template: &str) -> TextCatalogEntryV1 {
        TextCatalogEntryV1::new(text_id(name), template.to_owned()).expect("entry")
    }

    fn sample_catalog() -> TextCatalogV1 {
        TextCatalogV1::new(
            AssetId::from_bytes([0x91; 16]),
            1,
            TextLocaleTagV1::new("en").expect("locale"),
            None,
            vec![
                entry("hud.quest-state", "Quest: {0}"),
                entry("hud.health", "Health {0}/{1}"),
                entry("menu.title", "Paused"),
            ],
        )
        .expect("catalog")
    }

    #[test]
    fn diagnostic_code_is_stable() {
        assert_eq!(
            LOCALIZATION_RESOURCE_MISSING_CODE,
            "LOCALIZATION_RESOURCE_MISSING"
        );
        assert_eq!(TEXT_CATALOG_MAX_PLACEHOLDER_INDEX, 15);
    }

    #[test]
    fn locale_tags_validate_fail_closed() {
        for valid in ["en", "qps-ploc", "zh-hans-cn", "en-us", "abcdefgh-x9"] {
            assert!(
                TextLocaleTagV1::new(valid).is_ok(),
                "valid tag rejected: {valid}"
            );
        }
        for invalid in [
            "",
            "EN",
            "En",
            "-en",
            "en-",
            "en--us",
            "e",
            "a-bc",
            "123",
            "en_us",
            "en-Ü",
            "abcdefghij",
        ] {
            assert_eq!(
                TextLocaleTagV1::new(invalid),
                Err(TextCatalogErrorV1::InvalidLocale),
                "invalid tag accepted: {invalid}"
            );
        }
        let too_long = "a".repeat(TEXT_CATALOG_MAX_LOCALE_TAG_BYTES + 1);
        assert_eq!(
            TextLocaleTagV1::new(&too_long),
            Err(TextCatalogErrorV1::InvalidLocale)
        );
    }

    #[test]
    fn templates_validate_placeholders_fail_closed() {
        for valid in [
            "Health {0}/{1}",
            "{15}",
            "plain text",
            "Élan vital {0}",
            "{0}{1}{2}",
        ] {
            assert!(
                TextCatalogEntryV1::new(text_id("ok"), valid.to_owned()).is_ok(),
                "valid template rejected: {valid}"
            );
        }
        for invalid in [
            "", "{16}", "{x}", "{", "}", "{}", "{0", "0}", "{007}", "{{0}}", "e\u{301}",
        ] {
            assert_eq!(
                TextCatalogEntryV1::new(text_id("bad"), invalid.to_owned()),
                Err(TextCatalogErrorV1::InvalidTemplate),
                "invalid template accepted: {invalid}"
            );
        }
    }

    #[test]
    fn catalog_sorts_entries_and_round_trips_canonically() {
        let catalog = sample_catalog();
        assert_eq!(catalog.entries[0].text_id, text_id("hud.health"));
        assert_eq!(catalog.entries[2].text_id, text_id("menu.title"));
        let bytes = catalog.canonical_bytes().expect("encode");
        let decoded = TextCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode");
        assert_eq!(decoded, catalog);
        assert_eq!(
            decoded.record_sha256().expect("hash"),
            catalog.record_sha256().expect("hash")
        );
    }

    #[test]
    fn catalog_with_fallback_round_trips() {
        let catalog = TextCatalogV1::new(
            AssetId::from_bytes([0x92; 16]),
            3,
            TextLocaleTagV1::new("qps-ploc").expect("locale"),
            Some(TextLocaleTagV1::new("en").expect("fallback")),
            vec![entry("hud.health", "⟦Ħēåłŧħ⟧ {0}/{1}")],
        )
        .expect("catalog");
        let bytes = catalog.canonical_bytes().expect("encode");
        assert_eq!(
            TextCatalogV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode"),
            catalog
        );
    }

    #[test]
    fn catalog_rejects_duplicate_ids_zero_revision_and_self_fallback() {
        let locale = TextLocaleTagV1::new("en").expect("locale");
        assert_eq!(
            TextCatalogV1::new(
                AssetId::from_bytes([1; 16]),
                1,
                locale.clone(),
                None,
                vec![entry("a", "A"), entry("a", "B")],
            ),
            Err(TextCatalogErrorV1::DuplicateKey)
        );
        assert_eq!(
            TextCatalogV1::new(
                AssetId::from_bytes([1; 16]),
                0,
                locale.clone(),
                None,
                vec![entry("a", "A")],
            ),
            Err(TextCatalogErrorV1::InvalidRevision)
        );
        assert_eq!(
            TextCatalogV1::new(
                AssetId::from_bytes([1; 16]),
                1,
                locale.clone(),
                Some(locale.clone()),
                vec![entry("a", "A")],
            ),
            Err(TextCatalogErrorV1::InvalidLocale)
        );
        assert_eq!(
            TextCatalogV1::new(AssetId::from_bytes([1; 16]), 1, locale, None, Vec::new()),
            Err(TextCatalogErrorV1::EmptyEntries)
        );
    }

    #[test]
    fn decoder_rejects_tampered_hash_and_unsupported_version() {
        let catalog = sample_catalog();
        let bytes = catalog.canonical_bytes().expect("encode");
        let mut segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");

        let mut tampered_hash = segment.fields.clone();
        tampered_hash[6].payload[0] ^= 0xff;
        let tampered_hash_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            tampered_hash,
        )
        .expect("re-encode");
        assert_eq!(
            TextCatalogV1::from_canonical_bytes(
                &tampered_hash_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(TextCatalogErrorV1::HashMismatch)
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
            TextCatalogV1::from_canonical_bytes(
                &wrong_version_bytes,
                CanonicalDecodeLimits::default()
            ),
            Err(TextCatalogErrorV1::UnsupportedVersion(2))
        );
    }

    #[test]
    fn decoder_rejects_unsorted_entries_as_non_canonical() {
        let catalog = sample_catalog();
        let bytes = catalog.canonical_bytes().expect("encode");
        let segment =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("segment");
        let reversed: Vec<_> = catalog.entries.iter().rev().cloned().collect();
        let mut unsorted_entries = Vec::new();
        unsorted_entries.extend_from_slice(&(reversed.len() as u32).to_le_bytes());
        for entry in &reversed {
            for value in [entry.text_id.as_str(), entry.template.as_str()] {
                unsorted_entries.extend_from_slice(&(value.len() as u32).to_le_bytes());
                unsorted_entries.extend_from_slice(value.as_bytes());
            }
        }
        let mut fields = segment.fields.clone();
        fields[5].payload = unsorted_entries;
        let unsorted_bytes = encode_canonical_segment(
            &segment.owner_id,
            &segment.schema_id,
            &segment.segment_id,
            fields,
        )
        .expect("re-encode");
        assert_eq!(
            TextCatalogV1::from_canonical_bytes(&unsorted_bytes, CanonicalDecodeLimits::default()),
            Err(TextCatalogErrorV1::NonCanonical)
        );
    }
}
