use std::env;

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=NEXTENGINE_BUILD_PROFILE={profile}");
}
