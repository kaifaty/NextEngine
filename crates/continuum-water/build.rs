#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=RUSTDOCFLAGS");

    let out_dir = env::var("OUT_DIR").unwrap_or_default();
    let profile_directory = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let profile = if profile_directory == "water-oracle" {
        "water-oracle".to_owned()
    } else {
        env::var("PROFILE").unwrap_or_default()
    };
    emit("WATER_BUILD_PROFILE", profile);
    emit("WATER_BUILD_TARGET", env::var("TARGET").unwrap_or_default());
    emit(
        "WATER_BUILD_OPT_LEVEL",
        env::var("OPT_LEVEL").unwrap_or_default(),
    );
    emit("WATER_BUILD_DEBUG", env::var("DEBUG").unwrap_or_default());
    emit(
        "WATER_BUILD_RUSTFLAGS",
        env::var("CARGO_ENCODED_RUSTFLAGS")
            .or_else(|_| env::var("RUSTFLAGS"))
            .unwrap_or_default(),
    );

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let version = Command::new(rustc)
        .arg("-vV")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    emit("WATER_BUILD_RUSTC_VV", version);
}

fn emit(name: &str, value: String) {
    let normalized = value.replace('\r', "").replace('\n', "|");
    println!("cargo:rustc-env={name}={normalized}");
}
