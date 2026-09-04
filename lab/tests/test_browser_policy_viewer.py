from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from next_lab.browser_policy_viewer import VIEWER_HTML, BrowserPolicyViewer


class BrowserPolicyViewerTests(unittest.TestCase):
    def test_viewer_uses_physical_shapes_and_body_orientations(self) -> None:
        self.assertIn(b"BoxGeometry", VIEWER_HTML)
        self.assertIn(b"CylinderGeometry", VIEWER_HTML)
        self.assertIn(b"SphereGeometry", VIEWER_HTML)
        self.assertIn(b"body_quaternions_xyzw", VIEWER_HTML)

    def test_local_server_exposes_config_module_and_latest_state(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            assets = Path(temporary)
            (assets / "three.module.min.js").write_text("export {};", encoding="utf-8")
            (assets / "three.core.min.js").write_text("export {};", encoding="utf-8")
            files = {
                path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                for path in assets.glob("three.*.js")
            }
            (assets / "manifest.json").write_text(
                json.dumps(
                    {
                        "schema": "nextengine.browser-viewer-assets.v1",
                        "files": files,
                    }
                ),
                encoding="utf-8",
            )
            viewer = BrowserPolicyViewer(assets)
            try:
                url = viewer.start(
                    {
                        "checkpoint": "model_7.pt",
                        "checkpoints": [
                            {"iteration": 2, "name": "model_2.pt"},
                            {"iteration": 7, "name": "model_7.pt"},
                        ],
                    },
                    open_browser=False,
                )
                viewer.publish({"viewer_step": 12})
                with urlopen(url + "config.json") as response:
                    self.assertEqual(json.load(response)["checkpoint"], "model_7.pt")
                with urlopen(url + "state.json") as response:
                    self.assertEqual(json.load(response)["viewer_step"], 12)
                with urlopen(url + "three.module.js") as response:
                    self.assertEqual(response.read(), b"export {};")
                request = Request(
                    url + "checkpoint",
                    data=json.dumps({"checkpoint": "model_2.pt"}).encode("utf-8"),
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                with urlopen(request) as response:
                    self.assertEqual(response.status, 202)
                self.assertEqual(viewer.take_checkpoint_request(), "model_2.pt")
                self.assertIsNone(viewer.take_checkpoint_request())
                bad_request = Request(
                    url + "checkpoint",
                    data=json.dumps({"checkpoint": "model_99.pt"}).encode("utf-8"),
                    method="POST",
                )
                with self.assertRaises(HTTPError) as context:
                    urlopen(bad_request)
                self.assertEqual(context.exception.code, 400)
            finally:
                viewer.close()


if __name__ == "__main__":
    unittest.main()
