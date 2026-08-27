use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crc32fast::hash as crc32;
use flate2::read::DeflateDecoder;

use self::evidence::{
    AcquisitionReport, acquisition_metadata, inventory_manifest, pretty_json, provenance_review,
    publish_output, read_source_bundle,
};

use super::internet_sources::resolve_public_https_endpoint;
use super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, set_once, sha256_hex,
};

mod evidence;

const PROFILE: &str = "green-goblet-row-0-v1";
const ARCHIVE_URL: &str = "https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip";
const ARCHIVE_BYTES: u64 = 2_311_697_935;
const ARCHIVE_ETAG: &str = "6433e478-89c9b60f";
const ARCHIVE_LAST_MODIFIED_HTTP: &str = "Mon, 10 Apr 2023 10:27:04 GMT";
const ARCHIVE_LAST_MODIFIED_ISO: &str = "2023-04-10T10:27:04Z";
const CENTRAL_OFFSET: u64 = 2_311_696_618;
const CENTRAL_BYTES: usize = 1_295;
const CENTRAL_SHA256: &str = "083a0677196ee18688c42379d869137fc48e2efd756cd418bf4ef0cfa40eb3a4";
const ENTRY_COUNT: usize = 12;
const AUDIO_PREFIX_BYTES: usize = 1_048_576;
const AUDIO_PREFIX_SHA256: &str =
    "a9b68287eaa622455d3aa78de18b9dbf315e7fa3d5f719f9d67797b529ee270a";
const CORPUS_PLAN_SHA256: &str = "e082610c90dabff3c7a328df94671dca4f84f46cd629952c3e914ce600a3ea01";
const REPOSITORY_REVISION: &str = "commit-fca2bd6cbb7e9f96ac61328d2a0d51594bf01987";
const SOURCE_FILES: [SourceFile; 5] = [
    SourceFile {
        relative_path: "README.md",
        sha256: "3dd228b826651745f0cba8c8bfc0ff142f8fb4d9574f5adda91c272ffd8a2049",
    },
    SourceFile {
        relative_path: "LICENSE",
        sha256: "328ed037c524ac2f73c859183ca6bf07d034dc28dae918fb3897051a6fd8b937",
    },
    SourceFile {
        relative_path: "preprocess_measurements.py",
        sha256: "db55f2017a037fb50a7b8b59542e85e35a7c7d5581b15fb75b25dbdf67737bbc",
    },
    SourceFile {
        relative_path: "preprocess_annotations.py",
        sha256: "66c4ab81d39a1ef4e26a72561dd2fa5238224f31ed089b27ca9a80d33852985a",
    },
    SourceFile {
        relative_path: "dataset/download.sh",
        sha256: "4d6c2d7967d7dc2b8c56c7bfd7fe1550202099b6a7b1207aadc40e19d553a45a",
    },
];
const UNAVAILABLE_COMPONENTS: [&str; 4] = [
    "force-profile-bytes",
    "material-composition-revision",
    "repeat-recording-identity",
    "support-fixture-revision",
];
const ENTRIES: [EntrySpec; 8] = [
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/vertexXYZ.npy",
        159,
        258,
        441,
        72_128,
        0x695f_eafe,
        "cbc94c54a7ed8f35fd9743c8725678811566f17fc97d9739d110d390d909151e",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/micID.npy",
        699,
        794,
        225,
        24_128,
        0x082e_c0c6,
        "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/transformed.obj",
        2_783_815,
        2_783_916,
        557_368,
        3_528_966,
        0x101b_1df8,
        "96252fe0200699f06ae148d3eeeaf8eb67a1dc87dc4da94796bf375b5dd47192",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/vertexID.npy",
        3_341_284,
        3_341_382,
        164,
        24_128,
        0x4f58_b204,
        "ad1a3143fda803aeaace7b225cfe86ecea5e9393b8eb9f5c99ff5b54513e282d",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/listenerXYZ.npy",
        3_341_546,
        3_341_647,
        2_923,
        72_128,
        0xee42_6e91,
        "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/distance.npy",
        3_344_765,
        3_344_863,
        294,
        24_128,
        0x570b_48dd,
        "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/deconvolved_0db.npy",
        3_345_157,
        3_345_262,
        2_308_350_969,
        2_499_876_128,
        0xd41c_a14a,
        "",
    ),
    EntrySpec::new(
        "93_GreenGoblet/preprocessed/angle.npy",
        2_311_696_231,
        2_311_696_326,
        292,
        24_128,
        0xfabb_9a2e,
        "ed65ac28e45cc119b5d42c49293546f2749aa5f9629f6ffa9e0f35f2420874a3",
    ),
];

#[derive(Debug)]
pub(super) struct Request {
    profile: String,
    source_bundle: PathBuf,
    corpus_plan_report: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut profile = None;
    let mut source_bundle = None;
    let mut corpus_plan_report = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--profile" => set_once(&mut profile, value, &flag)?,
            "--source-bundle" => set_once(&mut source_bundle, PathBuf::from(value), &flag)?,
            "--corpus-plan-report" => {
                set_once(&mut corpus_plan_report, PathBuf::from(value), &flag)?
            }
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected realimpact-row argument: {flag}")),
        }
    }
    Ok(Request {
        profile: profile.ok_or_else(|| {
            "physical-sound-registry realimpact-row requires --profile green-goblet-row-0-v1"
                .to_owned()
        })?,
        source_bundle: source_bundle.ok_or_else(|| {
            "physical-sound-registry realimpact-row requires --source-bundle <external-directory>"
                .to_owned()
        })?,
        corpus_plan_report: corpus_plan_report.ok_or_else(|| {
            "physical-sound-registry realimpact-row requires --corpus-plan-report <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry realimpact-row requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    if request.profile != PROFILE {
        return Err(format!(
            "unsupported REALIMPACT row profile {}; expected {PROFILE}",
            request.profile
        ));
    }
    let output = resolve_output_path(root, &request.output)?;
    require_empty_output(&output)?;
    let source_bundle = resolve_cli_path(root, &request.source_bundle);
    let source_bytes = read_source_bundle(root, &source_bundle)?;
    let corpus_plan_path = canonical_external_file(
        root,
        &resolve_cli_path(root, &request.corpus_plan_report),
        "REALIMPACT corpus-plan report",
    )?;
    let corpus_plan = read_bounded_file(
        &corpus_plan_path,
        16 * 1024 * 1024,
        "REALIMPACT corpus-plan report",
    )?;
    require_hash(&corpus_plan, CORPUS_PLAN_SHA256, "corpus-plan report")?;

    let resolve = resolve_public_https_endpoint(ARCHIVE_URL)?
        .ok_or_else(|| "REALIMPACT archive host did not resolve".to_owned())?;
    let mut fetched_bytes = 0_u64;
    let eocd = fetch_range(&resolve, ARCHIVE_BYTES - 22, 22, &mut fetched_bytes)?;
    validate_archive_headers(&eocd)?;
    validate_eocd(&eocd.body)?;
    let central = fetch_range(&resolve, CENTRAL_OFFSET, CENTRAL_BYTES, &mut fetched_bytes)?;
    validate_archive_headers(&central)?;
    require_hash(&central.body, CENTRAL_SHA256, "ZIP central directory")?;
    let central_entries = parse_central_directory(&central.body)?;
    validate_central_entries(&central_entries)?;

    let mut raw_entries = BTreeMap::new();
    for spec in ENTRIES.iter().filter(|entry| !entry.raw_sha256.is_empty()) {
        let response = fetch_entry(&resolve, spec, spec.compressed_bytes, &mut fetched_bytes)?;
        let raw = decompress_complete(spec, &response)?;
        raw_entries.insert(spec.name, raw);
    }
    let audio_spec = entry("93_GreenGoblet/preprocessed/deconvolved_0db.npy")?;
    let audio_response = fetch_entry(
        &resolve,
        audio_spec,
        AUDIO_PREFIX_BYTES as u64,
        &mut fetched_bytes,
    )?;
    require_hash(
        &audio_response,
        AUDIO_PREFIX_SHA256,
        "deconvolved transfer compressed prefix",
    )?;
    let row = extract_audio_row_zero(&audio_response)?;
    let derived = validate_and_derive(&raw_entries, &row)?;

    let wav = normalized_wav(&row.bytes, row.peak_abs)?;
    let wav_sha256 = sha256_hex(&wav);
    let metadata = acquisition_metadata(&derived, &row, &wav_sha256);
    let metadata_bytes = pretty_json(&metadata)?;
    let provenance = provenance_review();
    let metadata_sha256 = sha256_hex(&metadata_bytes);
    let provenance_sha256 = sha256_hex(provenance.as_bytes());
    let manifest = inventory_manifest(&derived, &row, &metadata_sha256, &provenance_sha256);
    let manifest_bytes = pretty_json(&manifest)?;
    let report = AcquisitionReport {
        schema: "nextengine.experimental-realimpact-range-acquisition.report.v1",
        status: "Validated",
        decision: "E2TransferResponseFallbackOnly",
        profile: PROFILE,
        archive_url: ARCHIVE_URL,
        archive_content_length: ARCHIVE_BYTES,
        central_directory_sha256: CENTRAL_SHA256,
        http_range_payload_bytes: fetched_bytes,
        full_archive_fraction: fetched_bytes as f64 / ARCHIVE_BYTES as f64,
        dataset_object_id: "93_GreenGoblet",
        row_index: 0,
        row_sha256: &row.sha256,
        metadata_sha256: &metadata_sha256,
        provenance_sha256: &provenance_sha256,
        audition_wav_sha256: &wav_sha256,
        corpus_manifest_sha256: sha256_hex(&manifest_bytes),
        unavailable_components: &UNAVAILABLE_COMPONENTS,
    };
    let report_bytes = pretty_json(&report)?;

    publish_output(
        &output,
        &source_bytes,
        &corpus_plan,
        &row.bytes,
        &wav,
        &metadata_bytes,
        provenance.as_bytes(),
        &manifest_bytes,
        &report_bytes,
    )?;
    println!("REALIMPACT bounded row output: {}", output.display());
    println!("row sha256: {}", row.sha256);
    println!("metadata sha256: {metadata_sha256}");
    println!("provenance sha256: {provenance_sha256}");
    println!("audition wav sha256: {wav_sha256}");
    Ok(())
}

#[derive(Clone, Copy)]
struct SourceFile {
    relative_path: &'static str,
    sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EntrySpec {
    name: &'static str,
    local_offset: u64,
    data_offset: u64,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    crc32: u32,
    raw_sha256: &'static str,
}

impl EntrySpec {
    const fn new(
        name: &'static str,
        local_offset: u64,
        data_offset: u64,
        compressed_bytes: u64,
        uncompressed_bytes: u64,
        crc32: u32,
        raw_sha256: &'static str,
    ) -> Self {
        Self {
            name,
            local_offset,
            data_offset,
            compressed_bytes,
            uncompressed_bytes,
            crc32,
            raw_sha256,
        }
    }
}

#[derive(Debug)]
struct RangeResponse {
    body: Vec<u8>,
    etag: String,
    last_modified: String,
}

fn fetch_range(
    resolve: &str,
    start: u64,
    length: usize,
    fetched_bytes: &mut u64,
) -> Result<RangeResponse, String> {
    let length_u64 = u64::try_from(length).map_err(|_| "range length overflow".to_owned())?;
    let end = start
        .checked_add(length_u64)
        .and_then(|value| value.checked_sub(1))
        .ok_or_else(|| "range endpoint overflow".to_owned())?;
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--noproxy",
            "*",
            "--connect-timeout",
            "30",
            "--max-time",
            "120",
            "--resolve",
            resolve,
            "--range",
            &format!("{start}-{end}"),
            "--include",
            ARCHIVE_URL,
        ])
        .output()
        .map_err(|error| format!("start bounded REALIMPACT HTTPS range fetch: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "bounded REALIMPACT HTTPS range fetch failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let split = output
        .stdout
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "REALIMPACT range response has no HTTP header terminator".to_owned())?;
    let header = std::str::from_utf8(&output.stdout[..split])
        .map_err(|_| "REALIMPACT range response headers are not UTF-8".to_owned())?;
    let body = output.stdout[split + 4..].to_vec();
    if body.len() != length {
        return Err(format!(
            "REALIMPACT range {start}-{end} returned {} bytes, expected {length}",
            body.len()
        ));
    }
    let mut lines = header.split("\r\n");
    let status = lines
        .next()
        .ok_or_else(|| "REALIMPACT range response has no status".to_owned())?;
    if !status.ends_with(" 206") && !status.contains(" 206 ") {
        return Err(format!("REALIMPACT range response is not 206: {status}"));
    }
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let expected_content_range = format!("bytes {start}-{end}/{ARCHIVE_BYTES}");
    if headers.get("content-range") != Some(&expected_content_range) {
        return Err("REALIMPACT response Content-Range changed".to_owned());
    }
    *fetched_bytes = fetched_bytes
        .checked_add(length_u64)
        .ok_or_else(|| "REALIMPACT fetched byte count overflow".to_owned())?;
    Ok(RangeResponse {
        body,
        etag: headers
            .get("etag")
            .map(|value| value.trim_matches('"').to_owned())
            .ok_or_else(|| "REALIMPACT response has no ETag".to_owned())?,
        last_modified: headers
            .get("last-modified")
            .cloned()
            .ok_or_else(|| "REALIMPACT response has no Last-Modified".to_owned())?,
    })
}

fn validate_archive_headers(response: &RangeResponse) -> Result<(), String> {
    if response.etag != ARCHIVE_ETAG || response.last_modified != ARCHIVE_LAST_MODIFIED_HTTP {
        return Err("REALIMPACT archive HTTP identity changed".to_owned());
    }
    Ok(())
}

fn validate_eocd(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() != 22
        || &bytes[..4] != b"PK\x05\x06"
        || le_u16(bytes, 4)? != 0
        || le_u16(bytes, 6)? != 0
        || usize::from(le_u16(bytes, 8)?) != ENTRY_COUNT
        || usize::from(le_u16(bytes, 10)?) != ENTRY_COUNT
        || usize::try_from(le_u32(bytes, 12)?).ok() != Some(CENTRAL_BYTES)
        || u64::from(le_u32(bytes, 16)?) != CENTRAL_OFFSET
        || le_u16(bytes, 20)? != 0
    {
        return Err("REALIMPACT ZIP EOCD changed or is unsupported".to_owned());
    }
    Ok(())
}

fn parse_central_directory(bytes: &[u8]) -> Result<Vec<ParsedEntry>, String> {
    let mut entries = Vec::new();
    let mut offset = 0_usize;
    while offset < bytes.len() {
        let fixed = bytes
            .get(offset..offset + 46)
            .ok_or_else(|| "truncated ZIP central record".to_owned())?;
        if &fixed[..4] != b"PK\x01\x02" {
            return Err("invalid ZIP central record signature".to_owned());
        }
        let name_bytes = usize::from(le_u16(fixed, 28)?);
        let extra_bytes = usize::from(le_u16(fixed, 30)?);
        let comment_bytes = usize::from(le_u16(fixed, 32)?);
        let record_bytes = 46_usize
            .checked_add(name_bytes)
            .and_then(|value| value.checked_add(extra_bytes))
            .and_then(|value| value.checked_add(comment_bytes))
            .ok_or_else(|| "ZIP central record size overflow".to_owned())?;
        let record = bytes
            .get(offset..offset + record_bytes)
            .ok_or_else(|| "truncated ZIP central variable fields".to_owned())?;
        let name = std::str::from_utf8(&record[46..46 + name_bytes])
            .map_err(|_| "ZIP central name is not UTF-8".to_owned())?
            .to_owned();
        entries.push(ParsedEntry {
            name,
            flags: le_u16(fixed, 8)?,
            method: le_u16(fixed, 10)?,
            crc32: le_u32(fixed, 16)?,
            compressed_bytes: u64::from(le_u32(fixed, 20)?),
            uncompressed_bytes: u64::from(le_u32(fixed, 24)?),
            local_offset: u64::from(le_u32(fixed, 42)?),
        });
        offset += record_bytes;
    }
    if entries.len() != ENTRY_COUNT {
        return Err(format!(
            "REALIMPACT central directory has {} entries, expected {ENTRY_COUNT}",
            entries.len()
        ));
    }
    Ok(entries)
}

#[derive(Debug, Eq, PartialEq)]
struct ParsedEntry {
    name: String,
    flags: u16,
    method: u16,
    crc32: u32,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    local_offset: u64,
}

fn validate_central_entries(entries: &[ParsedEntry]) -> Result<(), String> {
    for expected in ENTRIES {
        let actual = entries
            .iter()
            .find(|entry| entry.name == expected.name)
            .ok_or_else(|| format!("ZIP central directory is missing {}", expected.name))?;
        if actual.flags != 0
            || actual.method != 8
            || actual.crc32 != expected.crc32
            || actual.compressed_bytes != expected.compressed_bytes
            || actual.uncompressed_bytes != expected.uncompressed_bytes
            || actual.local_offset != expected.local_offset
        {
            return Err(format!(
                "ZIP central metadata changed for {}",
                expected.name
            ));
        }
    }
    Ok(())
}

fn fetch_entry(
    resolve: &str,
    spec: &EntrySpec,
    compressed_bytes: u64,
    fetched_bytes: &mut u64,
) -> Result<Vec<u8>, String> {
    if compressed_bytes > spec.compressed_bytes {
        return Err(format!("requested range exceeds ZIP entry {}", spec.name));
    }
    let header_bytes = spec
        .data_offset
        .checked_sub(spec.local_offset)
        .ok_or_else(|| "ZIP local offset underflow".to_owned())?;
    let length = header_bytes
        .checked_add(compressed_bytes)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| "ZIP entry range length overflow".to_owned())?;
    let response = fetch_range(resolve, spec.local_offset, length, fetched_bytes)?;
    validate_archive_headers(&response)?;
    let header_len =
        usize::try_from(header_bytes).map_err(|_| "ZIP header too large".to_owned())?;
    validate_local_header(&response.body[..header_len], spec)?;
    Ok(response.body[header_len..].to_vec())
}

fn validate_local_header(bytes: &[u8], spec: &EntrySpec) -> Result<(), String> {
    if bytes.len() < 30 || &bytes[..4] != b"PK\x03\x04" {
        return Err(format!("invalid ZIP local header for {}", spec.name));
    }
    let name_bytes = usize::from(le_u16(bytes, 26)?);
    let extra_bytes = usize::from(le_u16(bytes, 28)?);
    if bytes.len() != 30 + name_bytes + extra_bytes
        || le_u16(bytes, 6)? != 0
        || le_u16(bytes, 8)? != 8
        || le_u32(bytes, 14)? != spec.crc32
        || u64::from(le_u32(bytes, 18)?) != spec.compressed_bytes
        || u64::from(le_u32(bytes, 22)?) != spec.uncompressed_bytes
        || bytes.get(30..30 + name_bytes) != Some(spec.name.as_bytes())
    {
        return Err(format!("ZIP local header changed for {}", spec.name));
    }
    Ok(())
}

fn decompress_complete(spec: &EntrySpec, compressed: &[u8]) -> Result<Vec<u8>, String> {
    let mut raw = Vec::new();
    DeflateDecoder::new(compressed)
        .take(spec.uncompressed_bytes + 1)
        .read_to_end(&mut raw)
        .map_err(|error| format!("decompress {}: {error}", spec.name))?;
    if raw.len() as u64 != spec.uncompressed_bytes || crc32(&raw) != spec.crc32 {
        return Err(format!(
            "decompressed ZIP integrity changed for {}",
            spec.name
        ));
    }
    require_hash(&raw, spec.raw_sha256, spec.name)?;
    Ok(raw)
}

fn entry(name: &str) -> Result<&'static EntrySpec, String> {
    ENTRIES
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| format!("internal REALIMPACT entry is missing: {name}"))
}

#[derive(Debug)]
struct AudioRow {
    bytes: Vec<u8>,
    sha256: String,
    sample_count: usize,
    peak_abs: f64,
    rms: f64,
}

fn extract_audio_row_zero(compressed: &[u8]) -> Result<AudioRow, String> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut header = [0_u8; 128];
    decoder
        .read_exact(&mut header)
        .map_err(|error| format!("decompress REALIMPACT transfer NPY header: {error}"))?;
    validate_npy_header_prefix(&header, "<f4", &[3_000, 208_323])?;
    let sample_count = 208_323_usize;
    let mut bytes = vec![0_u8; sample_count * 4];
    decoder
        .read_exact(&mut bytes)
        .map_err(|error| format!("decompress REALIMPACT transfer row 0: {error}"))?;
    require_hash(
        &bytes,
        "104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9",
        "REALIMPACT GreenGoblet transfer row 0",
    )?;
    let mut peak_abs = 0.0_f64;
    let mut sum_squared = 0.0_f64;
    for chunk in bytes.chunks_exact(4) {
        let value = f64::from(f32::from_le_bytes(chunk.try_into().expect("four bytes")));
        if !value.is_finite() {
            return Err("REALIMPACT transfer row contains a non-finite sample".to_owned());
        }
        peak_abs = peak_abs.max(value.abs());
        sum_squared += value * value;
    }
    Ok(AudioRow {
        sha256: sha256_hex(&bytes),
        bytes,
        sample_count,
        peak_abs,
        rms: (sum_squared / sample_count as f64).sqrt(),
    })
}

#[derive(Debug)]
struct DerivedMetadata {
    mesh_sha256: String,
    mesh_vertex_count: usize,
    mesh_bbox_min_metres: [f64; 3],
    mesh_bbox_max_metres: [f64; 3],
    impact_vertex_count: usize,
    listener_position_count: usize,
    azimuth_degrees: Vec<u32>,
    distance_offsets: Vec<u32>,
    microphone_ids: Vec<u32>,
    impact_vertex_id: usize,
    impact_position: [f64; 3],
    listener_position: [f64; 3],
    downloaded_hashes: BTreeMap<&'static str, String>,
}

fn validate_and_derive(
    raw_entries: &BTreeMap<&'static str, Vec<u8>>,
    row: &AudioRow,
) -> Result<DerivedMetadata, String> {
    if row.sample_count != 208_323 {
        return Err("REALIMPACT transfer row sample count changed".to_owned());
    }
    let vertex_xyz = f64_array(raw(raw_entries, "vertexXYZ.npy")?, &[3_000, 3])?;
    let listener_xyz = f64_array(raw(raw_entries, "listenerXYZ.npy")?, &[3_000, 3])?;
    let vertex_ids = i64_array(raw(raw_entries, "vertexID.npy")?, &[3_000])?;
    let microphone_ids = i64_array(raw(raw_entries, "micID.npy")?, &[3_000])?;
    let distances = i64_array(raw(raw_entries, "distance.npy")?, &[3_000])?;
    let angles = i64_array(raw(raw_entries, "angle.npy")?, &[3_000])?;
    let mesh = raw(raw_entries, "transformed.obj")?;
    let (vertices, bbox_min, bbox_max) = parse_mesh(mesh)?;
    let impact_vertex_id = usize::try_from(vertex_ids[0])
        .map_err(|_| "REALIMPACT row 0 vertex ID is negative".to_owned())?;
    let impact_position = [vertex_xyz[0], vertex_xyz[1], vertex_xyz[2]];
    let listener_position = [listener_xyz[0], listener_xyz[1], listener_xyz[2]];
    if vertices.get(impact_vertex_id) != Some(&impact_position) {
        return Err("REALIMPACT row 0 position does not match its mesh vertex".to_owned());
    }
    let unique_vertices = vertex_ids.iter().copied().collect::<BTreeSet<_>>();
    let angle_axis = u32_axis(&angles, "azimuth")?;
    let distance_axis = u32_axis(&distances, "distance")?;
    let microphone_axis = u32_axis(&microphone_ids, "microphone")?;
    if unique_vertices.len() != 5
        || angle_axis != [0, 20, 40, 60, 80, 100, 120, 140, 160, 180]
        || distance_axis != [0, 333, 666, 1_000]
        || microphone_axis != (0..15).collect::<Vec<_>>()
        || angles[0] != 0
        || distances[0] != 0
        || microphone_ids[0] != 0
        || impact_vertex_id != 31_676
        || impact_position != [-0.020_581_99, -0.039_491_5, 0.156_666_2]
        || listener_position != [0.23, -0.043_45, -0.91]
        || vertices.len() != 48_174
    {
        return Err("REALIMPACT GreenGoblet acquisition axes changed".to_owned());
    }
    let listener_position_count = angle_axis
        .len()
        .checked_mul(distance_axis.len())
        .and_then(|value| value.checked_mul(microphone_axis.len()))
        .ok_or_else(|| "REALIMPACT listener count overflow".to_owned())?;
    if listener_position_count != 600
        || listener_position_count * unique_vertices.len() != vertex_ids.len()
    {
        return Err("REALIMPACT row count does not match acquisition axes".to_owned());
    }
    let downloaded_hashes = [
        ("angle.npy", "angle.npy"),
        ("distance.npy", "distance.npy"),
        ("listenerXYZ.npy", "listenerXYZ.npy"),
        ("micID.npy", "micID.npy"),
        ("vertexID.npy", "vertexID.npy"),
        ("vertexXYZ.npy", "vertexXYZ.npy"),
    ]
    .into_iter()
    .map(|(key, suffix)| Ok((key, sha256_hex(raw(raw_entries, suffix)?))))
    .collect::<Result<_, String>>()?;
    Ok(DerivedMetadata {
        mesh_sha256: sha256_hex(mesh),
        mesh_vertex_count: vertices.len(),
        mesh_bbox_min_metres: bbox_min,
        mesh_bbox_max_metres: bbox_max,
        impact_vertex_count: unique_vertices.len(),
        listener_position_count,
        azimuth_degrees: angle_axis,
        distance_offsets: distance_axis,
        microphone_ids: microphone_axis,
        impact_vertex_id,
        impact_position,
        listener_position,
        downloaded_hashes,
    })
}

fn raw<'a>(entries: &'a BTreeMap<&'static str, Vec<u8>>, suffix: &str) -> Result<&'a [u8], String> {
    entries
        .iter()
        .find_map(|(name, bytes)| name.ends_with(suffix).then_some(bytes.as_slice()))
        .ok_or_else(|| format!("missing REALIMPACT decoded entry ending in {suffix}"))
}

fn f64_array(bytes: &[u8], shape: &[usize]) -> Result<Vec<f64>, String> {
    validate_npy_header(bytes, "<f8", shape)?;
    bytes[128..]
        .chunks_exact(8)
        .map(|chunk| {
            let value = f64::from_le_bytes(chunk.try_into().expect("eight bytes"));
            value
                .is_finite()
                .then_some(value)
                .ok_or_else(|| "REALIMPACT f64 NPY contains a non-finite value".to_owned())
        })
        .collect()
}

fn i64_array(bytes: &[u8], shape: &[usize]) -> Result<Vec<i64>, String> {
    validate_npy_header(bytes, "<i8", shape)?;
    Ok(bytes[128..]
        .chunks_exact(8)
        .map(|chunk| i64::from_le_bytes(chunk.try_into().expect("eight bytes")))
        .collect())
}

fn validate_npy_header(bytes: &[u8], dtype: &str, shape: &[usize]) -> Result<(), String> {
    let expected_values = shape
        .iter()
        .try_fold(1_usize, |total, value| total.checked_mul(*value))
        .ok_or_else(|| "NPY shape product overflow".to_owned())?;
    let item_bytes = match dtype {
        "<f4" => 4,
        "<f8" | "<i8" => 8,
        _ => return Err(format!("unsupported frozen NPY dtype {dtype}")),
    };
    let expected_bytes = 128_usize
        .checked_add(
            expected_values
                .checked_mul(item_bytes)
                .ok_or_else(|| "NPY payload size overflow".to_owned())?,
        )
        .ok_or_else(|| "NPY file size overflow".to_owned())?;
    if bytes.len() != expected_bytes {
        return Err("unsupported or truncated frozen NPY payload".to_owned());
    }
    validate_npy_header_prefix(bytes, dtype, shape)
}

fn validate_npy_header_prefix(bytes: &[u8], dtype: &str, shape: &[usize]) -> Result<(), String> {
    if bytes.len() < 128
        || bytes.get(..6) != Some(b"\x93NUMPY")
        || bytes.get(6..8) != Some(&[1, 0])
        || le_u16(bytes, 8)? != 118
    {
        return Err("unsupported or truncated frozen NPY header".to_owned());
    }
    let header =
        std::str::from_utf8(&bytes[10..128]).map_err(|_| "NPY header is not UTF-8".to_owned())?;
    let shape_text = match shape {
        [one] => format!("({one},)"),
        [one, two] => format!("({one}, {two})"),
        _ => return Err("unsupported frozen NPY rank".to_owned()),
    };
    let identity = format!("'descr': '{dtype}'");
    if !header.contains(&identity)
        || !header.contains("'fortran_order': False")
        || !header.contains(&format!("'shape': {shape_text}"))
        || !header.ends_with('\n')
    {
        return Err("frozen NPY descriptor changed".to_owned());
    }
    Ok(())
}

fn u32_axis(values: &[i64], role: &str) -> Result<Vec<u32>, String> {
    values
        .iter()
        .map(|value| u32::try_from(*value).map_err(|_| format!("negative or large {role} value")))
        .collect::<Result<BTreeSet<_>, _>>()
        .map(|values| values.into_iter().collect())
}

type ParsedMesh = (Vec<[f64; 3]>, [f64; 3], [f64; 3]);

fn parse_mesh(bytes: &[u8]) -> Result<ParsedMesh, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "REALIMPACT mesh is not UTF-8".to_owned())?;
    let mut vertices = Vec::new();
    let mut minimum = [f64::INFINITY; 3];
    let mut maximum = [f64::NEG_INFINITY; 3];
    for line in text.lines().filter(|line| line.starts_with("v ")) {
        let coordinates = line[2..]
            .split_ascii_whitespace()
            .map(|value| {
                value
                    .parse::<f64>()
                    .map_err(|error| format!("parse REALIMPACT mesh vertex: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let vertex: [f64; 3] = coordinates
            .try_into()
            .map_err(|_| "REALIMPACT mesh vertex is not three-dimensional".to_owned())?;
        if vertex.iter().any(|value| !value.is_finite()) {
            return Err("REALIMPACT mesh has a non-finite vertex".to_owned());
        }
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(vertex[axis]);
            maximum[axis] = maximum[axis].max(vertex[axis]);
        }
        vertices.push(vertex);
    }
    if vertices.is_empty() {
        return Err("REALIMPACT mesh has no vertices".to_owned());
    }
    Ok((vertices, minimum, maximum))
}

fn normalized_wav(row: &[u8], peak_abs: f64) -> Result<Vec<u8>, String> {
    if !row.len().is_multiple_of(4) || !peak_abs.is_finite() || peak_abs <= 0.0 {
        return Err("cannot normalize invalid REALIMPACT transfer row".to_owned());
    }
    let samples = row.len() / 4;
    let data_bytes = samples
        .checked_mul(2)
        .ok_or_else(|| "normalized WAV size overflow".to_owned())?;
    let riff_bytes = 36_usize
        .checked_add(data_bytes)
        .ok_or_else(|| "normalized WAV RIFF size overflow".to_owned())?;
    let mut wav = Vec::with_capacity(44 + data_bytes);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(
        &u32::try_from(riff_bytes)
            .map_err(|_| "WAV too large")?
            .to_le_bytes(),
    );
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&48_000_u32.to_le_bytes());
    wav.extend_from_slice(&96_000_u32.to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(
        &u32::try_from(data_bytes)
            .map_err(|_| "WAV too large")?
            .to_le_bytes(),
    );
    for chunk in row.chunks_exact(4) {
        let sample = f64::from(f32::from_le_bytes(chunk.try_into().expect("four bytes")));
        let normalized = (sample / peak_abs * 0.95 * f64::from(i16::MAX)).round();
        let quantized = normalized.clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16;
        wav.extend_from_slice(&quantized.to_le_bytes());
    }
    Ok(wav)
}

fn require_hash(bytes: &[u8], expected: &str, role: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "{role} SHA-256 changed: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn le_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "truncated little-endian u16".to_owned())
}

fn le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "truncated little-endian u32".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_bind_the_only_frozen_profile() {
        let request = parse_arguments(
            [
                "--profile",
                PROFILE,
                "--source-bundle",
                "/tmp/source",
                "--corpus-plan-report",
                "/tmp/plan.json",
                "--output",
                "/tmp/output",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("arguments parse");
        assert_eq!(request.profile, PROFILE);
        assert!(parse_arguments(std::iter::empty()).is_err());
    }

    #[test]
    fn frozen_npy_parser_rejects_shape_and_fortran_drift() {
        let mut bytes = vec![b' '; 128 + 16];
        bytes[..6].copy_from_slice(b"\x93NUMPY");
        bytes[6..8].copy_from_slice(&[1, 0]);
        bytes[8..10].copy_from_slice(&118_u16.to_le_bytes());
        let header = b"{'descr': '<f8', 'fortran_order': False, 'shape': (2,), }";
        bytes[10..10 + header.len()].copy_from_slice(header);
        bytes[127] = b'\n';
        validate_npy_header(&bytes, "<f8", &[2]).expect("frozen header validates");
        assert!(validate_npy_header(&bytes, "<f8", &[1, 2]).is_err());
        bytes[42..47].copy_from_slice(b"True ");
        assert!(validate_npy_header(&bytes, "<f8", &[2]).is_err());
    }

    #[test]
    fn normalized_wav_is_bounded_and_deterministic() {
        let row = [-2.0_f32, 0.0, 1.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let wav = normalized_wav(&row, 2.0).expect("WAV renders");
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(wav.len(), 50);
        assert_eq!(wav, normalized_wav(&row, 2.0).expect("WAV repeats"));
    }
}
