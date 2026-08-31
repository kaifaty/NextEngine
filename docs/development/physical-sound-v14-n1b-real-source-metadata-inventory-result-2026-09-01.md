# Physical Sound V14-N1b — real-source metadata inventory result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / COVERAGE_INSUFFICIENT` |
| Decision | `N1B_COVERAGE_INSUFFICIENT_FOR_FULL_SHAPE` |
| Protocol | [N1b protocol](physical-sound-v14-n1b-real-source-metadata-inventory-protocol-2026-09-01.md) |
| Roadmap | [V14 N1](../plans/physical-sound-synthesis-roadmap-v14.md) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed`. |

## Outcome

N1b now provides a deterministic, metadata-only inventory for the three
published source families. It binds 1,150 object rows, 70 remote archive
identities, cached ObjectFolder member structure, the N1a policy and the
historical Exposure Ledger without decoding a waveform, NPY header, force
value or protected signal.

The inventory falsifies the original assumption that the currently proven
real pool can supply eight fresh physical groups for every material. The exact
potential is:

| Material | All real metadata candidates | Unexposed real candidates | T1 synthetic teachers | N1a minimum |
| --- | ---: | ---: | ---: | ---: |
| Glass | `6` | `1` | `26` | `8` |
| Wood | `16` | `7` | `622` | `8` |
| Metal | `23` | `18` | `151` | `8` |

Synthetic teachers can start representation work only after N1 is honestly
scoped; they cannot replace T2/T3 protected real evidence. N1b therefore does
not assign roles or authorize N2.

## Repeat-exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v14-n1b-final.R3sjTV
```

`run-a` and `run-b` are byte-identical. Run-A hashes:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `inventory.json` | `1,118,080` | `2861bb0e42e40fe78e67dab7ed5e0897b40c42caf6ab639235948adfa38fcfc0` |
| `costs.json` | `1,468` | `c7710b447525dac744356561c6455fc42c27b058a66e6b724a901d0ebfad03df` |
| `report.json` | `1,641` | `45d81f04fd62854de288ff3a4a5fe47ac499033ff17ea85479034ae2c69f805c` |

The frozen successful HTTP identity snapshot is:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v14-n1b-http-sequential.IVz8AO/http-snapshot.json
bytes  = 29,151
sha256 = 4f4701f69b73845dcb59fa69296cfbd6a720d2473e1ce38d5d48b35eb081c7ea
```

All `70/70` endpoints returned status `200`, exact content length and byte-range
support when acquired sequentially. An earlier eight-worker diagnostic
snapshot received `61` HTTP 503 responses and is rejected from evidence; this
is why the committed acquisition default is one worker.

## Source/member facts

| Fact | Exact observation |
| --- | ---: |
| Source revisions | `4` |
| Object rows | `1,150` (`100` current real + `50` RealImpact + `1,000` synthetic) |
| Remote archive identities | `70` |
| Current/historical proven alias edges | `70` |
| Current/historical identity conflicts | `30`, IDs `71…100` |
| Compact split objects | `44` |
| Compact split object/contact pairs | `1,413` |
| Cached TAR member headers | `6,801` |

The contact-localization README says `53` real objects, while the exact
published split contains only `44` distinct object IDs. N1b records both facts
and does not manufacture the missing nine.

The current and historical ObjectFolder object tables differ for every ID from
`71` through `100`. RealImpact names follow the historical numbering, so an
equal number is not treated as a physical-object alias. No RealImpact row has
enough exact normalized-name evidence to inherit material identity in this
revision; those rows remain `source_ood` pending an independent publisher
mapping.

The only current-table Glass object that is both structurally reachable and
unexposed is `59 / Soap_Dish`; it still needs remote member preflight. Current
Glass `6`, `7`, `22`, `51` and `60` are historically exposed. Rows `82`, `92`
and `93` that looked fresh under the current table are inside the unresolved
revision-shift range and cannot be recycled as protected evidence.

## Acquisition cost ceiling

The successful header snapshot represents these remote bodies without
downloading them:

| Source | Archives | Remote bytes | Conservative full-member preflight |
| --- | ---: | ---: | ---: |
| ObjectFolder Real | `10` | `393,309,336,025` | `393,309,336,025` |
| RealImpact | `50` | `117,358,630,208` | `117,358,630,208` |
| ObjectFolder 2.0 | `10` | `39,160,858,491` | `39,160,858,491` |

These are cost bounds, not a download plan. RealImpact ZIP central directories
can later be ranged per object. ObjectFolder gzip/TAR archives remain
batch-granular unless a publisher-side index or already cached prefix proves
the required member identity. N1c may not spend roughly `393 GB` merely to
search for candidates.

## Access accounting

Each offline evidence run reports:

| Counter | Value |
| --- | ---: |
| Cached archive bytes hashed/traversed | `465,967,354` |
| Total cached bytes hashed | `466,705,568` |
| JSON scalar values decoded | `13,152` |
| Network requests during build | `0` |
| Network archive body bytes | `0` |
| WAV headers parsed | `0` |
| NPY headers parsed | `0` |
| Payload members extracted | `0` |
| PCM sample values decoded | `0` |
| Force sample values decoded | `0` |
| Protected signal values decoded | `0` |

## Focused verification

`python3.11 -m unittest -v
lab.tests.test_physical_sound_source_inventory_v1` passes six test methods.
They cover repeat-exact output, all zero-signal counters, source-revision drift,
numeric-ID alias rejection, unexpected endpoint/redirect/member failures,
duplicate JSON, exact real-input hashes and fresh external outputs.

The affected N1b + Research Record/Exposure Ledger/historical census/N1a set
passes `49/49`; Python bytecode compilation and `git diff --check` pass.
`cargo run --locked -q -p xtask -- content-package` passes with `123` records
and `64` chunks. The mapped boundary scan still fails only on the pre-existing
tracked `SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
N1b does not touch that file and no broad boundary-scan pass is claimed.

The implementation is:

- `lab/scripts/physical_sound_source_inventory_v1.py`;
- `lab/tests/test_physical_sound_source_inventory_v1.py`.

Their final SHA-256 identities are respectively
`abfabec334aadee6ded078a6aada03da4c94602d7860e5e0df1706b838761876`
and `8263c11f6310f664f46aa3862889997459aa80381462c171d331a9c11d647612`.

## Decision and next boundary

N1b is complete as a reproducible negative coverage result. N1c is now a
scope/source decision, not an automatic role-freeze step:

1. resolve the `71…100` revision lineage only through bounded publisher
   metadata or already cached metadata members;
2. search for an additional internet-published T2/T3 source with explicit
   material/object/recording lineage;
3. if eight fresh Glass and Wood groups still cannot be proven, narrow the
   first protected domain before any waveform decode;
4. freeze roles only after the chosen domain meets the unchanged N1a shape.

No per-object threshold, signal audition, full-batch speculative download or
synthetic-for-real substitution is authorized.

## Primary sources

- [ObjectFolder Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
- [Pinned ObjectFolder Real source](https://github.com/objectfolder/objectfolder.github.io/blob/d058ba09e7e7a1a8358d64d7b48e5f588b377eb8/source/_pages/objectfolder-real-download.md)
- [ObjectFolder 2.0 repository](https://github.com/rhgao/ObjectFolder/tree/3c6cd8930b2dcbadb6d94dadf2745c956bdcd236)
- [RealImpact repository](https://github.com/samuel-clarke/RealImpact/tree/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987)
- [Contact-localization benchmark](https://github.com/objectfolder/contact-localization/tree/4bb002f519cab9d250bbbe045a6df0248bf1639f)
