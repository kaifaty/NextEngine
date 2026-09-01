# Physical sound V16 L0a — known-truth neural modal-field protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V16 L0–L1](../plans/physical-sound-synthesis-roadmap-v16.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Allowed claim | `Capability`: the bounded implementation can or cannot recover a known modal field |
| Product effect | None; no real signal, content schema, runtime model or fallback change |

## Question

Can one small structured neural model recover object-global modal frequencies
and damping together with a contact-conditioned signed gain field on unseen
synthetic objects, while remaining stable, beating compatible simple controls
and rejecting declared corrupt/OOD cases?

This is an implementation and representation oracle. It cannot establish real
sound quality, material identity, source-domain coverage or validator
admission.

## Isolation and access boundary

- The fixture is generated entirely from the formulas below. It reads no
  external dataset, mesh, audio, force, checkpoint or prior experiment output.
- Real waveform, force, source-body, method-holdout, validator-calibration and
  admission-shadow access counters MUST all remain zero.
- The runner, checkpoints, arrays and reports write only to a fresh external
  directory under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- Runtime inference, public contracts, cooker promotion and demo integration
  remain unauthorized. Authored clips remain the complete fallback.

## Known-truth corpus

The corpus has `18` object groups, `8` modes per object and one deterministic
triangulated parametric surface per object. Roles are object- and mesh-revision
disjoint: `9 train`, `3 development` and `6 test`. Every material and surface
family occurs in every role, while grid dimensions and object parameters never
repeat across roles.

| ID | Role | Material | Surface | Support | Size `L` m | Aspect | Wall m | Grid |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | --- |
| `tr-steel-plate` | train | Steel | Plate | Free | 0.320 | 1.30 | 0.0040 | `9x10` |
| `tr-steel-cylinder` | train | Steel | Cylinder | BaseClamped | 0.280 | 1.10 | 0.0030 | `10x9` |
| `tr-steel-bowl` | train | Steel | Bowl | Free | 0.240 | 0.85 | 0.0035 | `11x9` |
| `tr-wood-plate` | train | Wood | Plate | BaseClamped | 0.380 | 1.50 | 0.0120 | `9x11` |
| `tr-wood-cylinder` | train | Wood | Cylinder | Free | 0.340 | 1.00 | 0.0090 | `10x10` |
| `tr-wood-bowl` | train | Wood | Bowl | BaseClamped | 0.300 | 0.90 | 0.0100 | `11x10` |
| `tr-glass-plate` | train | Glass | Plate | Free | 0.220 | 1.20 | 0.0040 | `9x12` |
| `tr-glass-cylinder` | train | Glass | Cylinder | BaseClamped | 0.260 | 0.85 | 0.0030 | `10x11` |
| `tr-glass-bowl` | train | Glass | Bowl | Free | 0.200 | 1.05 | 0.0025 | `11x11` |
| `dv-steel-cylinder` | development | Steel | Cylinder | Free | 0.310 | 0.92 | 0.0032 | `12x10` |
| `dv-wood-bowl` | development | Wood | Bowl | Free | 0.330 | 1.10 | 0.0110 | `12x11` |
| `dv-glass-plate` | development | Glass | Plate | BaseClamped | 0.240 | 1.40 | 0.0032 | `12x12` |
| `te-steel-plate` | test | Steel | Plate | BaseClamped | 0.290 | 0.80 | 0.0045 | `13x11` |
| `te-steel-bowl` | test | Steel | Bowl | BaseClamped | 0.270 | 1.20 | 0.0038 | `13x12` |
| `te-wood-plate` | test | Wood | Plate | Free | 0.350 | 1.00 | 0.0105 | `13x13` |
| `te-wood-cylinder` | test | Wood | Cylinder | BaseClamped | 0.310 | 1.25 | 0.0085 | `14x11` |
| `te-glass-cylinder` | test | Glass | Cylinder | Free | 0.230 | 1.15 | 0.0028 | `14x12` |
| `te-glass-bowl` | test | Glass | Bowl | BaseClamped | 0.210 | 0.95 | 0.0030 | `14x13` |

Material truth is fixed as follows:

| Material | Density `kg/m³` | Wave-speed proxy `m/s` | Base decay `s⁻¹` |
| --- | ---: | ---: | ---: |
| Steel | 7850 | 5000 | 6 |
| Wood | 650 | 3300 | 16 |
| Glass | 2500 | 5200 | 4 |

`Plate`, `Cylinder` and `Bowl` are analytic UV surfaces embedded in canonical
metres. The runner emits vertices, triangle faces, normals, two curvature
features, UV and an adjacency graph. Cylinder and bowl U edges are periodic;
plate edges are open. Mesh identity is the SHA-256 of canonical little-endian
vertices and faces plus the object table row.

For mode ordinal `m = 0..7`, use:

```text
q = [1.40, 2.30, 3.40, 4.80, 6.50, 8.50, 10.80, 13.40]
k = [1, 1, 2, 2, 3, 3, 4, 5]
l = [1, 2, 1, 3, 2, 4, 3, 2]

base = 0.5 * wave_speed * wall / L^2
frequency[m] = base * q[m]
  * topology_factor * (1 + 0.12 * abs(log(aspect)))
  * support_frequency_factor
damping[m] = base_decay * (1 + 0.10*m)
  * (1 + 0.25*mean_absolute_curvature*L)
  * support_damping_factor
```

Topology factors are Plate `1.00`, Cylinder `1.18`, Bowl `1.36`.
`BaseClamped` uses frequency/damping factors `1.22/1.12`; `Free` uses `1/1`.
Frequencies must remain strictly ordered inside `[40, 12_000] Hz`; damping
must remain inside `(0, 128] s⁻¹` or fixture generation fails.

Signed modal gain truth is:

```text
amplitude[m] = (1 + 0.08*log(density/650)) / sqrt(m + 1)
Plate:    phi = sin(k[m]*pi*(u+1)/2) * sin(l[m]*pi*(v+1)/2)
Cylinder: phi = cos(k[m]*pi*u + m*pi/11) * sin(l[m]*pi*(v+1)/2)
Bowl:     phi = cos(k[m]*pi*u + m*pi/13)
                 * cos(l[m]*pi*v/2) * (1 - 0.15*(u*u + v*v))
gain[m] = amplitude[m] * phi
  * (0.75 + 0.25*abs(dot(normal, canonical_impulse_direction)))
```

The canonical impulse direction is normalized `[0.31, -0.89, 0.33]`. No
random noise or residual is present. A waveform is the float64 sum of the
eight signed damped sinusoids at `16 kHz` for `8192` samples, peak-normalized
only for gain-invariant spectrum metrics.

## Contact split

Every object uses deterministic 25% farthest-point context selection in metric
XYZ with the lexicographically first vertex as the first point. The remaining
vertices are query. At least 20 context and 40 query vertices are required.
Only train-role query losses enter optimization. Development selects no model
or numerical quality gate; it supplies only the predeclared OOD-threshold
statistic and verifies the frozen configuration. Test truth is evaluated once
after all ensemble/control artifacts are immutable.

## Structured candidate

The candidate is fixed before execution:

1. train-only normalization of static object and local contact features;
2. object encoder `static -> 64 -> 64 -> latent32` with SiLU;
3. positive ordered frequency head using cumulative `softplus` increments;
4. positive damping head using `softplus`;
5. shared context encoder over `(local contact, signed gain)` rows,
   `input -> 64 -> 64`, followed by mean and maximum pooling;
6. surface field `(object latent, context latent, local contact) -> 128 -> 128
   -> 8 signed gains`, with SiLU;
7. three-member ensemble; the arithmetic mean is the only candidate and
   ensemble disagreement is diagnostic/OOD evidence.

Static features are material one-hot, support one-hot, surface one-hot,
`log(L)`, `log(aspect)`, `log(wall)`, `log(density)` and
`log(wave_speed)`. Local features are UV, normalized XYZ, unit normal and the
two dimensionless principal-curvature values. The geometry-agnostic ablation
retains material/support, UV and context gains but removes surface, metric size,
XYZ, normals and curvature. It receives the same width, update count and seeds.

Training is deterministic CPU PyTorch with one intra-op/inter-op thread,
float64, deterministic algorithms and seeds `160001`, `160002`, `160003`.
Each member uses full-batch AdamW for exactly `2500` inclusive-endpoint
half-cosine updates from `0.002` to `0.00001`, weight decay `1e-6`, no early
stopping and no checkpoint selection. Loss is the sum of train-normalized log-
frequency MSE, log-damping MSE and twice the signed-gain MSE. The complete
candidate plus ablation ceiling is six training runs and `15_000` optimizer
updates. Any CUDA use, nondeterministic kernel or unlisted retry invalidates the
run.

## Frozen controls

All controls receive exactly the evidence needed by their method and never see
query truth:

- material-mean global frequency/damping;
- nearest train object in normalized static-feature distance;
- context mean gain;
- nearest context gain in metric XYZ;
- Euclidean RBF over context with fixed bandwidth `0.25 * bbox diagonal`;
- graph-geodesic RBF with fixed bandwidth `0.25 * mesh geodesic diameter`;
- eight-neighbour local-linear ridge with `lambda = 1e-3`;
- the equal-training-budget geometry-agnostic neural ablation.

The structured candidate must beat the best compatible control at the
aggregate test gain and waveform-spectrum endpoints. Controls remain reported
even when their method is incompatible with a global frequency/damping
endpoint.

## Metrics and immutable gates

All aggregations group first by object and then take an unweighted aggregate
across the six test objects.

| Gate | Required result |
| --- | --- |
| Finite/stable | All parameters and PCM finite; frequencies strictly ordered in `[40, 12_000] Hz`; damping in `(0, 128] s⁻¹`. |
| Frequency recovery | Test median absolute error `<= 30 cents`; p95 `<= 80 cents`. |
| Damping recovery | Test median relative error `<= 0.12`; p95 `<= 0.30`. |
| Gain recovery | Mean object query NRMSE `<= 0.25`; every object `<= 0.40`. |
| Classical comparison | Candidate gain NRMSE and gain-matched multiresolution log-spectrum RMSE are each `<= 0.98` times the best compatible non-neural control. |
| Geometry ablation | Candidate gain NRMSE and spectrum RMSE are each `<= 0.95` times the equal-budget geometry-agnostic ablation. |
| Waveform | Mean query waveform NRMSE `<= 0.20`; median gain-matched multiresolution log-spectrum RMSE `<= 2.5 dB`; p95 normalized envelope RMSE `<= 0.15`. |
| Modal peaks | Every test object recovers all eight truth peaks within `80 cents`. |
| Surface continuity | p99 edge-wise gain-difference error divided by truth gain RMS `<= 0.20`; no nonfinite discontinuity. |
| Exact repeat | Manifest, report, prediction arrays and model parameter hashes match across two fresh complete executions. |
| Isolation | Every real/source/protected access count is zero and authored fallback remains required. |

## Mutation and OOD suite

The same frozen evaluator processes:

1. negative damping and frequency reordering — hard rejection for every case;
2. per-object query-contact cyclic shift by `17` — quality rejection;
3. cyclic wrong material `Steel -> Wood -> Glass -> Steel` with truth unchanged
   — quality or OOD rejection;
4. metric scale multiplied by `1.6` with truth unchanged — quality or OOD
   rejection;
5. context coverage collapsed to `u <= 0`, followed by queries at `u >= 0.5`
   — coverage OOD rejection.

OOD uses three raw components: ensemble standard deviation divided by the
train gain RMS for the corresponding mode, the L-infinity static-feature
distance outside the train min/max envelope divided feature-wise by the
nonzero train range, and nearest-context Euclidean distance divided by mesh
geodesic diameter. Each raw component is divided by its maximum over valid
development queries, with a denominator floor of `1e-12`; a component whose
valid-development maximum is exactly zero remains zero for an in-envelope
query and is `+infinity` for a positive out-of-envelope query. The OOD score is
the maximum of these three development-calibrated components. Its single
threshold is the larger of the frozen constant `1.0` and `1.25 *` the maximum
valid development score after calibration. Test truth cannot select it.

This development calibration is part of the frozen protocol, not a learned
test threshold. It corrects the dimensional contradiction in the initial
freeze: the raw nearest-context-distance/diameter ratio is bounded by `1`, so
it could not have crossed a threshold of at least `1` reliably. This correction
was made before implementation, training or inspection of any test result.

Required mutation results are:

- `100%` hard rejection of negative-damping and reordered-frequency cases;
- at least `95%` rejection across contact-shuffle, wrong-material and wrong-
  scale object cases;
- at least `95%` rejection of collapsed-coverage query vertices;
- at most `10%` OOD rejection of valid test query vertices.

## Decision and stop rule

`L0_CAPABILITY_PASS` requires every gate and mutation condition in two exact
runs. It authorizes implementation of the V16 validator scaffold and
preparation of the disclosed-real R1 protocol only after five exact generator
groups exist. It does not authorize real signal access by itself.

Any failure returns `L0_CAPABILITY_REJECT`. The first failing gate and every
control result remain immutable. No threshold, object, seed, update count,
width, context fraction or truth formula may change inside this revision. A
successor requires a falsifiable diagnosis and a new protocol; real training,
protected access, cooker and demo remain blocked.
