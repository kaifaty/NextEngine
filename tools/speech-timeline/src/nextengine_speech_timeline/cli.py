from __future__ import annotations

import argparse
import asyncio
import json
from pathlib import Path
import sys
from typing import Sequence

from . import SERVICE_PROTOCOL
from .profile import ProfileError, build_adapters, load_profile
from .service import SpeechTimelineRuntime
from .transport_websocket import SpeechTimelineWebSocketService


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="next-speech-timeline")
    root.add_argument("--version", action="version", version=SERVICE_PROTOCOL)
    commands = root.add_subparsers(dest="command", required=True)
    serve = commands.add_parser("serve", help="start the resident loopback service")
    serve.add_argument("--profile", type=Path, required=True)
    return root


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parser().parse_args(argv)
    if arguments.command == "serve":
        try:
            profile = load_profile(arguments.profile)
            transcriber, affect = build_adapters(profile)
            runtime = SpeechTimelineRuntime(transcriber, affect)
            service = SpeechTimelineWebSocketService(
                runtime,
                ready_file=profile.service.ready_file,
                port=profile.service.port,
                bounds=profile.service.bounds,
                model_identity={
                    "voxtral_sha256": profile.voxtral.model_sha256,
                    "transcribe_revision": profile.voxtral.runtime_revision,
                    "emotion_model_id": profile.emotion.model_id,
                    "emotion_revision": profile.emotion.model_revision,
                    "emotion_classification": profile.emotion.classification,
                },
            )
            return asyncio.run(_serve(service, profile.service.ready_file))
        except ProfileError as error:
            print(
                json.dumps(
                    {"schema_version": 1, "status": "error", "error": str(error)},
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
        except KeyboardInterrupt:
            return 130
    return 2


async def _serve(service: SpeechTimelineWebSocketService, ready_file: Path) -> int:
    try:
        await service.start()
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "status": "ready",
                    "uri": service.uri,
                    "ready_file": str(ready_file),
                },
                ensure_ascii=False,
            ),
            flush=True,
        )
        await service.serve_forever()
    finally:
        await service.close()
    return 0
