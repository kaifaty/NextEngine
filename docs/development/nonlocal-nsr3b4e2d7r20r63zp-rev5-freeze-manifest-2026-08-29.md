# R63ZP revision-5 freeze manifest

Status: `FROZEN / FORMAL_REVIEW_PENDING / NO_IMPLEMENTATION_AUTHORITY`.

Freeze date: `2026-08-29` (`Europe/Moscow`). This manifest freezes the first
formal-review candidate for exactly one direct `H*p0` product at the reviewed
R63ZM fixed-cache boundary. Revisions 2 through 4 are rejected pre-evidence and
are not review candidates.

## Source snapshot

```text
worktree  /home/kaifaty/Documents/NextEngine/target/nonlocal-r63zn-integration-IOy092/worktree
branch    codex/nonlocal-r63zn-integration
snapshot  1fe8ee4feed42f135c4cfeef6e1222971d367df7
tree      1098bb647442551020066ed26e170a3671d3df2c
base      38096294
base-to-snapshot binary diff SHA-256
          004c5cc292d92623e3b6b0f2b9842930532bcb8c5e038c7c567a81cc1dfd9e9e
```

The source snapshot is clean. This manifest and later status documentation are
author commentary, not part of the reviewed source identity.

## Frozen source and contract set

| File | SHA-256 |
|---|---|
| `r63zp/package_format.hpp` | `21777b1b560dd88fe31fc3d252e6ea3c151029d25ba7ec9083a39691d58433ef` |
| `r63zp/direct_product_producer.cpp` | `26812fab3c8a48d871e7979bde737b3d4b5463904e528f1b7da620135ba555da` |
| `r63zp/direct_product_checker.cpp` | `70fdde59ea15c9488cdcf52c167167231be66d74d6b3ecb8525ed3de386d9b1d` |
| `r63zp/run_controls.py` | `8b2fdb2f21048f73c3c1a73dda970c4002e4ce3562704f99c2d2f024d05d57c8` |
| nonlocal feasibility `CMakeLists.txt` | `783501a49ec957cb23366eb4db7469725ec7e870048c457b58c9f2ea5857b74d` |
| revision-5 contract | `a0e6fd5523d827bb6a7d855c3f56070e646bf61a44ef0d4d2e5abc56903da402` |

The producer and checker share only `package_format.hpp`, the already reviewed
R63ZM fixed SHA-256 implementation and the process allocation probe. They do
not link the formula-probe core or consume R63ZN/R63ZJ/K/L code or receipts.

## Frozen inputs

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| R63ZM parent cache | `1033625` | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| reviewed R63ZM artifact | `12916` | `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87` |
| reviewed R63ZM audit | `420` | `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80` |

## Toolchain and build contract

```text
g++ (Ubuntu 15.2.0-16ubuntu1) 15.2.0
Linux 7.0.0-30-generic x86_64
glibc 2.43

strict C++20, CXX_EXTENSIONS OFF
-Wall -Wextra -Wpedantic -Werror
-frounding-math -fno-fast-math -ffp-contract=off
Release: -O3 -DNDEBUG
```

Two independent clean CMake Release builds produced byte-identical binaries:

| Executable | Release A/B SHA-256 |
|---|---|
| direct-product producer | `ddb2c7e809752740d277a3a429cb73a6a2f52161c67284a35386e2d1df4575e5` |
| independent checker | `a1b9b250e3a13654d921cc5fc6a1bb1d758ffa2df2e3002ecae38bb0f4059727` |

`nm -C` found no formula-parent validator, boundary-reference, R63ZJ/K/L,
ladder or reclosure symbol. `ldd` showed only quadmath, the C++ ABI, math, GCC
support, libc and the ELF loader.

## Baseline and controls

Both Release builds and the sanitizer build reproduced:

| Evidence | Bytes | SHA-256 |
|---|---:|---|
| candidate revision 5 | `6152` | `82fc67403e86ab234ff985fe5694eb23f19a9d049f44d7cbf7cd14e4d3d8f9f3` |
| checker audit revision 5 | `1168` | `21dffa8867993ab6bdbcee6665a36075bb36a523c220c69444368a901914ca0a` |
| controls report | `24965` | `f7710c482bc613e913796a9f3a09b9426c18cfacc279bcfa2467449b21d4903f` |
| allocation receipt | `50` | `b47b57bc8c62f974e53867460ad37bc9ba94ef856316ea28815c2c7e1c5be55b` |

The baseline producer and checker both return route 7. The checker independently
reproduces exact signed-dyadic product root
`271facfdbbb97c777d1a661cf73f5a1eaa25853d42f9e520cab9c785af272f2f`.

The deterministic corpus passed `174/174` controls twice in Release and once
under ASan/UBSan; all `174/174` negative audit identities are unique. It covers
all 61 candidate-work fields, all 70 checker-work fields, candidate semantic,
event, seal and malformed classes, shortened/trailing candidates, and
same-size/truncated/trailing mutations of every parent input. Fixed early
paths have zero semantic-replay work rather than copied success counts.

ASan/UBSan binaries were producer
`b487a034e5227f8cf940a1ca1665d9ebcba6c8da7cb4dc0b9be7ed47713d17aa`
and checker
`10939fd21207d5418e4711b273ab4b83da4adea7bf243711e99e10e854c85630`.
LeakSanitizer is not claimed because the desktop ptrace environment rejects it;
the successful run used `ASAN_OPTIONS=detect_leaks=0` and
`UBSAN_OPTIONS=halt_on_error=1`.

GCC `-fanalyzer` compiled each package TU directly with `-Werror` and no
diagnostic. Object identities were producer
`763af3e0e56d0e3cf6d56eafc6b14c49dd0cbe3963310876877f4f074b7b2160`
and checker
`50ee8415736893f56712754099d676dc81bc8a3c7898dc48712f9ee483e34919`.
The combined CMake analyzer build is not claimed: GCC reports a known ownership
false positive in the shared global `operator new` allocation probe.

## Parent regressions

- R63ZM artifact controls: exact report
  `83c42c288e6bab63fab299a1569556536533201ee33543d04c13b899ee11739a`,
  artifact `ac6946e8...1f0b87`, audit `fd4bcf00...1f80`.
- R63ZO rounded-update preflight: exact stdout
  `d3cfe83ecf064ce042cae7bbb17a9a8a6f2376f6bc548edab621ce082f2d5ce3`.

## Formal review protocol

The reviewer must use the exact source snapshot and contract above, build in a
fresh detached worktree and treat this manifest only as an identity/request
index. Author research, task-state, README, roadmap and later synthesis are not
proof. The review must independently check:

1. source-to-binary and input closure;
2. producer/checker structural independence and exact dyadic arithmetic;
3. direct-product containment, curvature, step and late update consequences;
4. all actual work, root, event, output and causal-seal paths;
5. malformed/early-stop counts and first-specific route precedence;
6. every mutation class, clean Release repeat and parent regression.

`GO` requires no load-bearing semantic, causal, route, work, seal or identity
finding. One batched author repair and one re-review are the complete remaining
budget after an initial `NO-GO`.

## Claim ceiling

An independent `GO` would admit only this one fixed direct `H*p0`
value-plus-error product at the exact R63ZM identities. It would not authorize
recurrence, certificates, representation-family generalization, a dynamic
builder, corpus extrapolation, timing, runtime/game integration, Rust, GPU or
production. SPEC-38 and ADR-076 remain `Proposed`; ProductChecks remain
`NOT_RUN`.
