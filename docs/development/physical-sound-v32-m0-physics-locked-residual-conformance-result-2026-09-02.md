# Physical sound V32 M0 — physics-locked residual conformance result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT_PASS / OFFICIAL_VALUES_UNOPENED` |
| Protocol | [M0 physics-locked residual protocol](physical-sound-v32-m0-physics-locked-residual-protocol-2026-09-02.md) |
| Profile | [`physical-sound-v32-m0-physics-locked-residual.v1.json`](../../lab/profiles/physical-sound-v32-m0-physics-locked-residual.v1.json) |
| Owner | [`physical_sound_v32_m0_physics_locked_residual_v1.py`](../../lab/scripts/physical_sound_v32_m0_physics_locked_residual_v1.py) |
| Product effect | None; M1 sequencing only, authored clips authoritative |

## Result

The value-independent M0 owner conformance passes twice byte-identically.
It constructs one non-official plate fixture, runs exactly three disposable
training steps and verifies the frozen topology and composition without
materializing train development metrics or any method-holdout row.

Decision: `M0_OWNER_CONFORMANCE_PASS`.

The implementation proves:

- exact branch input widths `11 / 13 / 12`;
- exact trainable parameter count `1,491`;
- three correction outputs remain inside their `0.25 / 0.20 / 0.25` bounds;
- all ten P1 frequencies pass through exactly;
- positive multiplicative corrections preserve contact nodes and gain signs;
- the three-step smoke completes and its values are discarded;
- development, method holdout, real/protected signal and network access are all zero.

No official candidate metric, candidate freeze, checkpoint or M1 decision was
opened. The smoke loss decreased from `0.1502856595688021` to
`0.10008192052139474`, but that value is only a finite-gradient conformance
probe and cannot select a model or threshold.

## Exact evidence

Both external runs publish the same three files, `5,961` bytes and identical
stdout.

| Artifact | SHA-256 |
| --- | --- |
| Profile | `dd77fa3ee416061777775ddc00cf3eb4b5fc32d03574a569ca06935d4e23973d` |
| Owner | `2965a3aca9d3da9774bb9f51b0180276b073e60fb1df3e667d32e072128f0e51` |
| `conformance.json` | `a5b330a35d0e3237300e48a8131071bb784504326aa1530e733e3d225727af13` |
| `evidence.json` | `96f8f917e0f8b16a7a69a24da3cb83d78527e17fc5a0db9e985f4ca64a7faea4` |
| `report.json` / stdout | `61374d051dede59a4e6fadcbdfde704c772b839cf365e40ea38daa7364694a52` |

The runs complete in `3.04 / 3.01 s` and peak at `858,960 / 860,264 KiB` RSS,
inside the frozen `300 s / 1 GiB` envelope. The memory margin is bounded but
only about `164 MiB`; M1 must retain the same one-process CPU limit and report
a resource reject rather than increasing it after values open.

## Verification

| Check | Result |
| --- | --- |
| M0 focused suite | `PASS`, `4/4` |
| V24 T0 + V31 P0/P1 + V32 T0/V0/V0a/M0 inherited suites | `PASS`, `47/47` |
| Two complete external conformance publications | `PASS`, exact files/stdout |
| `git diff --check` | `PASS` |
| `cargo run -p xtask -- boundary-scan` | `FAIL`, known pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH` in unchanged `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs` |

The boundary diagnostic names no M0 path and still emits JSON `ERROR` despite
process exit zero. It remains a failed mapped check and grants no ProductCheck
or runtime credit.

## Consequence

M0 implementation conformance is closed. The next package implements the
complete M1 corpus/control/training owner and then spends the one-shot synthetic
development/holdout tournament exactly as frozen. No architecture, seed,
width, step count, coefficient, split, bound or gate may change from this
conformance result.
