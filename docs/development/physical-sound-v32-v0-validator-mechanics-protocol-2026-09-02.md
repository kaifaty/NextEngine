# Physical sound V32 V0 — validator mechanics protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_VALIDATOR_FEATURE_VALUES / SYNTHETIC_ONLY / NO_REAL_THRESHOLDS` |
| Roadmap package | V32.3 `V0` |
| Profile | [`physical-sound-v32-v0-validator-mechanics.v1.json`](../../lab/profiles/physical-sound-v32-v0-validator-mechanics.v1.json) |
| Truth parent | [V32 T0 truth/mutation result](physical-sound-v32-t0-truth-mutation-result-2026-09-02.md) |
| Product effect | None; external validator-mechanics evaluation only, authored clips authoritative |

## Question and allowed claim

Can an external validator, implemented independently of the P1/T0 generator,
reproduce the frozen T0 `9 Pass / 7 Reject` matrix from candidate PCM,
modal/lineage declarations and a separate clean truth catalog—without reading
the expected label, mutation name or precomputed invariant checks?

A V0 pass proves only deterministic feature/cache mechanics, mutation
coverage, reason precedence and OOD behavior on synthetic truth. It does not
create a real validator release, calibrate a material threshold or certify
Steel/Glass/Wood quality.

## Frozen parents

V0 is based on commit `d67bfabfb9d3d682b5c51be091a273131c7a8048` and
rejects unless these repository identities match:

| Parent | Path | SHA-256 |
| --- | --- | --- |
| T0 profile | `lab/profiles/physical-sound-v32-t0-truth-mutations.v1.json` | `44d83804398b2e08a54dbe28e4e3269098b1112785d8377dac34a11d4e5eebe8` |
| T0 owner | `lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py` | `4a0c5a5ea7cece8dfca2c8d346695bc826b9b375d94c2ef5aa1d723760391d6f` |
| T0 result | `docs/development/physical-sound-v32-t0-truth-mutation-result-2026-09-02.md` | `b17631b4032d632bb0ae9d30e05a421895756d65ce961109da94fed962c15ce9` |

The external T0 input must reproduce truth-release SHA-256
`6a5ff92000e27b8db325f29a670055147bcc891267087100876866b341a5b1d9`,
evidence `b55bb66e798e199d40e166686f94bfe35f7f09457585d3edd80250e2e5bcd9b3`
and report `60ab3dd66ed6fd1ae0eba0e25da9cc2a703528e8bf1b775ace56c3fd2be927ec`.
V0 never reads an arbitrary directory as trusted truth.

## Independence boundary

The validator implementation may import NumPy and its own profile loader. It
must not import P1 or T0 code, call their solver/render helpers, share a feature
cache with them or read generator thresholds.

The candidate view contains only:

```text
schema, record_id, target_case_id,
audio identity, modal declaration, lineage declaration
```

The fields `kind`, `source_case_id`, `expected`, `mutation` and
`invariant_checks` are ignored before feature/cache construction. V0 tests
must change all ignored fields without changing cache bytes or the decision.
The harness may compare the resulting decision with `expected` only after the
validator has returned.

The clean truth catalog is formed only from the nine profile-named record IDs.
It owns target modal/audio identities for this synthetic test. A candidate ID
or its `kind` field cannot make itself trusted truth.

## Canonical input and hard integrity

Every record JSON and `truth-release.json` must be canonical UTF-8, sorted-key,
two-space-indented, LF-terminated JSON. Every WAV must have the exact 44-byte
RIFF layout used by T0: IEEE float32 little-endian, mono, 48 kHz, 144,000
frames, exact chunk sizes and no trailing data. Record size/hash/rate/channel/
frame declarations must equal the payload.

Non-finite PCM, zero/near-zero RMS below `1e-12`, absolute sample at or beyond
`1.0`, or `abs(mean)/rms > 0.5` is an integrity failure. T0 clipping is mapped
specifically to `PcmClipping`; malformed headers, hashes, silence or DC use
`IntegrityMismatch`. Corrupt trusted release structure is `ContractReject`
before any decision publication.

## Frozen specialists and precedence

Exactly one first reason is selected in this order:

1. `ProvenanceMismatch`;
2. `PcmClipping`;
3. `RetrievalCopyDetected`;
4. `ModalCoverageCollapsed`;
5. `ModalParameterMismatch`;
6. `PhysicalDecayMismatch`;
7. `TemporalEvolutionFrozen`;
8. `EnvelopeOrderMismatch`;
9. `IntegrityMismatch`.

`IntegrityMismatch` header/hash/non-finite errors that prevent features are
returned immediately; the ordered position governs valid decodable PCM
diagnostics such as silence/DC. Unknown target case returns
`OutOfDomain(UnsupportedTargetCase)` rather than `Reject`.

### Provenance and retrieval

An optional parent lineage is valid only when actual and declared hashes are
both present, equal, and the actual hash exists in the clean catalog. An absent
pair is valid for a clean root. A half-present pair or mismatch is
`ProvenanceMismatch`.

If candidate audio bytes equal a clean audio hash belonging to a different
target case, the result is `RetrievalCopyDetected`. Equality to the candidate's
own target is not a retrieval failure.

### Modal structure and decay

Candidate and target must have the same ordered mode count. A smaller count is
`ModalCoverageCollapsed`; any other count difference or any exact mismatch in
ordinal, family indices, frequency, contact participation, pickup participation
or signed gain is `ModalParameterMismatch`. Synthetic V0 uses zero tolerance
for those declared binary64 values.

Decay values are checked separately with zero tolerance. Any mismatch is
`PhysicalDecayMismatch`. This separation prevents an honest wrong-decay record
from being misclassified as a generic modal mismatch.

### Temporal carrier evolution

All V0 target modes must share one exact declared decay. With target decay `d`,
candidate samples `x[n]` and lag `L=256`, compute:

```text
z[n] = x[n] * exp(d*n/48000)
periodicity = rms(z[L:] - z[:-L]) / max(rms(z), 1e-12)
```

`periodicity <= 1e-5` is `TemporalEvolutionFrozen`. The threshold is fixed from
float32 reconstruction error, not from opened mutation feature values.

### Envelope order

Independently reconstruct the target undamped carrier from target frequencies
and signed gains:

```text
c[n] = sum_i gain_i sin(2*pi*frequency_i*n/48000)
```

Split candidate and carrier into twelve 12,000-sample blocks. In each block,
retain samples with `abs(c) >= 1e-4 * max(abs(c))` and take the median of
`abs(x/c)`. The twelve positive finite amplitudes must be non-increasing within
relative tolerance `1e-6`. Any increase beyond that is
`EnvelopeOrderMismatch`.

The validator reconstructs only this diagnostic carrier. It does not import
the P1 renderer, use target waveform subtraction or require raw sample equality
for `Pass`.

## Deterministic feature cache

For each candidate, construct canonical candidate-view bytes after dropping
all ignored fields. The cache key is:

```text
SHA256(
  "nextengine.physical-sound-v32-v0-feature-cache.v1\0"
  || bytes(profile_sha256)
  || bytes(candidate_view_sha256)
  || bytes(candidate_audio_sha256)
  || bytes(target_clean_record_sha256)
)
```

All four hashes are decoded 32-byte values. Cache payloads contain only the
key, input identities and deterministic scalar/list features. Cache warmth,
path, run order and ignored labels cannot change bytes or decisions.

## Publication, OOD and resources

Each of 16 records publishes `features.json` and `decision.json`; the top level
adds `validator-mechanics-release.json`, `evidence.json` and `report.json`, for
35 files total. Output paths and artifact indexes are canonical, sorted and
unique. Two complete executions must match byte-for-byte.

The CLI accepts only an exact T0 directory and a fresh external output path.
Trusted input corruption, dependency drift, unsafe output, resource overflow
or internal inconsistency returns `ContractReject`, removes staging and
publishes nothing. A direct unknown-target probe must return
`OutOfDomain(UnsupportedTargetCase)` without a feature artifact.

One run is bounded to 16 candidates, 35 files, 64 MiB output, 1 GiB RSS and 300
seconds. It is single-threaded CPU-only and performs zero network, real signal,
protected signal or model access.

## V0 gates

V0 passes only when:

1. the complete CLI runs twice with byte-identical stdout and 35-file trees;
2. all nine clean candidates return `Pass` and all seven mutations return the
   exact frozen reason;
3. changing every ignored label field leaves candidate view, cache and decision
   bytes unchanged;
4. PCM/header/hash/provenance/retrieval/modal/decay/periodicity/envelope and
   unknown-target probes exercise their declared owner;
5. cold/warm and record-order permutations produce identical feature/cache and
   decision artifacts;
6. strict profile, duplicate/non-finite JSON, parent/release drift, artifact
   corruption, unsafe output and forced failure publish nothing;
7. resource bounds and zero-access counters pass.

A pass authorizes only V32 M0 protocol work and later V1 real qualification
after S1. It does not authorize a real encoder, real thresholds, a validator
release, training, admission, cooking, demo use, runtime integration or any
ProductCheck claim.

## Stop rules

- Do not inspect V0 feature values before this protocol/profile commit.
- Do not tune thresholds or precedence after seeing which mutation fails.
- Do not use the T0 expected label or mutation name inside the validator.
- Do not add CLAP/text-prompt similarity; real representation remains V1 work.
- If a frozen mechanic rejects clean truth or misses its mutation, V0 rejects.
  A successor needs a new hypothesis/profile rather than a threshold patch.
- Generated features, decisions and caches stay outside Git.
