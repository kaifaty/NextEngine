use std::process::{Command, Stdio};

use super::{DOWNLOAD_TIMEOUT_SECONDS, resolve_public_https_endpoint};
use crate::physical_sound_registry_command::internet_sources::canonical_https_host_and_path;

const OSF_STORAGE_HOST: &str = "files.de-1.osf.io";
const OSF_SIGNED_PREFIX: &str = "https://storage.googleapis.com/cos-osf-prod-files-de-1/";
const MAX_REDIRECT_BYTES: usize = 8 * 1024;

pub(super) enum RedirectResolution {
    Ready(String),
    FetchFailed,
    FetchToolUnavailable,
}

pub(super) fn resolve_storage_download(
    source_url: &str,
    expected_sha256: &str,
) -> Result<RedirectResolution, String> {
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
        Err(error) => return Err(format!("start OSF redirect probe: {error}")),
    };
    if !output.status.success() {
        return Ok(RedirectResolution::FetchFailed);
    }
    if output.stdout.len() > MAX_REDIRECT_BYTES {
        return Err("OSF redirect exceeds its byte bound".to_owned());
    }
    let response = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("OSF redirect is not UTF-8: {error}"))?;
    let (status, redirect) = response
        .split_once('\n')
        .ok_or_else(|| "OSF redirect probe returned no status delimiter".to_owned())?;
    if status != "302" || redirect.is_empty() {
        return Ok(RedirectResolution::FetchFailed);
    }
    validate_signed_url(redirect, expected_sha256)?;
    Ok(RedirectResolution::Ready(redirect.to_owned()))
}

fn validate_source_url(value: &str) -> Result<(), String> {
    let (host, path) = canonical_https_host_and_path(value, "OSF storage URL")?;
    let components = path.split('/').collect::<Vec<_>>();
    if host != OSF_STORAGE_HOST
        || components.len() != 6
        || components[0] != "v1"
        || components[1] != "resources"
        || !is_osf_node_id(components[2])
        || components[3] != "providers"
        || components[4] != "osfstorage"
        || !is_lower_hex(components[5], 24)
    {
        return Err("OSF redirect policy requires one canonical de-1 storage URL".to_owned());
    }
    Ok(())
}

fn validate_signed_url(value: &str, expected_sha256: &str) -> Result<(), String> {
    if value.len() > MAX_REDIRECT_BYTES
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.contains(['#', '\\', '@'])
    {
        return Err("OSF signed redirect is not bounded canonical ASCII".to_owned());
    }
    let remainder = value
        .strip_prefix(OSF_SIGNED_PREFIX)
        .ok_or_else(|| "OSF redirect target is not the approved storage bucket".to_owned())?;
    let (object_name, query) = remainder
        .split_once('?')
        .ok_or_else(|| "OSF redirect target has no signed query".to_owned())?;
    if object_name != expected_sha256 || !is_lower_hex(object_name, 64) {
        return Err("OSF redirect object does not match the expected SHA-256".to_owned());
    }
    let parameters = query.split('&').collect::<Vec<_>>();
    if parameters.len() != 4 {
        return Err("OSF signed redirect must contain exactly four parameters".to_owned());
    }
    let disposition = parameter(parameters[0], "response-content-disposition")?;
    let access_id = parameter(parameters[1], "GoogleAccessId")?;
    let expires = parameter(parameters[2], "Expires")?;
    let signature = parameter(parameters[3], "Signature")?;
    if disposition.is_empty()
        || disposition.len() > 512
        || !disposition.starts_with("attachment%3B%20filename%3D")
        || !bounded_query_value(disposition)
        || access_id != "files-de-1%40cos-osf-prod.iam.gserviceaccount.com"
        || expires.is_empty()
        || expires.len() > 20
        || !expires.bytes().all(|byte| byte.is_ascii_digit())
        || signature.len() < 128
        || signature.len() > 2_048
        || !bounded_query_value(signature)
    {
        return Err("OSF signed redirect parameters do not match the approved shape".to_owned());
    }
    Ok(())
}

fn parameter<'a>(value: &'a str, name: &str) -> Result<&'a str, String> {
    value
        .strip_prefix(name)
        .and_then(|value| value.strip_prefix('='))
        .ok_or_else(|| format!("OSF signed redirect is missing {name}"))
}

fn bounded_query_value(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"%._-*'".contains(&byte))
}

fn is_osf_node_id(value: &str) -> bool {
    value.len() == 5
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
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

    const SHA256: &str = "e2541a173879805eb718d9916c2b97551116970694f1b85138c1968f955673cd";

    #[test]
    fn accepts_only_canonical_osf_storage_origins() {
        assert!(
            validate_source_url(
                "https://files.de-1.osf.io/v1/resources/bj5w8/providers/osfstorage/622a10751e399c0b18602130"
            )
            .is_ok()
        );
        assert!(
            validate_source_url(
                "https://osf.io/v1/resources/bj5w8/providers/osfstorage/622a10751e399c0b18602130"
            )
            .is_err()
        );
    }

    #[test]
    fn signed_target_is_bound_to_the_expected_hash_and_bucket() {
        let url = format!(
            "{OSF_SIGNED_PREFIX}{SHA256}?response-content-disposition=attachment%3B%20filename%3D%22Clip_4.ogg%22&GoogleAccessId=files-de-1%40cos-osf-prod.iam.gserviceaccount.com&Expires=1787850644&Signature={}",
            "A".repeat(128)
        );
        assert!(validate_signed_url(&url, SHA256).is_ok());
        assert!(validate_signed_url(&url, &"a".repeat(64)).is_err());
        assert!(
            validate_signed_url(
                &url.replace("storage.googleapis.com", "example.org"),
                SHA256
            )
            .is_err()
        );
    }
}
