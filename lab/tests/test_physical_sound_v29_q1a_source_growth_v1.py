from __future__ import annotations

import copy
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
SCRIPTS = LAB / "scripts"
PROFILE = LAB / "profiles" / "physical-sound-v29-q1a-source-growth.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v29_q1a_source_growth_v1 as q1a  # noqa: E402


def sound_html(
    *,
    author: str = "publisher",
    author_id: int = 7,
    sound_id: int = 11,
    pack_id: int = 13,
    title: str = "Steel plate hit",
    description: str = "A 3mm steel plate hit with a hammer.",
    license_url: str = "https://creativecommons.org/publicdomain/zero/1.0/",
) -> bytes:
    return f"""<!doctype html>
<html><head>
<title>Freesound - {title} by {author}</title>
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
</head><body>
<div class="bw-player" data-sound-id="{sound_id}" data-username="{author}"
 data-user-id="{author_id}" data-title="{title}"></div>
<a href="/people/{author}/packs/{pack_id}/">pack</a>
<a href="{license_url}">license</a>
</body></html>""".encode()


def capture_document(profile_data: bytes, raw: bytes) -> dict[str, object]:
    url = "https://freesound.org/people/publisher/sounds/11/"
    return {
        "access": {
            **q1a.ZERO_SIGNAL_ACCESS,
            "network_requests": 1,
            "publisher_metadata_bytes_read": len(raw),
        },
        "baseline_commit": "a" * 40,
        "baseline_exposure": [],
        "pages": [
            {
                "bytes": len(raw),
                "content_type": "text/html",
                "file": "raw/page.html",
                "final_url": url,
                "sha256": q1a.sha256_bytes(raw),
                "url": url,
            }
        ],
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": q1a.sha256_bytes(profile_data),
        },
        "schema": q1a.CAPTURE_SCHEMA,
    }


class ProfileTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_text())

    def test_checked_in_profile_is_strict_and_has_expected_raw_power(self) -> None:
        validated = q1a.validate_profile(copy.deepcopy(self.profile))
        groups = [group for project in validated["projects"] for group in project["groups"]]
        self.assertEqual(len(validated["projects"]), 8)
        self.assertEqual(
            sum(group["relation"] == "exact_steel_candidate" for group in groups),
            9,
        )
        self.assertEqual(
            sum(group["relation"] == "non_metal_candidate" for group in groups),
            3,
        )

    def test_duplicate_publisher_pack_is_rejected(self) -> None:
        duplicate = copy.deepcopy(self.profile["projects"][0])
        duplicate["groups"][0]["group_id"] = "otherwise-unique-group"
        duplicate["groups"][0]["sound_ids"] = [999999]
        self.profile["projects"].append(duplicate)
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "duplicate publisher/pack"):
            q1a.validate_profile(self.profile)


class PublisherEvidenceTests(unittest.TestCase):
    def test_exact_sound_pack_uploader_and_license_are_normalized(self) -> None:
        sound = q1a.normalized_sound(sound_html(), "publisher", 13, 11)
        self.assertEqual(sound["license_expression"], "CC0-1.0")
        self.assertEqual(sound["title"], "Steel plate hit")
        self.assertEqual(sound["author_id"], "7")

    def test_wrong_uploader_pack_or_license_is_rejected(self) -> None:
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "uploader identity"):
            q1a.normalized_sound(sound_html(), "someone-else", 13, 11)
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "pack membership"):
            q1a.normalized_sound(sound_html(), "publisher", 99, 11)
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "unsupported license"):
            q1a.normalized_sound(
                sound_html(license_url="https://creativecommons.org/licenses/by-nc/4.0/"),
                "publisher",
                13,
                11,
            )

    def test_exact_steel_evidence_passes_but_qualified_steel_does_not(self) -> None:
        group = {
            "action_tokens": ["hit"],
            "evidence_phrase": "steel plate",
            "group_id": "steel-plate",
            "relation": "exact_steel_candidate",
        }
        q1a.validate_group_evidence(
            group,
            [{"title": "Steel plate hit", "description": "A steel plate hit."}],
        )
        for qualified in ("stainless steel plate", "carbon steel plate", "zinc-plated steel plate"):
            with self.subTest(qualified=qualified):
                with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "unqualified Steel"):
                    q1a.validate_group_evidence(
                        group,
                        [{"title": f"{qualified} hit", "description": ""}],
                    )

    def test_group_evidence_requires_whole_action_token(self) -> None:
        group = {
            "action_tokens": ["hit"],
            "evidence_phrase": "steel plate",
            "group_id": "steel-plate",
            "relation": "exact_steel_candidate",
        }
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "impact action"):
            q1a.validate_group_evidence(
                group,
                [{"title": "white steel plate", "description": "No impact verb."}],
            )


class CaptureTests(unittest.TestCase):
    def test_fetch_uses_one_bounded_keepalive_batch_and_normalizes_type(self) -> None:
        urls = [
            "https://freesound.org/people/publisher/sounds/11/",
            "https://freesound.org/people/publisher/sounds/12/",
        ]
        raw = sound_html()

        def fake_run(arguments: list[str], **_: object) -> object:
            for url in urls:
                self.assertEqual(arguments.count(url), 1)
            self.assertIn("--max-filesize", arguments)
            outputs = [
                Path(arguments[index + 1])
                for index, value in enumerate(arguments)
                if value == "--output"
            ]
            self.assertEqual(len(outputs), 2)
            for output in outputs:
                output.write_bytes(raw)
            return q1a.subprocess.CompletedProcess(
                arguments,
                0,
                stdout="".join(
                    f"{index}\t200\ttext/html; charset=utf-8\t{url}\n"
                    for index, url in enumerate(urls)
                ),
                stderr="",
            )

        with mock.patch.object(
            q1a.subprocess, "run", side_effect=fake_run
        ) as mocked_run:
            fetched = q1a.fetch_html_batch(urls)
        self.assertEqual(mocked_run.call_count, 1)
        for url in urls:
            data, final_url, content_type = fetched[url]
            self.assertEqual(data, raw)
            self.assertEqual(final_url, url)
            self.assertEqual(content_type, "text/html")

    def test_capture_rejects_forbidden_signal_counter(self) -> None:
        profile_data = b"{}\n"
        raw = sound_html()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "capture"
            (root / "raw").mkdir(parents=True)
            (root / "raw" / "page.html").write_bytes(raw)
            capture = capture_document(profile_data, raw)
            capture["access"]["audio_preview_bytes_read"] = 1  # type: ignore[index]
            (root / "capture.json").write_bytes(q1a.canonical_json(capture))
            with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "forbidden signal"):
                q1a.read_capture(root, profile_data)

    def test_capture_rejects_symlink_escape(self) -> None:
        profile_data = b"{}\n"
        raw = sound_html()
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary)
            root = temporary_root / "capture"
            outside = temporary_root / "outside.html"
            root.mkdir()
            (root / "raw").mkdir()
            outside.write_bytes(raw)
            os.symlink(outside, root / "raw" / "page.html")
            (root / "capture.json").write_bytes(
                q1a.canonical_json(capture_document(profile_data, raw))
            )
            with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "outside its root"):
                q1a.read_capture(root, profile_data)

    def test_output_inside_repository_is_rejected(self) -> None:
        with self.assertRaisesRegex(q1a.Q1ASourceGrowthError, "outside the repository"):
            q1a.write_fresh_directory(q1a.repository_root() / "forbidden", {"x": b"x"})


class PartitionTests(unittest.TestCase):
    @staticmethod
    def minimums() -> dict[str, int]:
        return {
            "protected_exact_steel_groups_per_role": 16,
            "protected_non_metal_groups_per_role": 35,
            "protected_projects_per_role": 2,
            "reserved_unprotected_projects": 5,
        }

    def test_aggregate_floors_do_not_imply_whole_project_feasibility(self) -> None:
        powers = [
            ("objectfolder", 17, 63),
            ("ycb", 6, 7),
            ("benboncan", 1, 0),
            ("ldezem", 3, 0),
            ("bibow", 1, 0),
            ("kitchen", 1, 1),
            ("juskiddink", 2, 0),
            ("can-drum", 1, 0),
            ("coffee-can", 0, 1),
            ("water-bottle", 0, 1),
        ]
        projects = [
            {
                "exact_steel_groups": steel,
                "non_metal_groups": rejects,
                "project_revision_id": name,
            }
            for name, steel, rejects in powers
        ]
        self.assertEqual(sum(row[1] for row in powers), 32)
        self.assertEqual(sum(row[2] for row in powers), 73)
        result = q1a.solve_protected_partition(projects, self.minimums())
        self.assertFalse(result["feasible"])
        self.assertIsNotNone(result["best_frontier"])

    def test_two_balanced_protected_roles_with_five_reserved_projects_pass(self) -> None:
        powers = [
            ("a-source", 16, 35),
            ("a-support", 0, 0),
            ("b-source", 16, 35),
            ("b-support", 0, 0),
            *[(f"reserved-{index}", 0, 0) for index in range(5)],
        ]
        projects = [
            {
                "exact_steel_groups": steel,
                "non_metal_groups": rejects,
                "project_revision_id": name,
            }
            for name, steel, rejects in powers
        ]
        result = q1a.solve_protected_partition(projects, self.minimums())
        self.assertTrue(result["feasible"])
        selected = result["selected_partition"]
        self.assertGreaterEqual(selected["reserved_unprotected_project_count"], 5)


if __name__ == "__main__":
    unittest.main()
