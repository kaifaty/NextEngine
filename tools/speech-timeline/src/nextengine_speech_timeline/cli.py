from __future__ import annotations

import argparse
from typing import Sequence

from . import SERVICE_PROTOCOL


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="next-speech-timeline")
    root.add_argument("--version", action="version", version=SERVICE_PROTOCOL)
    return root


def main(argv: Sequence[str] | None = None) -> int:
    parser().parse_args(argv)
    return 0
