# R63ZN revision-3 repair formal re-review freeze manifest

Status: `FROZEN / SINGLE_FORMAL_REREVIEW_PENDING / NO_ENDPOINT_AUTHORITY`.

Freeze date: `2026-08-29` (`Europe/Moscow`). This manifest identifies the
single batched repair after the independent revision-2 `NO-GO`. Any change to
a listed source, contract, input, binary or evidence artifact voids the
freeze. The manifest commit itself is not part of the candidate snapshot.

## Research lineage

```text
worktree  /home/kaifaty/Documents/NextEngine/target/nonlocal-r63zn-integration-IOy092/worktree
branch    codex/nonlocal-r63zn-integration
snapshot  321c32490ee3387aec14c762aa8d260934a20f93
parent    46dffbaa182284aa1f20bcb39c316f8b0d5d5425
tree      ab107137c422df6e85e135f1ec2824abdd581cd2
parent-to-snapshot binary diff SHA-256
          5b151e3336ccc08265df253518bb68d7416f6905677614f3d375ec46cd84e87a
```

The snapshot contains the whole repair plus the initial review evidence and
bounded task/roadmap updates. The original failed snapshot remains immutable
at `846dbac60285b36f8a162c2cb311eff69e0d7536`; its freeze manifest SHA-256 is
`bec5a8b8ba61ae7bf00df88a087bedc1d286c094555c6bb3ec779e4433398500`.
The revision-3 portable review package is
`/home/kaifaty/Documents/NextEngine/target/nonlocal-r63zn-rev3-review-freeze`.

## Frozen source and contract set

| File | SHA-256 |
|---|---|
| `crates/continuum-water/tools/nonlocal-feasibility/CMakeLists.txt` | `c4915a55c3d3c5a1ce9e6a5783535d70233f68c36348972846e037e2161da5f0` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zm/fixed_sha256.hpp` | `f091d0c608642175f69cba4282d3c5b22e177ae9ca3d33d382e85f5b33cb3432` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zm/allocation_probe.cpp` | `3b9fea28c004c5756ec75cbc20723e70e58f9c490eec21e3a479566f10074e6a` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/prefix_candidate.cpp` | `67e555503f4cdece227fd5accdf6f31ea5ee4bed6ba156b41c66f7cb5f88f50b` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/prefix_checker.cpp` | `c50a09d1786fbbf1c4ce7516ed9f63147c8d04918e8893179d9945dfca7bf824` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/receipt_mutator.cpp` | `68666c9dab3200a320077800e625b827d53f7f9d5ca250aed375cd30b519fd98` |
| `crates/continuum-water/tools/nonlocal-feasibility/r63zn/run_controls.py` | `036a54d4a2742ff776ddd6cf9b1487eabe1c8206b0521cca8049d50ad706f479` |
| `docs/plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zn-fixed-artifact-initial-recurrence-contract.md` | `e79295e60b48a7459205bd06bdd4acb9ecdd1420c3954107ea70e28041617d5c` |

Only the fixed SHA primitive and allocation probe are inherited
implementation support. Candidate and checker share no parser, solve, Dot2,
semantic-root, work-model, event reconstruction or route-classifier code.

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
CMake CXX_EXTENSIONS OFF for all six R63ZN targets

Dev:     common + -O0 -g3
Release: common + -O3 -DNDEBUG
Link candidate/checker: allocation_probe.cpp -lquadmath
Link mutator:            -lquadmath
```

Two independent clean Release build directories used exact `-std=c++20`
rather than `-std=gnu++20` and produced byte-identical binaries and reports.

| Executable | Release bytes | Dev SHA-256 | clean Release A/B SHA-256 |
|---|---:|---|---|
| prefix candidate | `83952` | `85ea7ce50d7a8cfebf2b929b57af0e2e5a3a23d287325e57f78a1d75e0a7da12` | `583f9558b739afac3bd65141e65df2cf78362970559e2cd004b926ab423fdffa` |
| independent checker | `88040` | `3f16a7ef89a51ad81ce04f0f62bf9b3211fca4376d09969d4742fe43e1cdec98` | `f899fa1f35c3c0dab341384d345ac6154f68db6f577dff90e37de7f75d0b83e0` |
| receipt mutator | `25488` | `eddd2fffef69ae5dddcd4e254352ccfb4d77ca9ad912337edfa281e5e9d51cbb` | `bc2ef0f22337806ddaa8efb611bf93b77540aed123bc457435f822d138a7d125` |

The binaries in the portable review package have exactly the Release hashes
above.

## Frozen evidence

| Evidence | Bytes | SHA-256 |
|---|---:|---|
| accepted candidate receipt v3 | `1368` | `1e42c8345293dc94c28be20c1d57bd2ea1d675397a6f2403a4e19a3fbceba1ac` |
| accepted checker audit v3 | `700` | `4f09b539a4e4803eae7bd415083b47b74b5d46e41d1f0b23a22de4a035e2e4eb` |
| x0-mismatch receipt | `1368` | `52455716c88bdb155e9c43701b8f390ba9fb55e7b0422eb7f8fa4c10b2d14127` |
| x0-mismatch checker audit | `700` | `36005885b5a3f2d1be150c69f4128dcbe6395eb70b3aa9d320e1c46c22bf9f04` |
| nonfinite-prefix receipt | `1368` | `699d123be7ae53c3c1a5d3c4614772110dbfcaddfc109862347218f2f3520134` |
| nonfinite-prefix checker audit | `700` | `2cd6a547d5ad15085e2544f55cc3d811ab31212ac47a7558372913c66b83bf54` |
| Release controls report v2 | `42466` | `24208010a78486977ce08a2a6278a6962520e794153b6afdd5f474c28f9d2def` |
| allocation receipt | `50` | `b47b57bc8c62f974e53867460ad37bc9ba94ef856316ea28815c2c7e1c5be55b` |

The accepted roots are:

```text
trace    834907ed6fe9d768dae9fd7063a2e9f49b0557c54b95f44290bb7d71c44e8fe7
result   b5907ca5f1870100f07d6ca9d4576fc8260c38adff9c053c0eae7a5e3f939d07
checker  b04bbb531d7425291c2f43cefe61ce3a2e99d4479ba8a05b083b9e24f0963774
```

Candidate and checker stdout are empty. Every applicable stderr is exactly
`allocation_probe_calls=0 allocation_probe_bytes=0` plus the newline.

## Repair closure and route boundary

Candidate routes remain the frozen first-specific routes `0..7`; checker
success/failure routes remain `0..14`. Receipt size remains `1368` bytes with
67 candidate work fields. The checker audit grows from `676` to `700` bytes
and from 48 to 51 work fields to own actual POSIX read calls, terminal read
stops and audit-field serialization.

The initial review findings are closed author-side as follows:

1. role-2 bytes, root and comparison occur only after all 102 `x0`
   comparisons succeed;
2. the route-4 mismatch receipt has zero `H*x0` and later roots/events, `205`
   quad decodes, `6` canonical-root calls and zero role-2 comparisons;
3. all R63ZN targets build under exact standard C++20 with extensions off;
4. checker read calls/stops and exactly 71 serialized audit fields are sealed;
5. candidate, checker and mutator assert the complete receipt offset chain;
6. selector `r63zn-prefix-nonfinite-v1` reaches route 5 before any residual
   root/event is published.

## Verification results

- `PASS` — Dev and Release compile under warnings-as-errors, strict FP and
  exact `-std=c++20`.
- `PASS` — two independent clean Release builds reproduce all three binaries
  and the exact complete report.
- `PASS` — two ordinary Release runs pass `118/118` controls with `118/118`
  distinct checker audits and the same report.
- `PASS` — the complete Dev control array equals Release.
- `PASS` — ASan/UBSan with `ASAN_OPTIONS=detect_leaks=0` and
  `UBSAN_OPTIONS=halt_on_error=1` passes `118/118`; LeakSanitizer is not
  claimed.
- `PASS` — direct route-4 early-stop and route-5 nonfinite assertions match
  the frozen evidence above.
- `PASS` — every package allocation probe reports zero calls and bytes.
- `PASS` — `nm -C` finds no forbidden parent/formula/R63ZJ/K/L/M helper;
  `ldd` shows only system runtime, quadmath, C++ ABI, math, GCC and libc.
- `PASS` — a fresh R63ZM checker run reproduces audit
  `fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80`,
  empty stdout and zero allocations.
- `PASS` — `git diff --check` and direct path/hash validation.

## Single formal re-review protocol

The same independent reviewer must first verify this manifest, the frozen
contract, snapshot/tree/parent diff and every source/input/binary/evidence
identity. The full candidate and checker sources, not only the parent diff,
are the review surface. Author research notes, task-state, README, roadmap and
later synthesis are not evidence and must not be consulted before the
verdict. Candidate files remain read-only during review.

The reviewer must independently rebuild and rerun the baseline, complete
control corpus and R63ZM regression, then explicitly retest all five initial
findings:

1. reproduce the old premature role-2 counterexample against the repaired
   first-failure schedule and require every unavailable field/work item zero;
2. verify source-to-binary closure under declared strict C++20;
3. account for every checker read, early stop and audit serialization path;
4. verify the complete compile-time receipt-layout chain in all three tools;
5. reach and independently classify the nonfinite arithmetic control;
6. audit complete candidate/checker independence, semantic roots, ordered
   events, terminal seals and all fully resealed mutations; and
7. confirm unchanged R63ZM artifact/audit acceptance.

This is the one re-review allowed by the research review budget. A verdict is
`GO` only if no load-bearing identity mismatch, shared authority, premature
consumption, unchecked semantic byte, invisible work, ambiguous route,
mutable post-seal decision or passing resealed drift remains. Any remaining
load-bearing finding closes R63ZN `INCONCLUSIVE`; there is no second repair or
re-review.

## Claim ceiling after GO

`GO` would admit only this exact fixed-input initial prefix: independently
solve `x0`, bind it to R63ZM role 2, consume that one `H*x0`, derive `r0`,
solve `z0` and certify `rho0.value-rho0.bound > 0`. It would not admit `p0`,
`H*p0`, `x1`, a certificate, complete recurrence, dynamic builder,
representation family, corpus, timing, runtime/game, Rust, GPU, cross-target
or production authority. SPEC-38/ADR-076 remain `Proposed`, ADR-081
guardrails remain binding and ProductChecks remain `NOT_RUN`.
