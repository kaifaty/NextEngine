use std::collections::BTreeSet;
use std::io::{Cursor, Read};

use next_contracts::ids::ContentHash;
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, ZipArchive};

const MAX_ARCHIVE_ENTRIES: usize = 1_024;
const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SINGLE_ENTRY_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn read_exact_entry(
    archive_bytes: &[u8],
    entry_name: &str,
    expected_bytes: u64,
    expected_sha256: &str,
) -> Result<Vec<u8>, String> {
    if expected_bytes == 0 || expected_bytes > MAX_SINGLE_ENTRY_BYTES {
        return Err("ZIP entry byte expectation exceeds the adapter bound".to_owned());
    }
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
        .map_err(|error| format!("open bounded ZIP archive: {error}"))?;
    validate_archive_index(&mut archive)?;
    let entry = archive
        .by_name(entry_name)
        .map_err(|error| format!("open ZIP entry {entry_name}: {error}"))?;
    if !entry.is_file()
        || entry.enclosed_name().is_none()
        || entry.size() != expected_bytes
        || entry.encrypted()
        || !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        )
    {
        return Err(format!(
            "ZIP entry {entry_name} does not match its bounded file declaration"
        ));
    }
    let limit = expected_bytes
        .checked_add(1)
        .ok_or_else(|| "ZIP entry read bound overflow".to_owned())?;
    let mut bytes = Vec::with_capacity(expected_bytes as usize);
    entry
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read ZIP entry {entry_name}: {error}"))?;
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    let actual_sha256 = ContentHash::from_bytes(digest).to_hex();
    if bytes.len() as u64 != expected_bytes || actual_sha256 != expected_sha256 {
        return Err(format!(
            "ZIP entry {entry_name} integrity mismatch: expected {expected_bytes} bytes/{expected_sha256}, got {} bytes/{actual_sha256}",
            bytes.len()
        ));
    }
    Ok(bytes)
}

fn validate_archive_index(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<(), String> {
    if archive.is_empty() || archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "ZIP archive entry count must be 1..={MAX_ARCHIVE_ENTRIES}"
        ));
    }
    let mut names = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("read ZIP central-directory entry {index}: {error}"))?;
        if !entry.name().is_ascii()
            || entry.enclosed_name().is_none()
            || entry.encrypted()
            || entry.is_symlink()
            || entry.size() > MAX_SINGLE_ENTRY_BYTES
            || !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(format!(
                "ZIP central-directory entry {index} violates the bounded archive policy"
            ));
        }
        if !names.insert(entry.name().to_owned()) {
            return Err(format!(
                "ZIP archive contains duplicate entry {}",
                entry.name()
            ));
        }
        total_bytes = total_bytes
            .checked_add(entry.size())
            .filter(|bytes| *bytes <= MAX_ARCHIVE_UNCOMPRESSED_BYTES)
            .ok_or_else(|| "ZIP archive exceeds its uncompressed byte bound".to_owned())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::*;
    use zip::write::SimpleFileOptions;

    #[test]
    fn reads_one_hash_closed_entry_without_extracting_paths() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("safe/audio.wav", SimpleFileOptions::default())
            .expect("start ZIP entry");
        writer.write_all(b"wave").expect("write ZIP entry");
        let bytes = writer.finish().expect("finish ZIP").into_inner();
        let sha256 = ContentHash::from_bytes(Sha256::digest(b"wave").into()).to_hex();
        assert_eq!(
            read_exact_entry(&bytes, "safe/audio.wav", 4, &sha256).expect("read exact entry"),
            b"wave"
        );
        assert!(read_exact_entry(&bytes, "../audio.wav", 4, &sha256).is_err());
        assert!(read_exact_entry(&bytes, "safe/audio.wav", 3, &sha256).is_err());
    }
}
