use std::io::{ErrorKind, Read, Write};

use next_contracts::ids::ContentHash;
use next_contracts::motor::{MotorContractError, MotorEnvironmentCheckpointEnvelopeV1};
use next_motor::{
    BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID, BiomechanicsStandingRunnerError,
    BiomechanicsStandingVectorRunner, MotorVectorRunner, TrainingEnvironmentError,
    VectorPolicyStepInput,
};

const MAGIC: &[u8; 8] = b"NEMLAB\0\0";
const PROTOCOL_VERSION: u16 = 2;
const REQUEST_HEADER_BYTES: usize = 24;
const MAX_REQUEST_BYTES: usize = 5 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 4_096;
const OP_CREATE: u16 = 1;
const OP_RESET: u16 = 2;
const OP_STEP: u16 = 3;
const OP_CHECKPOINT: u16 = 4;
const OP_RESTORE: u16 = 5;
const OP_PING: u16 = 6;
const OP_CLOSE: u16 = 255;

pub fn run(arguments: &[String]) -> i32 {
    if !arguments.is_empty() {
        eprintln!("next_headless motor-lab: protocol v2 accepts Create over stdin; no CLI options");
        return 2;
    }
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match serve(stdin.lock(), stdout.lock()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("next_headless motor-lab: {error}");
            1
        }
    }
}

fn serve(mut reader: impl Read, mut writer: impl Write) -> Result<(), String> {
    let mut session = ProtocolSession::default();
    loop {
        let Some(request) = read_request(&mut reader)? else {
            return Ok(());
        };
        let opcode = request.opcode;
        let request_id = request.request_id;
        let response = process_request(&mut session, request);
        let should_close = opcode == OP_CLOSE && response.is_ok();
        write_response(&mut writer, opcode, request_id, response)?;
        writer.flush().map_err(|error| error.to_string())?;
        if should_close {
            return Ok(());
        }
    }
}

#[derive(Default)]
struct ProtocolSession {
    last_request_id: Option<u64>,
    runner: Option<ProtocolRunner>,
}

#[derive(Debug)]
enum ProtocolRunner {
    Legacy(Box<MotorVectorRunner>),
    BiomechanicsStanding(Box<BiomechanicsStandingVectorRunner>),
}

impl ProtocolRunner {
    fn slot_count(&self) -> u32 {
        match self {
            Self::Legacy(runner) => runner.slot_count(),
            Self::BiomechanicsStanding(runner) => runner.slot_count(),
        }
    }

    fn profile_id(&self) -> &str {
        match self {
            Self::Legacy(runner) => runner.profile().profile_id(),
            Self::BiomechanicsStanding(runner) => runner.manifest().environment_id.as_str(),
        }
    }

    fn checkpoint_slot(
        &self,
        vector_slot: u32,
        episode_ordinal: u64,
    ) -> Result<MotorEnvironmentCheckpointEnvelopeV1, ProtocolFailure> {
        match self {
            Self::Legacy(runner) => Ok(runner.checkpoint_slot(vector_slot, episode_ordinal)?),
            Self::BiomechanicsStanding(_) => Err(ProtocolFailure::OperationUnsupported),
        }
    }
}

#[derive(Debug)]
struct Request {
    opcode: u16,
    request_id: u64,
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
    let request_id = u64::from_le_bytes(header[16..24].try_into().expect("fixed slice"));
    if payload_length > MAX_REQUEST_BYTES {
        return Err("MOTOR_LAB_REQUEST_CAPACITY_EXCEEDED".to_owned());
    }
    let mut payload = vec![0; payload_length];
    reader
        .read_exact(&mut payload)
        .map_err(|_| "MOTOR_LAB_REQUEST_TRUNCATED".to_owned())?;
    Ok(Some(Request {
        opcode,
        request_id,
        payload,
    }))
}

fn process_request(
    session: &mut ProtocolSession,
    request: Request,
) -> Result<Vec<u8>, ProtocolFailure> {
    if request.request_id == 0
        || session
            .last_request_id
            .is_some_and(|previous| request.request_id <= previous)
    {
        return Err(ProtocolFailure::StaleRequest);
    }
    session.last_request_id = Some(request.request_id);
    match request.opcode {
        OP_CREATE => process_create(session, &request.payload),
        OP_RESET => process_reset(require_runner(session)?, &request.payload),
        OP_STEP => process_step(require_runner(session)?, &request.payload),
        OP_CHECKPOINT => process_checkpoint(require_runner(session)?, &request.payload),
        OP_RESTORE => process_restore(require_runner_mut(session)?, &request.payload),
        OP_PING => process_ping(session, &request.payload),
        OP_CLOSE => {
            if !request.payload.is_empty() {
                return Err(ProtocolFailure::Payload);
            }
            session.runner = None;
            Ok(Vec::new())
        }
        _ => Err(ProtocolFailure::Opcode),
    }
}

fn process_create(
    session: &mut ProtocolSession,
    payload: &[u8],
) -> Result<Vec<u8>, ProtocolFailure> {
    if session.runner.is_some() {
        return Err(ProtocolFailure::AlreadyCreated);
    }
    let mut reader = PayloadReader::new(payload);
    let profile_id = reader.read_text(MAX_TEXT_BYTES)?;
    let slot_count = reader.read_u32()?;
    let run_root = reader.read_hash()?;
    reader.finish()?;
    let runner = if profile_id == BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID {
        ProtocolRunner::BiomechanicsStanding(Box::new(BiomechanicsStandingVectorRunner::create(
            slot_count, run_root,
        )?))
    } else {
        ProtocolRunner::Legacy(Box::new(MotorVectorRunner::create_profile(
            &profile_id,
            slot_count,
            run_root,
        )?))
    };
    let (manifest, manifest_hash) = match &runner {
        ProtocolRunner::Legacy(runner) => (runner.manifest(), runner.manifest_hash()),
        ProtocolRunner::BiomechanicsStanding(runner) => (runner.manifest(), runner.manifest_hash()),
    };
    manifest.validate_for_protocol_v2()?;
    let mut response = Vec::new();
    push_text(&mut response, manifest.environment_id.as_str())?;
    for hash in [
        manifest_hash,
        manifest.observation_layout_hash,
        manifest.action_layout_hash,
        manifest.command_schedule_profile_hash,
        manifest.reward_profile_hash,
        manifest.termination_profile_hash,
        manifest.rng_derivation_profile_hash,
        manifest.correspondence_profile_hash,
    ] {
        response.extend_from_slice(hash.as_bytes());
    }
    response.extend_from_slice(&runner.slot_count().to_le_bytes());
    response.extend_from_slice(&manifest.maximum_vector_slots.to_le_bytes());
    response.extend_from_slice(&manifest.maximum_episode_steps.to_le_bytes());
    response.extend_from_slice(&manifest.physics_hz.to_le_bytes());
    response.extend_from_slice(&manifest.motor_hz.to_le_bytes());
    response.extend_from_slice(&84_u32.to_le_bytes());
    response.extend_from_slice(&23_u32.to_le_bytes());
    push_len(&mut response, manifest.reward_components.len())?;
    for component in &manifest.reward_components {
        push_text(&mut response, component.component_id.as_str())?;
        response.extend_from_slice(&component.coefficient_q16.to_le_bytes());
        response.extend_from_slice(&component.minimum_raw.to_le_bytes());
        response.extend_from_slice(&component.maximum_raw.to_le_bytes());
    }
    session.runner = Some(runner);
    Ok(response)
}

fn process_reset(runner: &mut ProtocolRunner, payload: &[u8]) -> Result<Vec<u8>, ProtocolFailure> {
    let mut reader = PayloadReader::new(payload);
    let count = reader.read_len(256)?;
    let vector_slots = (0..count)
        .map(|_| reader.read_u32())
        .collect::<Result<Vec<_>, _>>()?;
    reader.finish()?;
    let values = match runner {
        ProtocolRunner::Legacy(runner) => runner.reset_slots(&vector_slots)?,
        ProtocolRunner::BiomechanicsStanding(runner) => runner.reset_slots(&vector_slots)?,
    };
    let mut response = Vec::new();
    push_len(&mut response, values.len())?;
    for value in values {
        response.extend_from_slice(&value.episode_ordinal.to_le_bytes());
        response.extend_from_slice(&value.vector_slot.to_le_bytes());
        push_i64_values(&mut response, &value.observation_raw)?;
        response.extend_from_slice(
            value
                .seed_set
                .seed_set_hash()
                .map_err(|_| ProtocolFailure::Encoding)?
                .as_bytes(),
        );
        response.extend_from_slice(value.reset_record.reset_root.as_bytes());
    }
    Ok(response)
}

fn process_step(runner: &mut ProtocolRunner, payload: &[u8]) -> Result<Vec<u8>, ProtocolFailure> {
    let mut reader = PayloadReader::new(payload);
    let count = reader.read_len(256)?;
    let mut inputs = Vec::with_capacity(count);
    for _ in 0..count {
        inputs.push(VectorPolicyStepInput {
            vector_slot: reader.read_u32()?,
            episode_ordinal: reader.read_u64()?,
            action_microradians: reader.read_i64_values(4_096)?,
        });
    }
    reader.finish()?;
    match runner {
        ProtocolRunner::Legacy(runner) => {
            encode_legacy_steps(runner.step_actions_lockstep(inputs)?)
        }
        ProtocolRunner::BiomechanicsStanding(runner) => {
            encode_biomechanics_steps(runner.step_actions_lockstep(inputs)?)
        }
    }
}

fn encode_legacy_steps(
    values: Vec<next_motor::VectorStepOutput>,
) -> Result<Vec<u8>, ProtocolFailure> {
    let mut response = Vec::new();
    push_len(&mut response, values.len())?;
    for value in values {
        response.extend_from_slice(&value.episode_ordinal.to_le_bytes());
        response.extend_from_slice(&value.vector_slot.to_le_bytes());
        response.extend_from_slice(&value.frame.motor_tick.to_le_bytes());
        for command in [value.command_raw, value.next_command_raw] {
            for component in command {
                response.extend_from_slice(&component.to_le_bytes());
            }
        }
        push_i64_values(&mut response, &value.frame.applied_action_microradians)?;
        push_i64_values(&mut response, &value.frame.observation_raw)?;
        push_len(&mut response, value.reward_components_raw.len())?;
        for (component_id, component_value) in &value.reward_components_raw {
            push_text(&mut response, component_id.as_str())?;
            response.extend_from_slice(&component_value.to_le_bytes());
        }
        response.extend_from_slice(&value.reward_total_q16.to_le_bytes());
        response.push(u8::from(value.terminated));
        response.push(u8::from(value.truncated));
        if let Some(reason) = &value.terminal_reason_id {
            response.push(1);
            push_text(&mut response, reason.as_str())?;
        } else {
            response.push(0);
        }
        response.extend_from_slice(value.step_record.physics_root.as_bytes());
        response.extend_from_slice(value.step_record.motor_root.as_bytes());
        response.extend_from_slice(value.step_record.step_root.as_bytes());
        let root = value
            .frame
            .snapshot
            .links
            .first()
            .ok_or(ProtocolFailure::Encoding)?;
        push_i64_array(&mut response, root.position_micrometres);
        push_i64_array(&mut response, root.rotation_q1_30);
        push_i64_array(&mut response, root.linear_velocity_micrometres_per_second);
        push_i64_array(&mut response, root.angular_velocity_microradians_per_second);
        push_i64_values(
            &mut response,
            value
                .frame
                .observation_raw
                .get(10..33)
                .ok_or(ProtocolFailure::Encoding)?,
        )?;
        push_i64_values(
            &mut response,
            value
                .frame
                .observation_raw
                .get(33..56)
                .ok_or(ProtocolFailure::Encoding)?,
        )?;
        let contact_offset = value.frame.observation_raw.len().saturating_sub(2);
        push_i64_values(
            &mut response,
            value
                .frame
                .observation_raw
                .get(contact_offset..)
                .ok_or(ProtocolFailure::Encoding)?,
        )?;
    }
    Ok(response)
}

fn encode_biomechanics_steps(
    values: Vec<next_motor::BiomechanicsStandingVectorStepOutput>,
) -> Result<Vec<u8>, ProtocolFailure> {
    let mut response = Vec::new();
    push_len(&mut response, values.len())?;
    for value in values {
        response.extend_from_slice(&value.episode_ordinal.to_le_bytes());
        response.extend_from_slice(&value.vector_slot.to_le_bytes());
        response.extend_from_slice(&value.frame.motor_tick.to_le_bytes());
        for command in [value.command_raw, value.next_command_raw] {
            for component in command {
                response.extend_from_slice(&component.to_le_bytes());
            }
        }
        push_i64_values(&mut response, &value.frame.applied_action_q1_30)?;
        push_i64_values(&mut response, &value.frame.observation_raw)?;
        push_len(&mut response, value.reward_components_raw.len())?;
        for (component_id, component_value) in &value.reward_components_raw {
            push_text(&mut response, component_id.as_str())?;
            response.extend_from_slice(&component_value.to_le_bytes());
        }
        response.extend_from_slice(&value.reward_total_q16.to_le_bytes());
        response.push(u8::from(value.terminated));
        response.push(u8::from(value.truncated));
        if let Some(reason) = &value.terminal_reason_id {
            response.push(1);
            push_text(&mut response, reason.as_str())?;
        } else {
            response.push(0);
        }
        response.extend_from_slice(value.step_record.physics_root.as_bytes());
        response.extend_from_slice(value.step_record.motor_root.as_bytes());
        response.extend_from_slice(value.step_record.step_root.as_bytes());
        let root = value
            .frame
            .snapshot
            .links
            .first()
            .ok_or(ProtocolFailure::Encoding)?;
        push_i64_array(&mut response, root.position_micrometres);
        push_i64_array(&mut response, root.rotation_q1_30);
        push_i64_array(&mut response, root.linear_velocity_micrometres_per_second);
        push_i64_array(&mut response, root.angular_velocity_microradians_per_second);
        push_i64_values(
            &mut response,
            value
                .frame
                .observation_raw
                .get(10..33)
                .ok_or(ProtocolFailure::Encoding)?,
        )?;
        push_i64_values(
            &mut response,
            value
                .frame
                .observation_raw
                .get(33..56)
                .ok_or(ProtocolFailure::Encoding)?,
        )?;
        push_i64_values(&mut response, &value.frame.contact_flags)?;
    }
    Ok(response)
}

fn process_checkpoint(runner: &ProtocolRunner, payload: &[u8]) -> Result<Vec<u8>, ProtocolFailure> {
    let mut reader = PayloadReader::new(payload);
    let vector_slot = reader.read_u32()?;
    let episode_ordinal = reader.read_u64()?;
    reader.finish()?;
    let envelope = runner.checkpoint_slot(vector_slot, episode_ordinal)?;
    let bytes = envelope.canonical_bytes()?;
    let mut response = Vec::new();
    push_bytes(&mut response, &bytes)?;
    Ok(response)
}

fn process_restore(
    runner: &mut ProtocolRunner,
    payload: &[u8],
) -> Result<Vec<u8>, ProtocolFailure> {
    let mut reader = PayloadReader::new(payload);
    let envelope_bytes = reader.read_bytes(MAX_REQUEST_BYTES)?;
    reader.finish()?;
    let envelope = MotorEnvironmentCheckpointEnvelopeV1::from_canonical_bytes(envelope_bytes)?;
    let observation = match runner {
        ProtocolRunner::Legacy(runner) => runner.restore_slot(&envelope)?,
        ProtocolRunner::BiomechanicsStanding(_) => {
            return Err(ProtocolFailure::OperationUnsupported);
        }
    };
    let mut response = Vec::new();
    response.extend_from_slice(&envelope.episode_ordinal.to_le_bytes());
    response.extend_from_slice(&envelope.vector_slot.to_le_bytes());
    push_i64_values(&mut response, &observation)?;
    Ok(response)
}

fn process_ping(session: &ProtocolSession, payload: &[u8]) -> Result<Vec<u8>, ProtocolFailure> {
    if !payload.is_empty() {
        return Err(ProtocolFailure::Payload);
    }
    let mut response = Vec::new();
    if let Some(runner) = &session.runner {
        response.push(1);
        response.extend_from_slice(&runner.slot_count().to_le_bytes());
        push_text(&mut response, runner.profile_id())?;
    } else {
        response.push(0);
        response.extend_from_slice(&0_u32.to_le_bytes());
        push_text(&mut response, "")?;
    }
    Ok(response)
}

fn require_runner(session: &mut ProtocolSession) -> Result<&mut ProtocolRunner, ProtocolFailure> {
    session.runner.as_mut().ok_or(ProtocolFailure::NotCreated)
}

fn require_runner_mut(
    session: &mut ProtocolSession,
) -> Result<&mut ProtocolRunner, ProtocolFailure> {
    require_runner(session)
}

fn write_response(
    writer: &mut impl Write,
    opcode: u16,
    request_id: u64,
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
        .write_all(&request_id.to_le_bytes())
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&payload)
        .map_err(|error| error.to_string())
}

fn push_len(bytes: &mut Vec<u8>, value: usize) -> Result<(), ProtocolFailure> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| ProtocolFailure::Encoding)?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ProtocolFailure> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_i64_values(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), ProtocolFailure> {
    push_len(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn push_i64_array<const N: usize>(bytes: &mut Vec<u8>, values: [i64; N]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProtocolFailure> {
    push_bytes(bytes, value.as_bytes())
}

struct PayloadReader<'a> {
    remaining: &'a [u8],
}

impl<'a> PayloadReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
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

    fn read_hash(&mut self) -> Result<ContentHash, ProtocolFailure> {
        Ok(ContentHash::from_bytes(
            self.take(32)?
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

    fn read_bytes(&mut self, maximum: usize) -> Result<&'a [u8], ProtocolFailure> {
        let length = self.read_len(maximum)?;
        self.take(length)
    }

    fn read_text(&mut self, maximum: usize) -> Result<String, ProtocolFailure> {
        std::str::from_utf8(self.read_bytes(maximum)?)
            .map(str::to_owned)
            .map_err(|_| ProtocolFailure::Payload)
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
    BiomechanicsStanding(BiomechanicsStandingRunnerError),
    Contract(MotorContractError),
    Opcode,
    Payload,
    Capacity,
    Encoding,
    StaleRequest,
    NotCreated,
    AlreadyCreated,
    OperationUnsupported,
}

impl ProtocolFailure {
    const fn status(&self) -> u32 {
        match self {
            Self::Training(_) | Self::BiomechanicsStanding(_) | Self::Contract(_) => 1,
            Self::Opcode => 2,
            Self::Payload => 3,
            Self::Capacity => 4,
            Self::Encoding => 5,
            Self::StaleRequest => 6,
            Self::NotCreated | Self::AlreadyCreated => 7,
            Self::OperationUnsupported => 8,
        }
    }

    const fn stable_code(&self) -> &'static str {
        match self {
            Self::Training(error) => error.stable_code(),
            Self::BiomechanicsStanding(error) => error.stable_code(),
            Self::Contract(error) => error.stable_code(),
            Self::Opcode => "MOTOR_LAB_OPCODE_UNSUPPORTED",
            Self::Payload => "MOTOR_LAB_PAYLOAD_INVALID",
            Self::Capacity => "MOTOR_LAB_CAPACITY_EXCEEDED",
            Self::Encoding => "MOTOR_LAB_RESPONSE_ENCODING_FAILED",
            Self::StaleRequest => "MOTOR_LAB_REQUEST_ID_STALE",
            Self::NotCreated => "MOTOR_LAB_ENVIRONMENT_NOT_CREATED",
            Self::AlreadyCreated => "MOTOR_LAB_ENVIRONMENT_ALREADY_CREATED",
            Self::OperationUnsupported => "MOTOR_LAB_OPERATION_UNSUPPORTED",
        }
    }
}

impl From<TrainingEnvironmentError> for ProtocolFailure {
    fn from(value: TrainingEnvironmentError) -> Self {
        Self::Training(value)
    }
}

impl From<BiomechanicsStandingRunnerError> for ProtocolFailure {
    fn from(value: BiomechanicsStandingRunnerError) -> Self {
        Self::BiomechanicsStanding(value)
    }
}

impl From<MotorContractError> for ProtocolFailure {
    fn from(value: MotorContractError) -> Self {
        Self::Contract(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    use next_motor::FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID;

    #[test]
    fn command_line_options_are_rejected_in_protocol_v2() {
        assert_eq!(run(&["--slots".to_owned(), "1".to_owned()]), 2);
    }

    #[test]
    fn request_header_rejects_version_before_payload_decode() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&OP_STEP.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        let error = read_request(&mut &bytes[..]).expect_err("old version");
        assert_eq!(error, "MOTOR_LAB_PROTOCOL_VERSION_UNSUPPORTED");
    }

    #[test]
    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn protocol_v2_create_partial_reset_step_checkpoint_restore_ping_and_close() {
        let mut requests = Vec::new();
        push_request(&mut requests, OP_CREATE, 1, &create_payload(2));
        let mut reset = Vec::new();
        reset.extend_from_slice(&2_u32.to_le_bytes());
        reset.extend_from_slice(&1_u32.to_le_bytes());
        reset.extend_from_slice(&0_u32.to_le_bytes());
        push_request(&mut requests, OP_RESET, 2, &reset);
        push_request(&mut requests, OP_STEP, 3, &step_payload(2, 1));
        let mut checkpoint = Vec::new();
        checkpoint.extend_from_slice(&0_u32.to_le_bytes());
        checkpoint.extend_from_slice(&1_u64.to_le_bytes());
        push_request(&mut requests, OP_CHECKPOINT, 4, &checkpoint);
        push_request(&mut requests, OP_PING, 5, &[]);
        push_request(&mut requests, OP_CLOSE, 6, &[]);
        let mut output = Vec::new();
        serve(&requests[..], &mut output).expect("serve");
        assert!(output.starts_with(MAGIC));
        assert_eq!(
            output
                .windows(MAGIC.len())
                .filter(|value| *value == MAGIC)
                .count(),
            6
        );

        let checkpoint_payload = response_payload(&output, 3).expect("checkpoint response");
        let mut restore_requests = Vec::new();
        push_request(&mut restore_requests, OP_CREATE, 1, &create_payload(2));
        push_request(&mut restore_requests, OP_RESTORE, 2, checkpoint_payload);
        push_request(&mut restore_requests, OP_RESTORE, 3, checkpoint_payload);
        push_request(&mut restore_requests, OP_CLOSE, 4, &[]);
        let mut restore_output = Vec::new();
        serve(&restore_requests[..], &mut restore_output).expect("restore serve");
        assert_eq!(
            restore_output
                .windows(MAGIC.len())
                .filter(|value| *value == MAGIC)
                .count(),
            4
        );
    }

    #[test]
    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn stale_request_id_and_malformed_batch_fail_without_mutation() {
        let mut session = ProtocolSession::default();
        process_request(
            &mut session,
            Request {
                opcode: OP_CREATE,
                request_id: 1,
                payload: create_payload(1),
            },
        )
        .expect("create");
        let reset_payload = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&1_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes
        };
        process_request(
            &mut session,
            Request {
                opcode: OP_RESET,
                request_id: 2,
                payload: reset_payload,
            },
        )
        .expect("reset");
        let before = session
            .runner
            .as_ref()
            .expect("runner")
            .checkpoint_slot(0, 1)
            .expect("checkpoint")
            .canonical_bytes()
            .expect("encode");
        let stale = process_request(
            &mut session,
            Request {
                opcode: OP_PING,
                request_id: 2,
                payload: Vec::new(),
            },
        )
        .expect_err("stale");
        assert_eq!(stale.stable_code(), "MOTOR_LAB_REQUEST_ID_STALE");
        let malformed = process_request(
            &mut session,
            Request {
                opcode: OP_STEP,
                request_id: 3,
                payload: vec![1, 0, 0],
            },
        )
        .expect_err("malformed");
        assert_eq!(malformed.stable_code(), "MOTOR_LAB_PAYLOAD_INVALID");
        let missing = process_request(
            &mut session,
            Request {
                opcode: OP_STEP,
                request_id: 4,
                payload: 0_u32.to_le_bytes().to_vec(),
            },
        )
        .expect_err("missing slot");
        assert_eq!(missing.stable_code(), "MOTOR_ENV_BATCH_INCOMPLETE");
        let after = session
            .runner
            .as_ref()
            .expect("runner")
            .checkpoint_slot(0, 1)
            .expect("checkpoint")
            .canonical_bytes()
            .expect("encode");
        assert_eq!(after, before);
    }

    #[test]
    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn create_accepts_only_engine_known_profile_id() {
        let mut session = ProtocolSession::default();
        let mut payload = Vec::new();
        push_text(&mut payload, "nextengine.motor.env.unknown").expect("profile");
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(&[3; 32]);
        let error = process_request(
            &mut session,
            Request {
                opcode: OP_CREATE,
                request_id: 1,
                payload,
            },
        )
        .expect_err("unknown profile");
        assert_eq!(error.stable_code(), "UNSUPPORTED_MOTOR_ENVIRONMENT_PROFILE");
        assert!(session.runner.is_none());
    }

    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn create_payload(slots: u32) -> Vec<u8> {
        let mut payload = Vec::new();
        push_text(&mut payload, FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID).expect("profile");
        payload.extend_from_slice(&slots.to_le_bytes());
        payload.extend_from_slice(&[2; 32]);
        payload
    }

    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn step_payload(slots: u32, episode: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&slots.to_le_bytes());
        for slot in 0..slots {
            payload.extend_from_slice(&slot.to_le_bytes());
            payload.extend_from_slice(&episode.to_le_bytes());
            payload.extend_from_slice(&23_u32.to_le_bytes());
            for _ in 0..23 {
                payload.extend_from_slice(&0_i64.to_le_bytes());
            }
        }
        payload
    }

    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn push_request(bytes: &mut Vec<u8>, opcode: u16, request_id: u64, payload: &[u8]) {
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        bytes.extend_from_slice(&opcode.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&request_id.to_le_bytes());
        bytes.extend_from_slice(payload);
    }

    #[cfg(any(feature = "mock-abi", feature = "physx-sdk"))]
    fn response_payload(output: &[u8], response_index: usize) -> Option<&[u8]> {
        let mut offset = 0;
        for index in 0..=response_index {
            let header = output.get(offset..offset + 28)?;
            let payload_length = u32::from_le_bytes(header[16..20].try_into().ok()?) as usize;
            let payload = output.get(offset + 28..offset + 28 + payload_length)?;
            if index == response_index {
                return Some(payload);
            }
            offset += 28 + payload_length;
        }
        None
    }
}
