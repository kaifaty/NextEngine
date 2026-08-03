//! Deterministic UI text resolution over cooked text catalogs (ADR-044).
//!
//! The resolver walks the declared fallback chain of the requested locale and
//! takes the first entry hit. Any miss — unknown text ID, missing argument,
//! missing locale, or a TextId recursion violation — yields the stable
//! `LOCALIZATION_RESOURCE_MISSING` diagnostic plus a readable placeholder
//! instead of failing presentation. Resolution is pure: the same catalog set
//! and text reference always produce the same text and diagnostic.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::ids::SchemaId;
use next_contracts::localization::{
    LOCALIZATION_RESOURCE_MISSING_CODE, TextCatalogV1, TextLocaleTagV1,
};
use next_contracts::presentation::{UiTextArgumentV1, UiTextRefV1};

pub const TEXT_RESOLUTION_MAX_DEPTH: u8 = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalizationDiagnosticV1 {
    pub code: &'static str,
    pub text_id: SchemaId,
}

impl LocalizationDiagnosticV1 {
    fn missing(text_id: &SchemaId) -> Self {
        Self {
            code: LOCALIZATION_RESOURCE_MISSING_CODE,
            text_id: text_id.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextResolutionV1 {
    pub text: String,
    pub diagnostic_or_none: Option<LocalizationDiagnosticV1>,
}

/// Resolves `UiTextRefV1` references against an immutable catalog set.
///
/// Construction re-validates the project-level fallback closure (unique
/// locales, exactly one source-locale root, existing fallback targets,
/// acyclic chains) as defense in depth: catalogs arriving outside the cook
/// cannot introduce a non-deterministic resolution order.
#[derive(Clone, Debug)]
pub struct TextCatalogResolverV1 {
    catalogs: Vec<TextCatalogV1>,
    requested_locale: TextLocaleTagV1,
    chain: Vec<usize>,
    requested_locale_missing: bool,
}

impl TextCatalogResolverV1 {
    pub fn new(
        catalogs: Vec<TextCatalogV1>,
        requested_locale: &str,
    ) -> Result<Self, TextResolverErrorV1> {
        let requested_locale = TextLocaleTagV1::new(requested_locale)
            .map_err(|_| TextResolverErrorV1::InvalidLocale)?;
        let mut locales = BTreeSet::new();
        let mut root_index_or_none = None;
        for (index, catalog) in catalogs.iter().enumerate() {
            if !locales.insert(catalog.locale.as_str()) {
                return Err(TextResolverErrorV1::ClosureInvalid);
            }
            if catalog.fallback_locale_or_none.is_none()
                && root_index_or_none.replace(index).is_some()
            {
                return Err(TextResolverErrorV1::ClosureInvalid);
            }
        }
        let root_index = root_index_or_none.ok_or(TextResolverErrorV1::ClosureInvalid)?;
        for catalog in &catalogs {
            let mut visited = BTreeSet::new();
            let mut current = catalog;
            loop {
                if !visited.insert(current.locale.as_str()) {
                    return Err(TextResolverErrorV1::ClosureInvalid);
                }
                let Some(fallback) = &current.fallback_locale_or_none else {
                    break;
                };
                current = catalogs
                    .iter()
                    .find(|candidate| candidate.locale.as_str() == fallback.as_str())
                    .ok_or(TextResolverErrorV1::ClosureInvalid)?;
            }
        }
        let start = catalogs
            .iter()
            .position(|catalog| catalog.locale == requested_locale)
            .unwrap_or(root_index);
        let requested_locale_missing = catalogs[start].locale != requested_locale;
        let mut chain = Vec::new();
        let mut current = start;
        loop {
            chain.push(current);
            let Some(fallback) = &catalogs[current].fallback_locale_or_none else {
                break;
            };
            current = catalogs
                .iter()
                .position(|candidate| candidate.locale.as_str() == fallback.as_str())
                .ok_or(TextResolverErrorV1::ClosureInvalid)?;
        }
        Ok(Self {
            catalogs,
            requested_locale,
            chain,
            requested_locale_missing,
        })
    }

    #[must_use]
    pub const fn requested_locale(&self) -> &TextLocaleTagV1 {
        &self.requested_locale
    }

    /// True when no catalog declares the requested locale; resolution then
    /// starts at the source-locale root instead of failing.
    #[must_use]
    pub const fn requested_locale_missing(&self) -> bool {
        self.requested_locale_missing
    }

    #[must_use]
    pub fn resolve(&self, text_ref: &UiTextRefV1) -> TextResolutionV1 {
        let mut visited = BTreeSet::new();
        let (text, diagnostic_or_none) =
            self.resolve_inner(&text_ref.text_id, &text_ref.arguments, 0, &mut visited);
        TextResolutionV1 {
            text,
            diagnostic_or_none,
        }
    }

    fn find_template(&self, text_id: &SchemaId) -> Option<&str> {
        self.chain.iter().find_map(|index| {
            self.catalogs[*index]
                .entry(text_id)
                .map(|entry| entry.template.as_str())
        })
    }

    fn resolve_inner(
        &self,
        text_id: &SchemaId,
        arguments: &[UiTextArgumentV1],
        depth: u8,
        visited: &mut BTreeSet<SchemaId>,
    ) -> (String, Option<LocalizationDiagnosticV1>) {
        if depth >= TEXT_RESOLUTION_MAX_DEPTH || visited.contains(text_id) {
            return (
                format!("[{}]", text_id.as_str()),
                Some(LocalizationDiagnosticV1::missing(text_id)),
            );
        }
        let Some(template) = self.find_template(text_id) else {
            return (
                format!("[{}]", text_id.as_str()),
                Some(LocalizationDiagnosticV1::missing(text_id)),
            );
        };
        visited.insert(text_id.clone());
        let mut text = String::with_capacity(template.len());
        let mut diagnostic_or_none = None;
        let bytes = template.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'{' {
                // The catalog contract guarantees 1-2 digits closed by `}`.
                let mut cursor = index + 1;
                let mut argument_index = 0_usize;
                while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                    argument_index = argument_index * 10 + usize::from(bytes[cursor] - b'0');
                    cursor += 1;
                }
                match arguments.get(argument_index) {
                    Some(UiTextArgumentV1::SignedInteger(value)) => {
                        text.push_str(&value.to_string());
                    }
                    Some(UiTextArgumentV1::TextId(nested_id)) => {
                        let (nested, nested_diagnostic) =
                            self.resolve_inner(nested_id, &[], depth + 1, visited);
                        text.push_str(&nested);
                        if diagnostic_or_none.is_none() {
                            diagnostic_or_none = nested_diagnostic;
                        }
                    }
                    None => {
                        text.push_str(&template[index..=cursor]);
                        if diagnostic_or_none.is_none() {
                            diagnostic_or_none = Some(LocalizationDiagnosticV1::missing(text_id));
                        }
                    }
                }
                index = cursor + 1;
            } else {
                let next = template[index..]
                    .find('{')
                    .map_or(bytes.len(), |offset| index + offset);
                text.push_str(&template[index..next]);
                index = next;
            }
        }
        visited.remove(text_id);
        (text, diagnostic_or_none)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TextResolverErrorV1 {
    InvalidLocale,
    ClosureInvalid,
}

impl Display for TextResolverErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLocale => formatter.write_str("requested text locale tag is invalid"),
            Self::ClosureInvalid => formatter.write_str("text catalog fallback closure is invalid"),
        }
    }
}

impl Error for TextResolverErrorV1 {}

#[cfg(test)]
mod tests {
    use super::{TextCatalogResolverV1, TextResolverErrorV1};
    use next_contracts::ids::{AssetId, SchemaId};
    use next_contracts::localization::{
        LOCALIZATION_RESOURCE_MISSING_CODE, TextCatalogEntryV1, TextCatalogV1, TextLocaleTagV1,
    };
    use next_contracts::presentation::{UiTextArgumentV1, UiTextRefV1};

    fn text_id(name: &str) -> SchemaId {
        SchemaId::new(format!("nextengine.test.text.{name}")).expect("text ID")
    }

    fn catalog(
        byte: u8,
        locale: &str,
        fallback: Option<&str>,
        entries: &[(&str, &str)],
    ) -> TextCatalogV1 {
        TextCatalogV1::new(
            AssetId::from_bytes([byte; 16]),
            1,
            TextLocaleTagV1::new(locale).expect("locale"),
            fallback.map(|value| TextLocaleTagV1::new(value).expect("fallback")),
            entries
                .iter()
                .map(|(name, template)| {
                    TextCatalogEntryV1::new(text_id(name), (*template).to_owned()).expect("entry")
                })
                .collect(),
        )
        .expect("catalog")
    }

    fn reference_catalogs() -> Vec<TextCatalogV1> {
        vec![
            catalog(
                0x91,
                "en",
                None,
                &[
                    ("greeting", "Hello {0}"),
                    ("farewell", "Bye"),
                    ("health", "Health {0}/{1}"),
                    ("quest", "Quest: {0}"),
                    ("state-on", "On"),
                    ("loop", "{0}"),
                ],
            ),
            catalog(0x92, "qps-ploc", Some("en"), &[("greeting", "⟦Ħēłłö⟧ {0}")]),
        ]
    }

    fn text_ref(name: &str, arguments: Vec<UiTextArgumentV1>) -> UiTextRefV1 {
        UiTextRefV1::new(text_id(name), arguments).expect("text ref")
    }

    #[test]
    fn resolves_requested_locale_and_signed_integer_arguments() {
        let resolver =
            TextCatalogResolverV1::new(reference_catalogs(), "qps-ploc").expect("resolver");
        assert!(!resolver.requested_locale_missing());
        let resolution = resolver.resolve(&text_ref(
            "greeting",
            vec![UiTextArgumentV1::SignedInteger(-3)],
        ));
        assert_eq!(resolution.text, "⟦Ħēłłö⟧ -3");
        assert_eq!(resolution.diagnostic_or_none, None);

        let en = TextCatalogResolverV1::new(reference_catalogs(), "en").expect("resolver");
        let resolution = en.resolve(&text_ref(
            "health",
            vec![
                UiTextArgumentV1::SignedInteger(37),
                UiTextArgumentV1::SignedInteger(100),
            ],
        ));
        assert_eq!(resolution.text, "Health 37/100");
        assert_eq!(resolution.diagnostic_or_none, None);
    }

    #[test]
    fn falls_back_along_the_declared_chain() {
        let resolver =
            TextCatalogResolverV1::new(reference_catalogs(), "qps-ploc").expect("resolver");
        let resolution = resolver.resolve(&text_ref("farewell", Vec::new()));
        assert_eq!(resolution.text, "Bye");
        assert_eq!(resolution.diagnostic_or_none, None);
    }

    #[test]
    fn missing_everywhere_yields_placeholder_and_stable_diagnostic() {
        let resolver = TextCatalogResolverV1::new(reference_catalogs(), "en").expect("resolver");
        let resolution = resolver.resolve(&text_ref("unknown", Vec::new()));
        assert_eq!(resolution.text, "[nextengine.test.text.unknown]");
        let diagnostic = resolution.diagnostic_or_none.expect("diagnostic");
        assert_eq!(diagnostic.code, LOCALIZATION_RESOURCE_MISSING_CODE);
        assert_eq!(diagnostic.text_id, text_id("unknown"));
    }

    #[test]
    fn missing_argument_keeps_placeholder_and_diagnoses() {
        let resolver = TextCatalogResolverV1::new(reference_catalogs(), "en").expect("resolver");
        let resolution = resolver.resolve(&text_ref(
            "health",
            vec![UiTextArgumentV1::SignedInteger(5)],
        ));
        assert_eq!(resolution.text, "Health 5/{1}");
        assert_eq!(
            resolution.diagnostic_or_none.expect("diagnostic").code,
            LOCALIZATION_RESOURCE_MISSING_CODE
        );
    }

    #[test]
    fn text_id_arguments_resolve_recursively_with_cycle_protection() {
        let resolver = TextCatalogResolverV1::new(reference_catalogs(), "en").expect("resolver");
        let resolution = resolver.resolve(&text_ref(
            "quest",
            vec![UiTextArgumentV1::TextId(text_id("state-on"))],
        ));
        assert_eq!(resolution.text, "Quest: On");
        assert_eq!(resolution.diagnostic_or_none, None);

        let resolution = resolver.resolve(&text_ref(
            "quest",
            vec![UiTextArgumentV1::TextId(text_id("unknown"))],
        ));
        assert_eq!(resolution.text, "Quest: [nextengine.test.text.unknown]");
        assert!(resolution.diagnostic_or_none.is_some());

        let resolution = resolver.resolve(&text_ref(
            "loop",
            vec![UiTextArgumentV1::TextId(text_id("loop"))],
        ));
        assert_eq!(resolution.text, "[nextengine.test.text.loop]");
        assert!(resolution.diagnostic_or_none.is_some());
    }

    #[test]
    fn missing_requested_locale_resolves_from_source_root() {
        let resolver = TextCatalogResolverV1::new(reference_catalogs(), "de").expect("resolver");
        assert!(resolver.requested_locale_missing());
        let resolution = resolver.resolve(&text_ref(
            "greeting",
            vec![UiTextArgumentV1::SignedInteger(7)],
        ));
        assert_eq!(resolution.text, "Hello 7");
        assert_eq!(resolution.diagnostic_or_none, None);
    }

    #[test]
    fn invalid_closure_and_locale_are_rejected_at_construction() {
        assert!(matches!(
            TextCatalogResolverV1::new(reference_catalogs(), "EN"),
            Err(TextResolverErrorV1::InvalidLocale)
        ));
        assert!(matches!(
            TextCatalogResolverV1::new(Vec::new(), "en"),
            Err(TextResolverErrorV1::ClosureInvalid)
        ));
        let duplicate_locale = vec![
            catalog(0x91, "en", None, &[("greeting", "Hello {0}")]),
            catalog(0x92, "en", None, &[("farewell", "Bye")]),
        ];
        assert!(matches!(
            TextCatalogResolverV1::new(duplicate_locale, "en"),
            Err(TextResolverErrorV1::ClosureInvalid)
        ));
        let no_root = vec![
            catalog(0x91, "en", Some("de"), &[("greeting", "Hello {0}")]),
            catalog(0x92, "de", Some("en"), &[("greeting", "Hallo {0}")]),
        ];
        assert!(matches!(
            TextCatalogResolverV1::new(no_root, "en"),
            Err(TextResolverErrorV1::ClosureInvalid)
        ));
        let missing_fallback = vec![
            catalog(0x91, "en", None, &[("greeting", "Hello {0}")]),
            catalog(0x92, "de", Some("fr"), &[("greeting", "Hallo {0}")]),
        ];
        assert!(matches!(
            TextCatalogResolverV1::new(missing_fallback, "de"),
            Err(TextResolverErrorV1::ClosureInvalid)
        ));
    }
}
