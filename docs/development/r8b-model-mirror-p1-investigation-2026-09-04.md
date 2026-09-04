# R8b biomechanics V3 MODEL-MIRROR-P1 investigation — 2026-09-04

## Result

`MODEL-MIRROR-P1` is `FAILED / WALKING_BLOCKED` for the current combination of
canonical CPU PhysX 5.9 and the Isaac Sim 5.1 GPU mirror. The standing policy
remains a valid nominal CPU standing result, but it is not an admissible parent
for an Isaac-accelerated walking run under ADR-102.

## Exact inputs

- body: `nextengine.body.humanoid-biomechanics-raja-1700.v3`;
- environment: `nextengine.motor.env.humanoid-biomechanics-standing.v2`;
- generation manifest: `614057d5129607e45b0683b047695216c6e66756d7ef6d28fb922b2e2698b696`;
- checkpoint: `model_249.pt`, SHA-256
  `254f9d3d3287533e8516c826de7e84cd883493a32b2712f1d9d8cee0985cf41d`;
- policy action source, command stream, checkpoint/generation identity and all
  environment profile hashes are byte-exact gates in trajectory schema V3.

Artifacts remain outside the repository under
`/home/kaifaty/NextEngine-training/r8b-biomechanics-standing-v3-final/`.

## Observations and discriminators

1. The first 256 x 600 diagnostic falsely reported joint RMSE `0.12694 rad`.
   The biomechanics motor-lab response exposed snapshot construction order,
   while observations and Isaac exposed actuator order. Re-encoding diagnostic
   positions/velocities from canonical observation slices reduces the
   three-tick policy control from `0.04292` to `0.00896 rad`. The serialization
   defect is fixed; the earlier report is invalid evidence.
2. Independent closed-loop execution after that repair completes 256 x 600 on
   both planes. Joint RMSE is `0.05634 rad` and root-velocity RMSE is
   `0.06655 m/s`, above the fixed `0.02 rad` and `0.05 m/s` limits. Root-position
   RMSE `0.02778 m`, contact agreement `1.0`, done-tick agreement `1.0` and
   reward MAE `0.00588` pass. Because actions diverge with observations, this is
   diagnostic rather than the final matched-action gate.
3. A canonical CPU-policy action tape replayed exactly on GPU loses the GPU
   episode at motor tick `171`. The reverse GPU-policy tape loses CPU slot `112`
   at tick `116` with `terminal.fall`. Therefore neither stable closed-loop
   controller supplies one common 600-tick action sequence for the current
   pair, and the normative sample floor cannot be completed.
4. Identical CPU slots are byte-identical. The 256-environment GPU batch has
   mean per-channel joint standard deviation `0.01772 rad` and maximum joint
   spread `0.81710 rad` despite identical reset/command semantics. Enabling
   Isaac enhanced determinism produces the same first-100 values and does not
   discriminate the failure.
5. Enabling PCM in canonical CPU PhysX to mimic the mandatory GPU narrow phase
   causes the nominal CPU policy to terminate on self-collision at tick `119`.
   This counterfactual is rejected and the accepted CPU scene flags are
   restored.

## External evidence

- NVIDIA documents that GPU contact generation requires PCM, whereas the
  canonical CPU profile intentionally disables PCM:
  <https://nvidia-omniverse.github.io/PhysX/physx/5.4.0/docs/GPURigidBodies.html>.
- NVIDIA limits deterministic expectations to the same PhysX release, platform
  and scene/API sequence; this investigation compares distinct pinned releases
  and execution planes:
  <https://nvidia-omniverse.github.io/PhysX/physx/5.8.0/docs/API.html#determinism>.
- Both planes use TGS, but NVIDIA notes solver/contact behavior and convergence
  depend on configuration:
  <https://nvidia-omniverse.github.io/PhysX/physx/5.4.0/docs/RigidBodyDynamics.html#projected-gauss-seidel-and-temporal-gauss-seidel>.

## Decision boundary

Do not relax the fixed correspondence thresholds, relabel the failed sample,
or start an admissible Isaac walking run. The smallest next product decision is
one of:

1. keep ADR-102 sequencing and investigate a newer/more compatible Isaac GPU
   mirror or a morphology/control change under a new identity; or
2. explicitly supersede only the sequencing clause and permit one bounded
   `R&D_ONLY` walking discriminator, while retaining CPU evaluation and
   `MODEL-MIRROR-P1` as hard promotion gates.

Direct canonical CPU PPO would avoid mirror risk but requires a new trainer
adapter and is not the smallest next experiment.
