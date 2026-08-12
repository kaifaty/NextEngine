use std::path::PathBuf;

fn main() {
    let output = next_motor::biomechanics_safety_contact_mirror_golden_json_v1()
        .expect("generate biomechanics safety/contact mirror");
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: export_biomechanics_safety_contact_mirror <output.json>");
    std::fs::write(path, output).expect("write biomechanics safety/contact mirror");
}
