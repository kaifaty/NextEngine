#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use sha2::{Digest, Sha256};

pub const PHYSX_VERSION: &str = "5.9.0";
pub const PHYSX_SOURCE_REVISION: &str = "517a0073715120e114ee055b63b26c95e00d9039";
pub const PHYSX_ARCHIVE_SHA256: &str =
    "bc894626070f0658a3235231c825c3e0f8ad1fd9d5077a63cb5bdaffe8816407";
pub const PHYSX_ARCHIVE_URL: &str = "https://github.com/NVIDIA-Omniverse/PhysX/archive/517a0073715120e114ee055b63b26c95e00d9039.tar.gz";
pub const PHYSX_BUILD_PROFILE: &str = "nextengine-physx-5.9.0-static-cpu-release-v1";
pub const PHYSX_BRIDGE_ABI: u32 = 4;

const MANIFEST_NAME: &str = "nextengine-physx-sdk-v1.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysxCommand {
    Setup,
    Doctor,
    CleanCache,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct PhysxSdkManifestV1 {
    schema: &'static str,
    physx_version: &'static str,
    source_revision: &'static str,
    archive_sha256: &'static str,
    target_triple: String,
    compiler_profile: String,
    build_profile: &'static str,
    bridge_abi: u32,
    profile_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct PhysxCommandReportV1 {
    command: &'static str,
    status: &'static str,
    version: &'static str,
    target: String,
    profile_hash: String,
    sdk_dir: String,
}

pub fn parse_command(mut arguments: impl Iterator<Item = String>) -> Result<PhysxCommand, String> {
    let subcommand = arguments
        .next()
        .ok_or_else(|| "physx requires setup, doctor or clean-cache".to_owned())?;
    if let Some(argument) = arguments.next() {
        return Err(format!("unexpected argument: {argument}"));
    }
    match subcommand.as_str() {
        "setup" => Ok(PhysxCommand::Setup),
        "doctor" => Ok(PhysxCommand::Doctor),
        "clean-cache" => Ok(PhysxCommand::CleanCache),
        _ => Err(format!("unknown physx command: {subcommand}")),
    }
}

pub fn run(command: PhysxCommand) -> Result<(), String> {
    match command {
        PhysxCommand::Setup => setup(),
        PhysxCommand::Doctor => doctor(),
        PhysxCommand::CleanCache => clean_cache(),
    }
}

fn setup() -> Result<(), String> {
    let identity = host_identity()?;
    let cache = cache_root()?;
    let profile_hash = profile_hash(&identity.target, &identity.compiler);
    let install = cache.join("sdk").join(&profile_hash);
    if verify_install(&install, &identity, &profile_hash).is_ok() {
        return emit_report("physx setup", &identity, &profile_hash, &install);
    }

    fs::create_dir_all(&cache)
        .map_err(|error| format!("PHYSX_CACHE_CREATE_FAILED {}: {error}", cache.display()))?;
    let archive_dir = cache.join("archives");
    fs::create_dir_all(&archive_dir).map_err(|error| error.to_string())?;
    let archive = archive_dir.join(format!("physx-{PHYSX_SOURCE_REVISION}.tar.gz"));
    if !archive.is_file() || sha256_file(&archive)? != PHYSX_ARCHIVE_SHA256 {
        let partial = archive.with_extension("tar.gz.partial");
        if partial.exists() {
            fs::remove_file(&partial).map_err(|error| error.to_string())?;
        }
        run_checked(
            Command::new(if cfg!(windows) { "curl.exe" } else { "curl" })
                .args(["--fail", "--location", "--proto", "=https", "--tlsv1.2"])
                .arg("--output")
                .arg(&partial)
                .arg(PHYSX_ARCHIVE_URL),
            "PHYSX_ARCHIVE_DOWNLOAD_FAILED",
        )?;
        let actual = sha256_file(&partial)?;
        if actual != PHYSX_ARCHIVE_SHA256 {
            return Err(format!(
                "PHYSX_ARCHIVE_HASH_MISMATCH expected={PHYSX_ARCHIVE_SHA256} actual={actual}"
            ));
        }
        fs::rename(&partial, &archive).map_err(|error| error.to_string())?;
    }

    let sources = cache.join("sources").join(PHYSX_SOURCE_REVISION);
    if !sources.join("physx/include/PxPhysicsAPI.h").is_file() {
        if sources.exists() {
            remove_scoped_tree(&sources, &cache)?;
        }
        fs::create_dir_all(&sources).map_err(|error| error.to_string())?;
        run_checked(
            Command::new("tar")
                .args(["-xzf"])
                .arg(&archive)
                .args(["--strip-components=1", "-C"])
                .arg(&sources)
                .arg(format!("PhysX-{PHYSX_SOURCE_REVISION}/physx"))
                .arg(format!("PhysX-{PHYSX_SOURCE_REVISION}/LICENSE.md")),
            "PHYSX_ARCHIVE_EXTRACT_FAILED",
        )?;
    }
    verify_source(&sources)?;

    let generated = generate_and_build(&sources, &identity)?;
    let staging = cache.join("staging").join(&profile_hash);
    if staging.exists() {
        remove_scoped_tree(&staging, &cache)?;
    }
    fs::create_dir_all(staging.join("include")).map_err(|error| error.to_string())?;
    fs::create_dir_all(staging.join("lib")).map_err(|error| error.to_string())?;
    copy_tree(&sources.join("physx/include"), &staging.join("include"))?;
    fs::copy(sources.join("LICENSE.md"), staging.join("LICENSE.md"))
        .map_err(|error| error.to_string())?;
    copy_static_libraries(&generated, &staging.join("lib"))?;

    let manifest = PhysxSdkManifestV1 {
        schema: "nextengine.physx-sdk-manifest.v1",
        physx_version: PHYSX_VERSION,
        source_revision: PHYSX_SOURCE_REVISION,
        archive_sha256: PHYSX_ARCHIVE_SHA256,
        target_triple: identity.target.clone(),
        compiler_profile: identity.compiler.clone(),
        build_profile: PHYSX_BUILD_PROFILE,
        bridge_abi: PHYSX_BRIDGE_ABI,
        profile_hash: profile_hash.clone(),
    };
    fs::write(
        staging.join(MANIFEST_NAME),
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    if install.exists() {
        remove_scoped_tree(&install, &cache)?;
    }
    fs::create_dir_all(install.parent().ok_or("PHYSX_CACHE_PATH_INVALID")?)
        .map_err(|error| error.to_string())?;
    fs::rename(&staging, &install).map_err(|error| error.to_string())?;
    let active_dir = cache.join("active");
    fs::create_dir_all(&active_dir).map_err(|error| error.to_string())?;
    let locator = active_dir.join(format!("{}.txt", identity.target));
    let locator_staging = locator.with_extension("txt.partial");
    fs::write(&locator_staging, install.to_string_lossy().as_bytes())
        .map_err(|error| error.to_string())?;
    fs::rename(locator_staging, locator).map_err(|error| error.to_string())?;
    verify_install(&install, &identity, &profile_hash)?;
    emit_report("physx setup", &identity, &profile_hash, &install)
}

fn doctor() -> Result<(), String> {
    let identity = host_identity()?;
    let profile_hash = profile_hash(&identity.target, &identity.compiler);
    let install = resolved_install_dir(&profile_hash)?;
    verify_install(&install, &identity, &profile_hash)?;
    emit_report("physx doctor", &identity, &profile_hash, &install)
}

fn clean_cache() -> Result<(), String> {
    let cache = cache_root()?;
    if cache.exists() {
        let resolved = fs::canonicalize(&cache).map_err(|error| error.to_string())?;
        let name = resolved
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if name != "physx" || resolved.parent().is_none() {
            return Err(format!("PHYSX_CACHE_SCOPE_INVALID {}", resolved.display()));
        }
        fs::remove_dir_all(&resolved).map_err(|error| error.to_string())?;
    }
    println!("{{\"command\":\"physx clean-cache\",\"status\":\"PASS\"}}");
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HostIdentity {
    target: String,
    compiler: String,
}

fn host_identity() -> Result<HostIdentity, String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .map_err(|error| format!("PHYSX_RUSTC_UNAVAILABLE: {error}"))?;
    if !output.status.success() {
        return Err("PHYSX_RUSTC_UNAVAILABLE".to_owned());
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let target = text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("PHYSX_TARGET_UNKNOWN")?
        .to_owned();
    let compiler = match target.as_str() {
        "x86_64-pc-windows-msvc" => {
            let vsdevcmd = find_vsdevcmd()?;
            format!("msvc-{}-static-release", msvc_version(&vsdevcmd)?)
        }
        "x86_64-unknown-linux-gnu" => {
            let output = Command::new("c++")
                .arg("-dumpfullversion")
                .output()
                .map_err(|error| format!("PHYSX_COMPILER_UNAVAILABLE: {error}"))?;
            if !output.status.success() {
                return Err("PHYSX_COMPILER_UNAVAILABLE".to_owned());
            }
            format!(
                "gcc-{}-static-release",
                String::from_utf8_lossy(&output.stdout).trim()
            )
        }
        _ => return Err(format!("PHYSX_TARGET_UNSUPPORTED {target}")),
    };
    Ok(HostIdentity { target, compiler })
}

fn cache_root() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("NEXTENGINE_PHYSX_CACHE_DIR") {
        return Ok(PathBuf::from(path).join("physx"));
    }
    if cfg!(windows) {
        return env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("NextEngine").join("cache").join("physx"))
            .ok_or_else(|| "PHYSX_CACHE_ROOT_UNAVAILABLE LOCALAPPDATA".to_owned());
    }
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(path).join("nextengine").join("physx"));
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| path.join(".cache").join("nextengine").join("physx"))
        .ok_or_else(|| "PHYSX_CACHE_ROOT_UNAVAILABLE XDG_CACHE_HOME/HOME".to_owned())
}

fn resolved_install_dir(profile_hash: &str) -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("NEXTENGINE_PHYSX_SDK_DIR") {
        Ok(PathBuf::from(path))
    } else {
        Ok(cache_root()?.join("sdk").join(profile_hash))
    }
}

fn profile_hash(target: &str, compiler: &str) -> String {
    let mut hasher = Sha256::new();
    let bridge_abi = PHYSX_BRIDGE_ABI.to_string();
    for value in [
        "nextengine.physx-build-profile.v1",
        PHYSX_VERSION,
        PHYSX_SOURCE_REVISION,
        PHYSX_ARCHIVE_SHA256,
        target,
        compiler,
        PHYSX_BUILD_PROFILE,
        bridge_abi.as_str(),
    ] {
        hasher.update((value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn verify_source(source: &Path) -> Result<(), String> {
    let license = fs::read_to_string(source.join("LICENSE.md"))
        .map_err(|error| format!("PHYSX_LICENSE_MISSING: {error}"))?;
    if !license.contains("Redistribution and use in source and binary forms") {
        return Err("PHYSX_LICENSE_INVALID".to_owned());
    }
    let version = fs::read_to_string(source.join("physx/include/foundation/PxPhysicsVersion.h"))
        .map_err(|error| format!("PHYSX_VERSION_HEADER_MISSING: {error}"))?;
    for expected in [
        "#define PX_PHYSICS_VERSION_MAJOR 5",
        "#define PX_PHYSICS_VERSION_MINOR 9",
        "#define PX_PHYSICS_VERSION_BUGFIX 0",
    ] {
        if !version.lines().any(|line| line.trim() == expected) {
            return Err("PHYSX_VERSION_MISMATCH".to_owned());
        }
    }
    Ok(())
}

fn generate_and_build(source: &Path, identity: &HostIdentity) -> Result<PathBuf, String> {
    let physx = source.join("physx");
    let preset = if identity.target.contains("windows-msvc") {
        let vsdevcmd = find_vsdevcmd()?;
        let compiler = msvc_generator_key(&vsdevcmd)?;
        if compiler == "vc18" {
            patch_physx_generator_for_vs18(&physx)?;
        }
        let preset = format!("nextengine-{compiler}win64-static-cpu");
        write_build_preset(&physx, &preset, "win64", compiler)?;
        preset
    } else {
        let preset = "nextengine-linux-gcc-static-cpu".to_owned();
        write_build_preset(&physx, &preset, "linux", "gcc")?;
        preset
    };
    if identity.target.contains("windows-msvc") {
        let vsdevcmd = find_vsdevcmd()?;
        let generated = physx.join("compiler").join(&preset);
        let script = source.join("nextengine-physx-build.bat");
        let body = format!(
            "@echo off\r\ncall \"{}\" -no_logo -arch=x64 -host_arch=x64\r\nif errorlevel 1 exit /b %errorlevel%\r\npushd \"{}\"\r\ncall generate_projects.bat {preset}\r\nif errorlevel 1 exit /b %errorlevel%\r\npopd\r\ncmake --build \"{}\" --config release --parallel\r\nexit /b %errorlevel%\r\n",
            vsdevcmd.display(),
            physx.display(),
            generated.display()
        );
        fs::write(&script, body).map_err(|error| error.to_string())?;
        let result = run_checked(
            Command::new("cmd").args(["/d", "/c"]).arg(&script),
            "PHYSX_BUILD_FAILED",
        );
        let _ = fs::remove_file(&script);
        result?;
    } else {
        run_checked(
            Command::new("bash")
                .arg("generate_projects.sh")
                .arg(&preset)
                .current_dir(&physx),
            "PHYSX_GENERATE_FAILED",
        )?;
    }
    let generated = generated_project_dir(&physx, &preset, &identity.target);
    if !identity.target.contains("windows-msvc") {
        run_checked(
            Command::new("cmake")
                .args(["--build"])
                .arg(&generated)
                .args(["--config", "release", "--parallel"]),
            "PHYSX_BUILD_FAILED",
        )?;
    }
    Ok(source.join("physx/bin"))
}

fn generated_project_dir(physx: &Path, preset: &str, target: &str) -> PathBuf {
    let directory = if target.contains("windows-msvc") {
        preset.to_owned()
    } else {
        format!("{preset}-release")
    };
    physx.join("compiler").join(directory)
}

fn verify_install(
    install: &Path,
    identity: &HostIdentity,
    expected_profile_hash: &str,
) -> Result<(), String> {
    for path in [
        install.join("include/PxPhysicsAPI.h"),
        install.join("include/foundation/PxPhysicsVersion.h"),
        install.join("LICENSE.md"),
        install.join(MANIFEST_NAME),
    ] {
        if !path.is_file() {
            return Err(format!("PHYSX_SDK_INCOMPLETE {}", path.display()));
        }
    }
    verify_source_like_install(install)?;
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(install.join(MANIFEST_NAME)).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("PHYSX_MANIFEST_INVALID: {error}"))?;
    for (field, expected) in [
        ("schema", "nextengine.physx-sdk-manifest.v1"),
        ("physx_version", PHYSX_VERSION),
        ("source_revision", PHYSX_SOURCE_REVISION),
        ("archive_sha256", PHYSX_ARCHIVE_SHA256),
        ("target_triple", identity.target.as_str()),
        ("compiler_profile", identity.compiler.as_str()),
        ("build_profile", PHYSX_BUILD_PROFILE),
        ("profile_hash", expected_profile_hash),
    ] {
        if manifest.get(field).and_then(serde_json::Value::as_str) != Some(expected) {
            return Err(format!("PHYSX_MANIFEST_MISMATCH {field}"));
        }
    }
    let library_count = fs::read_dir(install.join("lib"))
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| is_static_library(&entry.path()))
        .count();
    if library_count < 4 {
        return Err(format!(
            "PHYSX_STATIC_LIBRARIES_MISSING count={library_count}"
        ));
    }
    Ok(())
}

fn verify_source_like_install(install: &Path) -> Result<(), String> {
    let version = fs::read_to_string(install.join("include/foundation/PxPhysicsVersion.h"))
        .map_err(|error| error.to_string())?;
    if !version.contains("PX_PHYSICS_VERSION_MAJOR 5")
        || !version.contains("PX_PHYSICS_VERSION_MINOR 9")
        || !version.contains("PX_PHYSICS_VERSION_BUGFIX 0")
    {
        return Err("PHYSX_VERSION_MISMATCH".to_owned());
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err(format!("PHYSX_INCLUDE_TREE_MISSING {}", source.display()));
    }
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = destination.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            fs::create_dir_all(&target).map_err(|error| error.to_string())?;
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn copy_static_libraries(source: &Path, destination: &Path) -> Result<(), String> {
    let mut copied = 0_u32;
    visit_files(source, &mut |path| {
        if is_static_library(path) {
            let name = path.file_name().ok_or("PHYSX_LIBRARY_NAME_INVALID")?;
            fs::copy(path, destination.join(name)).map_err(|error| error.to_string())?;
            copied = copied.saturating_add(1);
        }
        Ok(())
    })?;
    if copied < 4 {
        return Err(format!("PHYSX_STATIC_LIBRARIES_MISSING count={copied}"));
    }
    Ok(())
}

fn is_static_library(path: &Path) -> bool {
    matches!(path.extension().and_then(OsStr::to_str), Some("a" | "lib"))
}

fn visit_files(
    path: &Path,
    visitor: &mut impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("PHYSX_BUILD_OUTPUT_MISSING {}", path.display()));
    }
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            visit_files(&entry.path(), visitor)?;
        } else {
            visitor(&entry.path())?;
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn find_vsdevcmd() -> Result<PathBuf, String> {
    for variable in ["ProgramFiles", "ProgramFiles(x86)"] {
        let Some(program_files) = env::var_os(variable) else {
            continue;
        };
        let visual_studio = PathBuf::from(program_files).join("Microsoft Visual Studio");
        let Ok(versions) = fs::read_dir(&visual_studio) else {
            continue;
        };
        let mut versions = versions.filter_map(Result::ok).collect::<Vec<_>>();
        versions.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
        for version in versions {
            if !version.path().is_dir() {
                continue;
            }
            let Ok(editions) = fs::read_dir(version.path()) else {
                continue;
            };
            for edition in editions.filter_map(Result::ok) {
                let candidate = edition.path().join("Common7/Tools/VsDevCmd.bat");
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    Err("PHYSX_MSVC_ENVIRONMENT_UNAVAILABLE VsDevCmd.bat".to_owned())
}

fn msvc_version(vsdevcmd: &Path) -> Result<String, String> {
    let edition = vsdevcmd
        .ancestors()
        .nth(3)
        .ok_or("PHYSX_MSVC_LAYOUT_INVALID")?;
    let toolchains = edition.join("VC/Tools/MSVC");
    let mut versions = fs::read_dir(&toolchains)
        .map_err(|error| format!("PHYSX_COMPILER_UNAVAILABLE: {error}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("bin/Hostx64/x64/cl.exe").is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    versions.sort();
    versions
        .pop()
        .ok_or_else(|| "PHYSX_COMPILER_UNAVAILABLE cl.exe".to_owned())
}

fn msvc_generator_key(vsdevcmd: &Path) -> Result<&'static str, String> {
    let version = vsdevcmd
        .ancestors()
        .nth(4)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .ok_or("PHYSX_MSVC_LAYOUT_INVALID")?;
    if version == "18" {
        Ok("vc18")
    } else {
        Ok("vc17")
    }
}

fn patch_physx_generator_for_vs18(physx: &Path) -> Result<(), String> {
    let path = physx.join("buildtools/cmake_generate_projects.py");
    let body = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    if body.contains(r#"'vc18': '\"Visual Studio 18 2026\"'"#) {
        return Ok(());
    }
    let needle = r#"'vc17': '\"Visual Studio 17 2022\"'"#;
    let replacement = format!(
        r#"{needle},
            'vc18': '\"Visual Studio 18 2026\"'"#
    );
    if !body.contains(needle) {
        return Err("PHYSX_GENERATOR_PATCH_POINT_MISSING".to_owned());
    }
    fs::write(path, body.replacen(needle, &replacement, 1)).map_err(|error| error.to_string())
}

fn write_build_preset(
    physx: &Path,
    name: &str,
    target_platform: &str,
    compiler: &str,
) -> Result<(), String> {
    let install = format!("install/{name}/PhysX");
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<preset name=\"{name}\" comment=\"Next Engine pinned static CPU PhysX\">\n\
  <platform targetPlatform=\"{target_platform}\" compiler=\"{compiler}\" />\n\
  <CMakeSwitches>\n\
    <cmakeSwitch name=\"PX_BUILDSNIPPETS\" value=\"False\" />\n\
    <cmakeSwitch name=\"PX_BUILDPVDRUNTIME\" value=\"False\" />\n\
    <cmakeSwitch name=\"PX_GENERATE_STATIC_LIBRARIES\" value=\"True\" />\n\
    <cmakeSwitch name=\"PX_GENERATE_GPU_PROJECTS\" value=\"False\" />\n\
    <cmakeSwitch name=\"NV_USE_STATIC_WINCRT\" value=\"False\" />\n\
    <cmakeSwitch name=\"NV_USE_DEBUG_WINCRT\" value=\"False\" />\n\
    <cmakeSwitch name=\"PX_FLOAT_POINT_PRECISE_MATH\" value=\"True\" />\n\
  </CMakeSwitches>\n\
  <CMakeParams>\n\
    <cmakeParam name=\"CMAKE_INSTALL_PREFIX\" value=\"{install}\" />\n\
  </CMakeParams>\n\
</preset>\n"
    );
    let path = physx.join("buildtools/presets").join(format!("{name}.xml"));
    fs::write(path, body).map_err(|error| error.to_string())
}

fn remove_scoped_tree(path: &Path, cache: &Path) -> Result<(), String> {
    let cache = if cache.exists() {
        fs::canonicalize(cache).map_err(|error| error.to_string())?
    } else {
        cache.to_path_buf()
    };
    let candidate = if path.exists() {
        fs::canonicalize(path).map_err(|error| error.to_string())?
    } else {
        path.to_path_buf()
    };
    if candidate == cache || !candidate.starts_with(&cache) {
        return Err(format!(
            "PHYSX_DELETE_SCOPE_INVALID {}",
            candidate.display()
        ));
    }
    fs::remove_dir_all(candidate).map_err(|error| error.to_string())
}

fn run_checked(command: &mut Command, code: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("{code}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{code}: {status}"))
    }
}

fn emit_report(
    command: &'static str,
    identity: &HostIdentity,
    profile_hash: &str,
    install: &Path,
) -> Result<(), String> {
    let report = PhysxCommandReportV1 {
        command,
        status: "PASS",
        version: PHYSX_VERSION,
        target: identity.target.clone(),
        profile_hash: profile_hash.to_owned(),
        sdk_dir: install.display().to_string(),
    };
    println!(
        "{}",
        serde_json::to_string(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bounded_subcommands() {
        assert_eq!(
            parse_command(["setup".to_owned()].into_iter()),
            Ok(PhysxCommand::Setup)
        );
        assert_eq!(
            parse_command(["doctor".to_owned()].into_iter()),
            Ok(PhysxCommand::Doctor)
        );
        assert!(parse_command(["setup".to_owned(), "extra".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn profile_hash_binds_target_and_compiler() {
        let windows = profile_hash("x86_64-pc-windows-msvc", "msvc-v143-static-release");
        let linux = profile_hash("x86_64-unknown-linux-gnu", "gcc-14-static-release");
        assert_ne!(windows, linux);
        assert_eq!(windows.len(), 64);
    }

    #[test]
    fn generated_project_directory_matches_platform_generator_layout() {
        let physx = Path::new("physx");
        let preset = "nextengine-static-cpu";
        assert_eq!(
            generated_project_dir(physx, preset, "x86_64-pc-windows-msvc"),
            physx.join("compiler/nextengine-static-cpu")
        );
        assert_eq!(
            generated_project_dir(physx, preset, "x86_64-unknown-linux-gnu"),
            physx.join("compiler/nextengine-static-cpu-release")
        );
    }
}
