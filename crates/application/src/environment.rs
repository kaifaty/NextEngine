use std::path::PathBuf;

use crate::ApplicationError;

pub fn default_user_state_root(application_id: &str) -> Result<PathBuf, ApplicationError> {
    if application_id.is_empty()
        || !application_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ApplicationError::StateRootUnavailable);
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| path.join("Library").join("Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|path| path.join(".local").join("state"))
        });
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    let base: Option<PathBuf> = None;

    base.map(|path| path.join("next-engine").join(application_id))
        .ok_or(ApplicationError::StateRootUnavailable)
}
