from __future__ import annotations

import json
import unittest

from nextengine_speech_timeline.protocol import (
    MAX_JSON_BYTES,
    ClientHello,
    ProtocolError,
    SessionStart,
    VadCalibration,
    encode_event,
    parse_client_message,
)


class ProtocolTests(unittest.TestCase):
    def test_hello_and_exact_audio_contract_are_parsed(self) -> None:
        hello = parse_client_message(
            json.dumps({"schema_version": 1, "type": "client.hello", "token": "secret"})
        )
        self.assertEqual(hello, ClientHello("secret"))
        start = parse_client_message(
            json.dumps(
                {
                    "schema_version": 1,
                    "type": "session.start",
                    "session_id": "turn-1",
                    "locale": "ru",
                    "sample_rate_hz": 16_000,
                    "encoding": "pcm_s16le",
                    "channels": 1,
                }
            )
        )
        self.assertEqual(start, SessionStart("turn-1", "ru", 16_000, "pcm_s16le", 1))

    def test_optional_vad_calibration_is_strictly_bounded(self) -> None:
        value = {
            "schema_version": 1,
            "type": "session.start",
            "session_id": "turn-1",
            "locale": "ru",
            "sample_rate_hz": 16_000,
            "encoding": "pcm_s16le",
            "channels": 1,
            "vad_calibration": {"noise_floor_dbfs": -52.5, "duration_ms": 2_000},
        }
        start = parse_client_message(json.dumps(value))
        self.assertEqual(
            start,
            SessionStart(
                "turn-1",
                "ru",
                16_000,
                "pcm_s16le",
                1,
                VadCalibration(-52.5, 2_000),
            ),
        )
        value["vad_calibration"] = {"noise_floor_dbfs": -9, "duration_ms": 2_000}
        with self.assertRaises(ProtocolError):
            parse_client_message(json.dumps(value))

    def test_asr_audio_route_defaults_to_raw_and_accepts_declared_routes(self) -> None:
        value = {
            "schema_version": 1,
            "type": "session.start",
            "session_id": "turn-route",
            "locale": "ru",
            "sample_rate_hz": 16_000,
            "encoding": "pcm_s16le",
            "channels": 1,
            "asr_audio_route": "enhanced",
            "asr_model": "gigaam-v3-e2e-rnnt",
        }
        for route in ("enhanced", "gain_only", "whisper"):
            value["asr_audio_route"] = route
            start = parse_client_message(json.dumps(value))
            self.assertIsInstance(start, SessionStart)
            self.assertEqual(start.asr_audio_route, route)
            self.assertEqual(start.asr_model, "gigaam-v3-e2e-rnnt")

        value["asr_audio_route"] = "automatic"
        with self.assertRaises(ProtocolError) as caught:
            parse_client_message(json.dumps(value))
        self.assertEqual(caught.exception.code, "INVALID_FIELD")
        value["asr_audio_route"] = ["raw"]
        with self.assertRaises(ProtocolError):
            parse_client_message(json.dumps(value))
        value["asr_audio_route"] = "raw"
        value["asr_model"] = "bad model"
        with self.assertRaises(ProtocolError):
            parse_client_message(json.dumps(value))

    def test_version_unknown_fields_and_audio_format_fail_closed(self) -> None:
        invalid = (
            {"schema_version": 2, "type": "client.hello", "token": "x"},
            {"schema_version": 1, "type": "client.hello", "token": "x", "extra": 1},
            {
                "schema_version": 1,
                "type": "session.start",
                "session_id": "turn",
                "locale": None,
                "sample_rate_hz": 44_100,
                "encoding": "pcm_s16le",
                "channels": 1,
            },
        )
        for value in invalid:
            with self.subTest(value=value), self.assertRaises(ProtocolError):
                parse_client_message(json.dumps(value))

    def test_json_bound_is_checked_before_parsing(self) -> None:
        with self.assertRaises(ProtocolError) as caught:
            parse_client_message(b" " * (MAX_JSON_BYTES + 1))
        self.assertEqual(caught.exception.code, "JSON_TOO_LARGE")

    def test_event_encoder_preserves_unicode_and_enforces_bound(self) -> None:
        encoded = encode_event({"schema_version": 1, "type": "x", "text": "Привет"})
        self.assertIn("Привет", encoded)
        with self.assertRaises(ProtocolError):
            encode_event({"schema_version": 1, "type": "x", "text": "x" * MAX_JSON_BYTES})


if __name__ == "__main__":
    unittest.main()
