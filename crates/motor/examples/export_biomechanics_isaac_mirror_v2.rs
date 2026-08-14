use next_motor::biomechanics_isaac_mirror_descriptor_json_v2;

fn main() {
    match biomechanics_isaac_mirror_descriptor_json_v2() {
        Ok(descriptor) => print!("{descriptor}"),
        Err(error) => {
            eprintln!("{}", error.stable_code());
            std::process::exit(1);
        }
    }
}
