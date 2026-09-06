# Physical sound V20 M0b — confound-resistant metric protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / DEVELOPMENT_ONLY / I1_AND_REAL_VALUES_SEALED` |
| Predecessor | [M0 repeat-exact reject](physical-sound-v20-m0-phase-consistent-metric-result-2026-09-01.md), SHA-256 `78503fbde6c706f06ac785b130e4009ca38ba2b4c0dbbde8656b816353a30bfa` |
| Research | [Confound-resistant metric research](physical-sound-v20-m0b-confound-resistant-metric-research-2026-09-01.md), SHA-256 `600554aacc76f3ddda777fa48e23aedba2e88a1a146d9fa3fafdd00d20950b7d` |
| Product effect | None; M0b may open only P0c and cannot authorize I1 values, real data, clips, demo or runtime |

## Question

Does residualizing per-frame spectral level and per-band decay level remove the
two M0 amplitude confounds while preserving all successful controls, physical
owners, causal attribution and the unchanged `0.90x` separation margin?

M0b validates a measuring instrument. It trains and modifies no B0/F0 weight,
chooses no object and generates no fresh identity.

## Immutable lineage and access

M0b pins:

- M0 implementation commit
  `b8bbeb39e574ab2fcbbe91a628e36183383baceb`;
- M0 common/oracle/test SHA-256
  `a742b4e941d08cd0dd9ed55d07e4d710359d61de1762a17e2b4a415e78c8535d`,
  `d1bbeb8f838c47c207a1e213cdb82d4ae6eaad5089a90b67a54d38da771aee69`
  and `cbe9235a106487a88735f441f1c159920a4d3669621044aeae66a50441932bf0`;
- M0 complete tree
  `12f3ce7298dadff53688262f9d3f34b202b9881d7bfbf4eaf2c60a753549fbfd`;
- the unchanged V18 B0 tree
  `bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111`,
  V19 F0 tree
  `7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718`
  and development row root
  `825d0aa5de9bda2ffefc0737cfd8221f8e55585d3d6431b19bf5ef4db33806a2`.

Reuse the exact M0 24 primary/twin views, eight ordered accepted queries per
view, 192 cases, 58 controls, 16 kHz, 8,192 samples, eight modes, periodic-Hann
STFT frames and common truth-RMS normalization. Every hard/physical,
transient, legacy, serialization, resource and zero-access rule remains
unchanged.

V19 `1601…1612`, V20 I1 `1701…1712`, optional `1801…2101`, every real waveform,
force, external dataset, protected calibration, method holdout and admission
shadow remain sealed. The M0 trees may be read only to verify identity and
predecessor evidence; M0b recomputes development controls from the pinned
B0/F0 artifacts and does not select cases from predecessor metric values.

## Changed metric 1: mean-centered log magnitude

Use the unchanged four M0 STFT resolutions. For every case, resolution and
time frame, first compute:

```text
LP[f] = log(|STFT(pred)[f]|  + 1e-7)
LT[f] = log(|STFT(truth)[f]| + 1e-7)

CLP[f] = LP[f] - mean_f(LP)
CLT[f] = LT[f] - mean_f(LT)
```

The resolution metric and case aggregate are:

```text
CLM_r = mean_frames,f(abs(CLP - CLT))
MCLM  = mean_r(CLM_r)
```

MRSC remains unchanged and blocking. MCLM replaces raw MRLM as the second
blocking spectrum metric. Raw MRLM is still computed exactly as M0 and reported
as diagnostic. Frame centering is deliberately independent for candidate and
truth; signed gain NRMSE, spatial gradient and gain-scale controls remain
binding owners of amplitude, so MCLM cannot waive an amplitude defect.

## Changed metric 2: normalized EDC log-slope

Reuse the M0 512/512/128 magnitude and the same three frequency bands. For
each band `b` and frame `d`:

```text
E[b,d] = sum_f_in_band(|STFT[f,d]|^2)
C[b,d] = sum_i=d..D-1(E[b,i])
N[b,d] = C[b,d] / C[b,0]
L[b,d] = log(max(N[b,d], 1e-10))
S[b,d] = L[b,d+1] - L[b,d]
```

A truth band is active iff `C_truth[b,0] > 1e-12`; inactive bands are counted
and omitted, and a case with no active band rejects. The per-case decay slope
residual is:

```text
DSR = sqrt(mean_active_b,d((S_pred[b,d] - S_truth[b,d])^2))
```

DSR replaces absolute DE as the blocking damping metric. The exact M0
12-cell absolute DE remains diagnostic. Backward integration smooths modal
beating; normalization removes band level; first difference makes attenuation
rate, not remaining energy level, the owner.

## Unchanged controls and summaries

All 58 M0 controls and classifications are byte-for-byte semantic copies:

- identity, B0-only, F0-only, combined and global polarity;
- uniform/alternating cents `0,5,10,20,40,60,90,120`;
- positive/negative damping fraction
  `0,.02,.05,.08,.12,.20,.35,.50`;
- gain scale `0,.05,.10,.20,.25,.40,.60`;
- removed modes `0,1,2,4`;
- onset delay `0,16,32,64,128` samples;
- sample-zero impulse `0,.5,2,8` times truth peak;
- alternating modal-gain signs.

Every metric reports per-case values and corpus median/p95. Separability and
severity use corpus p95 exactly as M0.

## Gates

One M0b execution passes only when:

1. exact lineage, 24 views, 192 cases, 58 controls and 11,136 complete finite
   rows pass before aggregation;
2. identity has `MRSC/MCLM/DSR/TE <=1e-12` and serialization is exact;
3. polarity has `MRSC/MCLM/DSR/TE <=1e-12`, raw waveform NRMSE `>=1.9`, and
   signed-gain physical rejection;
4. corpus-p95 maximum acceptable is `<=0.90x` minimum harmful for MRSC and
   MCLM in uniform frequency, alternating frequency and mode removal; DSR in
   both damping directions; TE in onset and impulse;
5. assigned-owner Spearman is `>=0.90` for MRSC/MCLM frequency and mode
   removal, DSR damping, signed gain NRMSE gain scale and TE onset/impulse;
6. every unchanged hard/physical gate and named harmful control passes its M0
   rejection rule;
7. M0 raw MRLM/DE failures are reproduced within `1e-12`, proving the
   successor did not silently change the corpus or old formulas;
8. V19 B0-only/F0-only/combined legacy diagnostics reproduce within `1e-12`;
9. zero sealed access, runtime below ten minutes, RSS below 4 GiB and output
   below 100 MiB;
10. two independent executions emit the exact same five files and bytes.

A reject closes M0b and leaves P0c/I1 sealed. A pass opens only a documentation
commit for P0c; no I1 value may exist before that protocol freezes exact fresh
metadata and thresholds between the measured acceptable/harmful intervals.

## Output and implementation boundary

After its implementation and focused tests are committed, each official run
emits only external `manifest.json`, `corpus.json`, `metrics.jsonl`,
`report.json` and `access-ledger.json`. The environment remains CPython
`3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`, PyTorch `2.13.0+cu130`, deterministic
float64 CPU and one numerical-library thread.

Generated artifacts stay under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/` and never enter
Git.
