# Isaac Sim 6 isolated installation — 2026-09-05

## Result and boundary

Isaac Sim **6.0.0.1** installed in the external directory
`/home/kaifaty/NextEngine-training/isaacsim-6.0`, using Python3.12.13 and
PyTorch2.11.0+cu128. Gain Tuner **3.5.2** loads in the bounded smoke test.
Host: RTX3080 10GiB, driver610.43.02, Ubuntu26.04. Driver unchanged.
This is an installation result, not validation of Gain Tuner's inertia/units,
GPU physics throughput, a new training environment, or a trained policy.
Isaac Lab2.3.2 and its Isaac5.1/Python3.11 environment remain separate.

## Installation and repairs

- User authorized uv-cache cleanup, temporary build123d MCP termination, and
  cleanup of this worktree's `target`. Used `uv cache clean` without force and
  `cargo clean --target-dir /home/kaifaty/.codex/worktrees/f0eb/NextEngine/target`.
  Source, checkpoints and training results were not deleted. Build artifacts
  require recompilation; package cache entries can be downloaded again.
- Followed [NVIDIA Python installation](https://docs.isaacsim.omniverse.nvidia.com/6.0.0/installation/install_python.html):
  torch2.11.0 from the cu128 index, then `isaacsim[all,extscache]==6.0.0.1`.
- uv's first-index policy initially rejected mujoco-usd-converter0.2.0.
  Retried with `--index-strategy unsafe-best-match` against NVIDIA and PyPI,
  matching pip's cross-index version selection. Installed167 compatible packages.
- Initial smoke passed its physics assertion but importer extensions could not
  find libxml2.so.2. External `python.sh` now supplies the already-existing
  `/home/kaifaty/NextEngine-training/system-libs/root/usr/lib/x86_64-linux-gnu`
  through process-local LD_LIBRARY_PATH; no system-library/vendor-source patch.
- Final environment occupies26GiB; final filesystem free space70GiB. Other host
  activity also changed disk usage; do not attribute the entire net change to
  this cleanup.

## Checks and exact evidence

All files below are in the external install directory above.

- PASS: `uv pip check` (167 packages), launcher `bash -n`.
- PASS: CUDA tensor sum523776 on RTX3080 with torch2.11.0+cu128.
- PASS: `python.sh smoke.py`, exit0. Dynamic0.2m cube after120 steps settles at
  z0.09999828040599823m; Gain Tuner enabled ID3.5.2. Steps use `render=False`;
  default World physics is not a GPU-training benchmark. Vulkan initializes.
- Final `smoke-local-libs.log` has no `[Error]`/Traceback entries; protobuf
  duplicate-registration and other nonfatal startup warnings remain.
- PASS: preserved old environment imports Isaac5.1.0.0 and torch2.7.0+cu128.
- NOT_RUN: full GUI interaction, Gain Tuner one-DOF oracle on6, Isaac Lab6
  compatibility/admission and training. Native Rust checks unnecessary for this
  installation/documentation-only repository change.
- `smoke.py` SHA256 `a851bc391d06b3c5a9173077a5c9eb3907b7bd1b098243e25e07859041f077ea`.
- `smoke-local-libs.log` SHA256 `159974e9b99cd16443921faf720414fda71f3ee6606d08966870f185ac82ecb1`.
- `requirements-installed.txt` SHA256 `1bd1ad9b00b728cc94167888ebbeb976bdbdb1c48bb190fd8cc9cd63fd69130a`.

## Resume

Launch scripts: external `python.sh SCRIPT.py` and `isaac-sim.sh` (GUI launcher,
not yet interactively tested). Logs include the initial resolution/importer
failures and successful retry. No environment activation/default was changed.
Next: run the existing known-inertia one-DOF comparison under this exact new
installation before selecting auto gains. Do not infer old bugs are fixed
merely because the extension loads. Existing5.1 environment is the fallback.
