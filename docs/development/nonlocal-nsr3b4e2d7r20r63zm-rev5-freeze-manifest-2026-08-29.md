# R63ZM revision-5 freeze manifest

Status: `FROZEN / FORMAL_REREVIEW_PENDING / NO_IMPLEMENTATION_AUTHORITY`.

Freeze date: `2026-08-29` (`Europe/Moscow`). This is the single batched repair
and re-review candidate allowed after the revision-4 formal `NO-GO`. Any change
to a listed source, contract, input, binary or evidence artifact voids this
freeze. The rejected revision-4 identities remain in
`R63ZM-FREEZE-MANIFEST.md` and are not part of this candidate.

## Research lineage

```text
worktree  /home/kaifaty/Documents/NextEngine-nonlocal-continuum-n0
branch    codex/nonlocal-nonlinear-r0
snapshot  fc8bcc53eb888e708be94ba0a08097b5169f87f2
parent    9a5a01db558821c5f73d20a8281d752f259e08f1
tree      7da4ff59ff8ca058ad3cda4080572ba1696471a9
parent-to-snapshot binary diff SHA-256
          863ded9a3a8f5d3cce7f3b1fdd272ab7ad388dc9372def0d45782ee974068e24
```

The research worktree is clean. The package is staged in the ignored directory
`/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zm-recovery` because the
active sandbox cannot write the sibling worktree. This is a review freeze, not
repository integration.

## Revision-4 NO-GO repaired as one batch

The formal initial review found three load-bearing classes:

1. skipped certificate/profile booleans were not validated and an invalid
   cache was mislocalized to admission predicate 27;
2. producer-model admission remained route authority on rejection paths, while
   model reconstruction and unused-slot scans were absent from checker work;
3. compile-time layout assertions did not cover every header, product and audit
   offset.

Revision 5 validates every skipped boolean, collection/string/vector bound,
uses a separately parsed 27-gate independent admission result as route and
first-failure authority on every readable-cache path, requires the producer
model to agree, seals six model/unused-scan counters, adds seven real typed
controls and asserts the complete layouts. No arithmetic schedule, product
root or product-set identity was changed.

## Frozen source and contract set

| File | SHA-256 |
|---|---|
| `fixed_sha256.hpp` | `f091d0c608642175f69cba4282d3c5b22e177ae9ca3d33d382e85f5b33cb3432` |
| `raw_semantic_probe.cpp` | `00e5f398b9b9ece9185d7a06f49741222d039b3d05bd0332647c968e0a7024b8` |
| `fixed_product_probe.cpp` | `0f13e7948da35a08f434610f3924c46caffd9dc339ccaf51daea24adefdfe4b9` |
| `fixed_binary_producer.cpp` | `d930d948f1f01c231d6b2823758ff9d0c7fbba71e5a20b73373c95f8acc99501` |
| `fixed_binary_checker.cpp` | `cd437eba00de6fb3c9a3f2ad5ec1cb182ba77bc02e3190b50a33e06cdfa1365e` |
| `allocation_probe.cpp` | `3b9fea28c004c5756ec75cbc20723e70e58f9c490eec21e3a479566f10074e6a` |
| `artifact_controls.py` | `a2648c1110cf843cb3cc574488579f26da094108a826939b8054bef69edc7f3e` |
| `R63ZM-CONTRACT-DRAFT-v2.md` revision 5 | `e7e92a8e40699b31215b6c5571263940f73621433186411e9b606e2ba0768770` |
| `RECOVERY.md` | `b75c95bbc0fb6ffb0d1fcef7ee5844c2a1a6ecd08b57c0454f7fa2307c864212` |

The historical contract filename is retained to avoid a path-only mutation;
its content identifies frozen revision 5.

## Frozen input

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `artifacts/parent-cache.bin` | `1033625` | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |

Its selected root is
`dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5`.

## Toolchain and build contract

```text
g++ (Ubuntu 15.2.0-16ubuntu1) 15.2.0
Linux 7.0.0-30-generic x86_64
glibc 2.43 (Ubuntu GLIBC 2.43-2ubuntu2.3)

common flags:
-std=c++20 -Wall -Wextra -Wpedantic -Werror
-frounding-math -fno-fast-math -ffp-contract=off

Dev:     common + -O0 -g3
Release: common + -O3 -DNDEBUG
Link:    allocation_probe.cpp -lquadmath
```

Two Release binaries were compiled independently for every executable. The
paired Release hashes are byte-identical.

| Executable | Dev SHA-256 | Release A/B SHA-256 |
|---|---|---|
| raw semantic probe | `fac3d908c911a548101d4ec4d05b12b1ea0009d091c35fd652e2ba5911322d06` | `4b0a4d088c1c2a1f4cd0ec1a10275c305d05306aa2b754e4b87a06f48bfd1894` |
| product probe | `319a09c599a5444129e15a7118fb2e7a83328c1885656c0e723286257cb6c49e` | `20c359ebd15f1e9d492e3ec70c3c06795e9e63fd88ab131bd9866df383fc3095` |
| fixed producer | `3a4e51f856bbf35af9b537bee583ba7a9e1162c4762ffbedf5d29f3a5ee5b8d7` | `42e934fa10aaa3a4720e46863ef7cc1df46fa682c142df5b4eb078ec041292f8` |
| independent checker | `c276965b51c3275c4c3202ab030ac3a6578ea2a952e56519b809376d51ba72e8` | `73296d7358c6c8522149b0c76209397ff968884e9067f65e6423575547c605fe` |

## Baseline evidence

Dev and both Release runs were byte-identical for every row below.

| Evidence | Bytes | SHA-256 |
|---|---:|---|
| raw semantic JSON | `759` | `4bcd13b3a1129042c90b572c04c06a3675586a529ff2719ea80b74c6057f6bd2` |
| six-product JSON | `3536` | `672a81639de00c323d847f65f211dc6b4091c5c46348e9e536a5504578c38710` |
| producer artifact | `12916` | `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87` |
| checker audit v3 | `420` | `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80` |
| controls report v3 | `56676` | `83c42c288e6bab63fab299a1569556536533201ee33543d04c13b899ee11739a` |
| allocation receipt | `50` | `b47b57bc8c62f974e53867460ad37bc9ba94ef856316ea28815c2c7e1c5be55b` |

Producer and checker stdout are empty. Every allocation receipt contains
exactly `allocation_probe_calls=0 allocation_probe_bytes=0`.

The accepted baseline roots are:

```text
product set  6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b
bundle       e1ec9d48e74630ea1698bc7ea4cccd84ecd3fd65e300048f47055e505e72970c
trace        8b6e3d1ebd261767aa6722440b9e61975ef49a00cd272a1b75dfc01364b93ce6
result       7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3
checker      b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4
```

The valid producer read tuple is:

```text
1,1,253,1033625,1,1033625,1,1033625,269,1033625,16,4,170,46,
15,51,87,269,7,560,523872,1,847,0,1,271
```

The accepted 26-field checker tuple is:

```text
253,4,6,1033625,12916,346,303,52,612,164,62,6,192780,192780,
192780,68,2212278,2,0,530,269,1033625,389,108,0,0
```

The last six fields are producer-model take calls/bytes, producer-model typed
operations, producer-model reconstruction operations and unused-slot
checks/bytes. Rejection receipts contain the actual nonzero unused scans; for
the verified missing-file route they are `74` checks and `11890` bytes.

## Verification results

- `PASS` — clean Dev plus two independent Release builds with all warnings as
  errors.
- `PASS` — raw/product/artifact/audit bytes identical across Dev and both
  Release runs.
- `PASS` — controls repeated twice; both reports are byte-identical,
  `56/56` controls pass and all `57/57` audit SHA-256 identities are unique.
- `PASS` — invalid early/profile/late certificate booleans, skipped string
  length, both certificate collection counts and profile-vector length all
  produce admission route 2 with producer first-failure ordinal 1, then receive
  independent checker success route 2.
- `PASS` — every cache control runs the real producer before the checker;
  artifact controls exercise independent reseal and all public reject routes.
- `PASS` — package allocation probes report zero calls/bytes for baseline and
  every applicable control.
- `PASS` — GCC `-fanalyzer` compiled producer and checker with `-Werror` and no
  diagnostics.
- `PASS` — ASan/UBSan candidate, invalid-boolean and missing-file producer and
  checker paths with `ASAN_OPTIONS=detect_leaks=0` and
  `UBSAN_OPTIONS=halt_on_error=1`; candidate artifact/audit equal Release.
  LeakSanitizer is not claimed.
- `PASS` — `nm -C` finds no formula-probe parent validator, parent reader,
  boundary-reference, R63ZL or ladder symbols in the Release producer/checker.
- `PASS` — `readelf`/`ldd` show only system runtime, quadmath, C++ ABI, math,
  GCC support and libc dependencies; no R63ZL/core library is linked.
- `PASS` — producer and checker contain compile-time assertions for
  every artifact header, product-relative and audit offset through the exact
  terminal sizes.

Static analyzer object identities are producer
`61fc5cb777e4612a834e3759fce0d2ca092a8165e5b3b022581fa91d13e2f15d`
and checker
`cd8c2f9285bd28b78a6c7c9c70d284c425909c495cc01dbcce2e66dd99b1cce7`.
Sanitizer binary identities are producer
`3e963b183081c0a55fe6e8cc0e7e43cbb152959cc46dcf5ade4ec731a11f2f06`
and checker
`badb294b70e44d3fa52c76effa8c18b4e6387beb1fa09bb650c85106095dc1b7`.

## Formal re-review protocol

The reviewer must treat only the exact revision-5 manifest-listed set as the
candidate and specifically rerun the revision-4 counterexamples. Author
commentary, task-state/README/roadmap and later synthesis are not evidence. The
re-review is read-only and must independently verify identities,
source-to-binary closure, every typed skipped field, producer/model/independent
parser separation, first-failure and route authority on all paths, complete
model/unused-scan work, full layout assertions, clean builds, baseline replay
and adversarial controls.

A verdict is `GO` only if every revision-4 load-bearing finding is closed and
no new load-bearing semantic omission, invisible package-owned work,
post-seal decision, shared producer/checker authority, route ambiguity or
identity mismatch remains. This is the single allowed formal re-review; a
remaining failure closes R63ZM as `INCONCLUSIVE`.

## Claim ceiling after GO

`GO` would authorize integration of this exact fixed-cache tangent boundary
into the research worktree. It would not authorize recurrence, representation
family generalization, timing/performance, runtime/game integration, Rust,
GPU, cross-target or production claims. Those remain separate future gates
under SPEC-38/ADR-076 Proposed and ADR-081 guardrails.
