# Physical sound V26 P0a — barycentric surface and padded alignment protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Protocol revision | `m0b-v1.1` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION_COMPLETION / MODEL_VALUES_UNOPENED / EXECUTION_EQUIVALENT_PREPROCESSING_REPAIR_ONLY` |
| Roadmap package | V26 `P0a`, prerequisite of `I0` and `M0b-E` |
| Supersedes | [P0 m0b-v1.0](physical-sound-v26-p0-barycentric-surface-query-protocol-2026-09-02.md) before implementation/root/model values |
| Rebaseline | [P0 implementation-conformance result](physical-sound-v26-p0-implementation-conformance-rebaseline-2026-09-02.md) |
| Product effect | None; external feasibility evidence only, with authored clips authoritative |

## Question and bounded claim

Can the frozen M0a experiment complete deterministic preprocessing when legal
surface queries are off-vertex and legal transfer peaks occur before the
declared output anchor, without changing a model, optimization, evidence role
or quality gate?

P0a may establish only implementation-ready preprocessing semantics. I0 must
still prove the complete owning entry point twice before M0b-E. No P0a/I0
result is real-material quality, automatic admission, public content or runtime
authority.

## Inherited P0 surface contract

Every surface-query rule, constant and fixture from P0 SHA-256
`4fff3ebd7b788e527b524ff8615d9e8a449b40121e194ba1eb947c5ea81e65e7`
remains unchanged:

- `surface_tolerance_m = max(1e-12, diagonal * 2^-40)`;
- barycentric tolerance `2^-40` and degeneracy factor `2^-80`;
- complete face validation, canonical sorted face key and lexicographic tie;
- float64 triangle projection, weights, reconstruction and field interpolation;
- refined interpolated T0 gain target, coarse independent remesh input and
  unchanged model remesh gate `<= 1e-5`;
- 48-query/24-pair official-shape affine fixture and every declared mutation;
- M0b manifest/schemas, nine canonical artifacts, role order, resources and
  stop rules.

P0a adds only the alignment contract below and its fixture coverage.

## Padded absolute-peak alignment V1

Input is exactly `230,215` little-endian finite float32 samples. Output is
exactly `144,000` float32 samples. Freeze:

```text
anchor = 512
maximum_leading_padding = 512
maximum_trailing_padding = 512
minimum_copied_source_samples = 144000 - 2*512 = 142976
```

Algorithm:

1. reject wrong byte length, non-finite input or an all-zero absolute peak;
2. choose the lowest source index whose absolute value equals the global
   absolute maximum; no threshold, onset detector or local search exists;
3. allocate an all-zero float32 output of length `144,000`;
4. set `raw_start = peak_index - 512`,
   `source_start = max(0, raw_start)` and
   `destination_start = max(0, -raw_start)`;
5. copy exactly
   `min(source_length-source_start, output_length-destination_start)` contiguous
   samples without resampling, scaling, denoising or reordering;
6. compute leading/trailing pad counts and reject if either exceeds `512`, if
   fewer than `142,976` source samples were copied, or if the first absolute
   output maximum is not exactly index `512`;
7. retain padding counts, source peak, copied count and output SHA-256 in the
   preprocessing transform record.

Padding bytes are positive zero float32. Signed zero, NaN payload preservation,
reflection, repetition, wrap, fade, crop search and response equalization are
forbidden. Zero padding supplies absence of pre/post-source evidence; it does
not fabricate measured sound.

## Alignment conformance fixture

The value-independent alignment fixture constructs exact-length finite float32
arrays and covers:

- early peaks at indices `0`, `39`, `73`, `87` and `511`;
- centered peaks at `512` and `72,000`;
- latest legal peak with exactly `512` trailing zeros;
- two equal absolute maxima, proving the lower source index wins;
- negative maximum, subnormal finite values and nonzero signed samples;
- all-zero, NaN, Inf, wrong byte count, excessive trailing padding and
  insufficient copied-source rejection;
- one-step mutations around both `512` padding ceilings and the `142,976`
  copied-sample floor;
- exact preservation of every copied source sample and zeros only outside the
  copied interval.

For the three disclosed adaptation contexts, the I0 preflight verifies only
the contract facts already opened by the rebaseline: peaks `87/73/39`, leading
pads `425/439/473`, no trailing pad, copy counts
`143575/143561/143527`, and output first maximum `512`. Candidate code receives
only the aligned arrays and the ordinary hash-closed transform records.

The final disclosed query is not an I0 selection input. Its bytes open only in
the inherited post-freeze role phase of a successful official M0b execution.

## Unchanged M0b value surface

The following remain exactly M0a v1.1 and P0:

- model ID `m0a-contact-modal-field-v1`, all layers, residual atoms, parameter
  count, inputs and unknown-axis masks;
- seed `3101`, CPU/one-thread determinism, `1500 + 500` steps, optimizer and
  batch schedule;
- every loss/weight, renderer, ridge/control/ablation and nuisance log gain;
- synthetic development, calibration, candidate freeze, one-shot method
  holdout, disclosed real query and sealed admission/row `2407` access order;
- all hard/causal/quality/ratio thresholds and stop decisions;
- canonical tensors, atomic output, diagnostic local MLflow and resource limits;
- unchanged combined V3, T0 evidence, X0 lineage and executable hashes.

No opened preprocessing fact may select an architecture, capacity, seed, loss,
threshold, contact, query, checkpoint or training length.

## Implementation identity and topology

The four frozen inherited M0a module hashes from P0 remain mandatory. No M0a
file is edited. M0b owns new common, surface, alignment, preprocessing and
entry-point modules. Its implementation root is the SHA-256 of the canonical
sorted filename-to-file-hash map containing all nine inherited/new modules.

Dynamic monkey-patching remains forbidden. The new preprocessor uses named
frozen M0a helpers, the P0 surface evaluator and the P0a alignment helper
through ordinary imports. The manifest pins this P0a SHA-256, the complete
implementation root and the inherited hash map.

The official output remains P0's nine files, including
`surface-query-report.json`. Alignment evidence belongs in
`preprocess-manifest.json`; no extra value-dependent file or selection surface
is added.

## I0 and official stop rules

- P0a freezes before alignment code is implemented or an M0b root exists.
- I0 runs the 48-query surface fixture, alignment fixture, complete owning
  entry point and all inherited/mutation/atomic tests twice exactly.
- P0 v1.0 partial code cannot be committed or executed as official M0b.
- M0a remains spent and unmodified.
- One failed official M0b execution spends M0b; B does not start after A fails.
- Opened M0b values cannot select any preprocessing/model/threshold change or
  retry. A successor requires a new hypothesis and protocol.
- A pass authorizes only V0/Metal research sequencing. Runtime, public schemas,
  material admission and removal of authored fallback remain unauthorized.
