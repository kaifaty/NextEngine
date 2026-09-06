# Physical sound V45 C0 — Clatter external-control protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| State | `FROZEN_BEFORE_NUMERIC_PAYLOAD_DECODE` |
| Source | `https://github.com/alters-mit/clatter` at commit `79cac6cbe3f7c452ba28b56c7da4a0124ad04806` |
| Scope | External empirical-modal prior and synthetic comparator only |
| Excluded authority | Real acoustic parent/project power, material truth, generator training, validator calibration, protected admission, cooker, demo, public contract and runtime |

## Falsifiable question

Can the exact published Clatter impact-material table be decoded into the
already-frozen Recipe V3 modal coordinates and rendered by one bounded,
source-independent control procedure twice with byte-identical outputs?

`Pass` means only that this external baseline is available and reproducible.
It does not mean that Clatter sounds natural, identifies a real material,
beats the analytic baseline or is safe to ship.

## Pre-access seal

Before any `.bytes` value is interpreted as a number, the owner and execution
profile must freeze:

- the exact Git commit and source URL;
- the expected `14 × 6 = 84` impact-material filenames;
- the path-neutral tree root: for filenames in UTF-8 byte order, hash each
  payload with SHA-256, emit `<hex><two spaces><basename><LF>`, concatenate the
  84 lines and SHA-256 the result;
- exact byte/hash bindings for the source files that define the binary layout,
  ten-mode synthesis, contact force, PRNG, sample rate, material enumeration,
  README and license;
- the binary decoder, Recipe V3 projection, seeded control renderer, resource
  limits, output layout and failure behavior below.

The expected tree root is
`8230a6192f189806b899a9113c08fefaf458e9e7f26322e2fc6fe5434a4c065e`.
Hashing a payload for integrity does not decode a numeric value. All integrity
checks finish before the first numeric unpack.

## Binary decoder

Each file is decoded independently as:

```text
little-endian int32 cf_count, op_count, rt_count
little-endian binary64 cf[cf_count]
little-endian binary64 op[op_count]
little-endian binary64 rt[rt_count]
EOF
```

The owner rejects a negative count, unequal array counts, fewer than ten rows,
trailing/truncated bytes, non-finite values, non-positive `cf` or non-positive
`rt`. It reads all values for structural validation but uses only paired rows
`0..9`, matching the frozen source implementation. Unknown filenames,
additional files, symlinks, Git mismatch or source semantic-hash drift reject
the complete C0 publication.

## Neutral Recipe V3 projection

For each source file, the ten paired rows are sorted by `(cf, source_ordinal)`.
The external empirical target observes only:

- `modal.mode_count`;
- `modal.base_frequency_millihz`;
- `modal.frequency_ratio_ppm`;
- `modal.rt60_milliseconds`;
- `modal.participation_ppm`.

Frequency and RT60 use round-to-nearest, ties-to-even when converted to integer
millihertz/milliseconds. Base frequency is the first sorted frequency. Ratios
are `round(cf / base × 1,000,000)`. Participation is proportional to the source
renderer amplitude law `10^(op/20)` and closes to `1,000,000 ppm` through
integer floor plus stable largest remainder. Uncertainty remains unobserved;
excitation, radiation/residual and OOD coordinates receive neutral defaults and
zero target mask. The frozen T0 projector remains the sole owner of ordering,
positivity, Nyquist, resource and energy bounds.

If projection would clamp a decoded modal frequency, RT60 or ratio, C0 rejects
instead of silently changing the source prior. The target and recipe stay in
the external run directory; Git records only their aggregate hashes and bounded
summary.

## Seeded control renderer

The renderer is a clean-room comparator, not a port or runtime dependency. For
each filename it derives one 64-bit seed from the frozen root seed and basename,
uses SplitMix64 plus Box–Muller, and applies the source-declared perturbations:

- frequency standard deviation `cf / 10`, retrying below `20 Hz`;
- onset level standard deviation `10 dB`;
- RT60 standard deviation `rt / 10`, retrying below `0.001 s`.

It renders the ten cosine modes at `48 kHz` for `48,000` samples with resonance
`1`, convolves them with a `48`-sample half-sine contact pulse, peak-normalizes
to amplitude `0.5`, quantizes to mono PCM16 with ties-to-even and writes a
canonical RIFF/WAVE file. One retry loop is capped at `128`; non-finite or zero
output rejects. The renderer is designed only to make the prior inspectable and
repeatable under this Python/platform profile. It does not claim bit parity
with Clatter's .NET runtime or audio output.

## Output and decision

A fresh external output directory contains:

- `access.json` — exact access counters;
- `inventory.json` — source bindings and per-payload structural metadata;
- `prior.json` — neutral Recipe V3 targets and recipe/render hashes;
- `renders/*.wav` — 84 external audition/control files;
- `report.json` — gates and bounded aggregate statistics.

C0 returns `ClatterExternalControlAvailable` only if all 84 rows project, every
WAV validates, every gate passes and two fresh runs are byte-identical. Any
integrity, decode, projection, render, resource or publication failure returns
`ExternalPriorUnavailableOrIncompatible` by absence of a published output.
There is no partial row acceptance and no authored/runtime fallback change.

## Resource and retention boundary

- exactly 84 payloads, at most `16,384` values per array and `4 MiB` per file;
- exactly ten rendered modes, `48,000` output frames and one mono PCM16 WAV per
  payload;
- external source and outputs only; no network during owner execution;
- no source code, payload, decoded values, WAV, cache, checkpoint or external
  model is committed to Next Engine;
- Clatter remains attributed under its published
  `Hippocratic-License-3.0-HL3-BDS-ECO-FFD-LAW-MEDIA-MIL-SV` terms. This C0
  consumes it only as an external research control and grants no redistribution
  or production/runtime authority.
