fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        next_motor::biomechanics_body_diagnostic_descriptor_json_v11()?
    );
    Ok(())
}
