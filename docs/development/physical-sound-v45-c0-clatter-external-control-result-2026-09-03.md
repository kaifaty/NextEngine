# Physical sound V45 C0 — Clatter external-control result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / REPEAT_EXACT / EXTERNAL_CONTROL_ONLY` |
| Decision | `ClatterExternalControlAvailable` |
| Pre-access seal | commit `52c1e168` |
| Claim | Reproducible external empirical-modal prior and synthetic comparator only |
| Next | S0 bounded NISR and VibraVerse provenance/sample preflights |

## Frozen before numeric access

The value-free [protocol](physical-sound-v45-c0-clatter-external-control-protocol-2026-09-03.md),
[profile](../../lab/profiles/physical-sound-v45-c0-clatter-external-control.v1.json),
[owner](../../lab/scripts/physical_sound_v45_c0_clatter_external_control_v1.py)
and [tests](../../lab/tests/test_physical_sound_v45_c0_clatter_external_control_v1.py)
were committed as `52c1e168` before any Clatter `.bytes` value was interpreted
as a number.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Protocol | 6,067 | `69433a61b2e668a378e2b4575f07702bcbdd552d03f0c8ed79e927345daf403a` |
| Profile | 4,999 | `070d93614ed163be925c1f8781d04cccab9b4ba66684117e16bfc1b33aa2631a` |
| Owner | 25,481 | `8a56bd6fa84718f66833875a575421683c675f6df940841a479ef39d4a618272` |
| Focused tests | 6,666 | `6793edb9537cd2a3af8ce9f74f273fa1e0a5547ac63efc4457792dd36a42f3af` |

The seal pins Clatter commit
`79cac6cbe3f7c452ba28b56c7da4a0124ad04806`, eight semantic source files and
the 84-file impact-material tree root
`8230a6192f189806b899a9113c08fefaf458e9e7f26322e2fc6fe5434a4c065e`.
It also binds the exact T0 Recipe V3 profile and owner. Execution performs no
network request and keeps the source, decoded values and outputs outside Git.

## Exact execution

Two fresh external runs completed in `1.44 s` and `1.46 s`, with peak RSS
`40,564 KiB` and `39,900 KiB`. `diff -qr` found no difference. Both complete
external output trees have the same path-neutral SHA-256 root:

`90019ff5b18527d0c4332108699c8f86f8c8bbabf13f99ff8556fbe76b762514`.

Each tree contains 84 mono PCM16 `48 kHz` control WAVs plus four canonical JSON
artifacts and totals `8,832,542` file bytes:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access.json` | 546 | `ce6124c1346f1b84f7082bac34eacb71423aa34f730e6291ab7843838dff4fc5` |
| `inventory.json` | 14,788 | `006d1e8409cf2631006a9a27f0be7606c42e1e4098b3624d2bbf639f534ef9ae` |
| `prior.json` | 748,297 | `067cc573445dd851fcdd9ca10371c0ad35807c498bb7228498e5bdda50122d7a` |
| `report.json` | 1,215 | `2a5790d69b1c7cc6bdf64160bd4cbc2966b343e45e7251b1a1f5340136ccc398` |

The 84 filenames and all eight source semantic bindings match the frozen seal.
The decoder reads `13,203` finite numeric values: `4,401` paired rows across
the `cf`, `op` and `rt60` arrays. Array-length distribution is:

| Rows per array | Payload files |
| ---: | ---: |
| 10 | 73 |
| 224 | 2 |
| 275 | 4 |
| 349 | 2 |
| 475 | 3 |

Only source rows `0..9` become Recipe V3 coordinates, as frozen before access.
All 84 targets project without clamp, observe only the five empirical-modal
fields and leave excitation, radiation/residual, OOD and uncertainty masks at
zero. The external renderer generates `4,032,000` PCM samples. Every WAV is
exactly `96,044` bytes and structurally valid.

All six report gates pass:

- source integrity and exact 84-file inventory;
- Recipe V3 projection without silent rewrite;
- canonical seeded control WAV production;
- zero forbidden access;
- zero real parent/project credit;
- exact T0 profile binding.

## Important limitation exposed by C0

The nominal `14 × 6 = 84` size-labelled table contains only `36` unique payload
hashes and exactly `36` unique modal-head value sets. Several size buckets are
byte-identical within a material. The per-material unique counts are:

| Material | Unique modal priors across six buckets |
| --- | ---: |
| cardboard | 3 |
| ceramic | 3 |
| fabric | 4 |
| glass | 2 |
| leather | 1 |
| metal | 4 |
| paper | 1 |
| plastic_hard | 4 |
| plastic_soft_foam | 3 |
| rubber | 3 |
| stone | 1 |
| wood_hard | 2 |
| wood_medium | 3 |
| wood_soft | 2 |

The 84 seeded WAVs are unique because the control seed includes the filename;
that does not create new modal priors or independent evidence. B0/MS must group
by the 36 value-identical components and must not treat nominal size buckets,
random draws or rendered variants as independent teacher power. In particular,
Clatter alone cannot prove continuous geometry scaling.

## Access and authority

The owner hashes `106,632` parameter bytes, decodes exactly 84 parameter files,
and reads no real audio, model, validator, protected or runtime value. Network
requests are zero. No Clatter source file, payload, decoded recipe or WAV is
stored in the repository.

C0 adds zero real-acoustic parents, zero project independence and no material-
truth, training, validator, protected, admission, cooker, demo or runtime
authority. The published license is retained as an external-control constraint;
no Next Engine distribution or runtime dependency is created.

## Verification

- bundled Python `py_compile`: `PASS`;
- pre-access artificial focused tests: `PASS`, nine tests;
- combined T0+C0 focused tests: `PASS`, 18 tests;
- two complete external owner runs: `PASS`;
- `diff -qr` and path-neutral tree-root comparison: `PASS`;
- source/profile/T0 hash closure and all C0 gates: `PASS`;
- `git diff --check` and direct changed-link/path validation: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
  C0 does not modify/import that path, keeps source/output roots external and
  receives no boundary-scan credit.

## Consequence

C0 is now a frozen baseline for the later lane-aware B0 tournament. S0 may open
only its separately preregistered bounded NISR and VibraVerse samples next.
C0 cannot select a material pack, train the real generator or calibrate its own
validator. Real support remains `71/105`, deficit `34`; PSEL, real fitting and
protected admission remain blocked, and authored clips remain the only current
production path.
