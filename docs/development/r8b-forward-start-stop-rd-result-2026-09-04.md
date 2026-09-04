# R8b forward start/stop R&D result — 2026-09-04

## Result

`R&D_ONLY / NEGATIVE / NO_WALKING / NO_RUNTIME_AUTHORITY`

The one ADR-103 optimizer budget completed, but it did not acquire meaningful
forward locomotion. It learned long safe GPU episodes while mostly remaining in
place. Canonical CPU PhysX also confirms no forward progress and exposes the
already-known mirror safety gap. Do not resume or repeat this optimizer profile.

## Exact lineage

- generation: `nextengine.training.generation.r8b-biomechanics-forward-start-stop-rd.v1`;
- generation manifest: `9554aaa25d3f09debb91ce861f0cadc3abfd72fa9e9528381bf55739dec4a814`;
- profile: `nextengine.isaac-rsl-rl.rtx3080-biomechanics-forward-start-stop.v1`,
  canonical hash `303971780a96e07187881d687be20e34b90d0e7173b83cbcc93a6c71ce1e469c`;
- completed run: `r8b-biomechanics-forward-start-stop-seed43-rd-v1-v2` on
  clean commit `06a574278e7e5c198dba06c67bd8664620a58214`;
- run manifest file SHA-256:
  `8f4d40b98824eca7f98fa90bb08f332def119b4a5dd8e481741f90083d8e4c49`;
- metrics: 250 finite records / 1,024,000 samples,
  `5f370f6d6571d99c882d6f01685b922f5a61e06d4d57366b07d68ab1a41c7400`;
- final `model_249.pt`:
  `b3c67a63862a143486787c3276210d618e5af73a5f1b85b4e61c5d3e32a4ac83`;
- initialization: standing `model_249.pt`
  `254f9d3d3287533e8516c826de7e84cd883493a32b2712f1d9d8cee0985cf41d`,
  model weights only; optimizer, counters, seed and run root are fresh.

The first attempted run stopped before an optimizer step because the shared
biomechanics tensor loader required the standing-only pose normalization. The
loader was corrected on commit `06a57427…8214`; the failed empty run remains
preserved, and only the completed `-v2` run consumes the ADR-103 budget.

## Training observation

Mean reported episode length rises from `16.50` at iteration 0 to `827.49` at
saved checkpoint 75 and ends at `735.97` at checkpoint 249. Mean action noise
stays near `0.34`; all recorded values are finite. This proves optimizer and
safety-survival signal, not forward motion. The generic training-diagnostics
script targets the separate reference-run schema and rejects this RSL-RL
manifest/metric vocabulary, so its artifact classification is inapplicable;
the native generation/checkpoint validators used by training and evaluation
accept this exact lineage.

## Deterministic outcome

| Plane/checkpoint | Safety outcome | Commanded forward distance | Achieved root-local forward distance | Forward-velocity MAE |
| --- | --- | ---: | ---: | ---: |
| GPU `model_249.pt`, fixed five-episode matrix | `5/5` full 1,199-step truncations; zero declared safety terminal | direct episode: `3.944 m` | `0.084 m` | `0.211 m/s` |
| GPU `model_75.pt`, same direct episode | one full 1,199-step truncation | `3.944 m` | `0.109 m` | `0.215 m/s` |
| CPU `model_249.pt`, five episodes | `0/5` timeout; mean length `461`, three joint-safety and two self-collision terminals | mean before termination `1.375 m` | mean `-0.025 m` | mean `0.252 m/s` |
| CPU `model_225.pt`, five episodes | `5/5` exact 1,200-tick timeout | mean `4.164 m` | mean `-0.031 m` | mean `0.210 m/s` |
| CPU `model_75.pt`, five episodes | `5/5` exact 1,200-tick timeout | mean `4.164 m` | mean `-0.345 m` | mean `0.227 m/s` |

The five-episode GPU safety report is
`bdf735cc…b95a`; direct final/checkpoint-75 reports are
`ea2e5d86…a8b` / `61e4b833…a809`. The direct CPU final and checkpoint-225
reports are `39092a88…753c` / `cf9001c1…b666`.

## Diagnosis and next discriminator

The first broken boundary is `Environment`, with `Evaluation` reinforcing it:

- at common forward commands, the inherited `2.5 m/s` planar-error
  normalization still gives a high tracking component to a stationary policy;
- upright/height/survival reward therefore admits a stable standing local
  optimum without meaningful displacement;
- the generated foundation schedule does not guarantee the roadmap's final
  180-tick zero-command interval, so it cannot prove start/stop even if motion
  existed;
- CPU/GPU safety disagreement is real but secondary to the fact that neither
  plane demonstrates forward travel.

Before any newly authorized optimizer budget, use no-training controls to
freeze a successor command/evaluation schedule with at least `3 m` commanded
forward distance and a final 180-tick stop, and a reward/acceptance rule under
which the standing parent and zero-action policy cannot pass the motion
criterion. Repairing `MODEL-MIRROR-P1` remains independently required before
runtime promotion. Video can illustrate this failed standing-local-optimum but
cannot convert it into walking evidence.
