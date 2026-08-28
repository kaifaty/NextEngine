use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::sync::{Arc, Mutex, MutexGuard};

use next_contracts::ids::ContentHash;
use rars::{ArchiveFamily, ArchiveReadOptions, ArchiveReader};
use sha2::{Digest, Sha256};

const MAX_ARCHIVE_ENTRIES: usize = 256;
const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SINGLE_ENTRY_BYTES: u64 = 8 * 1024 * 1024;

pub(super) struct ExpectedEntry<'a> {
    pub(super) name: &'a str,
    pub(super) bytes: u64,
    pub(super) sha256: &'a str,
}

pub(super) fn read_exact_entries(
    archive_bytes: &[u8],
    expected: &[ExpectedEntry<'_>],
) -> Result<Vec<Vec<u8>>, String> {
    if expected.is_empty() || expected.len() > MAX_ARCHIVE_ENTRIES {
        return Err("RAR entry expectation count exceeds the adapter bound".to_owned());
    }
    let mut expected_by_name = BTreeMap::new();
    for entry in expected {
        if entry.bytes == 0
            || entry.bytes > MAX_SINGLE_ENTRY_BYTES
            || !is_safe_ascii_entry_name(entry.name.as_bytes())
            || expected_by_name
                .insert(entry.name.as_bytes(), entry)
                .is_some()
        {
            return Err(format!(
                "RAR entry {} has an invalid bounded expectation",
                entry.name
            ));
        }
    }

    let options =
        ArchiveReadOptions::new().with_rar50_buffered_decode_limit(MAX_SINGLE_ENTRY_BYTES);
    let archive = ArchiveReader::read_with_options(archive_bytes, options)
        .map_err(|error| format!("open bounded RAR archive: {error}"))?;
    if archive.family() != ArchiveFamily::Rar50Plus || archive.sfx_offset() != 0 {
        return Err("source archive must be an ordinary RAR5 archive".to_owned());
    }
    validate_archive_index(&archive, &expected_by_name)?;

    let collected = Arc::new(Mutex::new(BTreeMap::<Vec<u8>, Vec<u8>>::new()));
    archive
        .extract_to_with_options(options, {
            let collected = Arc::clone(&collected);
            move |metadata| {
                let name = metadata.name_bytes();
                let Some(expectation) = expected_by_name.get(name) else {
                    return Ok(Box::new(io::sink()) as Box<dyn Write>);
                };
                let mut values = lock(&collected);
                if values
                    .insert(
                        name.to_vec(),
                        Vec::with_capacity(expectation.bytes as usize),
                    )
                    .is_some()
                {
                    return Err(rars::Error::InvalidHeader("duplicate extracted member"));
                }
                drop(values);
                Ok(Box::new(BoundedSharedBuffer {
                    values: Arc::clone(&collected),
                    name: name.to_vec(),
                    maximum_bytes: expectation.bytes,
                }) as Box<dyn Write>)
            }
        })
        .map_err(|error| format!("extract bounded RAR archive: {error}"))?;

    let mut values = lock(&collected);
    expected
        .iter()
        .map(|entry| {
            let bytes = values
                .remove(entry.name.as_bytes())
                .ok_or_else(|| format!("RAR archive is missing entry {}", entry.name))?;
            let digest: [u8; 32] = Sha256::digest(&bytes).into();
            let actual_sha256 = ContentHash::from_bytes(digest).to_hex();
            if bytes.len() as u64 != entry.bytes || actual_sha256 != entry.sha256 {
                return Err(format!(
                    "RAR entry {} integrity mismatch: expected {} bytes/{}, got {} bytes/{actual_sha256}",
                    entry.name,
                    entry.bytes,
                    entry.sha256,
                    bytes.len()
                ));
            }
            Ok(bytes)
        })
        .collect()
}

fn validate_archive_index(
    archive: &rars::Archive,
    expected: &BTreeMap<&[u8], &ExpectedEntry<'_>>,
) -> Result<(), String> {
    let members = archive.members().collect::<Vec<_>>();
    if members.is_empty() || members.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!(
            "RAR archive entry count must be 1..={MAX_ARCHIVE_ENTRIES}"
        ));
    }
    let mut names = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for member in members {
        let metadata = member.meta;
        if !is_safe_ascii_entry_name(metadata.name_bytes())
            || metadata.unpacked_size > MAX_SINGLE_ENTRY_BYTES
            || metadata.is_encrypted
            || metadata.is_split_before
            || metadata.is_split_after
            || !names.insert(metadata.name.clone())
        {
            return Err(format!(
                "RAR member {} violates the bounded archive policy",
                metadata.name_lossy()
            ));
        }
        total_bytes = total_bytes
            .checked_add(metadata.unpacked_size)
            .filter(|bytes| *bytes <= MAX_ARCHIVE_UNCOMPRESSED_BYTES)
            .ok_or_else(|| "RAR archive exceeds its uncompressed byte bound".to_owned())?;
        if let Some(entry) = expected.get(metadata.name_bytes())
            && (metadata.is_directory || metadata.unpacked_size != entry.bytes)
        {
            return Err(format!(
                "RAR entry {} does not match its bounded file declaration",
                entry.name
            ));
        }
    }
    if expected.keys().any(|name| !names.contains(*name)) {
        return Err("RAR archive is missing a declared entry".to_owned());
    }
    Ok(())
}

fn is_safe_ascii_entry_name(name: &[u8]) -> bool {
    !name.is_empty()
        && name.is_ascii()
        && !name.starts_with(b"/")
        && !name.contains(&b'\\')
        && name
            .split(|byte| *byte == b'/')
            .all(|component| !component.is_empty() && component != b"." && component != b"..")
}

struct BoundedSharedBuffer {
    values: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
    name: Vec<u8>,
    maximum_bytes: u64,
}

impl Write for BoundedSharedBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut values = lock(&self.values);
        let value = values
            .get_mut(&self.name)
            .ok_or_else(|| io::Error::other("RAR extraction buffer disappeared"))?;
        let next_len = value
            .len()
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::other("RAR extraction size overflow"))?;
        if next_len as u64 > self.maximum_bytes {
            return Err(io::Error::other("RAR extraction exceeded declared size"));
        }
        value.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
