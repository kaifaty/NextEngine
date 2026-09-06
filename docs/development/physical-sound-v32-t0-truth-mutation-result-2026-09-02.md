# Physical sound V32 T0 — truth and mutation result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / SYNTHETIC_VALIDATOR_MECHANICS_ONLY` |
| Protocol | [V32 T0 truth and mutation protocol](physical-sound-v32-t0-truth-mutation-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v32-t0-truth-mutations.v1.json`](../../lab/profiles/physical-sound-v32-t0-truth-mutations.v1.json) |
| Owner | [`physical_sound_v32_t0_truth_mutations_v1.py`](../../lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed` |

## Result

The complete T0 CLI ran twice into fresh external paths. Stdout and all 35
published files are byte-identical. The release contains exactly nine clean
`Pass` expectations and seven adversarial `Reject` expectations with the
frozen reason matrix. Every clean WAV equals a direct P1 render, every mutation
passes its operation-specific preserved/violated-invariant checks, and all
real/protected signal, model and network access counters remain zero.

Decision: `T0_TRUTH_MUTATION_LIBRARY_PASS`.

This authorizes only the V32 V0 validator-mechanics protocol. T0 is expected
truth, not a validator result. It grants no real material quality, learned
model, admission, cooker, demo, runtime or ProductCheck credit.

## Frozen identities

| Artifact | SHA-256 |
| --- | --- |
| T0 profile | `44d83804398b2e08a54dbe28e4e3269098b1112785d8377dac34a11d4e5eebe8` |
| T0 owner | `4a0c5a5ea7cece8dfca2c8d346695bc826b9b375d94c2ef5aa1d723760391d6f` |
| Truth release | `6a5ff92000e27b8db325f29a670055147bcc891267087100876866b341a5b1d9` |
| Evidence | `b55bb66e798e199d40e166686f94bfe35f7f09457585d3edd80250e2e5bcd9b3` |
| Report | `60ab3dd66ed6fd1ae0eba0e25da9cc2a703528e8bf1b775ace56c3fd2be927ec` |
| Baseline plate WAV | `3cc882dd9e367c5fd24047d9a711049583db1ee02bb722e3723c26eae60ef25b` |
| Baseline beam WAV | `911f765ea08f02a21e6d847f457d5fe9a32b31b6873b5cf7df58b775e0311e62` |

The two clean baseline hashes equal the P1 identities exactly. T0 rebuilt them
from the frozen owner rather than reading an earlier generated directory.

## Decision matrix

| Record class | Count | Expected decision |
| --- | ---: | --- |
| P1 clean cases | 9 | `Pass` with no reason |
| Frozen mutations | 7 | `Reject` with one exact reason |
| Out-of-domain records | 0 | none in T0 |

The mutation results are:

| Mutation | Audio SHA-256 | Frozen expected reason | Operation evidence |
| --- | --- | --- | --- |
| Wrong decay | `a0051215dbbadb1aab629fdb08e76972c9a1e64e72c59c985259c14973f5069c` | `PhysicalDecayMismatch` | all rates exactly `x0.25`; frequencies/gains/frame count exact |
| Frozen carrier | `232ad1f8528047066ad9a1e57b40c87b78003e105167409d4e64a68cdc01496f` | `TemporalEvolutionFrozen` | 256-sample carrier repetition; scalar decay envelope exact |
| Shuffled envelope | `75493080d644b57cb0f2701fdc3c1e2cab7a140e14bef02bcab7887b362580fb` | `EnvelopeOrderMismatch` | 12×12,000 blocks; bijective non-identity permutation and exact restored envelope |
| Mode collapse | `630072c37e707748102f5bb6435ea00c5c347d0b5819c276aa4a2487fb0df41c` | `ModalCoverageCollapsed` | exactly one retained mode, selected ordinal `1`, parameters exact |
| Spectral copy | `3cc882dd9e367c5fd24047d9a711049583db1ee02bb722e3723c26eae60ef25b` | `RetrievalCopyDetected` | payload equals plate, differs beam, source/target lineage explicit |
| Clipping | `30ea57cc1e71b19eb5e50c6931ff9f4c20d117460a7f143bf7937cdb34231bac` | `PcmClipping` | preclip peak `1.25`, output peak `1.0`, 14 clipped samples |
| Provenance mismatch | `3cc882dd9e367c5fd24047d9a711049583db1ee02bb722e3723c26eae60ef25b` | `ProvenanceMismatch` | payload/actual hash exact; declared parent is 64 zeroes |

Equal audio hashes for the clean plate, spectral-copy mutation and provenance
mutation are intentional. Their target/declared lineage differs, so V0 must
not rely on waveform content alone.

## Clean causal inheritance

T0 reruns the full owning P1 path for all nine clean cases:

| Gate | Result |
| --- | --- |
| Analytic plate/beam controls | `Pass` |
| P1 audio identity | `9/9 exact` |
| Remesh common vertices | `9/9 Pass` |
| Energy envelope | `9/9 Pass` |
| Young's modulus, density, thickness and scale ratios | `4/4 Pass` |
| Impulse linearity, contact node and support ratio | `3/3 Pass` |
| Mutation preserved/violated invariants | `7/7 Pass` |
| Expected decision matrix | `9 Pass / 7 Reject` |
| Real/protected signal, model and network access | `0 / 0 / 0` |

## Publication and resources

Each record publishes `audio.wav` and `record.json`; the release adds
`truth-release.json`, `evidence.json` and `report.json`. The final tree is 35
files and `9,315,757` bytes. The evidence-side indexed artifact count is 33,
its indexed bytes are `9,305,126`, and its canonical artifact root is
`fe3cdd7d7d11b3c5decd7c122f9f3f331ec55f5081a867f01a76e2e18aa4a3bf`.

The two measured complete processes each took `0.54 s`; peak RSS was `71,768`
and `72,168 KiB`. The deterministic live-array bound is `85,540,864` bytes.
All remain below the frozen `300 s`, `1 GiB` RSS and `256 MiB` output limits.

Generated WAVs, records and reports remain in external temporary storage and
are not committed.

## Verification

```text
lab/.venv/bin/python -m py_compile \
  lab/scripts/physical_sound_v32_t0_truth_mutations_v1.py \
  lab/tests/test_physical_sound_v32_t0_truth_mutations_v1.py

lab/.venv/bin/python -m unittest \
  lab.tests.test_physical_sound_v24_t0_teacher \
  lab.tests.test_physical_sound_v31_p0_causal_baseline_v1 \
  lab.tests.test_physical_sound_v31_p1_modal_owner_v1 \
  lab.tests.test_physical_sound_v32_t0_truth_mutations_v1

cargo run -p xtask -- boundary-scan
```

The T0-focused suite passes `9/9`; the inherited V24 + P0 + P1 + T0 suite
passes `32/32`. It covers full-process A/B identity, all clean P1 payloads,
every mutation equation and reason, exact WAV constraints, dependency/profile
corruption, duplicate/non-finite JSON, artifact corruption, unsafe output and
atomic failure cleanup.

The boundary scan still reports the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` at
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
That file is unchanged from the T0 protocol parent, and no T0 path appears in
the diagnostic. The scan therefore remains `Failed(KnownRepositoryBaseline)`;
it is not relabelled as a pass and grants no ProductCheck credit.

## Consequence

T0 closes the synthetic truth owner and unblocks V0 mechanics. The next commit
must freeze an independent validator interface, specialist ownership, cache
identity, abstention and the exact T0 decision matrix before implementing any
validator thresholds. Real encoder/threshold calibration, ML training,
admission, cooking and demo integration remain sealed behind their later gates.
