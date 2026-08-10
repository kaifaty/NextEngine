use std::io::{ErrorKind, Read, Write};

use next_contracts::ids::ContentHash;
use next_motor::{MotorVectorRunner, TrainingEnvironmentError, VectorStepInput};

const MAGIC: &[u8; 8] = b"NEMLAB\0\0";
const PROTOCOL_VERSION: u16 = 1;
const REQUEST_HEADER_BYTES: usize = 16;
const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
const OP_RESET: u16 = 1;
const OP_STEP: u16 = 2;
const OP_PING: u16 = 3;
const OP_CLOSE: u16 = 255;

pub fn run(arguments: &[String]) -> i32 {
    let (slot_count, run_root) = match parse_options(arguments) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("next_headless motor-lab: {error}");
            return 2;
        }
    };
    let runner = match MotorVectorRunner::create(slot_count, run_root) {
        Ok(runner) => runner,
        Err(error) => {
            eprintln!("next_headless motor-lab: {}", error.stable_code());
            return 1;
        }
    };
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match serve(stdin.lock(), stdout.lock(), runner) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("next_headless motor-lab: {error}");
            1
        }
    }
}

fn serve(
    mut reader: impl Read,
    mut writer: impl Write,
    mut runner: MotorVectorRunner,
) -> Result<(), String> {
    loop {
        let Some(request) = read_request(&mut reader)? else {
            return Ok(());
        };
        let opcode = request.opcode;
        let response = process_request(&mut runner, request);
        write_response(&mut writer, opcode, response)?;
        writer.flush().map_err(|error| error.to_string())?;
        if opcode == OP_CLOSE {
            return Ok(());
        }
    }
}

#[derive(Debug)]
struct Request {
    opcode: u16,
    payload: Vec<u8>,
}

fn read_request(reader: &mut impl Read) -> Result<Option<Request>, String> {
    let mut header = [0; REQUEST_HEADER_BYTES];
    match reader.read_exact(&mut header) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.to_string()),
    }
    if &header[..8] != MAGIC {
        return Err("MOTOR_LAB_PROTOCOL_MAGIC_MISMATCH".to_owned());
    }
    let version = u16::from_le_bytes([header[8], header[9]]);
    if version != PROTOCOL_VERSION {
        return Err("MOTOR_LAB_PROTOCOL_VERSION_UNSUPPORTED".to_owned());
    }
    let opcode = u16::from_le_bytes([header[10], header[11]]);
    let payload_length = u32::from_le_bytes(header[12..16].try_into().expect("fixed slice"));
    let payload_length = payload_length as usize;
    if payload_length > MAX_REQUEST_BYTES {
        return Err("MOTOR_LAB_REQUEST_CAPACITY_EXCEEDED".to_owned());
    }
    let mut payload = vec![0; payload_length];
    reader
        .read_exact(&mut payload)
        .map_err(|_| "MOTOR_LAB_REQUEST_TRUNCATED".to_owned())?;
    Ok(Some(Request { opcode, payload }))
}

fn process_request(
    runner: &mut MotorVectorRunner,
    request: Request,
) -> Result<Vec<u8>, ProtocolFailure> {
    match request.opcode {
        OP_RESET => {
            let mut reader = PayloadReader::new(&request.payload);
            let episode_ordinal = reader.read_u64()?;
            reader.finish()?;
            let values = runner.reset_all(episode_ordinal)?;
            let mut payload = Vec::new();
            push_len(&mut payload, values.len())?;
            for value in values {
                payload.extend_from_slice(&value.episode_ordinal.to_le_bytes());
                payload.extend_from_slice(&value.vector_slot.to_le_bytes());
                push_i64_values(&mut payload, &value.observation_raw)?;
                payload.extend_from_slice(
                    value
                        .seed_set
                        .seed_set_hash()
                        .map_err(|_| ProtocolFailure::Encoding)?
                        .as_bytes(),
                );
            }
            Ok(payload)
        }
        OP_STEP => {
            let mut reader = PayloadReader::new(&request.payload);
            let count = reader.read_len(256)?;
            let mut inputs = Vec::with_capacity(count);
            for _ in 0..count {
                inputs.push(VectorStepInput {
                    vector_slot: reader.read_u32()?,
                    action_microradians: reader.read_i64_values(4_096)?,
                    command_raw: [reader.read_i64()?, reader.read_i64()?, reader.read_i64()?],
                });
            }
            reader.finish()?;
            let values = runner.step_lockstep(inputs)?;
            let mut payload = Vec::new();
            push_len(&mut payload, values.len())?;
            for value in values {
                payload.extend_from_slice(&value.episode_ordinal.to_le_bytes());
                payload.extend_from_slice(&value.vector_slot.to_le_bytes());
                payload.extend_from_slice(&value.frame.motor_tick.to_le_bytes());
                push_i64_values(&mut payload, &value.frame.applied_action_microradians)?;
                push_i64_values(&mut payload, &value.frame.observation_raw)?;
                push_len(&mut payload, value.reward_components_raw.len())?;
                for (component_id, component_value) in value.reward_components_raw {
                    push_text(&mut payload, component_id.as_str())?;
                    payload.extend_from_slice(&component_value.to_le_bytes());
                }
                if let Some(reason) = value.terminal_reason_id {
                    payload.push(1);
                    push_text(&mut payload, reason.as_str())?;
                } else {
                    payload.push(0);
                }
                payload.extend_from_slice(
                    value
                        .frame
                        .deterministic_hash()
                        .map_err(|_| ProtocolFailure::Encoding)?
                        .as_bytes(),
                );
            }
            Ok(payload)
        }
        OP_PING => {
            if !request.payload.is_empty() {
                return Err(ProtocolFailure::Payload);
            }
            Ok(runner.slot_count().to_le_bytes().to_vec())
        }
        OP_CLOSE => {
            if !request.payload.is_empty() {
                return Err(ProtocolFailure::Payload);
            }
            Ok(Vec::new())
        }
        _ => Err(ProtocolFailure::Opcode),
    }
}

fn write_response(
    writer: &mut impl Write,
    opcode: u16,
    response: Result<Vec<u8>, ProtocolFailure>,
) -> Result<(), String> {
    let (status, payload) = match response {
        Ok(payload) => (0_u32, payload),
        Err(error) => (error.status(), error.stable_code().as_bytes().to_vec()),
    };
    let payload_length = u32::try_from(payload.len())
        .map_err(|_| "MOTOR_LAB_RESPONSE_CAPACITY_EXCEEDED".to_owned())?;
    writer.write_all(MAGIC).map_err(|error| error.to_string())?;
    writer
        .write_all(&PROTOCOL_VERSION.to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&opcode.to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&status.to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&payload_length.to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&payload)
        .map_err(|error| error.to_string())
}

fn parse_options(arguments: &[String]) -> Result<(u32, ContentHash), &'static str> {
    let mut slot_count = None;
    let mut run_root = None;
    let mut index = 0;
    while index < arguments.len() {
        let flag = arguments[index].as_str();
        index += 1;
        let value = arguments
            .get(index)
            .ok_or("missing motor-lab option value")?;
        index += 1;
        match flag {
            "--slots" if slot_count.is_none() => {
                slot_count = Some(value.parse().map_err(|_| "invalid --slots value")?);
            }
            "--run-root" if run_root.is_none() => run_root = Some(parse_hash(value)?),
            _ => return Err("unsupported or duplicate motor-lab option"),
        }
    }
    Ok((
        slot_count.ok_or("--slots is required")?,
        run_root.ok_or("--run-root is required")?,
    ))
}

fn parse_hash(value: &str) -> Result<ContentHash, &'static str> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("--run-root must be lowercase sha256");
    }
    let mut bytes = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(ContentHash::from_bytes(bytes))
}

const fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn push_len(bytes: &mut Vec<u8>, value: usize) -> Result<(), ProtocolFailure> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| ProtocolFailure::Encoding)?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_i64_values(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), ProtocolFailure> {
    push_len(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn push_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProtocolFailure> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

struct PayloadReader<'a> {
    remaining: &'a [u8],
}

impl<'a> PayloadReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], ProtocolFailure> {
        if self.remaining.len() < length {
            return Err(ProtocolFailure::Payload);
        }
        let (value, remaining) = self.remaining.split_at(length);
        self.remaining = remaining;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, ProtocolFailure> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| ProtocolFailure::Payload)?,
        ))
    }

    fn read_u64(&mut self) -> Result<u64, ProtocolFailure> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| ProtocolFailure::Payload)?,
        ))
    }

    fn read_i64(&mut self) -> Result<i64, ProtocolFailure> {
        Ok(i64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| ProtocolFailure::Payload)?,
        ))
    }

    fn read_len(&mut self, maximum: usize) -> Result<usize, ProtocolFailure> {
        let value = self.read_u32()? as usize;
        if value > maximum {
            Err(ProtocolFailure::Capacity)
        } else {
            Ok(value)
        }
    }

    fn read_i64_values(&mut self, maximum: usize) -> Result<Vec<i64>, ProtocolFailure> {
        let length = self.read_len(maximum)?;
        (0..length).map(|_| self.read_i64()).collect()
    }

    fn finish(self) -> Result<(), ProtocolFailure> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(ProtocolFailure::Payload)
        }
    }
}

#[derive(Debug)]
enum ProtocolFailure {
    Training(TrainingEnvironmentError),
    Opcode,
    Payload,
    Capacity,
    Encoding,
}

impl ProtocolFailure {
    const fn status(&self) -> u32 {
        match self {
            Self::Training(_) => 1,
            Self::Opcode => 2,
            Self::Payload => 3,
            Self::Capacity => 4,
            Self::Encoding => 5,
        }
    }

    const fn stable_code(&self) -> &'static str {
        match self {
            Self::Training(error) => error.stable_code(),
            Self::Opcode => "MOTOR_LAB_OPCODE_UNSUPPORTED",
            Self::Payload => "MOTOR_LAB_PAYLOAD_INVALID",
            Self::Capacity => "MOTOR_LAB_CAPACITY_EXCEEDED",
            Self::Encoding => "MOTOR_LAB_RESPONSE_ENCODING_FAILED",
        }
    }
}

impl From<TrainingEnvironmentError> for ProtocolFailure {
    fn from(value: TrainingEnvironmentError) -> Self {
        Self::Training(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_require_bounded_explicit_slots_and_run_root() {
        let options = [
            "--slots".to_owned(),
            "4".to_owned(),
            "--run-root".to_owned(),
            "01".repeat(32),
        ];
        let (slots, root) = parse_options(&options).expect("options");
        assert_eq!(slots, 4);
        assert_eq!(root, ContentHash::from_bytes([1; 32]));
        assert!(parse_options(&options[..2]).is_err());
    }

    #[test]
    fn request_header_rejects_version_before_payload_decode() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&OP_STEP.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        let error = read_request(&mut &bytes[..]).expect_err("old version");
        assert_eq!(error, "MOTOR_LAB_PROTOCOL_VERSION_UNSUPPORTED");
    }

    #[test]
    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn long_lived_protocol_resets_steps_pings_and_closes() {
        let mut requests = Vec::new();
        push_request(&mut requests, OP_RESET, &7_u64.to_le_bytes());
        let mut step = Vec::new();
        step.extend_from_slice(&1_u32.to_le_bytes());
        step.extend_from_slice(&0_u32.to_le_bytes());
        step.extend_from_slice(&23_u32.to_le_bytes());
        for _ in 0..23 {
            step.extend_from_slice(&0_i64.to_le_bytes());
        }
        for _ in 0..3 {
            step.extend_from_slice(&0_i64.to_le_bytes());
        }
        push_request(&mut requests, OP_STEP, &step);
        push_request(&mut requests, OP_PING, &[]);
        push_request(&mut requests, OP_CLOSE, &[]);
        let runner =
            MotorVectorRunner::create(1, ContentHash::from_bytes([2; 32])).expect("runner");
        let mut output = Vec::new();
        serve(&requests[..], &mut output, runner).expect("serve");
        assert!(output.starts_with(MAGIC));
        assert_eq!(
            output
                .windows(MAGIC.len())
                .filter(|value| *value == MAGIC)
                .count(),
            4
        );
    }

    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn push_request(bytes: &mut Vec<u8>, opcode: u16, payload: &[u8]) {
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        bytes.extend_from_slice(&opcode.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
    }
}
