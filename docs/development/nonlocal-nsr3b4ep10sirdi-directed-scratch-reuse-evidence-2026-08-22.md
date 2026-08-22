# NSR3-B4EP10SIRDI directed scratch reuse evidence -- 2026-08-22

Status: `PASS / DIRECTED_SCRATCH_REUSE_SELECTED_FOR_RESEARCH`

## Result

The transaction-local directed high-water buffer preserves B4EP10SII exactly
and clears every frozen performance gate. All three candidate reports are
byte-identical at stdout SHA-256 `539f1ec5...e7e7`; semantic result is
`b4f847cb...77e9`, embedded B4EP10SII result remains `f7b1542f...25fb2`,
correspondence remains `1e4bedbb...08a35d`, and all five physics roots match.

The candidate records exactly 685 reuse calls: 226 evaluation and 459 HVP.
It requests 454,936,226 full slots, overwrites the frozen 374,945,086 active
slots and grows once across the transaction to 670,229 slots / 16,085,496
bytes. One release leaves zero live buffers; all failure counters are zero.

## External A/B

One warmup per command preceded serialized `AB`, `BA`, `AB` rounds on CPUs
`0..7`. Every process exits zero and every program stderr file is empty.

| Pair | B4EP10SII wall ns | Scratch reuse wall ns | A/B |
|---:|---:|---:|---:|
| 1 | 5,516,171,413 | 4,295,337,753 | 1.2842229715573197 |
| 2 | 5,485,949,419 | 4,283,128,009 | 1.2808277986258056 |
| 3 | 5,514,278,898 | 4,290,430,589 | 1.2852506953818943 |

Median paired speedup is `1.2842229715573197x`; median wall falls from
5.514278898 s to 4.290430589 s. Candidate range ratio is
`1.0028506605392937`. Median RSS rises only 496 KiB, from 91,944 to 92,440
KiB.

This is also a real work reduction rather than extra-core substitution:
median total CPU falls from 40.28 to 31.58 seconds, ratio
`0.7840119165839126`. All gates pass with substantial margin.

## Build and artifacts

- implementation commit: `f33bf3aa68283a4391d91c74329a8ae9349927d2`;
- executable SHA-256: `3f692644...4a02`, size 4,318,368 bytes;
- Build ID: `218622804aa75cfd995b0b1dadd5a6d066a5126d`;
- `compile_commands.json` SHA-256: `30e9925f...a0bc`;
- raw metrics SHA-256: `4082d934...dfc6`.

Source SHA-256 values are `52b32927...f5c6` for `boundary_reference.cpp`,
`479b9a05...d8a8` for `boundary_reference.hpp` and `d49add40...beb6` for
`formula_reclosure_main.cpp`. Raw A/B artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdi.yCz0tE`.

## Decision

Select directed scratch reuse for the nominal research path and retain
B4EP10SII/B4EP10I as rollbacks. Reprofile the exact candidate before changing
another buffer or arithmetic loop. This host-specific one-macro result is not
broad-corpus, runtime, GPU or production evidence; B4E2 remains blocked.
