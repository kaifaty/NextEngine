use next_motor::biomechanics_forward_start_stop_isaac_descriptor_json_v1;

fn main() {
    match biomechanics_forward_start_stop_isaac_descriptor_json_v1() {
        Ok(descriptor) => {
            let mut arguments = std::env::args_os().skip(1);
            match (arguments.next(), arguments.next()) {
                (None, None) => print!("{descriptor}"),
                (Some(path), None) => {
                    if let Err(error) = std::fs::write(&path, descriptor) {
                        eprintln!("failed to write {}: {error}", path.to_string_lossy());
                        std::process::exit(1);
                    }
                }
                _ => {
                    eprintln!("usage: export_biomechanics_forward_start_stop_isaac [output.json]");
                    std::process::exit(2);
                }
            }
        }
        Err(error) => {
            eprintln!("{}", error.stable_code());
            std::process::exit(1);
        }
    }
}
