use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXPECTED_MAJOR: &str = "5";
const EXPECTED_MINOR: &str = "9";
const EXPECTED_PATCH: &str = "0";

fn main() {
    println!("cargo:rerun-if-env-changed=NEXTENGINE_PHYSX_SDK_DIR");
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
    let sdk = PathBuf::from(
        env::var_os("NEXTENGINE_PHYSX_SDK_DIR")
            .expect("NEXTENGINE_PHYSX_SDK_DIR must point to a PhysX 5.9.0 install prefix"),
    );
    let include = sdk.join("include");
    let library = sdk.join("lib");
    if !include.join("PxPhysicsAPI.h").is_file() || !library.is_dir() {
        panic!("NEXTENGINE_PHYSX_SDK_DIR must contain include/PxPhysicsAPI.h and lib/");
    }
    verify_version(&include);
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

fn verify_version(include: &Path) {
    let version_header = include.join("foundation/PxVersionNumber.h");
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
    let compiler = env::var_os("CXX").unwrap_or_else(|| "cl".into());
    let object = output.join("nextengine_physx_bridge.obj");
    run(
        Command::new(compiler)
            .args([
                "/nologo",
                "/c",
                "/std:c++17",
                "/EHsc",
                "/O2",
                "/DNDEBUG",
                "/DPX_PHYSX_STATIC_LIB",
            ])
            .arg(format!("/I{}", include.display()))
            .arg("native/nextengine_physx_bridge.cpp")
            .arg(format!("/Fo{}", object.display())),
        "compile PhysX bridge",
    );
    run(
        Command::new("lib")
            .args(["/nologo"])
            .arg(format!(
                "/OUT:{}",
                output.join("nextengine_physx_bridge.lib").display()
            ))
            .arg(object),
        "archive PhysX bridge",
    );
}

fn run(command: &mut Command, action: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to {action}: {error}"));
    if !status.success() {
        panic!("{action} failed with {status}");
    }
}
