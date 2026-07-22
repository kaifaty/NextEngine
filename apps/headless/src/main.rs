#![forbid(unsafe_code)]

use next_contracts::{CONTRACT_SCHEMA_VERSION, CommandPayload, PersistentId, WorldCommand};
use next_runtime::RuntimeState;
use next_verification::VerificationRecord;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = WorldCommand {
        schema_version: CONTRACT_SCHEMA_VERSION,
        sequence: 0,
        issuer: PersistentId::default(),
        payload: CommandPayload::Noop,
    };
    let mut runtime = RuntimeState::default();
    let event = runtime.apply(&command)?;
    let record = VerificationRecord::from_event(&event);
    println!(
        "{{\"status\":\"PASS\",\"tick\":{},\"event_hash\":{}}}",
        record.tick, record.event_hash
    );
    Ok(())
}
