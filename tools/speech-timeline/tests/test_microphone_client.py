from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest

from nextengine_speech_timeline.microphone_client import ClientError, load_ready_file


class MicrophoneClientTests(unittest.TestCase):
    def test_private_loopback_ready_file_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            ready = Path(temp_dir) / "ready.json"
            ready.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "uri": "ws://127.0.0.1:1234",
                        "token": "a" * 64,
                        "protocol": "nextengine.speech-timeline/1",
                    }
                ),
                encoding="utf-8",
            )
            ready.chmod(0o600)
            parsed = load_ready_file(ready)
        self.assertEqual(parsed.uri, "ws://127.0.0.1:1234")
        self.assertEqual(parsed.token, "a" * 64)

    def test_remote_or_public_ready_file_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            ready = Path(temp_dir) / "ready.json"
            ready.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "uri": "ws://example.com:1234",
                        "token": "a" * 64,
                        "protocol": "nextengine.speech-timeline/1",
                    }
                ),
                encoding="utf-8",
            )
            ready.chmod(0o644)
            with self.assertRaises(ClientError):
                load_ready_file(ready)
            ready.chmod(0o600)
            with self.assertRaises(ClientError):
                load_ready_file(ready)

    def test_loopback_prefix_cannot_hide_remote_userinfo_host(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            ready = Path(temp_dir) / "ready.json"
            ready.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "uri": "ws://127.0.0.1:1234@evil.example",
                        "token": "a" * 64,
                        "protocol": "nextengine.speech-timeline/1",
                    }
                ),
                encoding="utf-8",
            )
            ready.chmod(0o600)
            with self.assertRaises(ClientError):
                load_ready_file(ready)


if __name__ == "__main__":
    unittest.main()
