from __future__ import annotations

import struct
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Sequence

import numpy as np

MAGIC = b"NEMLAB\0\0"
PROTOCOL_VERSION = 2
REQUEST_HEADER = struct.Struct("<8sHHIQ")
RESPONSE_HEADER = struct.Struct("<8sHHIIQ")

OP_CREATE = 1
OP_RESET = 2
OP_STEP = 3
OP_CHECKPOINT = 4
OP_RESTORE = 5
OP_PING = 6
OP_CLOSE = 255

FLAT_LOCOMOTION_PROFILE_ID = "nextengine.motor.env.humanoid-flat-command.v1"
CURRICULUM_LOCOMOTION_PROFILE_ID = (
    "nextengine.motor.env.humanoid-flat-command-curriculum.v2"
)
STANDING_PROFILE_ID = "nextengine.motor.env.humanoid-standing.v1"
BOUNDED_STANDING_PROFILE_ID = "nextengine.motor.env.humanoid-standing.v2"
BIOMECHANICS_STANDING_PROFILE_ID = (
    "nextengine.motor.env.humanoid-biomechanics-standing.v2"
)
BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v1"
)
BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2 = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v2"
)
BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3 = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v3"
)
BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4 = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v4"
)
BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5 = (
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v5"
)


class MotorLabProtocolError(RuntimeError):
    """A stable motor-lab protocol or engine diagnostic."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


@dataclass(frozen=True)
class RewardComponent:
    component_id: str
    coefficient_q16: int
    minimum_raw: int
    maximum_raw: int


@dataclass(frozen=True)
class EnvironmentDescriptor:
    profile_id: str
    manifest_hash: bytes
    observation_layout_hash: bytes
    action_layout_hash: bytes
    command_schedule_profile_hash: bytes
    reward_profile_hash: bytes
    termination_profile_hash: bytes
    rng_derivation_profile_hash: bytes
    correspondence_profile_hash: bytes
    slots: int
    maximum_slots: int
    maximum_episode_steps: int
    physics_hz: int
    motor_hz: int
    observation_width: int
    action_width: int
    reward_components: tuple[RewardComponent, ...]


@dataclass(frozen=True)
class ResetResult:
    episode_ordinal: int
    vector_slot: int
    observation_raw: np.ndarray
    seed_set_hash: bytes
    reset_root: bytes


@dataclass(frozen=True)
class StepResult:
    episode_ordinal: int
    vector_slot: int
    motor_tick: int
    command_raw: np.ndarray
    next_command_raw: np.ndarray
    applied_action_raw: np.ndarray
    observation_raw: np.ndarray
    reward_components_raw: np.ndarray
    reward_total_q16: int
    terminated: bool
    truncated: bool
    terminal_reason_id: str | None
    physics_root: bytes
    motor_root: bytes
    step_root: bytes
    root_position_micrometres: np.ndarray
    root_quaternion_q1_30_xyzw: np.ndarray
    root_linear_velocity_micrometres_per_second: np.ndarray
    root_angular_velocity_microradians_per_second: np.ndarray
    joint_position_microradians: np.ndarray
    joint_velocity_microradians_per_second: np.ndarray
    contact_flags: np.ndarray


class MotorLabClient:
    """Sequential raw-integer client for canonical headless motor-lab v2."""

    def __init__(
        self,
        executable: str | Path | Sequence[str | Path],
        profile_id: str,
        slots: int,
        run_root: bytes | str,
    ) -> None:
        command = _command(executable)
        self._process = subprocess.Popen(
            [*command, "motor-lab"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self._request_id = 0
        self._closed = False
        try:
            payload = _pack_text(profile_id) + struct.pack("<I", slots) + _hash_bytes(run_root)
            self.descriptor = self._parse_descriptor(self._request(OP_CREATE, payload))
        except BaseException:
            self._terminate()
            raise

    def reset(self, vector_slots: Sequence[int]) -> list[ResetResult]:
        slots = [int(slot) for slot in vector_slots]
        payload = struct.pack("<I", len(slots)) + b"".join(
            struct.pack("<I", slot) for slot in slots
        )
        reader = _Reader(self._request(OP_RESET, payload))
        results = []
        for _ in range(reader.length(self.descriptor.maximum_slots)):
            results.append(
                ResetResult(
                    episode_ordinal=reader.u64(),
                    vector_slot=reader.u32(),
                    observation_raw=reader.i64_vector(self.descriptor.observation_width),
                    seed_set_hash=reader.take(32),
                    reset_root=reader.take(32),
                )
            )
        reader.finish()
        return results

    def step(
        self,
        episode_ordinals: Sequence[int] | np.ndarray,
        actions_raw: Sequence[Sequence[int]] | np.ndarray,
    ) -> list[StepResult]:
        episodes = np.asarray(episode_ordinals)
        actions = np.asarray(actions_raw)
        if episodes.dtype.kind not in "iu" or actions.dtype.kind not in "iu":
            raise TypeError("raw motor-lab step accepts integer episode/action arrays only")
        if episodes.shape != (self.descriptor.slots,):
            raise ValueError("episode_ordinals must have shape [slots]")
        if actions.shape != (self.descriptor.slots, self.descriptor.action_width):
            raise ValueError("actions_raw must have shape [slots, action_width]")
        actions = actions.astype("<i8", copy=False)
        payload = bytearray(struct.pack("<I", self.descriptor.slots))
        for slot in range(self.descriptor.slots):
            payload.extend(struct.pack("<IQI", slot, int(episodes[slot]), actions.shape[1]))
            payload.extend(actions[slot].tobytes(order="C"))
        return self._parse_steps(self._request(OP_STEP, bytes(payload)))

    def step_normalized(
        self,
        episode_ordinals: Sequence[int] | np.ndarray,
        normalized_actions: Sequence[Sequence[float]] | np.ndarray,
    ) -> list[StepResult]:
        return self.step(
            episode_ordinals,
            normalized_action_to_raw(
                normalized_actions,
                self.descriptor.action_width,
                q1_30=self.descriptor.profile_id
                in {
                    BIOMECHANICS_STANDING_PROFILE_ID,
                    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
                    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V2,
                    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V3,
                    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V4,
                    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID_V5,
                },
            ),
        )

    def checkpoint(self, vector_slot: int, episode_ordinal: int) -> bytes:
        reader = _Reader(
            self._request(OP_CHECKPOINT, struct.pack("<IQ", vector_slot, episode_ordinal))
        )
        envelope = reader.bytes(5 * 1024 * 1024)
        reader.finish()
        return envelope

    def restore(self, envelope: bytes) -> ResetResult:
        reader = _Reader(self._request(OP_RESTORE, _pack_bytes(envelope)))
        episode_ordinal = reader.u64()
        vector_slot = reader.u32()
        observation = reader.i64_vector(self.descriptor.observation_width)
        reader.finish()
        return ResetResult(
            episode_ordinal=episode_ordinal,
            vector_slot=vector_slot,
            observation_raw=observation,
            seed_set_hash=b"",
            reset_root=b"",
        )

    def ping(self) -> tuple[bool, int, str]:
        reader = _Reader(self._request(OP_PING, b""))
        created = reader.u8() == 1
        slots = reader.u32()
        profile_id = reader.text(4_096)
        reader.finish()
        return created, slots, profile_id

    def close(self) -> None:
        if self._closed:
            return
        try:
            self._request(OP_CLOSE, b"")
        finally:
            self._closed = True
            if self._process.stdin is not None:
                self._process.stdin.close()
            self._process.wait(timeout=10)
            if self._process.returncode != 0:
                stderr = self._process.stderr.read().decode("utf-8", errors="replace")
                raise RuntimeError(f"motor-lab exited with {self._process.returncode}: {stderr}")

    def __enter__(self) -> MotorLabClient:
        return self

    def __exit__(self, _type: object, _value: object, _traceback: object) -> None:
        self.close()

    def _request(self, opcode: int, payload: bytes) -> bytes:
        if self._closed or self._process.stdin is None or self._process.stdout is None:
            raise RuntimeError("motor-lab client is closed")
        self._request_id += 1
        request_id = self._request_id
        self._process.stdin.write(
            REQUEST_HEADER.pack(MAGIC, PROTOCOL_VERSION, opcode, len(payload), request_id)
        )
        self._process.stdin.write(payload)
        self._process.stdin.flush()
        header = _read_exact(self._process.stdout, RESPONSE_HEADER.size)
        magic, version, response_opcode, status, payload_length, response_id = (
            RESPONSE_HEADER.unpack(header)
        )
        if magic != MAGIC:
            raise MotorLabProtocolError("MOTOR_LAB_PROTOCOL_MAGIC_MISMATCH")
        if version != PROTOCOL_VERSION:
            raise MotorLabProtocolError("MOTOR_LAB_PROTOCOL_VERSION_UNSUPPORTED")
        if response_opcode != opcode or response_id != request_id:
            raise MotorLabProtocolError("MOTOR_LAB_RESPONSE_IDENTITY_MISMATCH")
        response = _read_exact(self._process.stdout, payload_length)
        if status != 0:
            raise MotorLabProtocolError(response.decode("ascii", errors="strict"))
        return response

    def _parse_descriptor(self, payload: bytes) -> EnvironmentDescriptor:
        reader = _Reader(payload)
        profile_id = reader.text(4_096)
        hashes = [reader.take(32) for _ in range(8)]
        slots = reader.u32()
        maximum_slots = reader.u32()
        maximum_episode_steps = reader.u64()
        physics_hz = reader.u32()
        motor_hz = reader.u32()
        observation_width = reader.u32()
        action_width = reader.u32()
        reward_components = tuple(
            RewardComponent(reader.text(4_096), reader.i64(), reader.i64(), reader.i64())
            for _ in range(reader.length(128))
        )
        reader.finish()
        return EnvironmentDescriptor(
            profile_id,
            *hashes,
            slots,
            maximum_slots,
            maximum_episode_steps,
            physics_hz,
            motor_hz,
            observation_width,
            action_width,
            reward_components,
        )

    def _parse_steps(self, payload: bytes) -> list[StepResult]:
        reader = _Reader(payload)
        results = []
        expected_component_ids = tuple(
            component.component_id for component in self.descriptor.reward_components
        )
        for _ in range(reader.length(self.descriptor.maximum_slots)):
            episode_ordinal = reader.u64()
            vector_slot = reader.u32()
            motor_tick = reader.u64()
            command = reader.i64_array(3)
            next_command = reader.i64_array(3)
            applied_action = reader.i64_vector(self.descriptor.action_width)
            observation = reader.i64_vector(self.descriptor.observation_width)
            reward_ids = []
            reward_values = []
            for _ in range(reader.length(128)):
                reward_ids.append(reader.text(4_096))
                reward_values.append(reader.i64())
            if tuple(reward_ids) != expected_component_ids:
                raise MotorLabProtocolError("MOTOR_LAB_REWARD_COMPONENT_ORDER_MISMATCH")
            reward_total_q16 = reader.i64()
            terminated = reader.u8() == 1
            truncated = reader.u8() == 1
            terminal_reason = reader.text(4_096) if reader.u8() == 1 else None
            results.append(
                StepResult(
                    episode_ordinal=episode_ordinal,
                    vector_slot=vector_slot,
                    motor_tick=motor_tick,
                    command_raw=command,
                    next_command_raw=next_command,
                    applied_action_raw=applied_action,
                    observation_raw=observation,
                    reward_components_raw=np.asarray(reward_values, dtype=np.int64),
                    reward_total_q16=reward_total_q16,
                    terminated=terminated,
                    truncated=truncated,
                    terminal_reason_id=terminal_reason,
                    physics_root=reader.take(32),
                    motor_root=reader.take(32),
                    step_root=reader.take(32),
                    root_position_micrometres=reader.i64_array(3),
                    root_quaternion_q1_30_xyzw=reader.i64_array(4),
                    root_linear_velocity_micrometres_per_second=reader.i64_array(3),
                    root_angular_velocity_microradians_per_second=reader.i64_array(3),
                    joint_position_microradians=reader.i64_vector(23),
                    joint_velocity_microradians_per_second=reader.i64_vector(23),
                    contact_flags=reader.i64_vector(2).astype(np.bool_),
                )
            )
        reader.finish()
        return results

    def _terminate(self) -> None:
        self._closed = True
        self._process.kill()
        self._process.wait(timeout=10)


def normalized_action_to_raw(
    normalized_actions: Sequence[Sequence[float]] | np.ndarray,
    action_width: int = 23,
    *,
    q1_30: bool = False,
) -> np.ndarray:
    values = np.asarray(normalized_actions, dtype=np.float64)
    if values.ndim != 2 or values.shape[1] != action_width:
        raise ValueError("normalized actions must have shape [slots, action_width]")
    if not np.isfinite(values).all():
        raise ValueError("normalized actions must be finite")
    clipped = np.clip(values, -1.0, 1.0)
    scale = float(1 << 30) if q1_30 else 1_000_000.0
    return np.rint(clipped * scale).astype(np.int64)


def _command(executable: str | Path | Sequence[str | Path]) -> list[str]:
    if isinstance(executable, (str, Path)):
        return [str(executable)]
    command = [str(value) for value in executable]
    if not command:
        raise ValueError("motor-lab executable command cannot be empty")
    return command


def _hash_bytes(value: bytes | str) -> bytes:
    if isinstance(value, str):
        if len(value) != 64:
            raise ValueError("run_root must be 32 bytes or lowercase sha256 hex")
        try:
            decoded = bytes.fromhex(value)
        except ValueError as error:
            raise ValueError("run_root must be 32 bytes or lowercase sha256 hex") from error
        if value != value.lower():
            raise ValueError("run_root hex must be lowercase")
        value = decoded
    if len(value) != 32:
        raise ValueError("run_root must contain exactly 32 bytes")
    return value


def _pack_bytes(value: bytes) -> bytes:
    return struct.pack("<I", len(value)) + value


def _pack_text(value: str) -> bytes:
    return _pack_bytes(value.encode("utf-8"))


def _read_exact(stream: object, length: int) -> bytes:
    remaining = length
    chunks = []
    while remaining:
        chunk = stream.read(remaining)  # type: ignore[attr-defined]
        if not chunk:
            raise EOFError("motor-lab closed its response stream")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


class _Reader:
    def __init__(self, payload: bytes) -> None:
        self._payload = memoryview(payload)
        self._offset = 0

    def take(self, length: int) -> bytes:
        end = self._offset + length
        if length < 0 or end > len(self._payload):
            raise MotorLabProtocolError("MOTOR_LAB_RESPONSE_TRUNCATED")
        value = self._payload[self._offset:end].tobytes()
        self._offset = end
        return value

    def u8(self) -> int:
        return self.take(1)[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.take(4))[0]

    def u64(self) -> int:
        return struct.unpack("<Q", self.take(8))[0]

    def i64(self) -> int:
        return struct.unpack("<q", self.take(8))[0]

    def length(self, maximum: int) -> int:
        value = self.u32()
        if value > maximum:
            raise MotorLabProtocolError("MOTOR_LAB_RESPONSE_CAPACITY_EXCEEDED")
        return value

    def bytes(self, maximum: int) -> bytes:
        return self.take(self.length(maximum))

    def text(self, maximum: int) -> str:
        try:
            return self.bytes(maximum).decode("utf-8")
        except UnicodeDecodeError as error:
            raise MotorLabProtocolError("MOTOR_LAB_RESPONSE_TEXT_INVALID") from error

    def i64_array(self, length: int) -> np.ndarray:
        return np.frombuffer(self.take(length * 8), dtype="<i8").copy()

    def i64_vector(self, maximum: int) -> np.ndarray:
        return self.i64_array(self.length(maximum))

    def finish(self) -> None:
        if self._offset != len(self._payload):
            raise MotorLabProtocolError("MOTOR_LAB_RESPONSE_TRAILING_BYTES")
