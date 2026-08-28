# PS-2 REALIMPACT metal batch execution preflight — 2026-08-28

## Decision

`RealImpactMetalBatchExecutionAndFixturesFrozen`.

The complete staged execution runner and its deterministic DSP fixtures emit
byte-identical zero-access preflights. This authorizes only the fit-role
acquisition for `86_MetalHoledSpoon`. It does not authorize Spoon holdout,
Spatula shadow, quality/domain/runtime admission, physics, Planter access or a
production schema.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Runner | `8484f9477b6ba9282bce3b6ce08185411c145bc095efa24c8cbe85b53db413cc` |
| Execution manifest | `bffaa21c58a14058ad4d05597c4406b24dd0b969a10d9c6b5b447e92058a8664` |
| Preflight A/B | `1af90ed4175407dcad608bfe816f7c64f1db7a584af630956fc6035af4ae4f68` |
| Parent protocol | `24231974…b724b` / `bd755741…709e3` |
| Runtime | NumPy `2.5.2`, SciPy `1.18.0` |
| New network/member payload | `0 / 0` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-metal-batch-execution-v1`.
The repository contains the runner, architecture boundary and evidence only;
recordings, decoded blocks and generated signals remain external.

## What is now executable

The runner publishes separate immutable stages:

1. `acquire-fit` and `decode-fit` for Holed Spoon only;
2. `select` over three development and two calibration families;
3. `acquire-holdout`, `decode-holdout` and `holdout` for Metal Spoon only,
   requiring a successful immutable selection report;
4. `acquire-shadow`, `decode-shadow` and `shadow` for both Metal Spatulas,
   requiring a successful immutable holdout report.

The fixed DSP profile binds onset detection, the all-region rank-7 modal
baseline, rectangular RFFT bands, SHA-256-counter Box-Muller excitation,
non-negative one/two-exponential fitting, per-listener RMS gain, three gating
metrics and the calibration tie-break. Retry, prefix growth, object
substitution and threshold changes after holdout/shadow remain forbidden.

## Fixture result

The offline control uses a known modal signal plus a seeded two-exponential
subband residual. All six fixture checks pass. The truncated transient fails
selection with energy/flatness ratios `1.0/1.0`; the seeded residual passes and
is selected with aggregate envelope/energy/flatness ratios approximately
`0.01428/0.00421/0.01429`. Normal samples are finite, their fixed 4,096-sample
hash is `9fa3d044…64079`, and all two-exponential fits retain two terms.

This proves implementation discrimination only on the declared synthetic
control. It is not evidence that the residual improves a real object or sounds
like metal.

## Leakage and fallback consequence

The 13 parent requests are no longer one undifferentiated acquisition. Their
opening order is enforced by report dependencies:

```text
repeated preflight
  -> Holed Spoon fit bytes
  -> development/calibration selection report
  -> Metal Spoon holdout bytes/report
  -> two-Spatula shadow bytes/report
```

Any failed stage ends this revision with `authored_clip_required`. A successful
shadow can support only representation transfer; absolute prediction quality,
selective validator risk, exact-domain admission and runtime promotion remain
open.

## Reproduction

The manifest and both preflights were run with:

```text
uv run --no-project --with numpy==2.5.2 --with scipy==1.18.0 \
  --directory lab/scripts python \
  physical_sound_realimpact_metal_batch_execute.py ...
```

`ruff format`, `ruff check`, Python byte compilation, both preflight runs and a
direct byte comparison passed. No Cargo/ProductCheck was run because this is
isolated external P0 Python tooling with no current production consumer.

## Next action

Run exactly `acquire-fit` using one repeated preflight directory as its input,
then `decode-fit`. Do not access `91_MetalSpoon`, either Metal Spatula, Planter
or physics until a separate immutable selection result permits the next stage.
