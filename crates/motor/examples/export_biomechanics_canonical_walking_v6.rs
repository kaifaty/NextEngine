fn main() {
    match next_motor::biomechanics_forward_start_stop_canonical_descriptor_json_v6() {
        Ok(descriptor) => print!("{descriptor}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
