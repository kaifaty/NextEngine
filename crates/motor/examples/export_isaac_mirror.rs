fn main() {
    let golden = std::env::args().nth(1).as_deref() == Some("--golden");
    let descriptor = if golden {
        next_motor::stage0_isaac_mirror_golden_json_v1()
    } else {
        next_motor::stage0_isaac_mirror_descriptor_json_v1()
    }
    .expect("the engine-owned Stage 0 descriptor must compile");
    print!("{descriptor}");
}
