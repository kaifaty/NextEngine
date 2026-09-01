# Physical sound V17 P0a — disjoint factorized truth protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V17 P0–I0](../plans/physical-sound-synthesis-roadmap-v17.md) |
| Research basis | [V17 factorized-operator research](physical-sound-v17-factorized-operator-research-2026-09-01.md) |
| Allowed claim | Four bounded capability certificates: global, intrinsic coverage, surface field and integrated modal rendering |
| Product effect | None; no real/protected access, public schema, runtime model, cooker or fallback change |

## Question and stop rule

Can a scale-separated global head, deterministic intrinsic coverage certificate
and masked intrinsic surface operator each pass an isolated new-truth exam and
then pass one integrated exam without using any V16 test value for selection?

`G0`, `O0` and `F0` are independent. A rejection closes that hypothesis and
blocks `I0`. Passing all three freezes their artifacts; no retraining occurs in
I0. Only two byte-identical complete I0 executions may emit
`V17_INTEGRATED_CAPABILITY_PASS`. No result earns real-quality, validator,
admission or runtime authority.

## Isolation

- The runner generates every scalar, mesh and waveform from this document. It
  reads no V16 output, external mesh, dataset, signal, checkpoint or report.
- V16 rows, parameter indices, mesh grids, seeds and checkpoints are forbidden
  inputs. V16 results may be compared only after every V17 artifact is frozen.
- Real waveform/force, source body, generator real, protected calibration,
  method holdout and admission-shadow counters remain exactly zero.
- Generated arrays, models, reports and optional debugging WAVs write only to a
  fresh directory below
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- Authored clips remain mandatory fallback. Runtime inference and product
  contracts remain unauthorized.

## Exact enumeration primitives

Orders are fixed:

```text
materials = [Steel, Wood, Glass]
topologies = [Plate, Cylinder, Bowl, RolledSheet]
supports = [Free, BaseClamped]
modes = 0..7
```

Material truth is Steel `(density=7850, wave=5000, decay=6)`, Wood
`(650,3300,16)` and Glass `(2500,5200,4)` in SI proxies. For positive integer
`n` and prime `b`, radical inverse is exact:

```text
H_b(n) = sum_{j>=0} digit_j(n in base b) * b^(-(j+1))
```

Canonical JSON stores the exact integer `n` and the resulting IEEE-754 float64
values. All generated numeric arrays are canonical little-endian float64/int64.
Object identity hashes the canonical row, vertices and faces.

## G corpus — object-global modes

There are `24` categorical cells
`c = material_index*8 + topology_index*2 + support_index`.

| Role | Replicates/cell | `n` | Count | Scale stratum |
| --- | ---: | --- | ---: | --- |
| train | 3 | `1 + 3*c + r`, `r=0..2` | 72 | interpolation |
| development | 1 | `101 + c` | 24 | interpolation |
| test-interpolation | 1 | `201 + c` | 24 | interpolation |
| test-scale-transfer | 1 | `301 + c` | 24 | short/long extrapolation |

For every row:

```text
aspect = 0.70 + 0.80*H_3(n)
slenderness = wall/L = 0.004 + 0.004*H_5(n)

interpolation: L = 0.22 + 0.24*H_2(n)
scale-transfer, even c: L = 0.15 + 0.05*H_2(n)
scale-transfer, odd c:  L = 0.50 + 0.12*H_2(n)
wall = L*slenderness
```

IDs are `g-{role}-{material}-{topology}-{support}-{r_or_0}` in lowercase.
Exact enumeration yields `L=0.17548828125…0.558359375`,
`aspect=0.7043895748…1.4912208505` and
`slenderness=0.004032…0.007968`; no complete V16 row repeats.

Global truth uses:

```text
q = [1.40, 2.25, 3.30, 4.55, 6.00, 7.65, 9.50, 11.55]
topology_frequency = [1.00, 1.16, 1.30, 1.08]
topology_damping   = [1.00, 1.08, 1.16, 1.04]
support_frequency = [1.00, 1.19]
support_damping   = [1.00, 1.10]

frequency_scale = wave_speed*wall/(L*L)
aspect_factor[m] = 1 + 0.08*abs(log(aspect))
                     + 0.012*sin((m+1)*log(aspect))
material_factor[m] = 1 + 0.01*material_index*cos(0.7*(m+1))
frequency[m] = frequency_scale*q[m]*topology_frequency
               *support_frequency*aspect_factor[m]*material_factor[m]

damping_scale = base_decay
damping[m] = damping_scale*(1+0.08*m)*topology_damping
             *support_damping*(1+0.06*abs(log(aspect)))
```

The enumerated truth is strictly ordered, frequencies are
`44.627366…3234.177717 Hz`, damping is `4.001685…32.574210 s^-1`,
and every adjacent frequency difference exceeds `26.756 Hz`. Generation fails
outside `[40,7500] Hz`, `(0,128] s^-1`, on nonfinite values or on an identity
collision.

### G candidate and controls

`G-candidate` receives material/topology/support one-hot, `log(aspect)` and
`log(slenderness)`. Deterministic preprocessing supplies `frequency_scale` and
`damping_scale`; the network predicts eight cumulative-softplus positive
dimensionless frequency ratios and eight softplus damping multipliers. The
object encoder is `input -> 64 -> 64 -> latent32` with SiLU and two linear
heads.

`G-raw-MLP` is equal-width/equal-budget and receives the same categorical
features plus `log(L)`, `log(wall)`, `log(density)`, `log(wave_speed)` and
`log(base_decay)`; it predicts dimensional frequencies/damping through the
same positive heads. It cannot call the scale preprocessor.

Non-neural controls are material/support mean, nearest train row in train-
normalized raw static features and degree-two ridge regression (`lambda=1e-6`)
on the candidate's dimensionless features. Hidden `q`, topology/support factors
and truth functions are unavailable to all candidate/control code.

Both neural families use train-only normalization, float64 deterministic CPU,
one intra/inter-op thread, seeds `[170001,170002,170003]`, full-batch AdamW,
`1500` inclusive-endpoint half-cosine updates from `0.002` to `0.00001`, weight
decay `1e-6`, no early stopping and no checkpoint selection. Loss is equal
train-normalized log-frequency and log-damping MSE, grouped equally by object.

G gates, grouped first by object/categorical cell, are:

- finite ordered frequencies in `[40,7500]` and damping in `(0,128]`;
- test median/p95 frequency error `<=20/60 cents`;
- test median/p95 damping relative error `<=0.08/0.20`;
- on scale-transfer, candidate frequency and damping error each `<=0.50x`
  G-raw-MLP and candidate wins at least `20/24` paired cells;
- overall candidate frequency/damping error each `<=0.90x` the best compatible
  non-neural control;
- wrong material cycle, support swap and corrupt scale (`L*1.7` with truth
  unchanged) reject all `72` mutation cases by quality or static OOD;
- two complete runs match manifests, reports, arrays and parameter hashes.

The `20/24` paired-cell rule has one-sided sign-test probability
`0.00077194` under equal win probability. It is a synthetic discriminator, not
a real-population confidence claim.

## F/I corpus — intrinsic contact fields

F cells are `c = material_index*4 + topology_index`, `12` total. Support is
`supports[(n+c) mod 2]`.

| Role | Replicates/cell | `n` | Count | Primary grid `(u,v)` | Remeshed twin |
| --- | ---: | --- | ---: | --- | --- |
| train | 2 | `401 + 2*c + r` | 24 | `(13+2r+c%2, 12+r+c%3)` | none |
| development | 1 | `501+c` | 12 | `(17+c%2, 15+c%3)` | `(23+c%2,20+c%3)` |
| test | 1 | `601+c` | 12 | `(19+c%2, 16+c%3)` | `(27+c%2,21+c%3)` |
| integration | 1 | `701+c` | 12 | `(21+c%2, 17+c%3)` | `(29+c%2,23+c%3)` |

Each row uses `L=0.21+0.27*H_2(n)`,
`aspect=0.72+0.76*H_3(n)`,
`slenderness=0.004+0.004*H_5(n)` and `wall=L*slenderness`.
Global truth is the G formula. Primary/twin meshes share the physical-group ID
but have distinct mesh hashes; no twin crosses roles.

UV/embedding is row-major. Plate includes both `u,v` endpoints and embeds as
`(0.5*L*u, 0.5*L/aspect*v, 0)`. Cylinder uses periodic
`u_i=-1+2i/n_u`, inclusive `v`, radius `L/(2*pi)` and height `L*aspect`.
Bowl uses periodic `u`, cell-centred `v_j=-1+2(j+0.5)/n_v`,
`theta=pi*(u+1)`, `alpha=pi*(v+1)/4`, radial/axial semiaxes
`a=0.5L`, `c=0.5L*aspect`, and embedding
`(a*sin(alpha)*cos(theta), a*sin(alpha)*sin(theta), -c*cos(alpha))`.
RolledSheet uses endpoint-inclusive `u,v`, open U connectivity, radius `L/(2*pi)`,
`theta=pi*(u+1)` and the same cylindrical embedding; its coincident ambient
seam remains intrinsically open. Quads use `(a,b,c),(a,c,d)`; only Cylinder and
Bowl wrap U.

The graph contains unique triangle edges weighted by metric length. Exact
all-pairs Dijkstra distance is authoritative for P0a; heat/diffusion matrices
are model features/caches only. Canonical normals and analytic dimensionless
curvatures accompany UV and bbox-normalized XYZ.

For modes, use `k=[1,1,2,2,3,3,4,5]`,
`l=[1,2,1,3,2,4,3,2]`, material amplitude
`(1+0.06*log(density/650))/sqrt(m+1)`, support phase
`phase=0` for Free and `pi/17` for BaseClamped, and:

```text
Plate: phi = cos(k*pi*u/2+phase)*cos(l*pi*v/2)
             +0.12*sin((k+l)*pi*u*v/2)
Cylinder: phi = cos(k*pi*u+m*pi/9+phase)*cos(l*pi*v/2)
Bowl: phi = cos(k*pi*u+m*pi/12+phase)*cos(l*pi*v/2)
            *(1-0.10*(u*u+v*v))
RolledSheet: phi = sin(k*pi*(u+1)/2+phase)*cos(l*pi*v/2)
             +0.10*cos((k+1)*pi*u*v)
gain = amplitude*phi*(0.72+0.28*abs(dot(normal, impulse_direction)))
```

Impulse direction is normalized `[0.27,-0.91,0.31]`. Truth is evaluated
analytically on each primary/twin; no noise/residual exists.

Valid context is `ceil(N/8)` intrinsic farthest-point vertices, minimum `16`,
starting at the lexicographically first XYZ vertex. Remaining vertices are
query. Train query loss alone updates weights. Development supplies frozen
normalization/calibration only. Test opens once after artifacts freeze;
integration remains sealed until G/O/F pass.

## O0 — intrinsic coverage certificate

Candidate raw coverage at every vertex is multi-source graph-geodesic distance
to valid context divided by mesh geodesic diameter. Development freezes
`threshold = max(0.05, 1.25*p99(valid development raw coverage))`. Static
feature-envelope and ensemble disagreement are separately reported but cannot
lower or replace intrinsic coverage.

Every development/test topology processes four mutations:

1. **intrinsic cap:** choose the lexicographic seed, keep as context its nearest
   `40%` of vertices by intrinsic distance and query the farthest `30%`;
2. **component isolation:** cut all graph edges crossing the median V row, keep
   context only in one component and query the other;
3. **thinning:** retain every fourth valid context in canonical context order
   and query the intrinsically farthest quartile;
4. **ambient shortcut:** RolledSheet context has `u<=-0.75`, query has
   `u>=0.75`; other topologies use the intrinsic-cap mutation as a balanced
   control.

An unreachable query has `+infinity` raw coverage and is always OOD. The
Euclidean nearest-context version, calibrated identically, is the frozen
negative control. O0 requires:

- valid-test OOD fraction `<=10%` on at least `11/12` objects and every
  topology aggregate;
- `>=95%` rejected mutation queries in every class/topology aggregate and at
  least `11/12` objects;
- intrinsic coverage beats Euclidean rejection-minus-false-rejection utility
  and specifically rejects `>=95%` RolledSheet ambient-shortcut queries;
- exact repeat of geometry, scores, decisions and report.

## F0 — masked intrinsic surface operator

Input vertex channels are context mask, eight normalized observed gains (zero
when unobserved), UV, normalized XYZ, normal, dimensionless curvatures and
broadcast object/global latent features. The primary model has width `96`:

1. pointwise input MLP `input -> 96 -> 96`;
2. four blocks concatenating the current state with graph diffusion at
   dimensionless times `[0.01,0.04,0.16,0.64]`, then `Linear -> SiLU -> Linear`
   back to width `96` with a residual connection;
3. pointwise `96 -> 96 -> 8` signed-gain decoder.

Diffusion is `exp(-t*L_norm)` from the symmetric normalized graph Laplacian
with edge conductance `1/max(edge_length,1e-12)`. Float64 symmetric
eigendecomposition constructs the complete matrix; nonfinite/asymmetric
caches fail closed.

Neural controls, with the same width, seeds and updates, are the V16-style
mean/max pooled context decoder and one four-head query-to-context attention
layer without graph/diffusion features. Classical controls are context mean,
nearest context, Euclidean/geodesic RBF at `0.25*diameter` and eight-neighbour
local-linear ridge `lambda=1e-3`.

All learned families use seeds `[170001,170002,170003]`, float64 deterministic
CPU, train-only normalization and full-batch AdamW for `1500` half-cosine
updates `0.002 -> 0.00001`, weight decay `1e-6`. Loss is normalized query gain
MSE plus `0.25` times normalized undirected edge-gradient MSE; edges with two
context endpoints are excluded from loss. No early stopping/selection.

F0 gates on twelve test physical groups are:

- mean/max object gain NRMSE `<=0.25/0.40`;
- p99 normalized edge-gradient error `<=0.20` and no nonfinite discontinuity;
- primary gain/continuity endpoints each `<=0.95x` best compatible classical
  control and `<=0.90x` both neural controls;
- primary wins both gain and continuity on at least `10/12` paired cells
  (`p=0.0192871` under equal sign probability);
- primary/twin predictions at canonical UV probes have NRMSE `<=0.10` per
  object and no metric changes by more than `10%` under remeshing;
- query cyclic shift `17`, alternating mode-sign flip, wrong topology and
  context removal are rejected at the frozen mutation rates (`100%` hard,
  `>=95%` ordinary/OOD);
- two exact complete executions.

## I0 — frozen integration

I0 loads only exact passing G0/O0/F0 parameter hashes and evaluates the twelve
untouched integration physical groups plus twins. It trains nothing. Modal PCM
is eight zero-phase damped sinusoids at `16 kHz`, `8192` samples. Metrics retain
P0a's object grouping, analytic-node handling, periodic Hann spectra
`[512,1024,2048,4096]` and Hilbert envelope definitions.

I0 gates are:

- every G0, O0 and F0 gate remains true on frozen artifacts;
- integration frequency median/p95 `<=20/60 cents`, damping `<=0.08/0.20`;
- gain mean/max `<=0.25/0.40`, continuity p99 `<=0.20`;
- first-`1024`-sample (`64 ms`) joint waveform NRMSE `<=0.35`;
- full-signal median multiresolution spectrum RMSE `<=2.5 dB` and envelope p95
  `<=0.15`; full-duration raw sample NRMSE is diagnostic, not a gate;
- every truth modal peak is within `60 cents`;
- negative damping/reordered modes hard reject `100%`; G/O/F ordinary mutation
  rates remain satisfied; valid integration OOD `<=10%`;
- manifest, report, every prediction array and all component parameter hashes
  match across two fresh executions.

The `64 ms` phase-sensitive window replaces V16's contradictory full `512 ms`
sample gate: a frequency prediction allowed by the cents gate can accumulate
many cycles of phase drift over the full signal. Spectrum, envelope and modal
peaks remain full-signal gates; no test outcome selected this definition.

## Reachability and implementation order

Before any learned official run, focused tests must prove:

1. analytic scale decomposition/recomposition recovers every generated G truth
   exactly when supplied the hidden dimensionless target;
2. identity gain prediction yields zero F/I metrics on nodal and non-nodal
   queries and twins;
3. authoritative intrinsic distance rejects every disconnected query and the
   constructed RolledSheet shortcut while valid geodesic FPS remains below the
   frozen development threshold in the successful control fixture;
4. role/object/mesh hashes are unique, all V16 exact rows are absent, output is
   external/atomic and all access counters are zero.

These successful controls prove only evaluator reachability. Candidate code
cannot call them or the hidden formulas. Official order is `G0 -> O0 -> F0 ->
I0`; any rejection stops downstream execution and is recorded without tuning.
