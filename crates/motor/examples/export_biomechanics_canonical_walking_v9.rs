fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!(
        "{}",
        next_motor::biomechanics_forward_start_stop_canonical_descriptor_json_v9()?
    );
    Ok(())
}
