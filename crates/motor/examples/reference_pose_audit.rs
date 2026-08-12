#[cfg(feature = "physx-sdk")]
use std::io::Read;

#[cfg(feature = "physx-sdk")]
fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read reference pose audit input from stdin");
    match next_motor::biomechanics_reference_pose_audit_json_v1(&input) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(not(feature = "physx-sdk"))]
fn main() {
    panic!("reference_pose_audit requires --features physx-sdk");
}
