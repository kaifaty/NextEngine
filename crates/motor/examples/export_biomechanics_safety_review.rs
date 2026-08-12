#[cfg(feature = "physx-sdk")]
fn main() {
    let output = next_motor::biomechanics_native_safety_review_json_v1()
        .expect("run native biomechanics safety review");
    let path = std::env::args_os()
        .nth(1)
        .expect("usage: export_biomechanics_safety_review <output.json>");
    std::fs::write(path, output).expect("write native safety review");
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("export_biomechanics_safety_review requires --features physx-sdk");
}
