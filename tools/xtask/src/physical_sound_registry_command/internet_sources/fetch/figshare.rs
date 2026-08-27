use std::process::{Command, Stdio};

use super::{DOWNLOAD_TIMEOUT_SECONDS, resolve_public_https_endpoint};
use crate::physical_sound_registry_command::internet_sources::canonical_https_host_and_path;

const FIGSHARE_DOWNLOAD_HOST: &str = "ndownloader.figshare.com";
const KILTHUB_STORAGE_PREFIX: &str =
    "https://s3-eu-west-1.amazonaws.com/pstorage-cmu-348901238291901/";
const MAX_REDIRECT_BYTES: usize = 8 * 1024;

pub(super) enum RedirectResolution {
    Ready(String),
    FetchFailed,
    FetchToolUnavailable,
}

pub(super) fn resolve_kilthub_download(source_url: &str) -> Result<RedirectResolution, String> {
    let file_id = validate_source_url(source_url)?;
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
            "--resolve",
            &curl_resolve,
            "--output",
            "/dev/null",
            "--write-out",
            "%{http_code}\n%{redirect_url}",
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
        Err(error) => return Err(format!("start Figshare redirect probe: {error}")),
    };
    if !output.status.success() {
        return Ok(RedirectResolution::FetchFailed);
    }
    if output.stdout.len() > MAX_REDIRECT_BYTES {
        return Err("Figshare redirect exceeds its byte bound".to_owned());
    }
    let response = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("Figshare redirect is not UTF-8: {error}"))?;
    let (status, redirect) = response
        .split_once('\n')
        .ok_or_else(|| "Figshare redirect probe returned no status delimiter".to_owned())?;
    if status != "302" || redirect.is_empty() {
        return Ok(RedirectResolution::FetchFailed);
    }
    validate_signed_url(redirect, file_id)?;
    Ok(RedirectResolution::Ready(redirect.to_owned()))
}

fn validate_source_url(value: &str) -> Result<&str, String> {
    let (host, path) = canonical_https_host_and_path(value, "Figshare download URL")?;
    let Some(file_id) = path.strip_prefix("files/") else {
        return Err("Figshare redirect policy requires one canonical file URL".to_owned());
    };
    if host != FIGSHARE_DOWNLOAD_HOST
        || file_id.is_empty()
        || file_id.len() > 20
        || file_id.starts_with('0')
        || !file_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("Figshare redirect policy requires one canonical file URL".to_owned());
    }
    Ok(file_id)
}

fn validate_signed_url(value: &str, file_id: &str) -> Result<(), String> {
    if value.len() > MAX_REDIRECT_BYTES
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['#', '\\', '@'])
    {
        return Err("Figshare signed redirect is not bounded canonical ASCII".to_owned());
    }
    let remainder = value
        .strip_prefix(KILTHUB_STORAGE_PREFIX)
        .ok_or_else(|| "Figshare redirect target is not the approved KiltHub bucket".to_owned())?;
    let (object_path, query) = remainder
        .split_once('?')
        .ok_or_else(|| "Figshare redirect target has no signed query".to_owned())?;
    let (target_file_id, file_name) = object_path
        .split_once('/')
        .ok_or_else(|| "Figshare redirect target has no file name".to_owned())?;
    if target_file_id != file_id || !bounded_file_name(file_name) {
        return Err("Figshare redirect object does not match the requested file".to_owned());
    }
    let parameters = query.split('&').collect::<Vec<_>>();
    if parameters.len() != 6 {
        return Err("Figshare signed redirect must contain exactly six parameters".to_owned());
    }
    let algorithm = parameter(parameters[0], "X-Amz-Algorithm")?;
    let credential = parameter(parameters[1], "X-Amz-Credential")?;
    let date = parameter(parameters[2], "X-Amz-Date")?;
    let expires = parameter(parameters[3], "X-Amz-Expires")?;
    let signed_headers = parameter(parameters[4], "X-Amz-SignedHeaders")?;
    let signature = parameter(parameters[5], "X-Amz-Signature")?;
    if algorithm != "AWS4-HMAC-SHA256"
        || !valid_credential(credential, date)
        || !valid_amz_date(date)
        || expires != "10"
        || signed_headers != "host"
        || !is_lower_hex(signature, 64)
    {
        return Err(
            "Figshare signed redirect parameters do not match the approved shape".to_owned(),
        );
    }
    Ok(())
}

fn parameter<'a>(value: &'a str, name: &str) -> Result<&'a str, String> {
    value
        .strip_prefix(name)
        .and_then(|value| value.strip_prefix('='))
        .ok_or_else(|| format!("Figshare signed redirect is missing {name}"))
}

fn bounded_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_credential(value: &str, date: &str) -> bool {
    let parts = value.split('/').collect::<Vec<_>>();
    parts.len() == 5
        && parts[0].len() == 20
        && parts[0]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        && parts[1].len() == 8
        && date.starts_with(parts[1])
        && parts[2] == "eu-west-1"
        && parts[3] == "s3"
        && parts[4] == "aws4_request"
}

fn valid_amz_date(value: &str) -> bool {
    value.len() == 16
        && value.as_bytes()[8] == b'T'
        && value.as_bytes()[15] == b'Z'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 15) || byte.is_ascii_digit())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGNATURE: &str = "e61c1b9944a73a5d6b120548d19572d95f26365de8f5835dd95d7cf83ee8b1ca";

    fn signed_url() -> String {
        format!(
            "{KILTHUB_STORAGE_PREFIX}36113411/Impacts_audio1.zip?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAI266R7V6O36O5JUA/20260827/eu-west-1/s3/aws4_request&X-Amz-Date=20260827T173753Z&X-Amz-Expires=10&X-Amz-SignedHeaders=host&X-Amz-Signature={SIGNATURE}"
        )
    }

    #[test]
    fn accepts_only_canonical_figshare_download_origins() {
        assert_eq!(
            validate_source_url("https://ndownloader.figshare.com/files/36113411")
                .expect("canonical Figshare URL"),
            "36113411"
        );
        assert!(validate_source_url("https://figshare.com/files/36113411").is_err());
        assert!(validate_source_url("https://ndownloader.figshare.com/files/../36113411").is_err());
    }

    #[test]
    fn signed_target_is_bound_to_kilthub_bucket_and_file_id() {
        let url = signed_url();
        assert!(validate_signed_url(&url, "36113411").is_ok());
        assert!(validate_signed_url(&url, "36113402").is_err());
        assert!(
            validate_signed_url(&url.replace("pstorage-cmu", "pstorage-other"), "36113411")
                .is_err()
        );
        assert!(
            validate_signed_url(
                &url.replace("X-Amz-Expires=10", "X-Amz-Expires=60"),
                "36113411"
            )
            .is_err()
        );
    }
}
