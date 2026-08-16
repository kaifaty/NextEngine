//! Shared cook-time helpers: schema reference derivation, uniqueness
//! validation and the ADR-044 text-catalog fallback closure check.
//!
//! These live outside `cook.rs` to keep each source file within the
//! repository 1000-line limit. Behavior is unchanged; visibility is
//! crate-local.

use std::collections::BTreeSet;

use next_contracts::ids::SchemaId;
use next_contracts::localization::TextCatalogV1;
use next_contracts::project::{SchemaEncodingV1, SchemaRefV1, SchemaRoleV1, domain_hash};

use crate::cook::ProjectCookError;

pub(crate) fn validate_text_catalog_closure(
    catalogs: &[TextCatalogV1],
) -> Result<(), ProjectCookError> {
    if catalogs.is_empty() {
        return Ok(());
    }
    let mut locales = BTreeSet::new();
    let mut root_count = 0_usize;
    for catalog in catalogs {
        if !locales.insert(catalog.locale.as_str()) {
            return Err(ProjectCookError::DuplicateIdentity);
        }
        if catalog.fallback_locale_or_none.is_none() {
            root_count += 1;
        }
    }
    if root_count != 1 {
        return Err(ProjectCookError::LocalizationClosureInvalid);
    }
    for catalog in catalogs {
        let mut visited = BTreeSet::new();
        let mut current = catalog;
        loop {
            if !visited.insert(current.locale.as_str()) {
                return Err(ProjectCookError::LocalizationClosureInvalid);
            }
            let Some(fallback) = &current.fallback_locale_or_none else {
                break;
            };
            current = catalogs
                .iter()
                .find(|candidate| candidate.locale.as_str() == fallback.as_str())
                .ok_or(ProjectCookError::MissingReference)?;
        }
    }
    Ok(())
}

pub(crate) fn schema_ref(
    schema_id: &str,
    role: SchemaRoleV1,
    encoding: SchemaEncodingV1,
) -> Result<SchemaRefV1, next_contracts::ids::IdentifierError> {
    schema_ref_with_version(schema_id, 1, role, encoding)
}

pub(crate) fn schema_ref_with_version(
    schema_id: &str,
    schema_version: u32,
    role: SchemaRoleV1,
    encoding: SchemaEncodingV1,
) -> Result<SchemaRefV1, next_contracts::ids::IdentifierError> {
    Ok(SchemaRefV1 {
        schema_id: SchemaId::new(schema_id)?,
        schema_version,
        descriptor_sha256: domain_hash("nextengine.schema-descriptor.v1", schema_id.as_bytes()),
        role,
        encoding,
    })
}

pub(crate) fn ensure_unique<T: Ord>(
    values: impl IntoIterator<Item = T>,
) -> Result<(), ProjectCookError> {
    let mut values: Vec<_> = values.into_iter().collect();
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(ProjectCookError::DuplicateIdentity)
    } else {
        Ok(())
    }
}
