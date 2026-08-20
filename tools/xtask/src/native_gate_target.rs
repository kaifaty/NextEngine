use xtask::native_gate::LINUX_TARGET_TRIPLE;

fn shipping_target_triple() -> Result<&'static str, String> {
    if cfg!(all(
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )) {
        Ok("x86_64-unknown-linux-gnu")
    } else {
        Err("TARGET_PACKAGE_REQUIRES_NATIVE_LINUX_X86_64_GNU".to_owned())
    }
}

pub(crate) fn native_shipping_target_for_host(rustc_host: &str) -> Result<String, String> {
    let target_triple = shipping_target_triple()
        .map_err(|_| {
            format!(
                "NATIVE_GATE_UNSUPPORTED_TARGET: native gate release evidence requires x86_64 Linux GNU, got {rustc_host}"
            )
        })?
        .to_owned();
    if rustc_host != target_triple {
        return Err(format!(
            "NATIVE_GATE_UNSUPPORTED_TARGET: rustc host {rustc_host} does not match native target {target_triple}"
        ));
    }
    if target_triple == LINUX_TARGET_TRIPLE && current_host_is_wsl() {
        return Err(
            "NATIVE_GATE_UNSUPPORTED_TARGET: WSL does not provide native Linux host evidence"
                .to_owned(),
        );
    }
    Ok(target_triple)
}

#[cfg(target_os = "linux")]
fn current_host_is_wsl() -> bool {
    let os_release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    wsl_markers_present(
        std::env::var_os("WSL_INTEROP").is_some(),
        std::env::var_os("WSL_DISTRO_NAME").is_some(),
        &os_release,
    )
}

#[cfg(not(target_os = "linux"))]
const fn current_host_is_wsl() -> bool {
    false
}

#[cfg(any(target_os = "linux", test))]
pub(crate) fn wsl_markers_present(
    has_wsl_interop: bool,
    has_wsl_distro_name: bool,
    os_release: &str,
) -> bool {
    has_wsl_interop || has_wsl_distro_name || os_release.to_ascii_lowercase().contains("microsoft")
}
