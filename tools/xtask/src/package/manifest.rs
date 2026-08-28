use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(super) struct PackageManifestVersionProbe {
    pub(super) schema_version: u32,
}

pub(super) const MAX_MANIFEST_BYTES: usize = 8 * 1024 * 1024;
pub(super) const REQUIRED_NOTICE_PATHS: [&str; 4] = [
    "LICENSE",
    "MIGRATION_PROVENANCE.md",
    "NOTICE",
    "THIRD_PARTY_NOTICES.md",
];
pub(super) const REFERENCE_PROJECT_DOCUMENT_PATHS: [(&str, &str); 2] = [
    ("projects/reference-alpha/ACCEPTANCE.md", "ACCEPTANCE.md"),
    ("projects/reference-alpha/NOTICE", "REFERENCE_ALPHA_NOTICE"),
];

pub(super) fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(value)
        .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
    let mut bytes = Vec::new();
    write_canonical_value(&value, &mut bytes)?;
    Ok(bytes)
}

pub(super) fn hash_bytes(bytes: &[u8]) -> String {
    next_contracts::ids::content_hash_from_bytes(next_contracts::canonical::sha256(bytes)).to_hex()
}

fn write_canonical_value(value: &serde_json::Value, output: &mut Vec<u8>) -> Result<(), String> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" });
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            serde_json::to_writer(output, value)
                .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_value(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(key, _)| *key);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)
                    .map_err(|error| format!("NATIVE_GATE_PACKAGE_INVALID: {error}"))?;
                output.push(b':');
                write_canonical_value(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}
