# Corrected-body training adapter foundation

Status: `PYTHON_ADAPTER_PASS / NATIVE_25_ENVIRONMENT_NOT_IMPLEMENTED / NO_TRAINING`.

The user prioritized connecting the corrected body to walking training over
another gain sweep. The existing Python adapter assumed 23 actions and 84/88
observations. It now derives joint-block sizes from the descriptor, preserving
actuator order, per-joint scales, native handshake checks and profile admission.
The existing native environments and their body/controller identities are unchanged.

## Implemented and checked

- Validate action count, unique joint/actuator references and descriptor/profile
  dimensions before worker creation. Reject old-width action batches and nonfinite
  actions before stepping any worker. Reject short observations instead of NumPy
  broadcasting them across joints.
- Derive sole-height telemetry offsets from the joint blocks. Preserve the final
  observation before autoreset and bootstrap only time-limit terminations.
- 41 focused Python tests pass: canonical adapter, prospective walking selection,
  corrected evaluation, known candidate and V7 checks. Synthetic 25-action clients
  exercise 90/94 observation layouts, distinct channel ordering, resets, terminal
  heights and one installed CPU PPO update. These are interface tests, not native
  25-joint physics or learning-quality evidence.
- Ruff 0.12.10 formatting and lint, Python compilation and `git diff --check` pass.
- Native `next_headless` was rebuilt with the pinned SDK and `physx-sdk` feature.
  `adapter_control` with the unchanged V8 descriptor and training profile V4,
  report-only `num_envs=4, shards=2`, passes 640 transitions and 29 terminal resets.
  All step fields, rewards, roots and pre/post-reset observations match independent
  direct motor-lab workers. Repeat after final shape checks gives the same digest.
  No generation was frozen and no native optimizer run was started.

Python: `/home/kaifaty/NextEngine-training/isaaclab-2.3.2/env/bin/python`,
`PYTHONPATH=lab:.`. Descriptor:
`/home/kaifaty/NextEngine-training/r8b-walking-stop-window-v8/evidence/descriptor-v8.json`.
The control calls `lab.scripts.canonical_walking_ppo.adapter_control` directly;
its reduced worker count is diagnostic only, not a changed training profile.

| Evidence | SHA-256 |
| --- | --- |
| Native binary | `6e94efa56b58290fadadf06933adfdbc3e26519c04fd397674795234dc4bdea7` |
| Descriptor | `d4e43b3e4ad0d08fc69a0327cd086d133375926de1eb4d9c296ef7987e6c560e` |
| Adapter source | `93beac7797590dd2933b6438ee2da4d155bef7c5cd0fd9f34986422fdcf6acb4` |
| Walking script | `4c73a92abfdc0fec9f8f50fcbd90ab69b9aa5ee7db49ccb664ce7518ff9af0fb` |
| Unchanged training profile V4 | `7984e531f19762db0ad94c25b40dafcdb88ab3ceb6bdbae63818bd1429af27d6` |
| Control step-root sequence | `558368fb38af52123877249d6f6818d2a1e3b0db739e524a781d3b2c97f68f6a` |

## Remaining work / next boundary

The native runner still creates BodySchema V4 / 23 DOFs. Its observations,
reward validation, rearfoot-only geometry/contact handling and controller creation
need an explicitly identified successor for the corrected 25-DOF body. Merely
changing its width constants would leave incorrect foot support and body identity.
ADR-125's support helper remains diagnostic-only; this change does not promote it
or choose it as the training controller. Define that successor, implement native
reset/action/contact/reward checks, then freeze a bounded real-body PPO smoke.
Preserve old descriptors, weights and control roots as regression controls.

No native contract, gameplay or persistence code changed in this adapter checkpoint;
workspace host/play/persistence/content checks were not rerun for it. Their passes
for the preceding support diagnostic remain recorded separately. Full calibration
and corrected-body walking readiness remain open.
