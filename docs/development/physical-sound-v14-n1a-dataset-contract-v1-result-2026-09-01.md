# Physical Sound V14-N1a — Dataset Contract V1 result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / ZERO_SOURCE_SIGNAL` |
| Decision | `N1A_DATASET_CONTRACT_V1_FIXTURE_PASS` |
| Protocol | [N1a protocol](physical-sound-v14-n1a-dataset-contract-v1-protocol-2026-09-01.md) |
| Roadmap | [V14 N1](../plans/physical-sound-synthesis-roadmap-v14.md) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed`. |

## Outcome

Dataset Contract V1 now freezes the multi-object dataset boundary before real
source inspection. It distinguishes `training_usable`, `evaluation_complete`
and `source_ood`; binds an exact Exposure Ledger V0; enforces physical-object
and recording-parent role disjointness; and prevents prior generator or
unknown exposure from becoming protected evidence.

The implementation is:

- `lab/scripts/physical_sound_dataset_contract_v1.py`;
- `lab/tests/test_physical_sound_dataset_contract_v1.py`.

The schema remains current-only experimental JSON. It is not a public content
contract, runtime model, validator release or admission record.

## Repeat-exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v14-n1a.PpQMVi
```

`run-a` and `run-b` are byte-identical for every emitted file. Run-A hashes:

| Artifact | SHA-256 |
| --- | --- |
| `contract.json` | `3093411beb53cccb362f1ee1d1fb922e9f312446b4cbb2f710107640e48f4278` |
| `descriptor.json` | `493f8bcde32190c046400ed65fbc49f0bc9fbdb017ce632c780bd55ecd194073` |
| `report.json` | `c4faa63ab046785ec7b53790cc0b803d08aa1069eca0a73359345b38ac1a88ea` |
| bound ledger | `2b2cc231c194c4d03ddfe9d5406159e28e104ab32e3b1384c59af245c603326f` |
| bound ledger role root | `c25417e4552b693839f40b0e9179c0bd0006a1c8f8d07a2648fa32dd9e03353f` |

## Frozen fixture shape

The positive fixture contains exactly three source-revision clusters and 24
distinct physical object/recording-parent groups:

| Material | Train | Generator dev | Validator calibration | Method holdout | Admission shadow |
| --- | ---: | ---: | ---: | ---: | ---: |
| Glass | 4 | 1 | 1 | 1 | 1 |
| Metal | 4 | 1 | 1 | 1 | 1 |
| Wood | 4 | 1 | 1 | 1 | 1 |

Three train objects are deliberately `training_usable` with absent
`support_condition`; they receive no evaluation role. Every protected object
is `evaluation_complete`, T2/T3-capable, unexposed and has a recording parent.

Known cross-source object aliases are queried together against the exposure
ledger, so a previously opened object cannot become protected evidence through
a second adapter. Source revision is reported as a statistical cluster.
Distinct physical objects inside one monolithic publisher revision may occupy
different roles;
the contract still forbids the same physical object or recording parent from
crossing roles.

## Access accounting

Each run reads only canonical contract and ledger metadata:

| Counter | Value |
| --- | ---: |
| Contract bytes read | `109,047` |
| Ledger bytes read | `2,781` |
| Metadata scalar values parsed | `1,773` |
| Network requests | `0` |
| Source artifact bytes read | `0` |
| PCM sample values decoded | `0` |
| Force sample values decoded | `0` |
| Protected signal values decoded | `0` |

## Focused verification

`python3.11 -m unittest lab.tests.test_physical_sound_dataset_contract_v1`
passes 12 test methods. The suite includes one valid non-counting source-OOD
branch and 30 fail-closed mutation cases for:

- ledger hash/schema/role-root drift;
- unknown contract vocabulary and changed derived group hashes;
- duplicate objects, cross-source alias exposure, and physical-object/
  recording-parent cross-role leakage;
- incomplete, training-only or synthetic protected evidence;
- prior exposure promotion/recycling;
- source OOD, failed structural evidence and nonzero signal accounting;
- per-material role undercoverage;
- noncanonical, duplicate-key, oversized and in-repository inputs/outputs.

The affected regression set—N1a plus Research Record V0, Exposure Ledger V0
and historical census—passes `43/43`; Python bytecode compilation also passes.
The mapped `cargo run --locked -q -p xtask -- boundary-scan` remains `FAILED`
only on the pre-existing tracked
`SOURCE_LAYOUT_ESCAPE_HATCH: tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
N1a does not touch that file and no broad boundary-scan pass is claimed.

## Decision

N1a is complete and opens only N1b metadata-only source inventory. N1b may
populate candidate source/object rows and acquisition costs, but it may not
decode real PCM or numeric force, freeze final roles, train a model, calibrate
the validator or change runtime/public authority.
