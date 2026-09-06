# Physical sound V44 G0B1c0 — noise-robust target synthetic result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / SYNTHETIC_TARGET_ADMITTED / 75_DIMENSION_FIXED / CLEAN_MODE_PASS / NOISE_REJECTION_PASS / MIXTURE_PASS / SCALE_IDENTITY_PASS / IETEASY_TARGET_ACCESS_AUTHORIZED / CORPUS_CREDIT_ZERO / G0B1C1_NEXT`

Planning authority: [Roadmap V44](../plans/physical-sound-synthesis-roadmap-v44.md)

## Outcome

G0B1c0 replaces the rejected arbitrary top-N mode list with a bounded,
fixed-size training target and tests it without opening any new IETeasy target
value. The terminal decision is `NoiseRobustTargetSyntheticAdmitted`.

The target has exactly `75` numerical values:

- `24` logarithmic-frequency bins of tonal excess above a local spectral
  baseline;
- `24` bins of the complementary broadband residual;
- `24` counts of energy-qualified prominent peaks;
- total tonal-excess mass, spectral flatness and total qualified-peak count.

Tonal and residual energy use one joint largest-remainder apportionment and
therefore sum to exactly `1,000,000 ppm`. A peak must have at least `6 dB`
local prominence and at least `3,000 ppm` of total valid-band excess energy;
the fixed target aggregates all qualifying evidence rather than truncating a
variable list at 12, 24, 48 or 96 entries. The ordered small modal recipe stays
a separate future renderer output.

The protocol and thresholds were frozen and exercised using only deterministic
synthetic signals. The access ledger records zero IETeasy, disclosed-audio,
candidate, model, validator, protected and network values. Passing G0B1c0
authorizes exactly one next operation: bind this owner and protocol unchanged
and apply it to the previously selected 15 IETeasy records in G0B1c1. It does
not grant corpus, PSEL, model, validator, admission, cooker, demo or runtime
credit.

## Why this representation

SciPy explicitly warns that noise can move or create local maxima, and suggests
smoothing or another peak method for noisy data; its prominence definition is
the height of a peak above the surrounding baseline, with a finite window able
to define intentionally local prominence
([`find_peaks`](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.find_peaks.html),
[`peak_prominences`](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.peak_prominences.html)).
That supports using local prominence as evidence, but not treating every local
maximum as a physical mode.

Prior game-audio work also reports that a modal reconstruction often needs a
residual and that some recordings are not well represented by modal synthesis
alone ([Raghuvanshi, Lloyd and Govindaraju, 2011](https://www.microsoft.com/en-us/research/publication/sound-synthesis-impact-sounds-video-games/)).
G0B1c0 therefore preserves tonal and residual energy explicitly instead of
forcing broadband energy into an ever-longer modal list. These sources motivate
the structure only; the tracked synthetic protocol, not a web page, owns the
exact algorithm and gates.

## Frozen algorithm

The input boundary is C0-compatible mono `float64`, `48 kHz`, exactly `144,000`
samples, with analysis after the `480`-sample pretrigger. The owner:

1. divides by absolute peak, then computes an `8,192`-sample Hann Welch spectrum
   with `2,048`-sample hop;
2. estimates a local log-power baseline with a deterministic 31-bin median;
3. finds peaks over `80–18,000 Hz` with `6 dB` prominence, `40 Hz` separation
   and a 61-bin local prominence window;
4. retains only peaks whose excess over the baseline is at least `3,000 ppm`
   of total valid-band energy;
5. assigns above-baseline energy within two bins of each qualified peak to the
   tonal plane and all remaining energy to the residual plane;
6. aggregates both planes and qualified-peak counts over the same 24 geometric
   bands and emits the fixed 75-value target.

The owner rejects non-finite, non-mono, wrong-length and silent input, more than
333 qualified peaks, dimension drift, non-closing energy/count partitions,
non-canonical dependencies, in-repository output and repeat publication.

## Exact synthetic result

| Control | Result |
| --- | ---: |
| Clean three-mode qualified peaks | `3` |
| Clean dominant bands | `7 / 12 / 17` |
| Clean tonal / residual energy | `999,692 / 308 ppm` |
| Broadband-noise qualified peaks | `0` |
| Broadband-noise tonal / residual energy | `0 / 1,000,000 ppm` |
| Broadband-noise spectral flatness | `808,017 ppm` |
| Modal-plus-noise qualified peaks | `3` |
| Modal-plus-noise tonal energy | `976,135 ppm` |
| Mixture-to-clean tonal cosine | `999,999 ppm` |
| Mixture-to-clean tonal L1 | `23,557 ppm` |
| Frequency-shift-to-clean tonal cosine | `0 ppm` |
| Positive scale identities | `4 / 4` |
| Target canonical JSON maximum allowed | `8,192 bytes` |
| Synthetic fixtures / target extractions | `4 / 8` |
| Synthetic sample values generated | `576,000` |

The clean control contains modes at `430 / 1,230 / 4,170 Hz`. Its noisy version
adds deterministic exponentially decaying white noise at `0.3×`; it preserves
the same three dominant bands. The shift control moves the modes to
`860 / 2,460 / 8,340 Hz` and becomes orthogonal to the clean 24-bin tonal
distribution. `0.5×` and `2×` scale mutations for the clean and mixed controls
produce byte-identical targets.

## Frozen implementation

- protocol:
  `lab/profiles/physical-sound-v44-g0b1c0-noise-robust-target-protocol.v1.json`,
  `5,293 bytes`,
  `sha256=a82fbd84a677a49ab95f1a68394a2cdefa4f885991f3cff7ff8ab34b201ddcec`;
- execution profile:
  `lab/profiles/physical-sound-v44-g0b1c0-synthetic-target-audit.v1.json`,
  `2,471 bytes`,
  `sha256=c1a0b1d10f4b76256ea2efe11ce3bd94b9a7d90f5df3be57ed07a98a7ad5bdbe`;
- owner:
  `lab/scripts/physical_sound_v44_g0b1c0_noise_robust_target_v1.py`,
  `31,087 bytes`,
  `sha256=378b96ba4edfc382a1840d255bde5a1425bd0f44e7d9759202e562bae4ab0266`;
- focused tests:
  `lab/tests/test_physical_sound_v44_g0b1c0_noise_robust_target_v1.py`,
  `6,314 bytes`,
  `sha256=a447876f43e21d5a12511b17f82e210da3b8a8a8b85584bd0d598febc8101f90`.

The profile binds SPEC-45, the G0B1b terminal evidence, the exact protocol,
C0's compatibility boundary, the owner and NumPy `2.5.2` / SciPy `1.18.0`.
The live roadmap is intentionally not a hash dependency.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v44-g0b1c0-synthetic-target-2026-09-03`

`run-a` and `run-b` are byte-identical. Each contains five files and `19,906`
bytes. SHA-256 of the canonical sorted array of `{bytes,path,sha256}` rows is
`f7b2755a3f8916bc58bcdf3b779a6a3ab9ba4f9e059a984e9f27ec87472dd2ce`.

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `2878cf6ac923dd11e17b511f131e1bcfd4a718865642f551815233da97dfaa73` |
| `audit.json` | `8794a3fbb3a2df665da60cd17355fd37b29d1ec4694de8728fd4f9735789d475` |
| `profile.json` | `c1a0b1d10f4b76256ea2efe11ce3bd94b9a7d90f5df3be57ed07a98a7ad5bdbe` |
| `protocol.json` | `a82fbd84a677a49ab95f1a68394a2cdefa4f885991f3cff7ff8ab34b201ddcec` |
| `report.json` | `971efac43a4a5ace7b0f201900bb12132fef62b93a466a9b9f831f7076cde9a4` |

## Consequence and next action

G0B1c1 may now open only the same 15 signal-blind IETeasy selections from
G0B1a and compute this exact target without parameter changes. Before access it
must freeze:

- exact G0B1a selection/media/payload and G0B1c0 owner/protocol bindings;
- per-record finite/dimension/partition/resource gates;
- dataset-level saturation and degeneracy diagnostics that cannot select a new
  threshold after access;
- one immutable disclosed-role assignment for all 15 parents;
- a terminal `CorpusSuccessorMaterialized` or `TargetDomainOOD` decision.

Even a G0B1c1 pass adds only one Steel project and at most 15 disclosed parents.
The second independent descriptor-to-signal Steel project and signal-blind PSEL
remain mandatory; current supported-parent accounting and the `49`-parent
planning deficit do not change at G0B1c0.

## Verification

- focused G0B1c0 suite: `PASS`, `8/8` tests;
- combined G0B0/G0B1a/G0B1b/G0B1c0 suite: `PASS`, `29/29` tests;
- Ruff `0.14.1` format/static analysis: `PASS`;
- Python compile: `PASS`;
- official external A/B and recursive byte comparison: `PASS`;
- canonical profile/protocol, fixed dimension, energy/count closure, target
  bounds, scale identity, frequency sensitivity, input mutation, output guard
  and atomic repeat rejection: `PASS`;
- IETeasy/disclosed-audio/candidate/model/validator/protected/network reads:
  `0 / 0 / 0 / 0 / 0 / 0 / 0`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  G0B1c0 adds no source-layout escape hatch and receives no boundary-scan
  credit;
- runtime ProductChecks: `NOT_RUN`, because this remains external research under
  Proposed SPEC-45 with no production consumer or public contract.
