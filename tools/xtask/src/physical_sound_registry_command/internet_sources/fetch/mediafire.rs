use std::process::{Command, Stdio};

use super::{DOWNLOAD_TIMEOUT_SECONDS, resolve_public_https_endpoint};
use crate::physical_sound_registry_command::internet_sources::canonical_https_host_and_path;

const SOURCE_HOST: &str = "www.mediafire.com";
const FILE_KEY: &str = "nxuj8iiakqecpnu";
const FILE_NAME: &str = "Glass_Recordings_by_kaffekrus.rar";
const SOURCE_PATH: &str = "file/nxuj8iiakqecpnu/Glass_Recordings_by_kaffekrus.rar/file";
const DIRECT_FILE_NAME: &str = "Glass+Recordings+by+kaffekrus.rar";
const MAX_LANDING_PAGE_BYTES: usize = 512 * 1024;
const MAX_DIRECT_URL_BYTES: usize = 2 * 1024;

pub(super) enum RedirectResolution {
    Ready(String),
    FetchFailed,
    FetchToolUnavailable,
}

pub(super) fn resolve_file_download(source_url: &str) -> Result<RedirectResolution, String> {
    validate_source_url(source_url)?;
    let Some(curl_resolve) = resolve_public_https_endpoint(source_url)? else {
        return Ok(RedirectResolution::FetchFailed);
    };
    let output = match Command::new("curl")
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
            "--max-filesize",
            &MAX_LANDING_PAGE_BYTES.to_string(),
            "--resolve",
            &curl_resolve,
            "--output",
            "-",
            source_url,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RedirectResolution::FetchToolUnavailable);
        }
        Err(error) => return Err(format!("start MediaFire landing-page probe: {error}")),
    };
    if !output.status.success() {
        return Ok(RedirectResolution::FetchFailed);
    }
    if output.stdout.len() > MAX_LANDING_PAGE_BYTES {
        return Err("MediaFire landing page exceeds its byte bound".to_owned());
    }
    let page = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("MediaFire landing page is not UTF-8: {error}"))?;
    let marker = "href=\"https://download";
    let start = page
        .find(marker)
        .map(|offset| offset + "href=\"".len())
        .ok_or_else(|| "MediaFire landing page has no direct download link".to_owned())?;
    let end = page[start..]
        .find('"')
        .map(|offset| start + offset)
        .ok_or_else(|| "MediaFire direct download link is incomplete".to_owned())?;
    let direct = &page[start..end];
    validate_direct_url(direct)?;
    Ok(RedirectResolution::Ready(direct.to_owned()))
}

fn validate_source_url(value: &str) -> Result<(), String> {
    let (host, path) = canonical_https_host_and_path(value, "MediaFire file URL")?;
    if host != SOURCE_HOST || path != SOURCE_PATH {
        return Err(format!(
            "MediaFire policy requires the canonical {FILE_NAME} landing URL"
        ));
    }
    Ok(())
}

fn validate_direct_url(value: &str) -> Result<(), String> {
    if value.len() > MAX_DIRECT_URL_BYTES
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['#', '\\', '@', '?'])
    {
        return Err("MediaFire direct URL is not bounded canonical ASCII".to_owned());
    }
    let (host, path) = canonical_https_host_and_path(value, "MediaFire direct URL")?;
    let Some(number) = host
        .strip_prefix("download")
        .and_then(|value| value.strip_suffix(".mediafire.com"))
    else {
        return Err("MediaFire direct URL has an unapproved host".to_owned());
    };
    if number.is_empty() || number.len() > 6 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("MediaFire direct URL has an invalid download shard".to_owned());
    }
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() != 3
        || parts[0].is_empty()
        || parts[0].len() > 512
        || !parts[0]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || parts[1] != FILE_KEY
        || parts[2] != DIRECT_FILE_NAME
    {
        return Err("MediaFire direct URL does not match the frozen file identity".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_URL: &str =
        "https://www.mediafire.com/file/nxuj8iiakqecpnu/Glass_Recordings_by_kaffekrus.rar/file";

    fn direct_url() -> String {
        "https://download1323.mediafire.com/abc_DEF-012/nxuj8iiakqecpnu/Glass+Recordings+by+kaffekrus.rar".to_owned()
    }

    #[test]
    fn accepts_only_the_frozen_landing_and_direct_file_shape() {
        assert!(validate_source_url(SOURCE_URL).is_ok());
        assert!(validate_source_url("https://www.mediafire.com/file/other/file.rar/file").is_err());
        assert!(validate_direct_url(&direct_url()).is_ok());
        assert!(validate_direct_url(&direct_url().replace(FILE_KEY, "other")).is_err());
        assert!(
            validate_direct_url(&direct_url().replace("mediafire.com", "example.com")).is_err()
        );
    }
}
