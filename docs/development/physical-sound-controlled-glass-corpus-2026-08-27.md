# Controlled exact-geometry glass corpus — 2026-08-27

## Result

The external P0 corpus is implemented and reproducible. It closes the previous
synthetic-control gap for one exact open rectangular glass-vessel proxy:

- one frozen geometry, material proxy, support patch and pickup;
- five strike positions and three impulse levels;
- explicit train, force-holdout, position-holdout and joint-holdout splits;
- a clean-room offline tetrahedral elasticity solve;
- engine-owned floating-point and signed-Q30 modal renders without
  per-condition normalization;
- deterministic physical controls, a deliberately simple spatial baseline and
  an independent Q1 evaluator bundle.

The result is
`CONTROLLED_SYNTHETIC_CORPUS_PASS / HUMAN_REFERENCE_OPEN / P1_BLOCKED`.
It is not physical identification of real glass, a generic glass material
model, a runtime FEM path or production-promotion evidence.

## Boundary and artifacts

The repository contains only the declarative recipe, solver/evaluator tooling,
tests and this report. Meshes, eigensolver arrays, rendered WAVs and blind-test
state remain under the external research store:

```text
/home/kaifaty/.cache/nextengine-research/controlled-glass-corpus-v0/
```

The ordinary authored-clip path and the existing Glass-H laboratory control
remain unchanged. No public schema, runtime dependency, DiffSound source,
generated PCM or solver state entered the repository.

## Frozen object and solve

| Property | Value |
| --- | --- |
| Object | open rectangular vessel proxy |
| Outer size | `84 × 120 × 72 mm` |
| Wall / bottom | `3 / 6 mm` |
| Tetrahedral cell | `3 mm` |
| Material proxy | density `2500 kg/m³`, Young's modulus `70 GPa`, Poisson ratio `0.22` |
| Support | four bottom back-left nodes fixed |
| Mesh | `9,775` nodes, `30,864` first-order tetrahedra |
| Retained modes | `16`, from `1558.210856` to `9979.065273 Hz` |
| Maximum relative eigen residual | `1.705755e-9` |
| Render | `48 kHz`, `19,200` frames (`400 ms`) |

The solver assembles linear tetrahedral stiffness and consistent mass matrices,
then solves `K φ = λ M φ`. The generalized symmetric eigensolver contract is
documented by the
[SciPy `eigsh` reference](https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.eigsh.html).
A fixed start vector and canonical mode signs make this bounded profile exactly
repeatable on the recorded environment: Python `3.12.13`, NumPy `2.5.2` and
SciPy `1.18.0`.

The recipe is
`lab/profiles/physical-sound-glass-vessel-corpus.v0.json`. Its SHA-256 is
`13c5d614d3bfff17b2836102069591da3babe901adef3e2a98182ffcb687e5b5`.
The two independent solver runs produced byte-identical meshes, fixed-node
arrays, eigenvectors and profile. Solver-profile SHA-256:
`a20265b32a3c23ffbe4185291b78641c8036ed9d3dfd347ebdbc61f0528d531e`.

## Experimental split

Five wall positions are crossed with low, medium and high normal impulses of
`0.01`, `0.02` and `0.04 N·s`:

| Split | Conditions | Purpose |
| --- | ---: | --- |
| train | 8 | four known positions at low/high impulse |
| force-holdout | 4 | medium impulse at the four known positions |
| position-holdout | 2 | low/high impulse at the unseen upper-middle position |
| joint-holdout | 1 | unseen upper-middle position at medium impulse |

This prevents a coefficient fit and its score from sharing every position and
force. It does not create object-level or real-recording generalization: there
is still only one synthetic object.

## Numerical and physical controls

`cargo run -p xtask -- physical-sound-corpus` validates every source hash,
renders all 15 conditions with the engine-owned recurrence and emits a frozen
quality manifest. The external report returned `PASS`:

| Control | Result |
| --- | --- |
| Repeated engine render | sample-identical |
| Q30 maximum absolute residual | `5.922971e-7` |
| Q30 maximum RMS residual | `7.986598e-8` |
| Q30 minimum correlation | `0.999999999928` |
| Force/modal scaling maximum relative error | `1.848632e-16` |
| Force/waveform RMS-ratio maximum relative error | `2.220446e-16` |
| Position modal-participation cosine distance | minimum `0.062908`, maximum `0.944901` |

The force check is an implementation control for the declared linear system,
not evidence that real hard impacts remain linear at arbitrary energy. The
position check establishes that strike location changes modal participation
while object frequencies and damping remain shared.

Render-report SHA-256:
`eb67392a394d73a0e84f1a41634d002ee8047966429072ff58a14846999be835`.
Its manifest SHA-256 is
`e0d80f03fe2c5c8ccc56132a2e4e96323a944300301d8960a2e9b7e61d5e4b0e`.

## Held-out spatial baseline

Inverse-distance weighting over the four known strike profiles was kept as an
intentionally weak baseline. On all three held-out forces it produced:

- correlation `0.907523`;
- signal-to-residual ratio `5.858210 dB`;
- medium-force RMS residual `0.0099603`;
- medium-force maximum absolute residual `0.135416`.

The independent evaluator also measured a `−998.900 Hz` spectral-centroid
shift and `2.616353 dB` gain-matched multiresolution log-spectrum RMSE for this
baseline. The useful conclusion is negative: interpolating whole modal
amplitude vectors by Euclidean strike-point distance is not an adequate model
of continuous contact position.

The smallest next counterfactual is to cook the surface mode-shape field and
interpolate modal participation on the struck triangle. That follows the
actual FEM state being sampled and can be tested against the same held-out
condition before any new solver run or coefficient search.

## Independent evaluator finding

The Q1 evaluator processed 18 matched pairs: 15 Q30-versus-f64 transfers and
three IDW-versus-exact held-out renders. It emitted a seeded 18-pair blind
browser and correctly retained
`Q1_MATCHED_ANALYSIS / HUMAN_CALIBRATION_REQUIRED`.

The Q30 modal-assignment cost remained at or below `1.704642e-6`, consistent
with the direct recurrence residual. Its log-spectrum RMSE nevertheless rose
as high as `12.672665 dB` on a low-level condition because the current
peak-normalized log spectrum magnifies differences in the quantized noise
floor; decay estimates are similarly unstable after the signal falls below
that floor. This is a calibration finding, not a Q30 acoustic mismatch. Direct
sample/modal checks and the evaluator descriptor must therefore remain
separate rather than being collapsed into one score.

All 18 entries remain `NeedsHumanAudit`. Evaluator-report SHA-256:
`af033cd5f913d3b48075ee3525c5348aec668b0a610dea11f0440c2ff8a061ef`.

## Audition files

The external render contains two deterministic audition files:

| File | Order | SHA-256 |
| --- | --- | --- |
| `audition-heldout-medium-exact-then-idw.wav` | exact held-out medium strike, then IDW prediction | `a496c1490da808f89805b8bc3d0eb5b3f1c1359f9259e31ca1223d8610cf2fb1` |
| `audition-heldout-force-low-medium-high.wav` | exact held-out low, medium, high strikes | `4553acf6d7db620eaff8b713b21b60bfb35185b02a43841c304778355e221ff8` |

Both WAV hashes repeated exactly from a second independent solver and renderer
run.

## Decision and remaining uncertainty

The experiment now supplies a valid controlled synthetic benchmark for force,
position and numeric-transfer regressions. It does not answer whether this
idealized vessel sounds like a particular real thin-walled container: support,
damping, radiation, pickup and material constants are synthetic and no matched
recording exists.

Proceed in two bounded steps:

1. replace IDW with surface mode-shape interpolation and require a large
   held-out residual reduction without weakening Q30/force controls;
2. record or obtain redistributable impacts for the same measured geometry,
   support, strike locations and force proxy, then calibrate the descriptor
   ensemble against blinded human judgments.

P1 remains blocked until SPEC-45's concrete consumer, complete committed
contact projection, content closure, whole-mixer budget and Accepted ADR exist.
