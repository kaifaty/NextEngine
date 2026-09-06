# PS-2 REALIMPACT colored-residual successor preregistration — 2026-08-28

## Decision

`RealImpactColoredResidualSuccessorProtocolFrozen`.

One successor is now hash-closed before any fresh archive or member access.
`modal_plus_seeded_colored_subband_residual_v1` changes only within-band
excitation coloration. Modal extraction, two-exponential amplitude envelope,
listener gains, deterministic seed path and rank-one spatial excitation remain
the old control.

This is a representation protocol and synthetic capacity result. It is not a
material label, real-data success, audible-quality result, domain admission,
runtime design or production authorization.

## Frozen lineage

| Artifact | SHA-256 / result |
| --- | --- |
| Successor runner | `fe8a12f516ee8f85dc515c23cf2b7f67cc6bcc4d12a2b7f20c7f69d0626cebf5` |
| Manifest | `f03e428ee9b6005b96519869541fd27a301e0343a4e0f800b5d74f770d3e564a` |
| Preflight A/B | `4c754f0630d16ade6fd7de3479fc15bb54a193a9f928b72eaf3e1342424da360`, byte-identical |
| Parent discovery / diagnostic | `578e143c…aec31` / `d8acc909…ba1eb` |
| Fresh network/member/shadow bytes | `0 / 0 / 0` |

Artifacts are external under
`/tmp/nextengine-physical-sound-colored-residual-successor-v1c`. An earlier
zero-access manifest/preflight `edf16549…fc2a7` / `6a16eb8e…d68ed` is rejected
because its descriptive string said “periodic Hann” while the code used
`np.hanning`, a symmetric Hann. The accepted lineage corrects only that text;
its numeric fixture roots remain unchanged.

## One changed representation

For each existing residual band the candidate:

1. measures mean power across 15 listeners and complete 512-sample symmetric-
   Hann frames with a 256-sample hop inside the unchanged 8,192-sample fit
   window;
2. converts power to dB, applies the frozen `1e-8` relative floor and removes
   the per-band mean;
3. stores at most eight non-DC orthogonal DCT-II coefficients, each bounded to
   `±18 dB`, and bounds reconstructed shape to `±18 dB`;
4. interpolates that shape onto the unchanged 32,768-point deterministic
   excitation RFFT, multiplies magnitudes, inverse-transforms and renormalizes
   to unit RMS;
5. applies the unchanged two-exponential envelope and listener gains.

The maximum additional representation is 64 bounded coefficients per object.
No runtime learning, model inference or recorded residual clip is introduced.

## Synthetic control

Eight deterministic colored bands validate the proposed capacity:

| Metric | Rectangular control | Colored candidate | Ratio | Gate |
| --- | ---: | ---: | ---: | ---: |
| Spectral-shape MAE | 3.616051 dB | 1.134416 dB | 0.313717 | ≤ 0.75 |
| Short-lag autocorrelation MAE | 0.135672 | 0.085300 | 0.628724 | ≤ 0.85 |

All eight profiles stay within coefficient/count bounds and repeat exactly.
This proves only that the compact representation can recover known synthetic
coloration; it does not predict a real-object result.

## Fresh grouped split

The [pinned official roster](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/dataset/object_names.txt)
hashes to `3ee26ac9…ad5a`. Names establish identity and grouping, not material.

| Role | Objects | Semantic family | Current access |
| --- | --- | --- | --- |
| Development diagnostic only | six already opened metal-batch objects including Metal Spoon | six prior groups | no new access; never selection/holdout credit |
| Calibration | `19_Pan`, `37_PiePan` | `pan` | archive identity unknown; zero bytes |
| Holdout | `22_Cup` | `cup` | archive identity unknown; zero bytes |
| Shadow | `89_MetalSpatula`, `92_MetalSpatula` | `metal_spatula` | inherited structural discovery; member payload zero |

The Pan pair cannot split across roles. Metal Spoon remains a diagnostic
counterexample and cannot return as calibration, holdout or shadow.

## Frozen gates

Calibration requires median ratios `≤0.95` for every prior gating metric,
maximum `≤1.05`, spectral-shape `≤0.75`, autocorrelation `≤0.85` and both Pan
objects improved. Holdout keeps prior metrics at `≤0.95`/`≤1.05`, plus
spectral-shape `≤0.80` and autocorrelation `≤0.90`. Shadow has its own frozen
family-median gates and cannot open before an immutable successful Cup report.

Waveform NRMSE, modulation power, coherence and effective rank remain
diagnostic. The known rank-one defect is deliberately unchanged so this cycle
tests one causal hypothesis.

## Access and stop rule

The next stage may make exactly three archive-identity requests, one for each
fresh Pan/PiePan/Cup archive. It may not read member payload. A later,
separately frozen discovery stage may read only exact ZIP tails and observation
local headers. Any identity, structure, decoder, repeat, calibration or holdout
failure stops later access without retry, object substitution or prefix growth.

## Checks

- Ruff format/check and Python byte compilation: pass;
- corrected manifest and preflight A/B: byte-identical;
- synthetic coloration fixture: all bounds/gates pass;
- lineage, official roster, rejected diagnostic and fallback flags: exact;
- network, new member payload and shadow member payload: zero;
- current ProductChecks: not run because no runtime/content/public contract
  changed.

## Next action

Implement the bounded discovery stage and query only the archive identities for
`19_Pan`, `37_PiePan` and `22_Cup`. Freeze those identities before writing any
tail/local-header range or payload decoder.
