# Physical Sound V13-M1b — Exposure Ledger V0 result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `M1B_EXPOSURE_LEDGER_V0_FIXTURE_PASS` |
| Protocol | [Exposure Ledger V0 protocol](physical-sound-v13-m1b-exposure-ledger-v0-protocol-2026-08-31.md) |
| Scope | Declarative catalog, JSON Pointer evidence, aggregation and leakage guards |
| Product effect | None; no source, model, content or runtime promotion |

## Outcome

M1b now has one schema-agnostic exposure-ledger builder. A catalog declares
the exact old manifest/report artifacts and maps publisher-specific fields to
stable sample identities through JSON Pointer plus canonical pointed-value
hash. The builder verifies artifact size/hash/schema, resolves the pointers,
groups payloads by object/contact/listener/mutation parent and emits an
M1a-compatible ledger summary.

The implementation does not infer semantics by recursively searching unknown
old schemas. That would be convenient but unsafe: a copied object ID or metric
array could otherwise become false evidence of sample exposure.

## Frozen implementation

| Artifact | SHA-256 |
| --- | --- |
| `lab/scripts/physical_sound_exposure_ledger_v0.py` | `3c872a9c4b029ae5830671557781021f42c9b1150e4c53c7eb984d8610331294` |
| `lab/tests/test_physical_sound_exposure_ledger_v0.py` | `c4932aae056201d13edc256f13a9979d19a965da76af1a8c058f0ea88d1cb084` |
| Protocol | `01b2afe7404a84a6a33bb925b1ab7bbd185ff98ec65b49f67678932ea7c78056` |

The CLI has separate decisions:

```text
--stage fixture -> M1B_EXPOSURE_LEDGER_V0_FIXTURE_PASS
--stage build   -> EXPOSURE_LEDGER_V0_BUILD_PASS
```

This prevents a future real historical build from being mislabeled as a
synthetic fixture.

## Repeat-exact evidence

Two independent fixture builds and one direct CLI build are external under:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m1b.KGn2ys/
```

`fixture-a` and `fixture-b` are byte-identical. The generic CLI build preserves
the exact catalog, ledger and summary bytes; only its report decision and
`opens_only` field correctly differ from the fixture wrapper.

| File | SHA-256 |
| --- | --- |
| `catalog.json` | `0e2b5ce60ebeb08c8f2c9e8149d8392d5d8e4d8dbf40535ad991fd72a212fc3c` |
| `ledger.json` | `d001269ad7f0f6c8334a855b7aee16aff18c6cb24c922378701e8df17594ccc4` |
| `summary.json` | `e5d092f1b0e7e5a5f8aea0aa6f62ffa3c164ce983032aa952d3f0c7dd3c6a66b` |
| fixture `report.json` | `d7dac5ed7e522da62d3570d16c4712f3a212ed294dfb9a29c3e89099b4fa1613` |

The summary has four synthetic payload identities, role root
`126f9bd6…5187`, `448` declared historical signal values and `192` declared
protected values. These are fixture history, not M1b build access.

Exact build-time access is:

```text
artifact_bytes_hashed = 786
metadata_files_parsed = 1
metadata_scalar_values_parsed = 15
network_requests = 0
source_bytes_read = 0
signal_values_decoded = 0
protected_signal_values_decoded = 0
```

## Guards exercised

The 13 focused tests cover:

- artifact size/hash/schema drift, path escape and symlink rejection;
- missing, changed, duplicate and unresolved evidence;
- identity/counter mismatch against pointed values;
- unknown schema, field, role, access and partition;
- duplicate exposure and invalid access accounting;
- protected fit/development misuse;
- contact-parent fit/validator leakage;
- mutation-parent leakage across different contacts/payloads;
- conservative historical-overlap reporting;
- canonical/size constraints and external output/store confinement;
- byte-identical repeat and M1a summary compatibility.

## Verification

- M1a + M1b focused tests: `PASS`, 25 tests total;
- Python byte compilation for both runners/tests: `PASS`;
- repeated fixture directory diff: `PASS`;
- direct generic CLI build versus fixture catalog/ledger/summary: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAILED` only on the known pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`,
  introduced by `a26f070f`; M1b adds no boundary finding.

## Decision and next action

M1b is complete and opens only M1c. M1c must freeze an exhaustive real catalog
for the current external physical-sound store, use `historical_union`, produce
two byte-identical zero-source-signal builds and derive a conservative fresh
source shortlist. Until that passes, no object is declared unexposed and M2
remains blocked.
