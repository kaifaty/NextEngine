# Physical Sound V13-M1a — Research Record V0 result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `M1A_RESEARCH_RECORD_V0_FIXTURE_PASS` |
| Protocol | [Research Record V0 protocol](physical-sound-v13-m1a-research-record-v0-protocol-2026-08-31.md) |
| Scope | Current-version schema/lifecycle, synthetic fixtures and zero-signal repeat |
| Product effect | None; authored clips remain authoritative |

## Outcome

M1a now has one strict external research-record implementation. It distinguishes
`canonical_impact_field` from `measured_transfer_field`, requires explicit
measurement axes, binds every successor to the canonical parent hash and keeps
an exact authored fallback through every state. The implementation rejects
unknown fields, unknown schemas, noncanonical JSON, malformed hashes, lifecycle
skips, source/claim drift, terminal reopening and admission without independent
validator/cooker evidence.

This is research bookkeeping, not a sound model. It decoded no source signal,
trained no weights, rendered no audio and created no public or runtime contract.

## Frozen implementation

| Artifact | SHA-256 |
| --- | --- |
| `lab/scripts/physical_sound_research_record_v0.py` | `57b406268e9bb0564855614567a6535aca9e026172c6f36cf7732961c2a95bb4` |
| `lab/tests/test_physical_sound_research_record_v0.py` | `905dfdf8b566b465a0042d3b4fb7a0b9d4bc7cffbf264198ce13203267958041` |
| Protocol | `eb6e7ac7ac2756f9a547b9481ff00cdab366def3ed314bf73396a5f7449c72df` |

The CLI supports:

```text
--stage fixture  -> deterministic synthetic branch set
--stage validate -> strict current-V0 roundtrip, with parent validation for successors
```

There is intentionally no migration path. Unknown record versions fail without
rewrite, consistent with ADR-046 for an unconsumed experimental format.

## Repeat-exact evidence

Two independent builds were written outside the repository under:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m1a.LcA42o/
```

`fixture-a` and `fixture-b` are byte-identical. Exact fixture-A hashes are:

| File | SHA-256 |
| --- | --- |
| `schema.json` | `98af3ee30f73e8eaaa644a90a0739858c79894c23015f162d7eefa4b59258d68` |
| `source-qualified.json` | `bb543ca2bcca37b50cab07756bcd778132d2769b4786d80b3e39ec50671ff646` |
| `formula-validated.json` | `eaa73816f2367641b23eb595b36f1fd23e7b9c4ecd5011c8373d5c82a5d43969` |
| `atlas-admitted.json` | `5c160195c64c506b5d0d3c90436ad1d0b66b0ec9b537edb52a5443b625b95d12` |
| `fallback-only.json` | `f0ae2a4e1b9c8fea486f77ce8967b23164109d86ef454026a451c96ba6292f2f` |
| `fallback-out-of-domain.json` | `ad1d0045c0017877293388b93b20272bf56138d2af0e7e86fd429a26a11b6de1` |
| `report.json` | `b0c55be7ea3f5afa472c2c7cf49d7c4adfbc5fb40eb75a3b39fffd1d505d8ec0` |

The FormulaValidated CLI roundtrip retained exact hash
`eaa73816…43969`. Both fixture reports state:

```text
network_requests = 0
source_bytes_read = 0
signal_values_decoded = 0
protected_signal_values_decoded = 0
public_contract = false
runtime_consumer_allowed = false
```

## Verification

- `python3.11 -m unittest lab/tests/test_physical_sound_research_record_v0.py`:
  `PASS`, 12 tests;
- Python byte compilation for runner and tests: `PASS`;
- repeated fixture directory diff: `PASS`, no differing bytes;
- successor CLI current-V0 roundtrip: `PASS`, identical record hash;
- `cargo run -p xtask -- boundary-scan`: `FAILED` only on the known pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`,
  introduced by `a26f070f`; the new M1a files add no boundary finding.

## Decision and next action

M1a is complete and opens only M1b: build the deterministic prior-exposure
ledger over existing manifests and reports. M1b must remain zero-signal and
must fail on role leakage, identity collision, changed hashes or unaccounted
access. M2 source selection and every model/audio step remain blocked.
