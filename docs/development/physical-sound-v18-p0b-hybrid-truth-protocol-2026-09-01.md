# Physical sound V18 P0b — hybrid truth protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V18 P0b–I0](../plans/physical-sound-synthesis-roadmap-v18.md) |
| Parent protocol | [V17 P0a](physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md), SHA-256 `39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e` |
| Closed evidence | [V17 G0 result](physical-sound-v17-g0-scale-separated-global-oracle-result-2026-09-01.md), SHA-256 `94e070d0aa594caf92a57b93ef1008c0cae5f2b6abc1f467df574e15a8c572ca` |
| Allowed claim | One deterministic global-baseline certificate, then the still-unopened intrinsic coverage, surface-field and integrated synthetic certificates |
| Product effect | None; no real/protected access, public schema, runtime model, cooker or fallback change |

## Question and dependency order

Can the unchanged degree-two scale-separated ridge that set the V17 complexity
floor pass a completely fresh object/scale exam, after which deterministic
intrinsic coverage and a learned surface field can be evaluated without
reopening or tuning against V17 G0?

The only execution order is:

```text
B0 deterministic global baseline
  -> O0 intrinsic coverage
  -> F0 masked intrinsic field
  -> I0 frozen hybrid integration
```

A rejection closes the current V18 scaffold and blocks every downstream step.
No threshold, feature, regularization, role, seed, mesh or truth revision may be
changed after its corresponding role is opened. Authored clips remain the
complete product fallback.

## Isolation and access ledger

- B0 generates its exact V17 training rows and all fresh rows directly from
  this protocol. It must not read V17 G0 reports, predictions, parameter files,
  corpus files, arrays or external run directories.
- The V17 G0 result is historical rationale only. Its per-object outcomes are
  forbidden inputs to B0 code and may not select a feature, gate or threshold.
- Candidate code receives declared row inputs only. Hidden truth functions and
  targets for development/test rows exist behind a scorer boundary and cannot
  be imported or called by the predictor.
- V16 rows and artifacts, external meshes, datasets, signals and checkpoints
  remain forbidden inputs.
- Real waveform/force, source body, generator real, protected calibration,
  method holdout and admission-shadow counters remain exactly zero.
- B0 additionally records zero bytes read for every opened V17 G0 artifact and
  zero network requests. O0/F0/I0 preserve the same ledger.
- Generated corpora, arrays, models, reports and optional debugging WAVs write
  atomically only to a fresh directory below
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- Symlinked output roots, existing nonempty output roots and paths outside that
  external experiment root fail before generation.

The pinned environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`
and PyTorch `2.13.0+cu130`, with float64 deterministic CPU algorithms and one
intra/inter-op thread. B0 itself uses NumPy/SciPy only; the PyTorch pin applies
to inherited F0.

## Exact shared enumeration

Orders and material/global truth are incorporated unchanged from V17 P0a:

```text
materials = [Steel, Wood, Glass]
topologies = [Plate, Cylinder, Bowl, RolledSheet]
supports = [Free, BaseClamped]
modes = 0..7
```

There are `24` categorical cells
`c = material_index*8 + topology_index*2 + support_index`. Radical inverse
`H_b(n)`, material proxies, dimensional scales, eight-mode global truth,
bounds and canonical little-endian representation are exactly P0a's definitions.
No implementation may replace those definitions with values recovered from an
opened V17 artifact.

## B corpus — fresh deterministic-baseline exam

The training input is exactly the `72` V17 G training identities, regenerated
from `n = 1 + 3*c + r`, `r=0..2`. Training is not enlarged and no V17
development/test row is generated or read by the B0 predictor.

Fresh roles contain one row per categorical cell:

| Role | `n` | Count | Scale stratum |
| --- | --- | ---: | --- |
| development/calibration | `801+c` | 24 | interpolation |
| test-interpolation | `901+c` | 24 | interpolation |
| test-scale-transfer | `1001+c` | 24 | short/long extrapolation |

For every fresh row:

```text
aspect = 0.70 + 0.80*H_3(n)
slenderness = wall/L = 0.004 + 0.004*H_5(n)

interpolation: L = 0.22 + 0.24*H_2(n)
scale-transfer, even c: L = 0.15 + 0.05*H_2(n)
scale-transfer, odd c:  L = 0.50 + 0.12*H_2(n)
wall = L*slenderness
```

IDs are `b-{role}-{material}-{topology}-{support}-0` in lowercase, with role
tokens `development`, `test-interpolation` and `test-scale-transfer`.
The exact pre-implementation reachability enumeration contains `72/72` unique
fresh rows, has zero complete-row overlap with V17 G, and yields:

| Quantity | Minimum | Maximum |
| --- | ---: | ---: |
| `L` | `0.178076171875` | `0.5598828125` |
| aspect | `0.7036579789666209` | `1.4904892546867856` |
| slenderness | `0.00418048` | `0.00791168` |
| wall | `0.000777103875` | `0.0041652315500000005` |
| frequency | `45.74235857965094 Hz` | `2901.0848945873163 Hz` |
| damping | `4.033146523100028 s^-1` | `32.476081554489724 s^-1` |

Every adjacent truth frequency differs by at least
`27.53202041965831 Hz`. These values prove enumeration reachability only; no
candidate prediction or error was evaluated before this freeze.

## B0 candidate — exact degree-two ridge

The base feature vector has this fixed order and width `11`:

1. material one-hot in `[Steel, Wood, Glass]` order;
2. topology one-hot in `[Plate, Cylinder, Bowl, RolledSheet]` order;
3. support one-hot in `[Free, BaseClamped]` order;
4. `log(aspect)`;
5. `log(slenderness)`.

Every base column is standardized from the `72` training rows using population
mean and population standard deviation. Zero or nonfinite standard deviation
fails closed. The design has exactly `78` columns in this order:

```text
intercept,
11 standardized linear terms,
x_i*x_j for i=0..10 and j=i..10 in lexicographic (i,j) order
```

Targets have width `16`: eight
`log(frequency[m]/frequency_scale)` values followed by eight
`log(damping[m]/damping_scale)` values in mode order. With `X` shaped
`72 x 78`, `Y` shaped `72 x 16` and `lambda=1e-6`, coefficients are the
unique float64 solution:

```text
beta = solve(X.T @ X + 1e-6*I_78, X.T @ Y)
```

The intercept is regularized because the identity is exactly `I_78`.
Prediction exponentiates the ratios/multipliers and reconstructs dimensional
frequencies/damping using only the declared scale preprocessor. There is no
optimizer, seed, checkpoint, early stopping, model selection or neural
comparison.

The artifact serializes, in fixed order, the feature names, training means and
standard deviations, `78 x 16` coefficients, target order, regularization,
training-row identity root and implementation/protocol hashes. Numeric payloads
are canonical little-endian float64. Loading recomputes and checks every shape,
bound and hash before prediction.

## B0 OOD and mutations

Static OOD uses only `[log(aspect), log(slenderness)]` and the V17-train
min/max box. Distance is L-infinity excess divided coordinate-wise by the
nonzero train range. Dimensional `L` is deliberately excluded so legal scale
transfer is not rejected. Development freezes exactly:

```text
threshold = max(0.25, 1.25*maximum_valid_development_distance)
```

The `24` fresh test-interpolation objects each produce three mutations:

1. material cycles `Steel -> Wood -> Glass -> Steel` while truth stays fixed;
2. support swaps while truth stays fixed;
3. scale corrupts `L -> 1.7*L` while wall and truth stay fixed.

A mutation rejects when static OOD exceeds the threshold or its predicted
object fails the same absolute quality limits used by B0. The unmutated
development/test roles never participate in threshold or feature selection.

## B0 gates

Object frequency error is the median of its eight absolute cents errors;
object damping error is the median of its eight relative errors. Pooled
median/p95 gates below are nevertheless computed over all eight-mode errors so
a bad mode cannot be hidden by object grouping. Endpoint means are reported
diagnostically only.

B0 passes only if all of the following are true:

- every prediction is finite, frequency-ordered and inside `[40,7500] Hz`,
  with damping in `(0,128] s^-1`;
- over both fresh test strata together, frequency median/p95 is `<=20/60`
  cents and damping median/p95 is `<=0.08/0.20`;
- test-interpolation and test-scale-transfer each separately meet the same
  frequency and damping limits;
- valid fresh test OOD fraction is `<=10%` overall and in each test stratum;
- all `72/72` frozen mutations reject by quality or static OOD;
- the serialized artifact round-trips without numeric or hash change;
- two complete executions in independent fresh roots match manifest, model,
  corpus, predictions and report byte-for-byte;
- all V16/V17-opened-artifact, real/source/protected/network access counters
  are zero.

There is intentionally no margin against another model. B0 is the frozen
complexity floor. Failure closes it instead of selecting a different
regression, polynomial degree, feature set, regularization or gate on opened
fresh rows.

## O0/F0/I0 inheritance and amendments

The complete F/I corpus, meshes, analytic gains, O0 mutations, F0 candidate
and controls, seeds, updates, losses, metrics, thresholds and gates are
incorporated unchanged from V17 P0a SHA-256
`39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e`.
This reuse is admissible because no O0, F0 or I0 implementation, prediction,
artifact, threshold or metric has ever been produced or opened. Their
identities are also disjoint from G0 and the fresh B roles.

Only these dependency/name substitutions apply:

- P0a `G0` becomes the exact passing B0 artifact and parameter hash;
- O0 runs only after B0 passes twice exactly;
- F0 runs only after exact B0 and O0 hashes freeze;
- I0 loads the exact passing B0/O0/F0 hashes, trains nothing and never loads
  the rejected V17 neural G0 head;
- I0's global modes are predicted by the serialized B0 ridge; every O0/F0/I0
  reference to a passing global component means B0;
- output schemas/reports use V18 B0/O0/F0/I0 study identities while corpus
  numbers, mesh formulas, neural controls and gates remain unchanged.

The inherited F/I role counts remain `24` train, `12` development, `12` test
and `12` integration physical groups, with remeshed twins on every non-train
group. Integration remains sealed until B0, O0 and F0 each pass twice exactly.

## Successful controls required before official execution

Focused tests must pass before any official run:

1. exact enumeration counts, ranges, row uniqueness and zero overlap with all
   V17 G rows;
2. hidden dimensionless targets recompose every truth row to the documented
   dimensional values;
3. the feature builder produces the exact `11 -> 78` order and an independent
   direct implementation matches every design cell;
4. serialized normalization/coefficient payloads round-trip byte-exactly and
   malformed shapes, hashes, endianness and nonfinite values fail closed;
5. an identity/global-truth control reaches zero B and integrated global
   metrics without exposing truth to the candidate;
6. P0a identity-gain, intrinsic-disconnection and RolledSheet-shortcut
   controls remain reachable unchanged;
7. perturbing hidden fresh truth after prediction changes scores but not
   predictions, proving predictor/scorer separation;
8. output-root escape, symlink, pre-existing output, artifact-read and access-
   ledger mutations all fail before partial publication;
9. short and complete baseline runs repeat every emitted byte exactly.

These controls validate the evaluator, not candidate quality. Official B0
fresh errors remain unopened until implementation and all focused tests are
committed.

## Compute and stop rules

- B0 is one deterministic `78 x 78` ridge solve and is expected to finish on
  CPU in seconds; no GPU or seed budget exists.
- O0 is exact all-pairs/multi-source graph Dijkstra with no training.
- F0 retains three seeds and `1500` full-batch updates per learned family from
  P0a; I0 trains nothing.
- B0 rejection closes V18 B/O/F/I. O0 rejection blocks F0/I0. F0 rejection
  blocks I0. I0 rejection blocks disclosed-real training.
- No failed stage receives a nearby retry on its opened role. A successor
  requires a new falsifiable family and a new disjoint protocol.
- Synthetic success cannot open a source or protected role by itself. Real
  work still requires the independent source sufficiency gates in Roadmap V18.
- No local recording, prompt-to-wave, runtime inference, raw PhysX callback,
  public contract or fallback removal is authorized.
