# Physical sound V32 T0 — truth and mutation protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_T0_GENERATED_VALUES / SYNTHETIC_ONLY / ZERO_REAL_SIGNAL_MODEL_NETWORK` |
| Roadmap package | V32.1 `T0` |
| Profile | [`physical-sound-v32-t0-truth-mutations.v1.json`](../../lab/profiles/physical-sound-v32-t0-truth-mutations.v1.json) |
| Parent | [V31 P1 deterministic modal owner](physical-sound-v31-p1-deterministic-modal-owner-result-2026-09-02.md) |
| Product effect | None; external validator-mechanics truth only, with authored clips authoritative |

## Question and allowed claim

Can one deterministic external owner publish an unambiguous library of clean
causal renders and adversarial corruptions before a validator or learned model
is implemented?

T0 freezes the expected decision for every record. A passing result may claim
only that the library is byte-reproducible, that each corruption preserves and
violates the declared invariants, and that the clean causal relations remain
identical to P1. It cannot claim that any waveform sounds like real Steel,
Glass or Wood; that a validator is calibrated; or that a generator, cooker,
demo or runtime path is admitted.

## Frozen parent identities

T0 is based on commit `101cc0be4ffa0def1ecb455fe57ffccc300af240` and must
reject before publication unless all of these bytes match:

| Parent | Path | SHA-256 |
| --- | --- | --- |
| P0 causal profile | `lab/profiles/physical-sound-v31-p0-causal-baseline.v1.json` | `c6b7f816d65bdcaa18618a72d40cfedb6ed970359d3a1cf8f28825cd4c686a3d` |
| P1 modal owner | `lab/scripts/physical_sound_v31_p1_modal_owner_v1.py` | `04b44d77ce073d07f30e94c3c361ca4c199cb554ecbe017947aa423842781650` |
| P1 result | `docs/development/physical-sound-v31-p1-deterministic-modal-owner-result-2026-09-02.md` | `16e3f6e23280eb3c6d77d52132b6d216768bf8383baa668ed3dd16ca5ac08683` |

T0 imports P1 as its only solver/render owner. It does not duplicate plate or
beam mechanics and does not read a previous generated P1 directory. Clean and
mutated records are rebuilt from the exact owner on each run.

## Clean truth

The library contains exactly nine clean cases, all with expected decision
`Pass` and no reason code:

1. `baseline-plate`;
2. `baseline-beam`;
3. `intervention-youngs-modulus`;
4. `intervention-density`;
5. `intervention-thickness`;
6. `intervention-uniform-scale`;
7. `intervention-impulse`;
8. `intervention-contact`;
9. `intervention-support`.

Every clean record contains the exact P1 fixture/formula/support identity,
modal records, render metrics and float32 WAV bytes. P1 analytic, energy,
remesh and seven isolated-intervention controls are rerun in the owning T0
entry point. Clean audio bytes must equal direct P1 encoding; T0 may not
normalize, filter or otherwise rewrite them.

The intervention expectations remain:

| Axis | Source | Expected relation |
| --- | --- | --- |
| Young's modulus | plate | every modal frequency ratio is exactly `2` within P1 tolerance |
| Density | plate | every modal frequency ratio is `0.5` |
| Thickness | plate | every modal frequency ratio is `2` |
| Uniform scale | plate | every modal frequency ratio is `0.5` |
| Impulse | plate | frequencies and gains are unchanged; samples are binary64-exact `x2` |
| Contact | plate | frequencies unchanged; target `(m=2,n=1)` participation and gain are exact zero |
| Support | beam | first-frequency ratio is `2.8070425317861318020883192852828806909956755557646` |

## Mutation equations

All mutations derive from P1 `baseline-plate` unless the table declares a
target. Let `N=144000`, `fs=48000`, `t[n]=n/fs`, `s=0.01`, `J` be the source
normal impulse, and for mode `i` let `f_i`, `d_i` and `g_i` be the exact P1
frequency, decay and signed gain.

```text
carrier[n] = sum_i g_i sin(2 pi f_i t[n])
clean[n]   = s J sum_i g_i exp(-d_i t[n]) sin(2 pi f_i t[n])
```

The baseline plate has one common decay rate. Operations that use one scalar
envelope must assert exact equality of all source decay values before writing.

| Mutation ID | Exact operation | Preserved invariant | Violated invariant | Expected outcome |
| --- | --- | --- | --- | --- |
| `mutation-wrong-decay` | Replace every `d_i` by `0.25*d_i` and re-render all modes. | Frequencies, gains, contact, impulse and sample count. | Declared physical decay. | `Reject(PhysicalDecayMismatch)` |
| `mutation-frozen-carrier` | Compute the undamped carrier for samples `0..255`; use `carrier[n mod 256] * exp(-d_0*t[n]) * s * J`. | Correct scalar decay envelope, peak scale family and lineage. | Continuous modal phase evolution. | `Reject(TemporalEvolutionFrozen)` |
| `mutation-shuffled-envelope` | Split the clean scalar envelope into twelve 12,000-sample blocks and assign output block `b` from input block `p[b]`, where `p=[0,5,10,3,8,1,6,11,4,9,2,7]`; multiply it by the time-aligned undamped carrier. | Carrier, modal spectrum, frame count and envelope values as a multiset. | Causal envelope order. | `Reject(EnvelopeOrderMismatch)` |
| `mutation-mode-collapse` | Select maximum `abs(signed_gain)`, breaking ties by lowest ordinal; render only that mode. | Selected mode frequency, decay, signed gain, contact, impulse and scale. | Required modal coverage. | `Reject(ModalCoverageCollapsed)` |
| `mutation-spectral-copy` | Use the exact clean `baseline-plate` WAV bytes as a candidate for target `baseline-beam`; record both honest source and target identities. | Payload integrity and honest copied-source lineage. | Target identity and non-retrieval requirement. | `Reject(RetrievalCopyDetected)` |
| `mutation-clipping` | Multiply clean plate samples by `1.25/max(abs(clean))`, then hard-clip to `[-1,1]` and encode float32 WAV. | Timing, relative preclip waveform and frame count. | Unclipped PCM integrity. | `Reject(PcmClipping)` |
| `mutation-provenance-mismatch` | Copy the exact clean plate WAV, but declare its parent audio SHA-256 as 64 zeroes while the release records the actual payload hash. | Audio bytes and actual hash. | Declared parent lineage. | `Reject(ProvenanceMismatch)` |

No mutation uses randomness, optimizer values, a learned feature, real audio
or a threshold selected from its output. The profile order and reason code are
part of the frozen truth.

## Record and publication contract

Each of the 16 records publishes exactly:

```text
records/<record-id>/audio.wav
records/<record-id>/record.json
```

`record.json` binds record kind, expected decision/reasons, source and target
case, parent identities, exact audio hash, operation parameters, modal data
when applicable and operation-specific invariant checks. The top level adds:

```text
truth-release.json
evidence.json
report.json
```

Therefore a complete publication has 35 files. Artifact paths are sorted and
unique; every size and SHA-256 is verified before atomic rename. Canonical JSON
is UTF-8, key-sorted, two-space-indented and LF-terminated. WAV is mono
little-endian IEEE float32, `48 kHz`, exactly `144000` frames. Clean P1 WAVs
remain strictly inside `(-1,1)`; only the clipping mutation may contain exact
`-1` or `1`, and it must contain at least one clipped sample.

The `TruthRelease` has exactly nine `Pass` expectations and seven `Reject`
expectations. It is validator input truth, not the output of a validator. V0
must later reproduce these decisions independently.

## Resources and atomic failure

One run is bounded to 16 records, 35 files, 256 MiB output, 1 GiB peak RSS and
300 seconds wall time. It is single-threaded and CPU-only. Network, real or
protected signals, pre-existing generated waveforms, checkpoints and model
parameters are forbidden.

Profile, parent, invariant, artifact, output-path or resource failure returns
`ContractReject`. The runner accepts only a fresh external output path, removes
staging on every failure and publishes no partial directory. Product behavior
continues to be the authored clip.

## T0 gates

T0 passes only when:

1. the complete CLI runs twice into fresh external paths and every file and
   stdout byte matches;
2. all three parent hashes and the canonical T0 profile match exactly;
3. all nine clean WAVs equal direct P1 renders and retain P1 controls;
4. all seven operations pass their exact preserved/violated invariant checks;
5. the release contains exactly `9 Pass / 7 Reject` expected outcomes and the
   frozen reason matrix;
6. strict profile, duplicate-key, non-finite, dependency-drift, unsafe-output,
   artifact-corruption and atomic-failure tests reject before publication;
7. signal/model/network access counters remain zero and resource bounds pass.

A pass authorizes only the V32 V0 validator-mechanics protocol. It grants no
real threshold, validator release, material-quality, ML, admission, cooker,
demo, runtime or ProductCheck credit.

## Stop rules

- Do not listen to or tune the synthetic values after freeze.
- Do not add real audio, material labels or encoder features to make a mutation
  easier to detect.
- Do not let T0 itself implement the V0 decision algorithm; it publishes
  expected truth and operation evidence only.
- Do not weaken or replace a mutation after V0 sees it. A semantic defect needs
  a new protocol/profile revision before V0 values.
- Generated records and WAVs stay outside Git.
