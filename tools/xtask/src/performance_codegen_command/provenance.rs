use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use xtask::performance_codegen::CodegenProfileOverrideV1;

pub(super) fn decode_build_toolchain() -> Result<String, String> {
    decode_hex_string(
        "embedded build toolchain",
        env!("NEXTENGINE_BUILD_TOOLCHAIN_HEX"),
    )
}

pub(super) fn decode_effective_rustflags() -> Result<Vec<String>, String> {
    let decoded = decode_hex_string(
        "embedded effective rustflags",
        env!("NEXTENGINE_EFFECTIVE_RUSTFLAGS_HEX"),
    )?;
    if decoded.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(decoded.split('\u{1f}').map(str::to_owned).collect())
    }
}

pub(super) fn decode_profile_overrides() -> Result<Vec<CodegenProfileOverrideV1>, String> {
    let encoded = env!("NEXTENGINE_PROFILE_OVERRIDES");
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    encoded
        .split(',')
        .map(|entry| {
            let (name, value) = entry
                .split_once(':')
                .ok_or_else(|| "embedded profile override is malformed".to_owned())?;
            Ok(CodegenProfileOverrideV1 {
                name: decode_hex_string("embedded profile override name", name)?,
                value: decode_hex_string("embedded profile override value", value)?,
            })
        })
        .collect()
}

fn decode_hex_string(description: &str, encoded: &str) -> Result<String, String> {
    let encoded = encoded.as_bytes();
    if !encoded.len().is_multiple_of(2) {
        return Err(format!("{description} has invalid hex length"));
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.chunks_exact(2) {
        let high =
            hex_nibble(pair[0]).ok_or_else(|| format!("{description} contains invalid hex"))?;
        let low =
            hex_nibble(pair[1]).ok_or_else(|| format!("{description} contains invalid hex"))?;
        bytes.push((high << 4) | low);
    }
    String::from_utf8(bytes).map_err(|error| format!("{description} is not UTF-8: {error}"))
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "hash input must be a non-symlink regular file: {}",
            path.display()
        ));
    }
    let file = fs::File::open(path)
        .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub(super) fn verified_pgo_profile_sha256(
    path: &Path,
    embedded_sha256: &str,
) -> Result<String, String> {
    if embedded_sha256.len() != 64
        || !embedded_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("CODEGEN_PGO_BUILD_PROFILE_HASH_INVALID".to_owned());
    }
    let current_sha256 = hash_file(path)?;
    if current_sha256 != embedded_sha256 {
        return Err(format!(
            "CODEGEN_PGO_PROFILE_CHANGED_SINCE_BUILD: {}",
            path.display()
        ));
    }
    Ok(embedded_sha256.to_owned())
}
