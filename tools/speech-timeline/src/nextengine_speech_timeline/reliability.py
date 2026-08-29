"""Public-data corpus preparation for utterance-level ASR reliability.

The module deliberately stops before replay, feature fitting, or runtime model
loading.  It validates one current-only recipe, verifies externally stored
source indexes and WAV files, applies the versioned Russian normalizer, and
publishes a deterministic prepared index outside the repository.
"""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import io
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import tempfile
from typing import Iterable, Mapping
import unicodedata
import wave

from .profile import REPOSITORY_ROOT
from .protocol import ASR_AUDIO_ROUTES


RECIPE_KIND = "nextengine.speech-reliability.dataset-recipe"
NORMALIZER_PROFILE = "ru-asr-normalize-v0"
SPLIT_PROFILE = "speaker-source-hash-v0"
PREPARED_INDEX_KIND = "nextengine.speech-reliability.prepared-index"
MAX_RECIPE_BYTES = 1024 * 1024
MAX_SOURCE_INDEX_BYTES = 128 * 1024 * 1024
MAX_SOURCE_INDEX_ROWS = 100_000
MAX_SOURCE_INDEX_LINE_BYTES = 64 * 1024
MAX_TEXT_BYTES = 4096
MAX_AUDIO_SAMPLES = 480_000
MAX_AUGMENTATION_ASSET_SAMPLES = 9_600_000
MAX_WAV_CONTAINER_OVERHEAD_BYTES = 1024 * 1024
_SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
_ID_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{0,127}$")


class ReliabilityCorpusError(RuntimeError):
    """A stable fail-closed corpus-preparation error."""


@dataclass(frozen=True)
class SourceRights:
    license_expression: str
    admission: str
    raw_redistribution: str
    notice: str


@dataclass(frozen=True)
class DatasetSource:
    source_id: str
    role: str
    release: str
    source_url: str
    status: str
    relative_index_path: str | None
    index_sha256: str | None
    rights: SourceRights


@dataclass(frozen=True)
class SplitPolicy:
    seed: str
    train_permyriad: int
    calibration_permyriad: int
    held_out_permyriad: int
    forced_held_out_sources: frozenset[str]


@dataclass(frozen=True)
class DatasetRecipe:
    path: Path
    dataset_id: str
    dataset_revision: int
    calibration_domain: str
    split_policy: SplitPolicy
    augmentation_profile: Mapping[str, object]
    replay_profile: Mapping[str, object]
    sources: tuple[DatasetSource, ...]
    manifest_hash: str


@dataclass(frozen=True)
class TranscriptScore:
    normalized_reference: str
    normalized_hypothesis: str
    word_substitutions: int
    word_deletions: int
    word_insertions: int
    reference_words: int
    character_substitutions: int
    character_deletions: int
    character_insertions: int
    reference_characters: int

    @property
    def exact_match(self) -> bool:
        return self.normalized_reference == self.normalized_hypothesis

    @property
    def wer(self) -> float:
        return _error_rate(
            self.word_substitutions + self.word_deletions + self.word_insertions,
            self.reference_words,
        )

    @property
    def cer(self) -> float:
        return _error_rate(
            self.character_substitutions
            + self.character_deletions
            + self.character_insertions,
            self.reference_characters,
        )

    def as_dict(self) -> dict[str, object]:
        return {
            "normalizer_profile": NORMALIZER_PROFILE,
            "normalized_reference": self.normalized_reference,
            "normalized_hypothesis": self.normalized_hypothesis,
            "exact_match": self.exact_match,
            "wer": self.wer,
            "cer": self.cer,
            "word_errors": {
                "substitutions": self.word_substitutions,
                "deletions": self.word_deletions,
                "insertions": self.word_insertions,
                "reference_units": self.reference_words,
            },
            "character_errors": {
                "substitutions": self.character_substitutions,
                "deletions": self.character_deletions,
                "insertions": self.character_insertions,
                "reference_units": self.reference_characters,
            },
        }


def normalize_ru_asr(text: str) -> str:
    """Apply the exact ``ru-asr-normalize-v0`` scoring transform."""

    if not isinstance(text, str):
        raise ReliabilityCorpusError("transcript must be a string")
    if len(text.encode("utf-8")) > MAX_TEXT_BYTES:
        raise ReliabilityCorpusError("transcript exceeds 4096 UTF-8 bytes")
    normalized = unicodedata.normalize("NFC", text).lower().replace("ё", "е")
    result: list[str] = []
    pending_space = False
    for character in normalized:
        if unicodedata.category(character).startswith("L"):
            if pending_space and result:
                result.append(" ")
            result.append(character)
            pending_space = False
        else:
            pending_space = True
    return "".join(result)


def contains_decimal_digit(text: str) -> bool:
    return any(unicodedata.category(character) == "Nd" for character in text)


def score_transcript(reference: str, hypothesis: str) -> TranscriptScore:
    if contains_decimal_digit(reference):
        raise ReliabilityCorpusError(
            "ru-asr-normalize-v0 excludes references containing decimal digits"
        )
    normalized_reference = normalize_ru_asr(reference)
    normalized_hypothesis = normalize_ru_asr(hypothesis)
    if not normalized_reference:
        raise ReliabilityCorpusError("normalized reference must not be empty")
    word_subs, word_dels, word_ins = _levenshtein_counts(
        normalized_reference.split(), normalized_hypothesis.split()
    )
    reference_characters = tuple(normalized_reference.replace(" ", ""))
    hypothesis_characters = tuple(normalized_hypothesis.replace(" ", ""))
    char_subs, char_dels, char_ins = _levenshtein_counts(
        reference_characters, hypothesis_characters
    )
    return TranscriptScore(
        normalized_reference=normalized_reference,
        normalized_hypothesis=normalized_hypothesis,
        word_substitutions=word_subs,
        word_deletions=word_dels,
        word_insertions=word_ins,
        reference_words=len(normalized_reference.split()),
        character_substitutions=char_subs,
        character_deletions=char_dels,
        character_insertions=char_ins,
        reference_characters=len(reference_characters),
    )


def load_dataset_recipe(path: Path) -> DatasetRecipe:
    resolved = path.expanduser().resolve()
    raw_bytes = _read_bounded_file(resolved, "dataset recipe", MAX_RECIPE_BYTES)
    try:
        raw = json.loads(raw_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ReliabilityCorpusError(
            f"cannot decode dataset recipe JSON: {error}"
        ) from error
    root = _strict_object(
        raw,
        "dataset recipe",
        {
            "schema_version",
            "kind",
            "dataset_id",
            "dataset_revision",
            "calibration_domain",
            "text_normalizer_profile",
            "split_policy",
            "augmentation_profile",
            "replay_profile",
            "sources",
        },
    )
    _bounded_int(root["schema_version"], "dataset recipe schema_version", 0, 0)
    if root["kind"] != RECIPE_KIND:
        raise ReliabilityCorpusError("unsupported dataset recipe kind")
    dataset_id = _identifier(root["dataset_id"], "dataset_id")
    if dataset_id not in {"public-safe-v0", "whisper-research-v0"}:
        raise ReliabilityCorpusError("unsupported speech reliability dataset_id")
    dataset_revision = _bounded_int(
        root["dataset_revision"], "dataset_revision", 1, 2**31 - 1
    )
    calibration_domain = _identifier(root["calibration_domain"], "calibration_domain")
    expected_domain = {
        "public-safe-v0": "generic_public_ru_v0",
        "whisper-research-v0": "whisper_research_ru_v0",
    }[dataset_id]
    if calibration_domain != expected_domain:
        raise ReliabilityCorpusError(
            f"{dataset_id} calibration_domain must be {expected_domain}"
        )
    if root["text_normalizer_profile"] != NORMALIZER_PROFILE:
        raise ReliabilityCorpusError(
            f"text_normalizer_profile must be {NORMALIZER_PROFILE}"
        )
    sources = _parse_sources(root["sources"])
    split_policy = _parse_split_policy(root["split_policy"], sources)
    augmentation = _parse_augmentation_profile(root["augmentation_profile"])
    replay = _parse_replay_profile(root["replay_profile"])
    return DatasetRecipe(
        path=resolved,
        dataset_id=dataset_id,
        dataset_revision=dataset_revision,
        calibration_domain=calibration_domain,
        split_policy=split_policy,
        augmentation_profile=augmentation,
        replay_profile=replay,
        sources=sources,
        manifest_hash=_canonical_hash(root),
    )


def dry_run_reliability_corpus(recipe: DatasetRecipe, store: Path) -> dict[str, object]:
    store_root = _external_directory(store, "corpus store")
    blockers: list[dict[str, str]] = []
    source_rows: list[dict[str, object]] = []
    for source in recipe.sources:
        source_blockers: list[str] = []
        if source.rights.admission == "excluded":
            source_blockers.append("SOURCE_EXCLUDED_BY_RIGHTS")
        elif not _source_is_admitted(recipe, source):
            source_blockers.append("SOURCE_NOT_PUBLIC_SAFE")
        if source.status != "closed":
            source_blockers.append("SOURCE_NOT_HASH_CLOSED")
        else:
            assert source.relative_index_path is not None
            assert source.index_sha256 is not None
            try:
                index_path = _resolve_store_path(store_root, source.relative_index_path)
                actual_hash = f"sha256:{_sha256_file(index_path)}"
                if actual_hash != source.index_sha256:
                    source_blockers.append("SOURCE_INDEX_HASH_MISMATCH")
            except ReliabilityCorpusError:
                source_blockers.append("SOURCE_INDEX_UNAVAILABLE")
        for code in source_blockers:
            blockers.append({"code": code, "source_id": source.source_id})
        source_rows.append(
            {
                "source_id": source.source_id,
                "role": source.role,
                "release": source.release,
                "status": source.status,
                "admission": source.rights.admission,
                "ready": not source_blockers,
                "blockers": source_blockers,
            }
        )
    if recipe.replay_profile["identity_status"] != "closed":
        blockers.append({"code": "REPLAY_IDENTITY_NOT_HASH_CLOSED", "source_id": ""})
    return {
        "schema_version": 0,
        "kind": "nextengine.speech-reliability.corpus-dry-run",
        "status": "ready" if not blockers else "planned_with_blockers",
        "ready_for_prepare": not blockers,
        "dataset_id": recipe.dataset_id,
        "dataset_revision": recipe.dataset_revision,
        "calibration_domain": recipe.calibration_domain,
        "recipe_hash": recipe.manifest_hash,
        "normalizer_profile": NORMALIZER_PROFILE,
        "split_profile": SPLIT_PROFILE,
        "augmentation_profile_hash": _canonical_hash(recipe.augmentation_profile),
        "replay_profile_hash": _canonical_hash(recipe.replay_profile),
        "sources": source_rows,
        "blockers": blockers,
        "store": "explicit_external_store",
        "downloads_performed": 0,
        "privacy": "no_audio_or_transcript_read",
    }


def prepare_reliability_index(
    recipe: DatasetRecipe,
    store: Path,
    output_index: Path,
) -> dict[str, object]:
    store_root = _external_directory(store, "corpus store")
    dry_run = dry_run_reliability_corpus(recipe, store_root)
    if not dry_run["ready_for_prepare"]:
        codes = sorted({str(row["code"]) for row in dry_run["blockers"]})
        raise ReliabilityCorpusError(
            "dataset recipe is not ready for prepare: " + ", ".join(codes)
        )
    destination = _external_output_path(output_index, "prepared index")
    seen_entries: set[tuple[str, str]] = set()
    seen_audio_hashes: set[str] = set()
    prepared: list[dict[str, object]] = []
    excluded_empty = 0
    excluded_digits = 0
    verified_source_indexes: list[dict[str, object]] = []
    for source in recipe.sources:
        if not _source_is_admitted(recipe, source):
            continue
        assert source.relative_index_path is not None
        assert source.index_sha256 is not None
        index_path = _resolve_store_path(store_root, source.relative_index_path)
        source_rows = _load_source_index(
            index_path, source.index_sha256, source.role
        )
        verified_source_indexes.append(
            {
                "source_id": source.source_id,
                "role": source.role,
                "index_sha256": source.index_sha256,
                "rows": len(source_rows),
            }
        )
        for row_number, row in enumerate(source_rows, start=1):
            if source.role != "speech":
                asset_id = _identifier(
                    row["asset_id"],
                    f"{source.source_id} row {row_number} asset_id",
                )
                _record_unique_entry(seen_entries, source.source_id, asset_id)
                partition_group_id = _bounded_string(
                    row["partition_group_id"],
                    f"{source.source_id}/{asset_id} partition_group_id",
                    512,
                )
                relative_audio_path, expected_hash, samples = _verify_indexed_wav(
                    store_root,
                    row,
                    source.source_id,
                    asset_id,
                    MAX_AUGMENTATION_ASSET_SAMPLES,
                )
                _record_unique_audio(seen_audio_hashes, expected_hash, source, asset_id)
                prepared.append(
                    {
                        "schema_version": 0,
                        "entry_kind": source.role,
                        "source_id": source.source_id,
                        "asset_id": asset_id,
                        "partition_group_hash": _partition_group_hash(
                            source.role, source.source_id, partition_group_id
                        ),
                        "split": assign_split(
                            recipe, source.source_id, partition_group_id
                        ),
                        "relative_audio_path": relative_audio_path,
                        "audio_sha256": expected_hash,
                        "samples": samples,
                        "duration_ms": samples * 1000 // 16_000,
                    }
                )
                continue
            clip_id = _identifier(
                row["clip_id"],
                f"{source.source_id} row {row_number} clip_id",
            )
            _record_unique_entry(seen_entries, source.source_id, clip_id)
            speaker_id = _bounded_string(
                row["speaker_id"], f"{source.source_id}/{clip_id} speaker_id", 512
            )
            transcript = _bounded_string(
                row["transcript"],
                f"{source.source_id}/{clip_id} transcript",
                MAX_TEXT_BYTES,
            )
            relative_audio_path, expected_hash, samples = _verify_indexed_wav(
                store_root,
                row,
                source.source_id,
                clip_id,
                MAX_AUDIO_SAMPLES,
            )
            _record_unique_audio(seen_audio_hashes, expected_hash, source, clip_id)
            if contains_decimal_digit(transcript):
                excluded_digits += 1
                continue
            normalized_reference = normalize_ru_asr(transcript)
            if not normalized_reference:
                excluded_empty += 1
                continue
            split = assign_split(recipe, source.source_id, speaker_id)
            prepared.append(
                {
                    "schema_version": 0,
                    "entry_kind": "speech",
                    "source_id": source.source_id,
                    "clip_id": clip_id,
                    "speaker_group_hash": _speaker_group_hash(
                        source.source_id, speaker_id
                    ),
                    "split": split,
                    "relative_audio_path": relative_audio_path,
                    "audio_sha256": expected_hash,
                    "samples": samples,
                    "duration_ms": samples * 1000 // 16_000,
                    "normalized_reference": normalized_reference,
                    "normalizer_profile": NORMALIZER_PROFILE,
                }
            )
    if not any(row["entry_kind"] == "speech" for row in prepared):
        raise ReliabilityCorpusError(
            "no admissible non-empty speech samples were prepared"
        )
    prepared.sort(
        key=lambda item: (
            str(item["source_id"]),
            str(item.get("clip_id", item.get("asset_id", ""))),
        )
    )
    index_bytes = b"".join(_canonical_json(row) + b"\n" for row in prepared)
    _atomic_write(destination, index_bytes)
    prepared_index_hash = f"sha256:{hashlib.sha256(index_bytes).hexdigest()}"
    split_counts: dict[str, int] = {"train": 0, "calibration": 0, "held_out": 0}
    role_counts: dict[str, int] = {"speech": 0, "noise": 0, "rir": 0}
    for row in prepared:
        split_counts[str(row["split"])] += 1
        role_counts[str(row["entry_kind"])] += 1
    split_root = _canonical_hash(
        [
            {
                "entry_kind": row["entry_kind"],
                "source_id": row["source_id"],
                "entry_id": row.get("clip_id", row.get("asset_id")),
                "partition_group_hash": row.get(
                    "speaker_group_hash", row.get("partition_group_hash")
                ),
                "split": row["split"],
            }
            for row in prepared
        ]
    )
    speaker_partition_root = _canonical_hash(
        [
            {
                "source_id": row["source_id"],
                "clip_id": row["clip_id"],
                "speaker_group_hash": row["speaker_group_hash"],
            }
            for row in prepared
            if row["entry_kind"] == "speech"
        ]
    )
    augmentation_partition_root = _canonical_hash(
        [
            {
                "entry_kind": row["entry_kind"],
                "source_id": row["source_id"],
                "asset_id": row["asset_id"],
                "partition_group_hash": row["partition_group_hash"],
            }
            for row in prepared
            if row["entry_kind"] != "speech"
        ]
    )
    return {
        "schema_version": 0,
        "kind": PREPARED_INDEX_KIND,
        "status": "prepared",
        "dataset_id": recipe.dataset_id,
        "dataset_revision": recipe.dataset_revision,
        "calibration_domain": recipe.calibration_domain,
        "recipe_hash": recipe.manifest_hash,
        "prepared_index_sha256": prepared_index_hash,
        "sample_index_root": prepared_index_hash,
        "speaker_partition_root": speaker_partition_root,
        "augmentation_partition_root": augmentation_partition_root,
        "split_assignment_root": split_root,
        "normalizer_profile": NORMALIZER_PROFILE,
        "augmentation_profile_hash": _canonical_hash(recipe.augmentation_profile),
        "replay_profile_hash": _canonical_hash(recipe.replay_profile),
        "feature_schema_hash": _canonical_hash(
            {"feature_schema": recipe.replay_profile["feature_schema"]}
        ),
        "label_schema_hash": _canonical_hash(
            {"target": "exact_match", "normalizer_profile": NORMALIZER_PROFILE}
        ),
        "entries_prepared": len(prepared),
        "samples_prepared": role_counts["speech"],
        "augmentation_assets_prepared": role_counts["noise"] + role_counts["rir"],
        "role_counts": role_counts,
        "split_counts": split_counts,
        "excluded": {
            "empty_normalized_reference": excluded_empty,
            "reference_contains_digits": excluded_digits,
        },
        "source_indexes": verified_source_indexes,
        "output": "explicit_external_prepared_index",
        "privacy": "report_omits_audio_transcript_and_raw_partition_ids",
    }


def verify_indexed_audio(
    store: Path,
    row: Mapping[str, object],
    source_id: str,
    entry_id: str,
    *,
    maximum_samples: int = MAX_AUDIO_SAMPLES,
) -> tuple[str, str, int]:
    """Verify one externally stored indexed WAV and return its identity.

    Public wrapper over the strict prepare-time verification: relative path
    containment, SHA-256 match and the 16 kHz mono pcm_s16le WAV contract.
    """

    return _verify_indexed_wav(store, row, source_id, entry_id, maximum_samples)


def assign_split(recipe: DatasetRecipe, source_id: str, partition_group_id: str) -> str:
    policy = recipe.split_policy
    if source_id in policy.forced_held_out_sources:
        return "held_out"
    payload = (
        f"{SPLIT_PROFILE}\0{policy.seed}\0{recipe.dataset_id}\0"
        f"{source_id}\0{partition_group_id}"
    ).encode("utf-8")
    bucket = int.from_bytes(hashlib.sha256(payload).digest()[:8], "big") % 10_000
    if bucket < policy.train_permyriad:
        return "train"
    if bucket < policy.train_permyriad + policy.calibration_permyriad:
        return "calibration"
    return "held_out"


def write_reliability_report(path: Path, report: Mapping[str, object]) -> None:
    destination = _external_output_path(path, "reliability report")
    encoded = json.dumps(
        report, ensure_ascii=False, indent=2, sort_keys=True
    ).encode("utf-8")
    _atomic_write(destination, encoded)


def _source_is_admitted(recipe: DatasetRecipe, source: DatasetSource) -> bool:
    if source.rights.admission == "allowed":
        return True
    return (
        recipe.dataset_id == "whisper-research-v0"
        and source.rights.admission == "research_only"
    )


def _parse_sources(value: object) -> tuple[DatasetSource, ...]:
    if not isinstance(value, list) or not 1 <= len(value) <= 32:
        raise ReliabilityCorpusError("sources must contain between 1 and 32 entries")
    sources: list[DatasetSource] = []
    seen_ids: set[str] = set()
    for index, item in enumerate(value):
        raw = _strict_object(
            item,
            f"source {index}",
            {
                "source_id",
                "role",
                "release",
                "source_url",
                "status",
                "relative_index_path",
                "index_sha256",
                "rights",
            },
        )
        source_id = _identifier(raw["source_id"], f"source {index} source_id")
        if source_id in seen_ids:
            raise ReliabilityCorpusError(f"duplicate source_id: {source_id}")
        seen_ids.add(source_id)
        role = raw["role"]
        if role not in {"speech", "noise", "rir"}:
            raise ReliabilityCorpusError(f"source {source_id} has unsupported role")
        status = raw["status"]
        if status not in {"planned", "closed"}:
            raise ReliabilityCorpusError(f"source {source_id} has unsupported status")
        relative_index_path = raw["relative_index_path"]
        index_sha256 = raw["index_sha256"]
        if status == "planned":
            if relative_index_path is not None or index_sha256 is not None:
                raise ReliabilityCorpusError(
                    f"planned source {source_id} cannot claim a closed index"
                )
        else:
            relative_index_path = _relative_path(
                relative_index_path, f"source {source_id} relative_index_path"
            )
            index_sha256 = _sha256_value(
                index_sha256, f"source {source_id} index_sha256"
            )
        source_url = _bounded_string(raw["source_url"], f"source {source_id} URL", 2048)
        if not source_url.startswith("https://") or "@" in source_url.split("/", 3)[2]:
            raise ReliabilityCorpusError(
                f"source {source_id} URL must be credential-free HTTPS"
            )
        rights_raw = _strict_object(
            raw["rights"],
            f"source {source_id} rights",
            {"license_expression", "admission", "raw_redistribution", "notice"},
        )
        admission = rights_raw["admission"]
        if admission not in {"allowed", "research_only", "excluded"}:
            raise ReliabilityCorpusError(
                f"source {source_id} has unsupported admission"
            )
        sources.append(
            DatasetSource(
                source_id=source_id,
                role=str(role),
                release=_bounded_string(
                    raw["release"], f"source {source_id} release", 256
                ),
                source_url=source_url,
                status=str(status),
                relative_index_path=(
                    str(relative_index_path)
                    if relative_index_path is not None
                    else None
                ),
                index_sha256=str(index_sha256) if index_sha256 is not None else None,
                rights=SourceRights(
                    license_expression=_bounded_string(
                        rights_raw["license_expression"],
                        f"source {source_id} license_expression",
                        128,
                    ),
                    admission=str(admission),
                    raw_redistribution=_bounded_string(
                        rights_raw["raw_redistribution"],
                        f"source {source_id} raw_redistribution",
                        128,
                    ),
                    notice=_bounded_string(
                        rights_raw["notice"], f"source {source_id} notice", 2048
                    ),
                ),
            )
        )
    return tuple(sources)


def _parse_split_policy(
    value: object, sources: tuple[DatasetSource, ...]
) -> SplitPolicy:
    raw = _strict_object(
        value,
        "split_policy",
        {
            "profile",
            "seed",
            "train_permyriad",
            "calibration_permyriad",
            "held_out_permyriad",
            "forced_held_out_sources",
        },
    )
    if raw["profile"] != SPLIT_PROFILE:
        raise ReliabilityCorpusError(f"split_policy.profile must be {SPLIT_PROFILE}")
    train = _bounded_int(raw["train_permyriad"], "train_permyriad", 1, 9998)
    calibration = _bounded_int(
        raw["calibration_permyriad"], "calibration_permyriad", 1, 9998
    )
    held_out = _bounded_int(raw["held_out_permyriad"], "held_out_permyriad", 1, 9998)
    if train + calibration + held_out != 10_000:
        raise ReliabilityCorpusError("split permyriad values must sum to 10000")
    forced_raw = raw["forced_held_out_sources"]
    if not isinstance(forced_raw, list) or len(forced_raw) > 32:
        raise ReliabilityCorpusError("forced_held_out_sources must be a bounded list")
    forced = frozenset(
        _identifier(item, "forced held-out source") for item in forced_raw
    )
    if len(forced) != len(forced_raw):
        raise ReliabilityCorpusError("forced_held_out_sources contains duplicates")
    source_ids = {source.source_id for source in sources if source.role == "speech"}
    missing = forced - source_ids
    if missing:
        raise ReliabilityCorpusError(
            "forced held-out sources are not declared speech sources: "
            + ", ".join(sorted(missing))
        )
    return SplitPolicy(
        seed=_bounded_string(raw["seed"], "split seed", 256),
        train_permyriad=train,
        calibration_permyriad=calibration,
        held_out_permyriad=held_out,
        forced_held_out_sources=forced,
    )


def _parse_augmentation_profile(value: object) -> Mapping[str, object]:
    raw = _strict_object(
        value,
        "augmentation_profile",
        {"profile", "seed", "max_variants_per_clip", "conditions"},
    )
    if raw["profile"] != "speech-reliability-degradation-v0":
        raise ReliabilityCorpusError("unsupported augmentation profile")
    _bounded_string(raw["seed"], "augmentation seed", 256)
    _bounded_int(raw["max_variants_per_clip"], "max_variants_per_clip", 0, 5)
    conditions = raw["conditions"]
    if not isinstance(conditions, list) or not 1 <= len(conditions) <= 16:
        raise ReliabilityCorpusError(
            "augmentation conditions must contain 1 to 16 entries"
        )
    kinds: set[str] = set()
    for index, condition in enumerate(conditions):
        item = _strict_object(
            condition,
            f"augmentation condition {index}",
            {"kind", "parameters"},
        )
        kind = _identifier(item["kind"], f"augmentation condition {index} kind")
        if kind in kinds:
            raise ReliabilityCorpusError(
                f"duplicate augmentation condition kind: {kind}"
            )
        kinds.add(kind)
        if not isinstance(item["parameters"], dict):
            raise ReliabilityCorpusError(
                "augmentation condition parameters must be an object"
            )
        _ensure_bounded_json(item["parameters"], 16 * 1024, "augmentation parameters")
    return raw


def _parse_replay_profile(value: object) -> Mapping[str, object]:
    raw = _strict_object(
        value,
        "replay_profile",
        {
            "profile",
            "identity_status",
            "asr_adapter_id",
            "model_id",
            "model_revision",
            "model_artifact_sha256",
            "runtime_revision",
            "model_delay_ms",
            "partial_decode_interval_ms",
            "asr_audio_route",
            "sample_rate_hz",
            "channels",
            "encoding",
            "chunk_ms",
            "explicit_finish",
            "retain_diagnostic_audio",
            "feature_schema",
        },
    )
    if raw["profile"] != "speech-timeline-replay-v0":
        raise ReliabilityCorpusError("unsupported replay profile")
    status = raw["identity_status"]
    if status not in {"planned", "closed"}:
        raise ReliabilityCorpusError("replay identity_status must be planned or closed")
    for field in (
        "asr_adapter_id",
        "model_id",
        "model_revision",
        "runtime_revision",
        "feature_schema",
    ):
        _bounded_string(raw[field], f"replay {field}", 256)
    artifact_hash = raw["model_artifact_sha256"]
    if status == "closed":
        _sha256_value(artifact_hash, "replay model_artifact_sha256")
    elif artifact_hash is not None:
        raise ReliabilityCorpusError(
            "planned replay profile cannot claim an artifact hash"
        )
    _bounded_int(raw["model_delay_ms"], "replay model_delay_ms", 0, 10_000)
    _bounded_int(
        raw["partial_decode_interval_ms"],
        "replay partial_decode_interval_ms",
        20,
        10_000,
    )
    if raw["asr_audio_route"] not in ASR_AUDIO_ROUTES:
        raise ReliabilityCorpusError("unsupported replay asr_audio_route")
    _bounded_int(raw["sample_rate_hz"], "replay sample_rate_hz", 16_000, 16_000)
    _bounded_int(raw["channels"], "replay channels", 1, 1)
    if raw["encoding"] != "pcm_s16le":
        raise ReliabilityCorpusError("replay audio must be 16 kHz mono pcm_s16le")
    _bounded_int(raw["chunk_ms"], "replay chunk_ms", 80, 80)
    if (
        raw["explicit_finish"] is not True
        or raw["retain_diagnostic_audio"] is not False
    ):
        raise ReliabilityCorpusError(
            "replay requires explicit_finish=true and retain_diagnostic_audio=false"
        )
    return raw


def _load_source_index(
    path: Path, expected_hash: str, role: str
) -> list[dict[str, object]]:
    data = _read_bounded_file(path, "source index", MAX_SOURCE_INDEX_BYTES)
    actual_hash = f"sha256:{hashlib.sha256(data).hexdigest()}"
    if actual_hash != expected_hash:
        raise ReliabilityCorpusError("source index changed after recipe validation")
    rows: list[dict[str, object]] = []
    for line_number, line in enumerate(data.splitlines(), start=1):
        if not line.strip():
            continue
        if len(line) > MAX_SOURCE_INDEX_LINE_BYTES:
            raise ReliabilityCorpusError(
                f"source index line {line_number} is too large"
            )
        if len(rows) >= MAX_SOURCE_INDEX_ROWS:
            raise ReliabilityCorpusError("source index exceeds 100000 rows")
        try:
            value = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ReliabilityCorpusError(
                f"cannot decode source index line {line_number}: {error}"
            ) from error
        required = {
            "schema_version",
            "relative_audio_path",
            "audio_sha256",
            "samples",
            "sample_rate_hz",
            "channels",
            "encoding",
        }
        if role == "speech":
            required.update(
                {
                    "clip_id",
                    "speaker_id",
                    "transcript",
                }
            )
        else:
            required.update(
                {
                    "asset_id",
                    "partition_group_id",
                }
            )
        row = _strict_object(
            value,
            f"source index line {line_number}",
            required,
        )
        _bounded_int(
            row["schema_version"],
            f"source index line {line_number} schema_version",
            0,
            0,
        )
        _bounded_int(
            row["sample_rate_hz"],
            f"source index line {line_number} sample_rate_hz",
            16_000,
            16_000,
        )
        _bounded_int(
            row["channels"],
            f"source index line {line_number} channels",
            1,
            1,
        )
        if row["encoding"] != "pcm_s16le_wav":
            raise ReliabilityCorpusError(
                f"source index line {line_number} audio must be "
                "16 kHz mono pcm_s16le_wav"
            )
        rows.append(row)
    if not rows:
        raise ReliabilityCorpusError("source index contains no rows")
    return rows


def _record_unique_entry(
    identities: set[tuple[str, str]], source_id: str, entry_id: str
) -> None:
    identity = (source_id, entry_id)
    if identity in identities:
        raise ReliabilityCorpusError(
            f"duplicate source/entry identity: {source_id}/{entry_id}"
        )
    identities.add(identity)


def _record_unique_audio(
    hashes: set[str],
    audio_hash: str,
    source: DatasetSource,
    entry_id: str,
) -> None:
    if audio_hash in hashes:
        raise ReliabilityCorpusError(
            f"duplicate audio content: {source.source_id}/{entry_id}"
        )
    hashes.add(audio_hash)


def _verify_indexed_wav(
    store: Path,
    row: Mapping[str, object],
    source_id: str,
    entry_id: str,
    maximum_samples: int,
) -> tuple[str, str, int]:
    label = f"{source_id}/{entry_id}"
    relative_audio_path = _relative_path(
        row["relative_audio_path"], f"{label} relative_audio_path"
    )
    audio_path = _resolve_store_path(store, relative_audio_path)
    expected_hash = _sha256_value(row["audio_sha256"], f"{label} audio_sha256")
    samples = _bounded_int(row["samples"], f"{label} samples", 1, maximum_samples)
    audio_bytes = _read_bounded_file(
        audio_path,
        f"audio {label}",
        samples * 2 + MAX_WAV_CONTAINER_OVERHEAD_BYTES,
    )
    if f"sha256:{hashlib.sha256(audio_bytes).hexdigest()}" != expected_hash:
        raise ReliabilityCorpusError(f"audio SHA-256 mismatch: {label}")
    _validate_wav(audio_bytes, samples, source_id, entry_id)
    return relative_audio_path, expected_hash, samples


def _validate_wav(
    data: bytes, samples: int, source_id: str, clip_id: str
) -> None:
    try:
        with wave.open(io.BytesIO(data), "rb") as audio:
            metadata = (
                audio.getframerate(),
                audio.getnchannels(),
                audio.getsampwidth(),
                audio.getcomptype(),
            )
            frames = audio.getnframes()
            if metadata != (16_000, 1, 2, "NONE") or frames != samples:
                raise ReliabilityCorpusError(
                    f"audio contract mismatch: {source_id}/{clip_id}"
                )
            if len(audio.readframes(frames)) != frames * 2:
                raise ReliabilityCorpusError(
                    f"audio ended before declared samples: {source_id}/{clip_id}"
                )
    except (OSError, EOFError, wave.Error) as error:
        raise ReliabilityCorpusError(
            f"cannot read audio {source_id}/{clip_id}: {error}"
        ) from error


def _levenshtein_counts(
    reference: Iterable[str], hypothesis: Iterable[str]
) -> tuple[int, int, int]:
    reference_units = tuple(reference)
    hypothesis_units = tuple(hypothesis)
    previous = [(index, 0, 0, index) for index in range(len(hypothesis_units) + 1)]
    for ref_index, ref_unit in enumerate(reference_units, start=1):
        current = [(ref_index, 0, ref_index, 0)]
        for hyp_index, hyp_unit in enumerate(hypothesis_units, start=1):
            if ref_unit == hyp_unit:
                current.append(previous[hyp_index - 1])
                continue
            diagonal = previous[hyp_index - 1]
            deletion = previous[hyp_index]
            insertion = current[hyp_index - 1]
            candidates = (
                (diagonal[0] + 1, diagonal[1] + 1, diagonal[2], diagonal[3]),
                (deletion[0] + 1, deletion[1], deletion[2] + 1, deletion[3]),
                (insertion[0] + 1, insertion[1], insertion[2], insertion[3] + 1),
            )
            current.append(min(candidates, key=lambda item: item[0]))
        previous = current
    _, substitutions, deletions, insertions = previous[-1]
    return substitutions, deletions, insertions


def _error_rate(errors: int, reference_units: int) -> float:
    if reference_units <= 0:
        return math.inf
    return errors / reference_units


def _speaker_group_hash(source_id: str, speaker_id: str) -> str:
    payload = (
        f"speech-reliability-speaker-v0\0{source_id}\0{speaker_id}"
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def _partition_group_hash(role: str, source_id: str, group_id: str) -> str:
    payload = (
        f"speech-reliability-partition-v0\0{role}\0{source_id}\0{group_id}"
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def _strict_object(
    value: object,
    name: str,
    required: set[str],
) -> dict[str, object]:
    if not isinstance(value, dict) or any(
        not isinstance(key, str) for key in value
    ):
        raise ReliabilityCorpusError(f"{name} must be an object with string keys")
    keys = set(value)
    missing = required - keys
    extra = keys - required
    if missing:
        raise ReliabilityCorpusError(
            f"{name} is missing fields: {', '.join(sorted(missing))}"
        )
    if extra:
        raise ReliabilityCorpusError(
            f"{name} has unknown fields: {', '.join(sorted(extra))}"
        )
    return dict(value)


def _identifier(value: object, name: str) -> str:
    text = _bounded_string(value, name, 128)
    if not _ID_RE.fullmatch(text):
        raise ReliabilityCorpusError(
            f"{name} is not a valid lowercase stable identifier"
        )
    return text


def _bounded_string(value: object, name: str, maximum_bytes: int) -> str:
    if not isinstance(value, str) or not value or "\x00" in value:
        raise ReliabilityCorpusError(f"{name} must be a non-empty string without NUL")
    if len(value.encode("utf-8")) > maximum_bytes:
        raise ReliabilityCorpusError(f"{name} exceeds {maximum_bytes} UTF-8 bytes")
    return value


def _bounded_int(value: object, name: str, minimum: int, maximum: int) -> int:
    if (
        isinstance(value, bool)
        or not isinstance(value, int)
        or not minimum <= value <= maximum
    ):
        raise ReliabilityCorpusError(
            f"{name} must be an integer in [{minimum}, {maximum}]"
        )
    return value


def _sha256_value(value: object, name: str) -> str:
    if not isinstance(value, str) or _SHA256_RE.fullmatch(value) is None:
        raise ReliabilityCorpusError(
            f"{name} must be sha256: followed by 64 lowercase hex"
        )
    return value


def _relative_path(value: object, name: str) -> str:
    text = _bounded_string(value, name, 1024)
    path = PurePosixPath(text)
    if (
        path.is_absolute()
        or not path.parts
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise ReliabilityCorpusError(f"{name} must be a normalized relative POSIX path")
    return path.as_posix()


def _external_directory(path: Path, name: str) -> Path:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise ReliabilityCorpusError(f"{name} must be outside the repository")
    if not resolved.is_dir():
        raise ReliabilityCorpusError(f"{name} does not exist or is not a directory")
    return resolved


def _external_output_path(path: Path, name: str) -> Path:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise ReliabilityCorpusError(f"{name} must be outside the repository")
    if not resolved.parent.is_dir():
        raise ReliabilityCorpusError(f"{name} parent directory does not exist")
    if resolved.is_dir():
        raise ReliabilityCorpusError(f"{name} must be a file path")
    return resolved


def _resolve_store_path(store: Path, relative: str) -> Path:
    candidate = (store / relative).resolve()
    if not candidate.is_relative_to(store):
        raise ReliabilityCorpusError("external corpus path escapes the explicit store")
    if not candidate.is_file():
        raise ReliabilityCorpusError(f"external corpus file does not exist: {relative}")
    return candidate


def _read_bounded_file(path: Path, name: str, maximum_bytes: int) -> bytes:
    try:
        if not path.is_file():
            raise ReliabilityCorpusError(
                f"{name} does not exist or is not a regular file"
            )
        size = path.stat().st_size
        if not 0 < size <= maximum_bytes:
            raise ReliabilityCorpusError(
                f"{name} size must be between 1 and {maximum_bytes}"
            )
        data = path.read_bytes()
    except OSError as error:
        raise ReliabilityCorpusError(f"cannot read {name}: {error}") from error
    if len(data) != size:
        raise ReliabilityCorpusError(f"{name} changed while it was read")
    return data


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as stream:
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
    except OSError as error:
        raise ReliabilityCorpusError(
            f"cannot hash external corpus file: {error}"
        ) from error
    return digest.hexdigest()


def _canonical_json(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        allow_nan=False,
    ).encode("utf-8")


def _canonical_hash(value: object) -> str:
    return f"sha256:{hashlib.sha256(_canonical_json(value)).hexdigest()}"


def _ensure_bounded_json(value: object, maximum_bytes: int, name: str) -> None:
    try:
        encoded = _canonical_json(value)
    except (TypeError, ValueError) as error:
        raise ReliabilityCorpusError(f"{name} is not finite canonical JSON") from error
    if len(encoded) > maximum_bytes:
        raise ReliabilityCorpusError(f"{name} exceeds {maximum_bytes} bytes")


def _atomic_write(path: Path, data: bytes) -> None:
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
