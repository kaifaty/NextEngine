# Physical sound V44 G0B1b — target-cap/normalization audit result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / NORMALIZATION_PASS / REFERENCE_NON_SATURATING / 12_24_48_96_MODE_CAPS_REJECT / TARGET_REPRESENTATION_REDESIGN_REQUIRED / CORPUS_CREDIT_ZERO / PSEL_BLOCKED`

Planning authority: [Roadmap V44](../plans/physical-sound-synthesis-roadmap-v44.md)

## Outcome

G0B1b answers the anomaly found by G0B1a without tuning a cap after seeing
alternative targets. Before computing any higher-cap value, the tracked
protocol froze one `512`-mode non-saturating reference, the candidate caps
`12/24/48/96`, three per-record similarity gates and the rule “choose the
smallest cap passing every selected record”. It also froze positive input-gain
mutations `0.5×/2×` and exact PCM/target identity as the normalization gate.

The terminal decision is `TargetRepresentationRedesignRequired`.
Normalization is not the problem: all `30/30` gain mutations reproduce the
canonical PCM and target exactly, and every normalized segment contains zero
clipped samples. The modal list is the problem. The non-saturating reference
finds `138–225` peaks per three-second recording; no fixed candidate cap passes
all `15` records. Even cap `96` passes only `9/15`, retaining as little as
`77.1135%` of reference linear modal amplitude and missing the preregistered
histogram-distance bound.

This is evidence against raising `maximum_modes` until the test turns green.
The current peak list mixes stable resonances with broadband/noise-local
maxima, while the downstream B0 representation collapses that list into a
fixed 24-bin modal histogram. A larger arbitrary top-N would consume more
space without resolving which peaks are physically meaningful. G0B1b grants
no target-policy, corpus, supported-parent, PSEL, model, validator, admission,
cooker, demo or runtime credit.

## Frozen audit

The reference cap `512` is above the protocol's analytical maximum of `449`
peaks implied by the `80–18,000 Hz` interval and `40 Hz` minimum separation, so
the reference cannot truncate a valid peak set. Every other C0 extraction and
B0 vectorization parameter remains unchanged.

For every candidate cap and every selected record, the protocol requires:

- retained linear modal amplitude `>= 0.95` of the reference;
- 24-bin B0 modal-histogram cosine similarity `>= 0.99`;
- normalized modal-histogram L1 distance `<= 0.10`.

The cap decision is the smallest candidate satisfying all three gates on all
records. Reference saturation, normalization failure and “no cap passes” are
distinct terminal results. Candidate/model/validator/protected values and
network access are forbidden.

## Exact result

| Candidate cap | Records passing | Minimum retained amplitude | Minimum histogram cosine | Maximum normalized L1 |
| ---: | ---: | ---: | ---: | ---: |
| `12` | `0 / 15` | `0.254627` | `0.706813` | `0.700164` |
| `24` | `0 / 15` | `0.315733` | `0.802129` | `0.568366` |
| `48` | `1 / 15` | `0.593411` | `0.939520` | `0.365937` |
| `96` | `9 / 15` | `0.771135` | `0.986856` | `0.186058` |

| Other observation | Result |
| --- | ---: |
| Selected waveforms / physical parents | `15 / 15` |
| Selected payload bytes | `1,657,896` |
| Decoded sample values | `6,801,630` |
| Target extractions | `105` |
| Reference mode-count range | `138–225` (median `177`) |
| Reference cap saturated | `0 / 15` |
| Gain mutations reproducing PCM/target | `30 / 30` |
| Post-normalization clipped samples | `0` |
| Candidate/model/validator/protected/network reads | `0 / 0 / 0 / 0 / 0` |

The protocol compares the actual B0 24-bin modal histogram, not raw waveform
phase or listening preference. It stores only hashes, counts and bounded audit
metrics outside Git; no source audio or alternative target values enter the
repository.

## Consequence and successor shape

G0B1c must preregister a fixed-size, noise-robust target before computing its
values. The smallest credible successor is not a longer mode list:

1. keep C0 spectral bands, transient envelope and global time/spectral fields;
2. replace top-N peak truncation in the ML target with a fixed-bin
   prominence/energy distribution over all admissible peaks;
3. add explicit tonal-mass, peak-density and broadband-residual summaries so
   “many local maxima” is not mislabeled as “many physical modes”;
4. retain a small ordered modal recipe only as renderer output, recovered by a
   separate bounded projection rather than copied from every spectral peak;
5. require synthetic clean-mode recovery, noise-only rejection,
   scale-invariance, finite/resource limits and exact C0/B0 compatibility
   fixtures before re-reading the 15 IETeasy values.

If that value-independent protocol cannot produce a stable fixed-size target,
IETeasy remains catalogue/payload evidence only and the corpus deficit stays
`49`. The metadata-first search for a second independent Steel project remains
parallel and cannot use this disclosed audit as PSEL credit.

## Frozen implementation

- preregistered protocol:
  `lab/profiles/physical-sound-v44-g0b1b-target-audit-protocol.v1.json`,
  `3,333 bytes`,
  `sha256=1e3cd03160adf771d7417f3e8458e50fa01333625976e4e863191c1b671921c0`;
- execution profile:
  `lab/profiles/physical-sound-v44-g0b1b-target-audit.v1.json`,
  `3,004 bytes`,
  `sha256=afde75d8e99af0a8b07376356612e1ed1aa6f1b10b1f0f16dd569cffb9ce5c03`;
- owner:
  `lab/scripts/physical_sound_v44_g0b1b_target_audit_v1.py`,
  `28,415 bytes`,
  `sha256=60aef9958acb6762ced7a12d129303390ed938fd1a8dbc84f0b4435af8a5b9cb`;
- focused tests:
  `lab/tests/test_physical_sound_v44_g0b1b_target_audit_v1.py`,
  `7,015 bytes`,
  `sha256=61c4765e6d7769f78fe6424a9f6dc0c91d82cef040d5878d89623e4758ef835d`.

The profile binds SPEC-45, G0B1a evidence/owner, the immutable G0B1b protocol,
C0 extraction, B0 vectorization, exact NumPy `2.5.2`, SciPy `1.18.0` and
ffmpeg. The live roadmap is not a hash dependency.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v44-g0b1b-target-audit-2026-09-03`

`run-a` and `run-b` consume the same frozen G0B1a evidence and payloads and are
byte-identical. Each output has five files and `52,264` bytes. SHA-256 of the
canonical sorted array of `{bytes,path,sha256}` rows is
`62c9a59e045c17bd7b177671df91a1469ee40b9d10b29f230c8e84c81a33ac36`.

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `58be020fd6b259eb35d9e3f5f0c0ef1623ce0b40931d420f264966bb11ee0399` |
| `audit.json` | `1582ec35beb3ae526518a05f8d7884a256f3668f89fc36148a4f1b1eb451824e` |
| `profile.json` | `afde75d8e99af0a8b07376356612e1ed1aa6f1b10b1f0f16dd569cffb9ce5c03` |
| `protocol.json` | `1e3cd03160adf771d7417f3e8458e50fa01333625976e4e863191c1b671921c0` |
| `report.json` | `1b898bfd04ebae54dd9f43b5c699c4496c2b240bb6827ecbed0211320a5e036c` |

## Verification and next action

- focused G0B1b suite: `PASS`, `7/7` tests;
- official external A/B and recursive byte comparison: `PASS`;
- exact legacy target replay, non-saturating reference, four cap candidates,
  gain mutations, target/vector identity and zero-clipping checks: `PASS`;
- input mutation, reference saturation, normalization failure, target-owner
  state restoration, repository output and forbidden network imports are
  covered by deterministic terminal or rejection tests;
- Ruff `0.14.1` format/check and Python compile: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  G0B1b adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`; this remains external research tooling
  under SPEC-45 `Proposed` and changes no production consumer.

G0B1c is next: freeze the noise-robust fixed-size target contract and synthetic
fixtures before reading new IETeasy target values. A corpus successor can
follow only if that target passes its own audit. PSEL, B1, validator/model
training, protected access, cooking and demo integration remain blocked;
authored clips remain the only production path.
