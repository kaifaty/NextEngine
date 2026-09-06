# Physical sound V20 M0 — phase-consistent metric protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / DEVELOPMENT_ONLY / I1_AND_REAL_VALUES_SEALED` |
| Roadmap | [V20](../plans/physical-sound-synthesis-roadmap-v20.md), adoption commit `d26c63e7e40fbe0639fccfe88e00b897f664e4f0` |
| Scope | Validate automatic integration metrics on already-opened V19 development identities |
| Product effect | None; no I1, real-audio, validator, cooker, demo, public-contract or runtime credit |

## Question

Can a deterministic metric ensemble accept physically admissible B0/F0
composition while rejecting defects in frequency structure, decay, transient,
signed contact response, coverage and fallback—without treating accumulated
oscillator phase as sound-quality failure?

M0 validates the measuring instrument, not a model. It trains no weights,
chooses no physical formula and generates no fresh object identity.

## Immutable inputs

| Dependency | Identity |
| --- | --- |
| V19 I0 development-control result | `7608cff54a09ff322aba0c5f4c0f696b56231262f7cda7793fbf3b2440bd69da` |
| Fail-closed V19 I0 implementation commit | `22988542b834ce1eac75a8aa57ba3dabdb716143` |
| V19 I0 common / oracle | `9606f4c7a3a2d3a2ed893e5cabdc0b49c976da836540790a9836a4888eaa5170` / `704265828cd7213eec6ac8d6df301cc484d70eaf0c170a6b2f62d5077a981f7d` |
| V18 B0 complete external tree | `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111` |
| V19 F0 complete external tree | `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718` |
| V19 development row root | `825d0aa5de9bda2ffefc0737cfd8221f8e55585d3d6431b19bf5ef4db33806a2` |

The exact 24 primary/twin `1401…1412` development views, B0/F0 artifacts,
16 kHz rate, 8,192 samples, eight modes, C0 decisions and F0 predictions are
reused. M0 must fail before rendering if any identity changes.

The V19 integration band `1601…1612`, V20 I1 `1701…1712`, every optional
successor band `1801…2101`, real waveform, force, external dataset, protected
calibration, method holdout and admission shadow remain sealed.

## Deterministic evaluation subset

Both primary and remeshed twin views participate. From each view, select eight
C0-accepted queries at ordered positions
`floor(i*(count-1)/7), i=0…7`; duplicate positions are invalid. Context and
rejected queries are never substituted. This yields exactly 192 waveform
cases, or rejects before metrics.

Every candidate waveform is compared with the zero-phase analytic truth at the
same object/query. A common truth RMS normalization is applied before acoustic
metrics; candidate and truth never receive independent peak normalization.

## Metric layers

### Hard and physical layer

The following V19 gates are copied unchanged and remain independently binding:

- exact dependency/context/mesh identity, finite/order/range checks;
- C0 global/local acceptance and complete reason-coded fallback;
- B0 frequency median/p95 `<=20/60 cents`, maximum `<=60 cents`, damping
  median/p95 `<=0.08/0.20`;
- F0 gain NRMSE mean/max `<=0.25/0.40`, edge-gradient p99 mean/max
  `<=0.45/0.80`, control ratios, paired wins, remesh and corruption gates;
- negative damping, unordered frequency, missing fallback and dependency-hash
  mutations reject `12/12` primary objects.

### Multiresolution spectrum

Use periodic Hann windows with `(FFT, window, hop)`:

```text
(256,  256,  64)
(512,  512, 128)
(1024, 1024, 256)
(2048, 2048, 512)
```

Frames start at sample zero; right-pad only with zeros so the final sample is
covered. For each resolution `r` and normalized waveform pair:

```text
SC_r = || |STFT(pred)|-|STFT(truth)| ||_F
       / max(|| |STFT(truth)| ||_F, 1e-12)

LM_r = mean(abs(log(|STFT(pred)|+1e-7)
                -log(|STFT(truth)|+1e-7)))
```

Report per-case means `MRSC=mean_r(SC_r)` and `MRLM=mean_r(LM_r)`, then corpus
median and p95. No complex phase term enters either metric.

### Decay energy

Reuse the 512/512/128 STFT magnitude. Sum squared magnitude in Cartesian cells
formed by frequency bands `[0,500)`, `[500,2000)`, `[2000,8000] Hz` and sample
intervals `[0,256)`, `[256,1024)`, `[1024,4096)`, `[4096,8192)`.

For the 12 cells with nondegenerate truth energy:

```text
DE = sqrt(mean((log(E_pred+1e-10)-log(E_truth+1e-10))^2))
```

Report corpus median/p95. Empty truth cells are omitted and counted; an object
with no active cell rejects.

### Transient energy

At sample prefixes `[64,256,1024]` (`4/16/64 ms`), compute cumulative squared
energy divided by complete-clip energy. `TE` is RMS candidate-minus-truth
difference across the three fractions. Report corpus median/p95.

### Diagnostics only

Raw early/full waveform NRMSE and mixed Hilbert-envelope NRMSE are reproduced
for causal comparison. They receive no M0 pass/fail threshold. A global
polarity control must make their phase sensitivity visible while the acoustic
magnitude/energy metrics stay invariant.

## Frozen control corpus

Controls are rendered from truth, B0 and F0 values without training:

| Class | Control | Intended owner |
| --- | --- | --- |
| acceptable | exact identity | every metric must be numerical zero within tolerance |
| acceptable | frozen B0 + truth gains | spectrum/decay must coexist with passing B0 |
| acceptable | truth modes + frozen F0 gains | spectrum/energy must coexist with passing F0 |
| acceptable | frozen B0 + frozen F0 | complete component-pass composition |
| invariant | global waveform polarity | acoustic magnitude/energy invariant; signed-gain physical gate detects it |
| harmful-frequency | uniform `+90 cents` | MRSC/MRLM and physical frequency |
| harmful-frequency | alternating `-90/+90 cents` | MRSC/MRLM and modal correspondence |
| harmful-decay | damping multiplied by `1.35` | DE and physical damping |
| harmful-decay | damping multiplied by `0.65` | DE and physical damping |
| harmful-spectrum | remove the two modes with largest truth absolute gain per query | MRSC/MRLM and modal/gain evidence |
| harmful-field | alternate modal-gain signs | signed gain/gradient; acoustic metrics are diagnostic |
| harmful-transient | prepend 64 zeros and truncate to 8,192 samples | TE plus onset identity |
| harmful-transient | add a sample-zero impulse of `8.0*truth_peak` | TE plus transient hard policy |

The exact severity ladders are also fixed before implementation:

```text
uniform/alternating frequency cents: 0, 5, 10, 20, 40, 60, 90, 120
positive/negative damping fraction:  0, .02, .05, .08, .12, .20, .35, .50
uniform gain-scale error:             0, .05, .10, .20, .25, .40, .60
largest-gain modes removed:           0, 1, 2, 4
onset delay samples:                  0, 16, 32, 64, 128
sample-zero impulse / truth peak:     0, .5, 2, 8
```

For separation, frequency `0…20 cents`, damping `0….08` and gain `0….25`
are acceptable; their higher values are harmful because the unchanged median
gate fails. Onset `64/128`, impulse `8` and any mode removal are assigned
harmful; their intermediate points are trend diagnostics. A control may be
detected by more than one layer, but the assigned owner must succeed
independently.

## Gates

One complete M0 run passes only when:

1. exactly 24 views and 192 waveform cases are evaluated; every scalar is
   finite and every expected metric/control cell is present;
2. identity has `MRSC/MRLM/DE/TE <=1e-12` and serialization round-trip changes
   no value;
3. global polarity has `MRSC/MRLM/DE/TE <=1e-12`, while raw waveform NRMSE is
   `>=1.9`; this proves phase-sensitive diagnostics are not silently blocking;
4. the maximum assigned acceptable value is no more than `0.90x` the minimum
   assigned harmful value for both MRSC and MRLM in each frequency/spectrum
   family, for DE in the decay family, and for TE in the transient family;
5. Spearman severity correlation is `>=0.90` for each assigned owner across
   the frequency, damping, gain, mode-removal, onset and impulse ladders;
6. both harmful-frequency controls fail the unchanged physical frequency gate,
   both harmful-decay controls fail damping or DE attribution, and removed-mode
   and alternating-sign controls fail their physical/modal owner;
7. the frozen B0-only/F0-only/combined ordering reproduces the V19 causal
   conclusion within `1e-12` for legacy waveform diagnostics;
8. no sealed role is generated/read, runtime stays under ten minutes, RSS under
   4 GiB and output under 100 MiB;
9. two independent executions emit the exact same file set and bytes.

Gate 4 is a metric-family separability test, not the final I1 threshold. If it
passes, P0c freezes I1 thresholds between the measured acceptable and harmful
intervals without reading I1. If any assigned family lacks 10% separation, M0
rejects and I1 remains sealed.

## Output and implementation boundary

The committed runner emits only external:

- `manifest.json` with protocol, code, dependency and environment hashes;
- `corpus.json` with the exact development identities/query indices;
- `metrics.jsonl` with every case/control/metric value;
- `report.json` with aggregates, separation ratios and gates;
- `access-ledger.json` proving zero sealed-role access.

The environment is CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, PyTorch
`2.13.0+cu130`, float64 deterministic CPU and one numerical-library thread.
Implementation and focused development-only tests must be committed before two
official M0 runs. Generated files stay under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.

## Stop rule

M0 cannot change B0/F0, open I1, select a real source, calibrate the independent
validator or authorize a cooked clip. A reject returns to a new metric-protocol
revision with explicit causal evidence. A pass opens only P0c, which must freeze
the fresh I1 identities and final integration gates before their values exist.
