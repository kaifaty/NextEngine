use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use goblin::elf::{Elf, header as elf_header};
use goblin::pe::{PE, header as pe_header};

use super::{
    PackageBinaryRuntimeV3, PackageExternalPrerequisiteV3, PackageRuntimeAbiV3,
    PackageRuntimeProfileV3, PackageWindowsCrtV3,
};

const WINDOWS_TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";
const LINUX_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const LINUX_MINIMUM_GLIBC: &str = "2.35";
const LINUX_X86_64_INTERPRETER: &str = "/lib64/ld-linux-x86-64.so.2";
const MAX_AUDITED_BINARY_BYTES: u64 = 512 * 1024 * 1024;

pub(super) fn build_runtime_profile(
    package_root: &Path,
    target_triple: &str,
    binary_paths: &[&str],
) -> Result<PackageRuntimeProfileV3, String> {
    let abi = expected_abi(target_triple)?;
    let mut binaries = binary_paths
        .iter()
        .map(|binary_path| {
            audit_binary(
                &package_root.join(binary_path.replace('/', std::path::MAIN_SEPARATOR_STR)),
                binary_path,
                target_triple,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    binaries.sort_by(|left, right| left.binary_path.cmp(&right.binary_path));

    Ok(PackageRuntimeProfileV3 {
        abi,
        binaries,
        external_prerequisites: expected_external_prerequisites(target_triple)?,
    })
}

pub(super) fn validate_runtime_profile(
    package_root: &Path,
    target_triple: &str,
    profile: &PackageRuntimeProfileV3,
    binary_paths: &[&str],
) -> Result<(), String> {
    let expected = build_runtime_profile(package_root, target_triple, binary_paths)?;
    if profile.abi != expected.abi {
        return runtime_profile_error("runtime ABI does not match the package target baseline");
    }
    if profile.external_prerequisites != expected.external_prerequisites {
        return runtime_profile_error(
            "external prerequisite list is incomplete, non-canonical, or target-incompatible",
        );
    }
    if profile.binaries != expected.binaries {
        return runtime_profile_error(
            "declared binary runtime dependencies do not match packaged binaries",
        );
    }
    Ok(())
}

fn expected_abi(target_triple: &str) -> Result<PackageRuntimeAbiV3, String> {
    match target_triple {
        WINDOWS_TARGET_TRIPLE => Ok(PackageRuntimeAbiV3::WindowsMsvcX64 {
            crt: PackageWindowsCrtV3::DynamicSystem,
        }),
        LINUX_TARGET_TRIPLE => Ok(PackageRuntimeAbiV3::LinuxGnuX64 {
            minimum_glibc: LINUX_MINIMUM_GLIBC.to_owned(),
        }),
        _ => runtime_abi_error(format!("unsupported package target {target_triple}")),
    }
}

fn expected_external_prerequisites(
    target_triple: &str,
) -> Result<Vec<PackageExternalPrerequisiteV3>, String> {
    let mut prerequisites = match target_triple {
        WINDOWS_TARGET_TRIPLE => vec![
            prerequisite(
                "msvc-runtime-x64",
                "vcruntime140.dll",
                "Microsoft Visual C++ 2015-2022 Redistributable x64 dynamic system CRT",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
            prerequisite(
                "vulkan-icd",
                r"registry:HKLM\SOFTWARE\Khronos\Vulkan\Drivers",
                "system GPU driver exposing a Vulkan 1.3-capable x86_64 physical device",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
            prerequisite(
                "vulkan-loader",
                "vulkan-1.dll",
                "system Vulkan loader exposing Vulkan API 1.3",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
        ],
        LINUX_TARGET_TRIPLE => vec![
            prerequisite(
                "desktop-session",
                "DISPLAY|WAYLAND_DISPLAY",
                "active X11 or Wayland desktop session",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
            prerequisite(
                "glibc",
                "libc.so.6",
                "GNU libc 2.35 or newer",
                "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED",
            ),
            prerequisite(
                "vulkan-icd",
                "/usr/share/vulkan/icd.d/*.json",
                "system GPU driver exposing a Vulkan 1.3-capable x86_64 physical device",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
            prerequisite(
                "vulkan-loader",
                "libvulkan.so.1",
                "system Vulkan loader exposing Vulkan API 1.3",
                "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING",
            ),
        ],
        _ => return runtime_abi_error(format!("unsupported package target {target_triple}")),
    };
    prerequisites.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(prerequisites)
}

fn prerequisite(
    id: &str,
    locator: &str,
    requirement: &str,
    diagnostic_code: &str,
) -> PackageExternalPrerequisiteV3 {
    PackageExternalPrerequisiteV3 {
        diagnostic_code: diagnostic_code.to_owned(),
        id: id.to_owned(),
        locator: locator.to_owned(),
        requirement: requirement.to_owned(),
    }
}

fn audit_binary(
    binary: &Path,
    binary_path: &str,
    target_triple: &str,
) -> Result<PackageBinaryRuntimeV3, String> {
    let metadata = fs::metadata(binary).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING: failed to inspect {}: {error}",
            binary.display()
        )
    })?;
    if !metadata.is_file() {
        return runtime_dependency_error(format!("{} is not a regular binary", binary.display()));
    }
    if metadata.len() > MAX_AUDITED_BINARY_BYTES {
        return runtime_profile_error(format!(
            "{} has {} bytes; runtime audit limit is {MAX_AUDITED_BINARY_BYTES}",
            binary.display(),
            metadata.len()
        ));
    }
    let bytes = fs::read(binary).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING: failed to read {}: {error}",
            binary.display()
        )
    })?;
    match target_triple {
        WINDOWS_TARGET_TRIPLE => audit_pe64(&bytes, binary_path),
        LINUX_TARGET_TRIPLE => audit_elf64(&bytes, binary_path),
        _ => runtime_abi_error(format!("unsupported package target {target_triple}")),
    }
}

fn audit_pe64(bytes: &[u8], binary_path: &str) -> Result<PackageBinaryRuntimeV3, String> {
    let pe = PE::parse(bytes).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED: {binary_path} is not valid PE64: {error}"
        )
    })?;
    if !pe.is_64 || pe.header.coff_header.machine != pe_header::COFF_MACHINE_X86_64 {
        return runtime_abi_error(format!("{binary_path} is not an x86_64 PE32+ executable"));
    }
    let optional_header = pe.header.optional_header.as_ref().ok_or_else(|| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED: {binary_path} has no PE optional header"
        )
    })?;
    validate_pe_delay_imports(
        binary_path,
        optional_header
            .data_directories
            .get_delay_import_descriptor()
            .is_some(),
    )?;

    let mut direct_libraries = pe
        .libraries
        .iter()
        .map(|library| library.to_ascii_lowercase())
        .collect::<Vec<_>>();
    direct_libraries.sort();
    direct_libraries.dedup();
    validate_direct_libraries(binary_path, WINDOWS_TARGET_TRIPLE, &direct_libraries)?;

    Ok(PackageBinaryRuntimeV3 {
        binary_path: binary_path.to_owned(),
        direct_libraries,
        maximum_required_glibc: None,
    })
}

fn validate_pe_delay_imports(binary_path: &str, delay_imports_present: bool) -> Result<(), String> {
    if delay_imports_present {
        runtime_profile_error(format!(
            "{binary_path} declares PE delay imports, which are not supported by the R1 dependency auditor"
        ))
    } else {
        Ok(())
    }
}

fn audit_elf64(bytes: &[u8], binary_path: &str) -> Result<PackageBinaryRuntimeV3, String> {
    let elf = Elf::parse(bytes).map_err(|error| {
        format!(
            "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED: {binary_path} is not valid ELF64: {error}"
        )
    })?;
    if !elf.is_64 || elf.header.e_machine != elf_header::EM_X86_64 {
        return runtime_abi_error(format!("{binary_path} is not an x86_64 ELF64 executable"));
    }
    validate_elf_layout(binary_path, elf.interpreter, &elf.rpaths, &elf.runpaths)?;

    let mut direct_libraries = elf
        .libraries
        .iter()
        .map(|library| (*library).to_owned())
        .collect::<Vec<_>>();
    direct_libraries.sort();
    direct_libraries.dedup();
    validate_direct_libraries(binary_path, LINUX_TARGET_TRIPLE, &direct_libraries)?;

    let maximum_required_glibc = maximum_required_glibc(&elf)?;
    let Some(maximum_required_glibc) = maximum_required_glibc else {
        return runtime_abi_error(format!(
            "{binary_path} does not declare a GLIBC symbol-version requirement"
        ));
    };
    validate_glibc_baseline(binary_path, &maximum_required_glibc)?;

    Ok(PackageBinaryRuntimeV3 {
        binary_path: binary_path.to_owned(),
        direct_libraries,
        maximum_required_glibc: Some(maximum_required_glibc),
    })
}

fn validate_direct_libraries(
    binary_path: &str,
    target_triple: &str,
    direct_libraries: &[String],
) -> Result<(), String> {
    for library in direct_libraries {
        if is_sdl_dynamic_library(library) {
            return runtime_profile_error(format!(
                "{binary_path} dynamically imports forbidden {library}; SDL3 must remain static"
            ));
        }
        let classified = match target_triple {
            WINDOWS_TARGET_TRIPLE => is_windows_runtime_library(library),
            LINUX_TARGET_TRIPLE => is_linux_runtime_library(library),
            _ => return runtime_abi_error(format!("unsupported package target {target_triple}")),
        };
        if !classified {
            return runtime_profile_error(format!(
                "{binary_path} imports unclassified runtime library {library}"
            ));
        }
    }
    if target_triple == WINDOWS_TARGET_TRIPLE
        && !direct_libraries
            .iter()
            .any(|library| is_windows_crt_library(library))
    {
        return runtime_abi_error(format!(
            "{binary_path} does not import the declared dynamic system CRT"
        ));
    }
    Ok(())
}

fn validate_elf_layout(
    binary_path: &str,
    interpreter: Option<&str>,
    rpaths: &[&str],
    runpaths: &[&str],
) -> Result<(), String> {
    if interpreter != Some(LINUX_X86_64_INTERPRETER) {
        return runtime_abi_error(format!(
            "{binary_path} uses unsupported ELF interpreter {interpreter:?}; expected {LINUX_X86_64_INTERPRETER}"
        ));
    }
    if !rpaths.is_empty() || !runpaths.is_empty() {
        return runtime_profile_error(format!(
            "{binary_path} declares RPATH/RUNPATH; packaged binaries must use only system lookup"
        ));
    }
    Ok(())
}

fn validate_glibc_baseline(binary_path: &str, maximum_required_glibc: &str) -> Result<(), String> {
    if compare_versions(maximum_required_glibc, LINUX_MINIMUM_GLIBC)? == Ordering::Greater {
        return runtime_abi_error(format!(
            "{binary_path} requires GLIBC_{maximum_required_glibc}, newer than the {LINUX_MINIMUM_GLIBC} package baseline"
        ));
    }
    Ok(())
}

fn maximum_required_glibc(elf: &Elf<'_>) -> Result<Option<String>, String> {
    let mut maximum: Option<String> = None;
    if let Some(verneed) = &elf.verneed {
        for need_file in verneed.iter() {
            for need_version in need_file.iter() {
                let name = elf.dynstrtab.get_at(need_version.vna_name).ok_or_else(|| {
                    format!(
                        "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID: invalid ELF version-need string index {}",
                        need_version.vna_name
                    )
                })?;
                consider_glibc_requirement(&mut maximum, name)?;
            }
        }
    }
    Ok(maximum)
}

fn consider_glibc_requirement(
    maximum: &mut Option<String>,
    requirement: &str,
) -> Result<(), String> {
    let Some(version) = requirement.strip_prefix("GLIBC_") else {
        return Ok(());
    };
    parse_version(version)?;
    if maximum
        .as_deref()
        .is_none_or(|current| compare_versions(version, current) == Ok(Ordering::Greater))
    {
        *maximum = Some(version.to_owned());
    }
    Ok(())
}

fn compare_versions(left: &str, right: &str) -> Result<Ordering, String> {
    Ok(parse_version(left)?.cmp(&parse_version(right)?))
}

fn parse_version(value: &str) -> Result<Vec<u32>, String> {
    let components = value
        .split('.')
        .map(|component| {
            if component.is_empty()
                || (component.len() > 1 && component.starts_with('0'))
                || !component.bytes().all(|byte| byte.is_ascii_digit())
            {
                return runtime_profile_error(format!(
                    "invalid canonical runtime version {value}"
                ));
            }
            component.parse::<u32>().map_err(|error| {
                format!(
                    "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID: invalid runtime version {value}: {error}"
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if components.len() < 2 {
        return runtime_profile_error(format!(
            "runtime version {value} must contain major and minor components"
        ));
    }
    Ok(components)
}

fn is_sdl_dynamic_library(library: &str) -> bool {
    let lower = library.to_ascii_lowercase();
    lower == "sdl3.dll" || lower.starts_with("libsdl3.so")
}

fn is_windows_runtime_library(library: &str) -> bool {
    if library.starts_with("api-ms-win-") || library.starts_with("ext-ms-win-") {
        return true;
    }
    matches!(
        library,
        "advapi32.dll"
            | "avrt.dll"
            | "bcrypt.dll"
            | "cfgmgr32.dll"
            | "combase.dll"
            | "concrt140.dll"
            | "crypt32.dll"
            | "d3d11.dll"
            | "d3d12.dll"
            | "d3dcompiler_47.dll"
            | "dcomp.dll"
            | "dinput8.dll"
            | "dwmapi.dll"
            | "dxgi.dll"
            | "gameinput.dll"
            | "gdi32.dll"
            | "hid.dll"
            | "imm32.dll"
            | "kernel32.dll"
            | "msvcp140.dll"
            | "msvcp140_1.dll"
            | "msvcp140_2.dll"
            | "msvcrt.dll"
            | "ntdll.dll"
            | "ole32.dll"
            | "oleaut32.dll"
            | "opengl32.dll"
            | "powrprof.dll"
            | "propsys.dll"
            | "rpcrt4.dll"
            | "sechost.dll"
            | "setupapi.dll"
            | "shell32.dll"
            | "shlwapi.dll"
            | "ucrtbase.dll"
            | "user32.dll"
            | "userenv.dll"
            | "uxtheme.dll"
            | "vcruntime140.dll"
            | "vcruntime140_1.dll"
            | "version.dll"
            | "vulkan-1.dll"
            | "winmm.dll"
            | "wintrust.dll"
            | "ws2_32.dll"
            | "wtsapi32.dll"
            | "xinput1_4.dll"
    )
}

fn is_windows_crt_library(library: &str) -> bool {
    library.starts_with("api-ms-win-crt-")
        || matches!(
            library,
            "concrt140.dll"
                | "msvcp140.dll"
                | "msvcp140_1.dll"
                | "msvcp140_2.dll"
                | "msvcrt.dll"
                | "ucrtbase.dll"
                | "vcruntime140.dll"
                | "vcruntime140_1.dll"
        )
}

fn is_linux_runtime_library(library: &str) -> bool {
    matches!(
        library,
        "ld-linux-x86-64.so.2"
            | "libX11.so.6"
            | "libXcursor.so.1"
            | "libXext.so.6"
            | "libXfixes.so.3"
            | "libXi.so.6"
            | "libXrandr.so.2"
            | "libXss.so.1"
            | "libasound.so.2"
            | "libatomic.so.1"
            | "libc.so.6"
            | "libdbus-1.so.3"
            | "libdecor-0.so.0"
            | "libdl.so.2"
            | "libgcc_s.so.1"
            | "libjack.so.0"
            | "libm.so.6"
            | "libpthread.so.0"
            | "libpulse.so.0"
            | "librt.so.1"
            | "libsndio.so.7"
            | "libstdc++.so.6"
            | "libudev.so.1"
            | "libutil.so.1"
            | "libvulkan.so.1"
            | "libwayland-client.so.0"
            | "libwayland-egl.so.1"
            | "libxcb.so.1"
            | "libxkbcommon.so.0"
    )
}

fn runtime_profile_error<T>(message: impl std::fmt::Display) -> Result<T, String> {
    Err(format!(
        "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID: {message}"
    ))
}

fn runtime_dependency_error<T>(message: impl std::fmt::Display) -> Result<T, String> {
    Err(format!(
        "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING: {message}"
    ))
}

fn runtime_abi_error<T>(message: impl std::fmt::Display) -> Result<T, String> {
    Err(format!(
        "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED: {message}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_library_classification_rejects_sdl_and_unknown_dependencies() {
        for (target, library) in [
            (WINDOWS_TARGET_TRIPLE, "SDL3.dll"),
            (WINDOWS_TARGET_TRIPLE, "vendor-renderer.dll"),
            (LINUX_TARGET_TRIPLE, "libSDL3.so.0"),
            (LINUX_TARGET_TRIPLE, "libvendor-renderer.so.1"),
        ] {
            let error = validate_direct_libraries("bin/fixture", target, &[library.to_owned()])
                .expect_err("dependency must be rejected");
            assert!(error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID:"));
        }
        validate_direct_libraries(
            "bin/fixture.exe",
            WINDOWS_TARGET_TRIPLE,
            &[
                "api-ms-win-crt-runtime-l1-1-0.dll".to_owned(),
                "kernel32.dll".to_owned(),
                "vcruntime140.dll".to_owned(),
            ],
        )
        .expect("Windows runtime dependencies");
        assert!(
            validate_direct_libraries(
                "bin/fixture.exe",
                WINDOWS_TARGET_TRIPLE,
                &["kernel32.dll".to_owned()],
            )
            .expect_err("dynamic CRT declaration must be evidenced")
            .starts_with("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED:")
        );
        validate_direct_libraries(
            "bin/fixture",
            LINUX_TARGET_TRIPLE,
            &["libc.so.6".to_owned(), "libgcc_s.so.1".to_owned()],
        )
        .expect("Linux runtime dependencies");
    }

    #[test]
    fn pe_audit_fails_closed_when_delay_import_directory_is_present() {
        validate_pe_delay_imports("bin/fixture.exe", false).expect("no delay imports");
        let error = validate_pe_delay_imports("bin/fixture.exe", true)
            .expect_err("delay imports must fail closed");
        assert!(error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID:"));
    }

    #[test]
    fn elf_layout_rejects_nonstandard_interpreter_and_search_paths() {
        validate_elf_layout("bin/fixture", Some(LINUX_X86_64_INTERPRETER), &[], &[])
            .expect("canonical ELF layout");
        assert!(
            validate_elf_layout("bin/fixture", Some("/tmp/ld-linux.so"), &[], &[])
                .expect_err("interpreter")
                .starts_with("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED:")
        );
        assert!(
            validate_elf_layout(
                "bin/fixture",
                Some(LINUX_X86_64_INTERPRETER),
                &["/developer/lib"],
                &[],
            )
            .expect_err("rpath")
            .starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID:")
        );
        assert!(
            validate_elf_layout(
                "bin/fixture",
                Some(LINUX_X86_64_INTERPRETER),
                &[],
                &["$ORIGIN"],
            )
            .expect_err("runpath")
            .starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID:")
        );
    }

    #[test]
    fn glibc_baseline_rejects_versions_newer_than_ubuntu_2204() {
        validate_glibc_baseline("bin/fixture", "2.35").expect("baseline");
        validate_glibc_baseline("bin/fixture", "2.17").expect("older-compatible binary");
        let error =
            validate_glibc_baseline("bin/fixture", "2.36").expect_err("newer GLIBC rejected");
        assert!(error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED:"));
        assert_eq!(
            compare_versions("2.35.1", "2.35").expect("version comparison"),
            Ordering::Greater
        );
        let mut maximum = None;
        consider_glibc_requirement(&mut maximum, "GLIBC_2.34").expect("numeric GLIBC version");
        assert_eq!(maximum.as_deref(), Some("2.34"));
        assert!(
            consider_glibc_requirement(&mut maximum, "GLIBC_PRIVATE")
                .expect_err("malformed GLIBC requirement must fail closed")
                .starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID:")
        );
    }

    #[test]
    fn runtime_profile_prerequisites_are_target_specific_and_canonical() {
        let windows =
            expected_external_prerequisites(WINDOWS_TARGET_TRIPLE).expect("Windows profile");
        let linux = expected_external_prerequisites(LINUX_TARGET_TRIPLE).expect("Linux profile");
        assert!(windows.windows(2).all(|pair| pair[0].id < pair[1].id));
        assert!(linux.windows(2).all(|pair| pair[0].id < pair[1].id));
        assert!(windows.iter().any(|entry| entry.locator == "vulkan-1.dll"));
        assert!(linux.iter().any(|entry| entry.locator == "libvulkan.so.1"));
        assert!(linux.iter().any(|entry| entry.id == "desktop-session"));
    }
}
