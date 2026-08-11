use next_contracts::canonical::CanonicalError;
use next_contracts::command::WorldCommand;

pub(super) fn sort_command_batch(commands: &mut [WorldCommand]) -> Result<(), CanonicalError> {
    let mut keys = Vec::with_capacity(commands.len());
    for command in commands.iter() {
        keys.push((
            command.stream_id,
            command.sequence,
            command.body_hash()?,
            command.claimed_command_id,
        ));
    }
    let mut indexed = commands
        .iter()
        .cloned()
        .zip(keys)
        .collect::<Vec<(WorldCommand, _)>>();
    indexed.sort_by_key(|entry| entry.1);
    for (slot, (command, _)) in commands.iter_mut().zip(indexed) {
        *slot = command;
    }
    Ok(())
}
