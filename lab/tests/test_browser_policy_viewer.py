from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from urllib.request import urlopen

from next_lab.browser_policy_viewer import BrowserPolicyViewer


class BrowserPolicyViewerTests(unittest.TestCase):
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
                url = viewer.start({"checkpoint": "model_7.pt"}, open_browser=False)
                viewer.publish({"viewer_step": 12})
                with urlopen(url + "config.json") as response:
                    self.assertEqual(json.load(response)["checkpoint"], "model_7.pt")
                with urlopen(url + "state.json") as response:
                    self.assertEqual(json.load(response)["viewer_step"], 12)
                with urlopen(url + "three.module.js") as response:
                    self.assertEqual(response.read(), b"export {};")
            finally:
                viewer.close()


if __name__ == "__main__":
    unittest.main()
