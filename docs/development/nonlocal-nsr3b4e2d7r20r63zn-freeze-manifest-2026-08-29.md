# R63ZN revision-2 author-package freeze manifest

Status: `FROZEN / FORMAL_REVIEW_PENDING / NO_ENDPOINT_AUTHORITY`.

Freeze date: `2026-08-29` (`Europe/Moscow`). This manifest identifies the
first complete author package for the already frozen revision-2 question. Any
change to a listed source, contract, input, binary or evidence artifact voids
the freeze. The manifest commit itself is not part of the candidate source
snapshot.

## Research lineage

```text
worktree  /home/kaifaty/Documents/NextEngine/target/nonlocal-r63zn-integration-IOy092/worktree
branch    codex/nonlocal-r63zn-integration
snapshot  846dbac60285b36f8a162c2cb311eff69e0d7536
parent    d1fbc338df9e136a4a688c48a0789c01efa47b9b
tree      a186fbb3f0622baaa8eee387541475d5bf7ead8d
parent-to-snapshot binary diff SHA-256
          faa4e91905ec50b86f693cd3b1772113584b134268737823cefe7fe22d6612c8
```

The source worktree was clean when the candidate snapshot was frozen. The
portable review artifacts are in
`/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zn-review-freeze`.

## Frozen source and contract set

| File | SHA-256 |
|---|---|
| `crates/continuum-water/tools/nonlocal-feasibility/CMakeLists.txt` | `0773f3fbc1924245d7516b85bbe8f0c83ed525012761d23ece71927cdc685e8b` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zm/fixed_sha256.hpp` | `f091d0c608642175f69cba4282d3c5b22e177ae9ca3d33d382e85f5b33cb3432` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zm/allocation_probe.cpp` | `3b9fea28c004c5756ec75cbc20723e70e58f9c490eec21e3a479566f10074e6a` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/prefix_candidate.cpp` | `e782332da862d0b0347af96b227e1e59bc9209de315ac249c92f3fd924d642c4` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/prefix_checker.cpp` | `10ddc3b0ef4414d33d40a8da6340d512d5a0872a689b770d99cabb82b40a62cb` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/receipt_mutator.cpp` | `4259ade66c0984ff08869e41c34302632eeac0fa9ce70988ae97376ebc46f707` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/run_controls.py` | `592075606615df76d5c28b89206c5158fabce74f38036de9d3e2f3e8c00b5b49` |
| `docs/plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zn-fixed-artifact-initial-recurrence-contract.md` | `16e2758c617975de7a301671d2ff22b61f92dff596b6619a3deaad19c0d90910` |

Only `fixed_sha256.hpp` and the allocation probe are inherited implementation
support. Candidate and checker share no parser, solve, Dot2, semantic root,
work model, event reconstruction or route classifier.

## Frozen inputs

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zm-recovery/artifacts/parent-cache.bin` | `1033625` | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| `/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zm-recovery/artifacts/r63zm-v5-repair-a/artifact.bin` | `12916` | `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87` |
| `/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zm-recovery/artifacts/r63zm-v5-repair-a/audit.bin` | `420` | `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80` |

The consumed R63ZM role-2 value root is
`8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce`.

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
Link candidate/checker: allocation_probe.cpp -lquadmath
Link mutator:            -lquadmath
```

Two independent clean Release build directories produced byte-identical
binaries and byte-identical complete control reports.

| Executable | Bytes | Dev SHA-256 | clean Release A/B SHA-256 |
|---|---:|---|---|
| prefix candidate | `79624` | `725ab44cbff15d730a77d1b022a5edd233911150be30e3aa9b137f633f59fcc5` | `ea497b3306d60a1ff9dab6cf08f59285bb5190f4145581fa61e9fc8815499f9e` |
| independent checker | `88040` | `46a9c387e46cd6bac72c728da913321d214e8257e12d8c8c3f56c4ef36dca1f2` | `e3d5a729593c71c6edb5e9e34e4bba838f65d870133332a7d1bffbdfd7ae355b` |
| receipt mutator | `25488` | `7e9b3759fad60b89776093834864808bb6420c843b1c67f5dca5b9c879be5a41` | `bc2ef0f22337806ddaa8efb611bf93b77540aed123bc457435f822d138a7d125` |

The frozen Release binaries are under the manifest-listed review artifact
directory. Their hashes equal the clean A/B hashes above.

## Baseline evidence

Dev, both in-tree Release runs and both clean Release runs have the same
semantic control records.

| Evidence | Bytes | SHA-256 |
|---|---:|---|
| accepted candidate receipt v3 | `1368` | `e0015763eec6f77854568505da47d5874de389ecd505c91c5537e6b12a9af8da` |
| accepted checker audit v2 | `676` | `d0a98ac66b06847278469bf36f7781dd2eaf886bd157cc8fbbb3cf9e739d2cc3` |
| Release controls report v1 | `41698` | `745eafba951678bb584e79bf569b554468355c17ff564fdd3e553f10bc264a02` |
| allocation receipt | `50` | `b47b57bc8c62f974e53867460ad37bc9ba94ef856316ea28815c2c7e1c5be55b` |

The accepted roots are:

```text
trace    6aa6f881294eb5b68b20e778ed608a4ad4499a12207de7474a6832a68ebad908
result   3303938b7b2dca351324302e1da0ddb43f3059ac7b6ad2238dfde943fec49963
checker  000de639269409d6edc6862bbab167316e030c3480b7492987b345e924d82b32
```

Candidate and checker stdout are empty. Every applicable stderr is exactly
`allocation_probe_calls=0 allocation_probe_bytes=0` plus the newline.

## Route and work boundary

Candidate routes are first-specific and terminal:

| Candidate route | Meaning | Checker success route |
|---:|---|---:|
| `0` | cache read rejected | `1` |
| `1` | parent artifact rejected | `2` |
| `2` | parent audit rejected | `3` |
| `3` | cache semantics rejected | `4` |
| `4` | independently solved `x0` does not match role 2 | `5` |
| `5` | prefix arithmetic nonfinite/inexact/underflow | `6` |
| `6` | exact `rho0` lower bound is not positive | `7` |
| `7` | accepted initial prefix | `0` |

Checker failures `8..14` distinguish checker read, malformed receipt,
arithmetic reconstruction, semantic correspondence, work, events and seal.
The receipt contains 67 candidate work fields. The checker audit contains 48
separate checker fields. Every candidate work field has a fully resealed
single-field negative control.

## Verification results

- `PASS` — Dev and Release targets compile under the frozen warnings-as-errors
  and strict-FP flags.
- `PASS` — two independent clean Release builds produce the exact three
  manifest binaries.
- `PASS` — two ordinary Release runs and both clean builds produce the exact
  same control report.
- `PASS` — `116/116` controls pass and all `116/116` checker audit hashes are
  distinct in each run.
- `PASS` — the Dev control record array equals Release even though top-level
  executable identities differ.
- `PASS` — ASan/UBSan with `ASAN_OPTIONS=detect_leaks=0` and
  `UBSAN_OPTIONS=halt_on_error=1` passes the complete `116/116` corpus.
  LeakSanitizer is not claimed.
- `PASS` — every package allocation probe reports zero calls and bytes.
- `PASS` — `nm -C` finds no parent reader/validator, boundary-reference,
  R63ZL, ladder or R63ZM producer/checker symbol in candidate/checker; `ldd`
  shows only system runtime, quadmath, C++ ABI, math, GCC support and libc.
- `PASS` — a fresh unchanged R63ZM checker run reproduces its exact 420-byte
  audit `fd4bcf00...1f80`, empty stdout and zero allocations.
- `PASS` — `git diff --check` and direct manifest link/path validation.
- `NOT_CLAIMED` — GCC `-fanalyzer` is not evidence for this package: the
  current analyzer reports inherited span/allocation-probe diagnostics. The
  clean compiler, sanitizer, allocation probe, controls and direct source
  audit above remain the claimed checks.

## Formal review protocol

The reviewer must first verify this manifest, frozen contract, candidate
snapshot/tree/diff and every source/input/binary/evidence identity. The full
candidate and checker sources, not only the parent diff, are the review
surface. Author research notes, task-state, README, roadmap and any later
synthesis are not evidence and must not be consulted before the verdict.
Candidate files are read-only during review.

The reviewer must independently rebuild and rerun baseline plus adversarial
controls, then audit:

1. complete selected-cache parsing and typed validation;
2. role-2 input/value binding and the exact initial arithmetic schedule;
3. independent candidate/checker authority and first-failure precedence;
4. all semantic roots, ordered events and terminal seals;
5. complete candidate and checker work ownership, including early stops,
   zero scans, file/hash/root paths, receipt/audit writes and controls;
6. resealed semantic/work/event/route mutations and wrong-selector cases;
7. the independent small solve and binary128 state-difference counterexample;
8. no post-seal decision, hidden allocation or forbidden parent helper; and
9. unchanged R63ZM artifact/audit acceptance.

A verdict is `GO` only if no load-bearing identity mismatch, shared authority,
unchecked semantic byte, invisible package work, ambiguous route, mutable
post-seal decision or passing resealed drift remains.

## Claim ceiling after GO

`GO` would admit only this exact fixed-input initial prefix:
independently solve `x0`, bind it to R63ZM role 2, consume that one `H*x0`,
derive `r0`, solve `z0` and certify `rho0.value-rho0.bound > 0`. It would not
admit `p0`, `H*p0`, `x1`, any certificate, a complete recurrence, a dynamic
builder, representation-family generalization, corpus, timing, runtime/game,
Rust, GPU, cross-target or production authority. SPEC-38/ADR-076 remain
`Proposed`, ADR-081 guardrails remain binding and ProductChecks remain
`NOT_RUN`.
