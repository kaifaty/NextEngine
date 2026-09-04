use next_motor::biomechanics_forward_start_stop_isaac_descriptor_json_v4;

fn main() {
    match biomechanics_forward_start_stop_isaac_descriptor_json_v4() {
        Ok(json) => {
            if let Some(path) = std::env::args_os().nth(1) {
                if let Err(error) = std::fs::write(path, json) {
                    eprintln!("failed to write descriptor: {error}");
                    std::process::exit(1);
                }
            } else {
                print!("{json}");
            }
        }
        Err(error) => {
            eprintln!("failed to build descriptor: {error}");
            std::process::exit(1);
        }
    }
}
