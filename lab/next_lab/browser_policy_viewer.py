from __future__ import annotations

import hashlib
import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any


ASSET_MANIFEST_SCHEMA = "nextengine.browser-viewer-assets.v1"


VIEWER_HTML = b"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>NextEngine policy viewer</title>
  <style>
    html, body { width: 100%; height: 100%; margin: 0; overflow: hidden; background: #07111f; }
    body { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; color: #e5edf8; }
    canvas { display: block; }
    #panel { position: fixed; left: 16px; top: 16px; padding: 12px 14px; min-width: 300px;
      border: 1px solid #29415f; border-radius: 8px; background: rgba(5, 13, 25, .86);
      box-shadow: 0 8px 30px rgba(0,0,0,.35); white-space: pre; line-height: 1.45; }
    #hint { position: fixed; right: 16px; bottom: 14px; color: #9bb0ca; font-size: 12px; }
    .ok { color: #6ee7b7; } .wait { color: #fbbf24; }
  </style>
</head>
<body>
  <div id="panel"><span class="wait">waiting for Isaac...</span></div>
  <div id="hint">drag: orbit &nbsp; wheel: zoom</div>
  <script type="module">
    import * as THREE from '/three.module.js';

    const config = await fetch('/config.json').then(response => response.json());
    const panel = document.getElementById('panel');
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x07111f);
    scene.fog = new THREE.Fog(0x07111f, 8, 24);

    const camera = new THREE.PerspectiveCamera(52, innerWidth / innerHeight, 0.01, 100);
    const renderer = new THREE.WebGLRenderer({antialias: true});
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.setSize(innerWidth, innerHeight);
    renderer.shadowMap.enabled = true;
    document.body.appendChild(renderer.domElement);

    scene.add(new THREE.HemisphereLight(0xb9d8ff, 0x172033, 2.1));
    const sun = new THREE.DirectionalLight(0xffffff, 2.8);
    sun.position.set(4, 7, 3);
    sun.castShadow = true;
    scene.add(sun);

    const floor = new THREE.Mesh(
      new THREE.PlaneGeometry(200, 200),
      new THREE.MeshStandardMaterial({color: 0x111d2c, roughness: .92})
    );
    floor.rotation.x = -Math.PI / 2;
    floor.receiveShadow = true;
    scene.add(floor);
    const grid = new THREE.GridHelper(200, 400, 0x416487, 0x1c3047);
    grid.position.y = .002;
    scene.add(grid);

    const bodyMeshes = config.bodies.map(body => {
      const mesh = new THREE.Mesh(
        new THREE.SphereGeometry(body.radius_m, 18, 12),
        new THREE.MeshStandardMaterial({color: body.color, roughness: .55, metalness: .08})
      );
      mesh.castShadow = true;
      scene.add(mesh);
      return mesh;
    });
    const linkGeometry = new THREE.CylinderGeometry(.027, .027, 1, 10);
    const linkMaterial = new THREE.MeshStandardMaterial({color: 0x9fb7d4, roughness: .65});
    const links = config.bodies.map(body => {
      if (body.parent_index < 0) return null;
      const mesh = new THREE.Mesh(linkGeometry, linkMaterial);
      mesh.castShadow = true;
      scene.add(mesh);
      return mesh;
    });
    const commandArrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 0, 1), new THREE.Vector3(), 1, 0x4ade80, .18, .09
    );
    scene.add(commandArrow);

    let target = new THREE.Vector3(0, 1, 0);
    let yaw = .78, pitch = .32, distance = 3.8;
    let dragging = false, previousX = 0, previousY = 0;
    renderer.domElement.addEventListener('pointerdown', event => {
      dragging = true; previousX = event.clientX; previousY = event.clientY;
      renderer.domElement.setPointerCapture(event.pointerId);
    });
    renderer.domElement.addEventListener('pointerup', () => dragging = false);
    renderer.domElement.addEventListener('pointermove', event => {
      if (!dragging) return;
      yaw -= (event.clientX - previousX) * .006;
      pitch = Math.max(-.05, Math.min(1.25, pitch + (event.clientY - previousY) * .005));
      previousX = event.clientX; previousY = event.clientY;
    });
    renderer.domElement.addEventListener('wheel', event => {
      distance = Math.max(1.5, Math.min(10, distance * Math.exp(event.deltaY * .001)));
    }, {passive: true});

    function point(value) { return new THREE.Vector3(value[0], value[2], -value[1]); }
    function updateLink(mesh, start, end) {
      const delta = end.clone().sub(start);
      const length = delta.length();
      mesh.position.copy(start).add(end).multiplyScalar(.5);
      mesh.scale.set(1, Math.max(length, .001), 1);
      mesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), delta.normalize());
    }
    function applyState(state) {
      if (!state.body_positions_w) return;
      const positions = state.body_positions_w.map(point);
      positions.forEach((position, index) => {
        bodyMeshes[index].position.copy(position);
        const parent = config.bodies[index].parent_index;
        if (parent >= 0) updateLink(links[index], positions[parent], position);
      });
      target.lerp(positions[config.root_index], .18);
      const right = state.command.right_mps, forward = state.command.forward_mps;
      const length = Math.hypot(right, forward);
      commandArrow.position.set(target.x, .025, target.z);
      commandArrow.visible = length > .001;
      if (commandArrow.visible) {
        commandArrow.setDirection(new THREE.Vector3(right, 0, forward).normalize());
        commandArrow.setLength(Math.min(2.4, .35 + length * .55), .18, .09);
      }
      panel.innerHTML = `<span class="ok">connected</span>  checkpoint ${config.checkpoint}\n` +
        `mode       ${state.command.mode}\n` +
        `command    right ${right.toFixed(2)}  forward ${forward.toFixed(2)} m/s\n` +
        `           yaw ${state.command.yaw_rps.toFixed(2)} rad/s\n` +
        `reward     ${state.reward.toFixed(4)}\n` +
        `root       ${state.root_height_m.toFixed(3)} m\n` +
        `step       ${state.viewer_step}${state.episode_reset ? '  RESET' : ''}`;
    }
    async function poll() {
      while (true) {
        try {
          const response = await fetch('/state.json', {cache: 'no-store'});
          if (response.ok) applyState(await response.json());
        } catch (_) {
          panel.innerHTML = '<span class="wait">waiting for Isaac...</span>';
        }
        await new Promise(resolve => setTimeout(resolve, 16));
      }
    }
    poll();

    function animate() {
      requestAnimationFrame(animate);
      const horizontal = Math.cos(pitch) * distance;
      camera.position.set(
        target.x + Math.sin(yaw) * horizontal,
        target.y + Math.sin(pitch) * distance + .45,
        target.z + Math.cos(yaw) * horizontal
      );
      camera.lookAt(target.x, target.y + .25, target.z);
      renderer.render(scene, camera);
    }
    animate();
    addEventListener('resize', () => {
      camera.aspect = innerWidth / innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(innerWidth, innerHeight);
    });
  </script>
</body>
</html>
"""


class BrowserPolicyViewer:
    """Serve a localhost-only WebGL view of streamed Isaac body facts."""

    def __init__(self, assets_root: Path, host: str = "127.0.0.1", port: int = 0) -> None:
        assets_root = assets_root.resolve()
        manifest_path = assets_root / "manifest.json"
        if not manifest_path.is_file():
            raise FileNotFoundError(f"WebGL viewer manifest does not exist: {manifest_path}")
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if manifest.get("schema") != ASSET_MANIFEST_SCHEMA:
            raise ValueError("unsupported WebGL viewer asset manifest")
        records = manifest.get("files")
        if not isinstance(records, dict):
            raise ValueError("WebGL viewer asset manifest has no file hashes")
        module_path = assets_root / "three.module.min.js"
        core_path = assets_root / "three.core.min.js"
        for path in (module_path, core_path):
            if not path.is_file():
                raise FileNotFoundError(f"Three.js viewer asset does not exist: {path}")
        self._three_module = _closed_asset(module_path, records)
        self._three_core = _closed_asset(core_path, records)
        self._host = host
        self._port = port
        self._config = b"{}"
        self._state = b"{}"
        self._lock = threading.Lock()
        self._server: ThreadingHTTPServer | None = None
        self._thread: threading.Thread | None = None

    @property
    def url(self) -> str:
        if self._server is None:
            raise RuntimeError("browser viewer is not running")
        host, port = self._server.server_address[:2]
        return f"http://{host}:{port}/"

    def start(self, config: dict[str, Any], *, open_browser: bool) -> str:
        if self._server is not None:
            raise RuntimeError("browser viewer is already running")
        self._config = _json_bytes(config)
        self._server = ThreadingHTTPServer((self._host, self._port), _ViewerHandler)
        self._server.daemon_threads = True
        self._server.viewer = self  # type: ignore[attr-defined]
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)
        self._thread.start()
        if open_browser:
            browser_environment = os.environ.copy()
            browser_environment.pop("LD_LIBRARY_PATH", None)
            browser_environment.pop("PYTHONPATH", None)
            subprocess.Popen(
                ["xdg-open", self.url],
                env=browser_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
        return self.url

    def publish(self, state: dict[str, Any]) -> None:
        payload = _json_bytes(state)
        with self._lock:
            self._state = payload

    def close(self) -> None:
        if self._server is None:
            return
        self._server.shutdown()
        self._server.server_close()
        if self._thread is not None:
            self._thread.join(timeout=2.0)
        self._server = None
        self._thread = None

    def response(self, path: str) -> tuple[int, str, bytes]:
        if path == "/":
            return 200, "text/html; charset=utf-8", VIEWER_HTML
        if path == "/three.module.js":
            return 200, "text/javascript; charset=utf-8", self._three_module
        if path == "/three.core.min.js":
            return 200, "text/javascript; charset=utf-8", self._three_core
        if path == "/config.json":
            return 200, "application/json", self._config
        if path == "/state.json":
            with self._lock:
                return 200, "application/json", self._state
        return 404, "text/plain; charset=utf-8", b"not found\n"


class _ViewerHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802
        viewer: BrowserPolicyViewer = self.server.viewer  # type: ignore[attr-defined]
        status, content_type, payload = viewer.response(self.path.split("?", 1)[0])
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, format: str, *args: Any) -> None:
        return


def _json_bytes(value: dict[str, Any]) -> bytes:
    return json.dumps(value, separators=(",", ":"), sort_keys=True).encode("utf-8")


def _closed_asset(path: Path, records: dict[str, Any]) -> bytes:
    payload = path.read_bytes()
    expected = records.get(path.name)
    actual = hashlib.sha256(payload).hexdigest()
    if not isinstance(expected, str) or expected != actual:
        raise ValueError(f"WebGL viewer asset hash mismatch: {path.name}")
    return payload
