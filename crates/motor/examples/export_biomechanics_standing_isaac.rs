use next_motor::biomechanics_standing_isaac_descriptor_json_v1;

fn main() {
    match biomechanics_standing_isaac_descriptor_json_v1() {
        Ok(value) => print!("{value}"),
        Err(error) => {
            eprintln!("{}", error.stable_code());
            std::process::exit(1);
        }
    }
}
