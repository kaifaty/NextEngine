# Physical sound V44 C1 runtime descriptor contract result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / CONTRACT_FROZEN / ZERO_DESCRIPTOR_COMPLETE / G0_AUTHORIZED / NO_AUDIO_FEATURE_VALIDATOR_MODEL_PROTECTED_OR_NETWORK_ACCESS`

## Outcome

C1 freezes the experimental, runtime-available input plane that future physical
sound controls and bounded neural candidates may use. It deliberately does not
train a model, qualify a validator, select a material pack, open protected data,
or authorize a runtime consumer.

The contract has nine ordered fields:

1. `material_label`;
2. `shape_topology`;
3. `extent_micrometres`;
4. `characteristic_wall_thickness_micrometres`;
5. `cavity_kind`;
6. `opening_kind`;
7. `mass_milligrams`;
8. `support_condition`;
9. `impact_zone`.

Dimensions use integer micrometres and mass uses integer milligrams. Every
field carries an observed mask, an explicit missing reason and, when published,
an integer confidence in parts per million. The future model input is only the
ordered values, observed masks and confidence masks/values. Project, publisher,
source, URL, filename, audio, waveform and capture identities remain provenance
only and cannot enter the predictor.

Unknown values are never inferred from audio, dataset names, filenames or
project identity. Conflicting sources produce the masked reason
`source_conflict`; source ordering cannot choose a winner. C0R material labels
cannot be overridden, and the quarantined object-80 material conflict remains
masked.

## Frozen implementation

- contract:
  [`physical-sound-v44-c1-runtime-descriptor-contract.v1.json`](../../lab/profiles/physical-sound-v44-c1-runtime-descriptor-contract.v1.json),
  `6,404` bytes, SHA-256
  `a8bdf313204b89c55f3fbc1a9b2f174ced48a3a337274e8d79c27c14066b9ed1`;
- intake profile:
  [`physical-sound-v44-c1-runtime-descriptor-intake.v1.json`](../../lab/profiles/physical-sound-v44-c1-runtime-descriptor-intake.v1.json);
- owner:
  [`physical_sound_v44_c1_runtime_descriptor_intake_v1.py`](../../lab/scripts/physical_sound_v44_c1_runtime_descriptor_intake_v1.py),
  `56,114` bytes, SHA-256
  `8734186d031766f98ccb93b223c6a644450d90a0c38cc8c93fbd8efe44b88348`;
- focused tests:
  [`test_physical_sound_v44_c1_runtime_descriptor_intake_v1.py`](../../lab/tests/test_physical_sound_v44_c1_runtime_descriptor_intake_v1.py).

The checked-in profile binds SPEC-45, the corrected C0R and B0R/R0R results,
the C0R identity, every C0R metadata input, the contract and owner. Its official
`source_inputs` list is empty. G0 must publish a successor profile that binds
each internet metadata source manifest and artifact by path, byte count and
SHA-256 before that source can add a descriptor.

The owner reads only the five hash-bound C0R metadata/projection files and
optional C1 metadata-source manifests/artifacts. It has no audio, acoustic
feature, validator projection, candidate model, protected-payload or network
reader. Publication is an atomic external directory; an input mutation,
lineage/scope violation, source-artifact mutation, path escape, output collision
or forbidden repository destination publishes nothing.

## Corrected-corpus coverage

The official C1 run intentionally adds no unverified metadata source:

| Measure | Result |
| --- | ---: |
| Physical parents | `64` |
| Records | `134` |
| Material-observed parents / records | `63 / 129` |
| Material-conflict records | `5` |
| Shape, extent, wall, cavity, opening or mass observed | `0` |
| Support or impact-zone observed | `0` |
| Descriptor-complete parents / records | `0 / 0` |
| Source inputs / artifacts / observations | `0 / 0 / 0` |

This is the expected honest terminal for C1: the schema and intake gate are
ready, but the existing corpus does not publish the runtime geometry and
condition metadata needed for descriptor learning. G0 is therefore source
growth, not model tuning.

Priority-pack parent counts remain:

| Material | Train parents | Development parents | Total parents |
| --- | ---: | ---: | ---: |
| Glass | `14` | `3` | `17` |
| Steel | `0` | `3` | `3` |
| Wood | `7` | `3` | `10` |

These counts do not select the first pack because descriptor completeness and
independent validator/source power are still absent.

## Repeat-exact evidence

Runs A and B were published outside the repository under
`physical-sound-v44-c1-runtime-descriptor-intake-2026-09-03/release/` and are
byte-identical across all eight artifacts. Run A contains `502,112` bytes; its
canonical file-inventory root is
`d52e2c91555d6917733d19ff24cb3867a34e5125778259ebb83f0af76640b41b`.

Key artifact hashes:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `descriptor-records.json` | `445,269` | `19712611369192f76656fd6c7504bb14e0747edef5aecea583ec2269986573ff` |
| `provenance-ledger.json` | `38,703` | `81bc02b3979f8542a7ead81cf061c4dfa55446a0a30dd3ed244b648607b03bb1` |
| `coverage.json` | `3,181` | `cf185605221901c7de3f7aa401a0c08584ff5d4b3f8efbef9be4e25cc5f0f880` |
| `report.json` | `3,449` | `5b5e0c585479a519f29515cb5e47c86b90e47caefa85f2043b57ef492f7596a4` |

Both runs report decision `C1_DESCRIPTOR_CONTRACT_FROZEN_G0_AUTHORIZED`. They
read `185,217` bytes of corrected-corpus metadata and exactly zero PCM,
acoustic-feature, validator, candidate-model and protected bytes, with zero
network requests.

## Verification and authority

- focused C1 suite: `PASS`, `9/9` tests;
- Ruff `0.14.1` format/check: `PASS`;
- Python bytecode compilation: `PASS`;
- official external A/B comparison: `PASS`, all eight artifacts byte-identical;
- combined C0R/B0R-R0R/C1 focused suite: `PASS`, `24/24` tests;
- architecture boundary scan: `ERROR` on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  C1 adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`, because C1 is external research under
  `Proposed` SPEC-45 and adds no public/runtime consumer.

C1 authorizes only V44 G0 descriptor-source growth. B1, PSEL, V0, model
training, protected admission, cooking and runtime use remain blocked by their
documented prerequisites. Authored clips remain the only production path.
