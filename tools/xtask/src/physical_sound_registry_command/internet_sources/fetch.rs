use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use next_contracts::ids::ContentHash;
use sha2::{Digest, Sha256};

use super::{
    CacheStatus, FetchNormalizationPolicy, FetchRedirectPolicy, canonical_https_host_and_path,
    verify_cache_artifact,
};

mod figshare;
mod freesound;
mod osf;

const DOWNLOAD_BUFFER_BYTES: usize = 128 * 1024;
const DOWNLOAD_TIMEOUT_SECONDS: &str = "120";

static NEXT_STAGING_FILE: AtomicU64 = AtomicU64::new(0);

pub(super) struct FetchPolicies {
    pub(super) redirect: Option<FetchRedirectPolicy>,
    pub(super) normalization: Option<FetchNormalizationPolicy>,
}

struct StagingFileGuard {
    path: PathBuf,
    active: bool,
}

impl StagingFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, active: true }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn cleanup(mut self) -> Result<(), String> {
        remove_staging_file(&self.path)?;
        self.active = false;
        Ok(())
    }
}

impl Drop for StagingFileGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(super) fn fetch_exact_artifact(
    cache: &Path,
    target: &Path,
    url: &str,
    policies: FetchPolicies,
    expected_sha256: &str,
    expected_bytes: u64,
    maximum_bytes: u64,
) -> Result<CacheStatus, String> {
    let download_url = match policies.redirect {
        None => url.to_owned(),
        Some(FetchRedirectPolicy::FigshareKiltHubV1) => {
            match figshare::resolve_kilthub_download(url)? {
                figshare::RedirectResolution::Ready(url) => url,
                figshare::RedirectResolution::FetchFailed => {
                    return Ok(CacheStatus::FetchFailed);
                }
                figshare::RedirectResolution::FetchToolUnavailable => {
                    return Ok(CacheStatus::FetchToolUnavailable);
                }
            }
        }
        Some(FetchRedirectPolicy::OsfStorageV1) => {
            match osf::resolve_storage_download(url, expected_sha256)? {
                osf::RedirectResolution::Ready(url) => url,
                osf::RedirectResolution::FetchFailed => return Ok(CacheStatus::FetchFailed),
                osf::RedirectResolution::FetchToolUnavailable => {
                    return Ok(CacheStatus::FetchToolUnavailable);
                }
            }
        }
    };
    if let Some(policy) = policies.normalization {
        return fetch_normalized_artifact(
            cache,
            target,
            &download_url,
            policy,
            expected_sha256,
            expected_bytes,
            maximum_bytes,
        );
    }
    let Some(curl_resolve) = resolve_public_https_endpoint(&download_url)? else {
        return Ok(CacheStatus::FetchFailed);
    };
    let sequence = NEXT_STAGING_FILE.fetch_add(1, Ordering::Relaxed);
    let staging = cache
        .join("staging")
        .join(format!("download-{}-{sequence}", std::process::id()));
    let staging = StagingFileGuard::new(staging);
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(staging.path())
        .map_err(|error| format!("create download staging file: {error}"))?;
    let mut child = match Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--proto",
            "=https",
            "--noproxy",
            "*",
            "--connect-timeout",
            "30",
            "--max-time",
            DOWNLOAD_TIMEOUT_SECONDS,
            "--resolve",
            &curl_resolve,
            "--output",
            "-",
            &download_url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            drop(output);
            staging.cleanup()?;
            return Ok(CacheStatus::FetchToolUnavailable);
        }
        Err(error) => {
            drop(output);
            staging.cleanup()?;
            return Err(format!("start bounded HTTPS fetch: {error}"));
        }
    };
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("bounded HTTPS fetch has no stdout".to_owned());
    };
    let mut hasher = Sha256::new();
    let mut byte_count = 0_u64;
    let mut buffer = [0_u8; DOWNLOAD_BUFFER_BYTES];
    loop {
        let count = match stdout.read(&mut buffer) {
            Ok(count) => count,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("read bounded HTTPS response: {error}"));
            }
        };
        if count == 0 {
            break;
        }
        let Some(next_byte_count) = byte_count.checked_add(count as u64) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("download byte count overflow".to_owned());
        };
        byte_count = next_byte_count;
        if byte_count > maximum_bytes {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(CacheStatus::DownloadLimitExceeded);
        }
        if let Err(error) = output.write_all(&buffer[..count]) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("write download staging file: {error}"));
        }
        hasher.update(&buffer[..count]);
    }
    let status = child
        .wait()
        .map_err(|error| format!("wait for bounded HTTPS fetch: {error}"))?;
    output
        .sync_all()
        .map_err(|error| format!("sync download staging file: {error}"))?;
    drop(output);
    if !status.success() {
        return Ok(CacheStatus::FetchFailed);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    let actual_sha256 = ContentHash::from_bytes(digest).to_hex();
    if byte_count != expected_bytes || actual_sha256 != expected_sha256 {
        return Err(format!(
            "download integrity mismatch: expected {expected_bytes} bytes/{expected_sha256}, got {byte_count} bytes/{actual_sha256}"
        ));
    }
    match fs::hard_link(staging.path(), target) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            verify_cache_artifact(target, expected_sha256, expected_bytes)?;
        }
        Err(error) => {
            return Err(format!("publish cached artifact: {error}"));
        }
    }
    staging.cleanup()?;
    verify_cache_artifact(target, expected_sha256, expected_bytes)?;
    Ok(CacheStatus::CachedVerified)
}

fn fetch_normalized_artifact(
    cache: &Path,
    target: &Path,
    url: &str,
    policy: FetchNormalizationPolicy,
    expected_sha256: &str,
    expected_bytes: u64,
    maximum_transfer_bytes: u64,
) -> Result<CacheStatus, String> {
    let Some(curl_resolve) = resolve_public_https_endpoint(url)? else {
        return Ok(CacheStatus::FetchFailed);
    };
    let mut child = match Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--proto",
            "=https",
            "--noproxy",
            "*",
            "--connect-timeout",
            "30",
            "--max-time",
            DOWNLOAD_TIMEOUT_SECONDS,
            "--resolve",
            &curl_resolve,
            "--output",
            "-",
            url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CacheStatus::FetchToolUnavailable);
        }
        Err(error) => return Err(format!("start bounded normalized HTTPS fetch: {error}")),
    };
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("bounded normalized HTTPS fetch has no stdout".to_owned());
    };
    let capacity = usize::try_from(maximum_transfer_bytes.min(1024 * 1024))
        .map_err(|_| "normalized transfer capacity overflow".to_owned())?;
    let mut raw = Vec::with_capacity(capacity);
    let mut buffer = [0_u8; DOWNLOAD_BUFFER_BYTES];
    loop {
        let count = match stdout.read(&mut buffer) {
            Ok(count) => count,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("read bounded normalized HTTPS response: {error}"));
            }
        };
        if count == 0 {
            break;
        }
        let next = raw
            .len()
            .checked_add(count)
            .ok_or_else(|| "normalized transfer byte count overflow".to_owned())?;
        if u64::try_from(next).map_or(true, |value| value > maximum_transfer_bytes) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(CacheStatus::DownloadLimitExceeded);
        }
        raw.extend_from_slice(&buffer[..count]);
    }
    let status = child
        .wait()
        .map_err(|error| format!("wait for bounded normalized HTTPS fetch: {error}"))?;
    if !status.success() {
        return Ok(CacheStatus::FetchFailed);
    }
    let normalized = match policy {
        FetchNormalizationPolicy::FreesoundPackIdentityV1 => {
            freesound::normalize_pack_identity(url, &raw)?
        }
    };
    let actual_bytes = u64::try_from(normalized.len())
        .map_err(|_| "normalized artifact byte count overflow".to_owned())?;
    let actual_sha256 = {
        let digest: [u8; 32] = Sha256::digest(&normalized).into();
        ContentHash::from_bytes(digest).to_hex()
    };
    if actual_bytes != expected_bytes || actual_sha256 != expected_sha256 {
        return Err(format!(
            "normalized download integrity mismatch: expected {expected_bytes} bytes/{expected_sha256}, got {actual_bytes} bytes/{actual_sha256}"
        ));
    }

    let sequence = NEXT_STAGING_FILE.fetch_add(1, Ordering::Relaxed);
    let staging = cache
        .join("staging")
        .join(format!("normalized-{}-{sequence}", std::process::id()));
    let staging = StagingFileGuard::new(staging);
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(staging.path())
        .map_err(|error| format!("create normalized staging file: {error}"))?;
    output
        .write_all(&normalized)
        .map_err(|error| format!("write normalized staging file: {error}"))?;
    output
        .sync_all()
        .map_err(|error| format!("sync normalized staging file: {error}"))?;
    drop(output);
    match fs::hard_link(staging.path(), target) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            verify_cache_artifact(target, expected_sha256, expected_bytes)?;
        }
        Err(error) => return Err(format!("publish normalized cached artifact: {error}")),
    }
    staging.cleanup()?;
    verify_cache_artifact(target, expected_sha256, expected_bytes)?;
    Ok(CacheStatus::CachedVerified)
}

pub(crate) fn resolve_public_https_endpoint(url: &str) -> Result<Option<String>, String> {
    let (host, _) = canonical_https_host_and_path(url, "remote artifact URL")?;
    let addresses = match (host, 443).to_socket_addrs() {
        Ok(addresses) => addresses.collect::<Vec<_>>(),
        Err(_) => return Ok(None),
    };
    if addresses.is_empty() {
        return Ok(None);
    }
    if addresses.iter().any(|address| !is_public_ip(address.ip())) {
        return Err(format!(
            "remote artifact host {host} resolves to a non-public address"
        ));
    }
    let mut addresses = addresses
        .into_iter()
        .map(|address| address.ip())
        .collect::<Vec<_>>();
    addresses.sort_by_key(ToString::to_string);
    addresses.dedup();
    let address = addresses[0];
    let rendered = match address {
        IpAddr::V4(address) => address.to_string(),
        IpAddr::V6(address) => format!("[{address}]"),
    };
    Ok(Some(format!("{host}:443:{rendered}")))
}

pub(super) fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let [a, b, c, _] = address.octets();
    !address.is_private()
        && !address.is_loopback()
        && !address.is_link_local()
        && !address.is_broadcast()
        && !address.is_documentation()
        && !address.is_unspecified()
        && !address.is_multicast()
        && a != 0
        && a < 224
        && !(a == 100 && (64..=127).contains(&b))
        && !(a == 192 && b == 0 && c == 0)
        && !(a == 198 && (18..=19).contains(&b))
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if let Some(address) = address.to_ipv4_mapped() {
        return is_public_ipv4(address);
    }
    let segments = address.segments();
    !address.is_loopback()
        && !address.is_unspecified()
        && !address.is_multicast()
        && (segments[0] & 0xfe00) != 0xfc00
        && (segments[0] & 0xffc0) != 0xfe80
        && (segments[0] & 0xffc0) != 0xfec0
        && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
}

fn remove_staging_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("remove download staging file: {error}"))?;
    }
    Ok(())
}
