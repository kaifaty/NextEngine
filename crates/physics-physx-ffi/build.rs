use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXPECTED_MAJOR: &str = "5";
const EXPECTED_MINOR: &str = "9";
const EXPECTED_PATCH: &str = "0";

fn main() {
    println!("cargo:rerun-if-env-changed=NEXTENGINE_PHYSX_SDK_DIR");
    println!("cargo:rerun-if-env-changed=NEXTENGINE_PHYSX_CACHE_DIR");
    println!("cargo:rerun-if-env-changed=LOCALAPPDATA");
    println!("cargo:rerun-if-env-changed=XDG_CACHE_HOME");
    println!("cargo:rerun-if-env-changed=HOME");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=AR");
    println!("cargo:rerun-if-changed=native/nextengine_physx_bridge.cpp");
    if env::var_os("CARGO_FEATURE_PHYSX_SDK").is_none() {
        return;
    }
    if env::var_os("CARGO_FEATURE_MOCK_ABI").is_some() {
        panic!("features `physx-sdk` and `mock-abi` are mutually exclusive");
    }

    let target = env::var("TARGET").expect("cargo provides TARGET");
    if !target.contains("x86_64-unknown-linux-gnu") && !target.contains("x86_64-pc-windows-msvc") {
        panic!("PhysX v1 supports only x86_64 Linux GNU and x86_64 Windows MSVC");
    }
    let sdk = resolve_sdk(&target);
    let include = sdk.join("include");
    let library = sdk.join("lib");
    if !include.join("PxPhysicsAPI.h").is_file() || !library.is_dir() {
        panic!("prepared PhysX SDK must contain include/PxPhysicsAPI.h and lib/");
    }
    verify_version(&include);
    verify_manifest(&sdk, &target);
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("cargo provides OUT_DIR"));
    if target.contains("windows-msvc") {
        compile_msvc(&include, &output);
    } else {
        compile_unix(&include, &output);
    }

    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-search=native={}", library.display());
    println!("cargo:rustc-link-lib=static=nextengine_physx_bridge");
    for library in [
        "PhysX_static_64",
        "PhysXCommon_static_64",
        "PhysXFoundation_static_64",
    ] {
        println!("cargo:rustc-link-lib=static={library}");
    }
    if target.contains("windows-msvc") {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=user32");
    } else {
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=rt");
    }
}

fn resolve_sdk(target: &str) -> PathBuf {
    if let Some(path) = env::var_os("NEXTENGINE_PHYSX_SDK_DIR") {
        return PathBuf::from(path);
    }
    let cache = if let Some(path) = env::var_os("NEXTENGINE_PHYSX_CACHE_DIR") {
        PathBuf::from(path).join("physx")
    } else if target.contains("windows-msvc") {
        PathBuf::from(
            env::var_os("LOCALAPPDATA")
                .expect("LOCALAPPDATA is required to locate the prepared PhysX SDK"),
        )
        .join("NextEngine")
        .join("cache")
        .join("physx")
    } else if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path).join("nextengine").join("physx")
    } else {
        PathBuf::from(env::var_os("HOME").expect("HOME is required to locate the PhysX cache"))
            .join(".cache")
            .join("nextengine")
            .join("physx")
    };
    let locator = cache.join("active").join(format!("{target}.txt"));
    println!("cargo:rerun-if-changed={}", locator.display());
    let value = fs::read_to_string(&locator).unwrap_or_else(|_| {
        panic!("PhysX SDK is not prepared for {target}; run `cargo run -p xtask -- physx setup`")
    });
    PathBuf::from(value.trim())
}

fn verify_manifest(sdk: &Path, target: &str) {
    let path = sdk.join("nextengine-physx-sdk-v1.json");
    let body =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    for expected in [
        "\"schema\": \"nextengine.physx-sdk-manifest.v1\"".to_owned(),
        "\"physx_version\": \"5.9.0\"".to_owned(),
        format!("\"target_triple\": \"{target}\""),
        "\"build_profile\": \"nextengine-physx-5.9.0-static-cpu-release-v1\"".to_owned(),
        "\"bridge_abi\": 2".to_owned(),
    ] {
        if !body.contains(&expected) {
            panic!("PhysX SDK manifest mismatch: {expected}");
        }
    }
}

fn verify_version(include: &Path) {
    let version_header = include.join("foundation/PxPhysicsVersion.h");
    let body = fs::read_to_string(&version_header)
        .unwrap_or_else(|error| panic!("{}: {error}", version_header.display()));
    let major = format!("#define PX_PHYSICS_VERSION_MAJOR {EXPECTED_MAJOR}");
    let minor = format!("#define PX_PHYSICS_VERSION_MINOR {EXPECTED_MINOR}");
    let patch = format!("#define PX_PHYSICS_VERSION_BUGFIX {EXPECTED_PATCH}");
    if !body.lines().any(|line| line.trim() == major)
        || !body.lines().any(|line| line.trim() == minor)
        || !body.lines().any(|line| line.trim() == patch)
    {
        panic!("PhysX SDK version must be exactly 5.9.0");
    }
}

fn compile_unix(include: &Path, output: &Path) {
    let compiler = env::var_os("CXX").unwrap_or_else(|| "c++".into());
    let archiver = env::var_os("AR").unwrap_or_else(|| "ar".into());
    let object = output.join("nextengine_physx_bridge.o");
    run(
        Command::new(compiler)
            .args([
                "-std=c++17",
                "-fPIC",
                "-O2",
                "-DNDEBUG",
                "-DPX_PHYSX_STATIC_LIB",
                "-c",
            ])
            .arg("native/nextengine_physx_bridge.cpp")
            .arg("-I")
            .arg(include)
            .arg("-o")
            .arg(&object),
        "compile PhysX bridge",
    );
    run(
        Command::new(archiver)
            .args(["crs"])
            .arg(output.join("libnextengine_physx_bridge.a"))
            .arg(object),
        "archive PhysX bridge",
    );
}

fn compile_msvc(include: &Path, output: &Path) {
    let object = output.join("nextengine_physx_bridge.obj");
    let library = output.join("nextengine_physx_bridge.lib");
    let vsdevcmd = find_vsdevcmd();
    let script = output.join("build-nextengine-physx-bridge.bat");
    let compiler = env::var("CXX").unwrap_or_else(|_| "cl".to_owned());
    let body = format!(
        "@echo off\r\ncall \"{}\" -no_logo -arch=x64 -host_arch=x64\r\nif errorlevel 1 exit /b %errorlevel%\r\n{} /nologo /c /std:c++17 /EHsc /O2 /DNDEBUG /DPX_PHYSX_STATIC_LIB /I\"{}\" native\\nextengine_physx_bridge.cpp /Fo\"{}\"\r\nif errorlevel 1 exit /b %errorlevel%\r\nlib /nologo /OUT:\"{}\" \"{}\"\r\nexit /b %errorlevel%\r\n",
        vsdevcmd.display(),
        compiler,
        include.display(),
        object.display(),
        library.display(),
        object.display()
    );
    fs::write(&script, body).unwrap_or_else(|error| panic!("{}: {error}", script.display()));
    run(
        Command::new("cmd").args(["/d", "/c"]).arg(&script),
        "compile and archive PhysX bridge",
    );
    let _ = fs::remove_file(script);
}

fn find_vsdevcmd() -> PathBuf {
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
            let Ok(editions) = fs::read_dir(version.path()) else {
                continue;
            };
            for edition in editions.filter_map(Result::ok) {
                let candidate = edition.path().join("Common7/Tools/VsDevCmd.bat");
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    panic!("Visual Studio C++ environment not found; PhysX requires VsDevCmd.bat")
}

fn run(command: &mut Command, action: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to {action}: {error}"));
    if !status.success() {
        panic!("{action} failed with {status}");
    }
}
