fn main() {
    match next_motor::biomechanics_body_diagnostic_descriptor_json_v8() {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
