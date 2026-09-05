fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!(
        "{}",
        next_motor::biomechanics_body_diagnostic_descriptor_json_v9()?
    );
    Ok(())
}
