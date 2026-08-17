# Continuum water W1 clean-tree discriminator — 2026-08-17

Status: `NUMERIC_GATE_FAILED / PRODUCT_CHECK_NOT_RUN`.

## Scope and provenance

This report records the first clean-tree numerical discriminator for the W1
serial CPU DFSPH oracle. It is a bounded summary; the raw JSON reports remain
outside Git.

- Implementation commit:
  `74730e208cfeb70b05a3ec44b2bb9c2f5002fe97`.
- Branch: `codex/continuum-water-w1` in the dedicated water worktree.
- Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM `22.1.6`.
- Target and profile: `x86_64-unknown-linux-gnu`, `water-oracle`.
- Exact flags:
  `-Ctarget-cpu=x86-64`,
  `-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma`,
  `-Cllvm-args=-fp-contract=off`.
- Both reports bind `tool_commit` to the implementation commit and record
  `tool_tree_state = CLEAN`.
- W0B document root:
  `d357bca64983fbd2074961a462743a4fb5d3fedc2af09631d32ec178bd299550`.
- Float-profile root:
  `d6152c575fd88bb53d0d63d0e1e8b2e86a82465268fc8d102ebdb1d77c092d63`.
- Corpus root:
  `cb091e0f3a04f2c052b614aeab72034bddd3c0b32430fba85854ede2d183aa91`.
- Execution-profile root:
  `bc594e7e78f9662c1c0573221799489cb618c66a65e9bc369b4b17e3bfbe43c8`.

The exact invocation shape was:

```text
CARGO_ENCODED_RUSTFLAGS=<the three exact flags above> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water oracle --scenario <ID> \
  --output <absolute-path-outside-Git> --repeat 2 --storage-order reverse
```

## Repository verification

The implementation content passed `cargo run --locked -p xtask -- host-check`,
including workspace formatting, Clippy, tests and `boundary-scan`. Focused
results included 29/29 `next_continuum_water` tests and 99/99 plus 44/44
`xtask` tests.

## Numerical results

| Scenario | Scenario-local result | First causal observation | Bounded artifact identity |
| --- | --- | --- | --- |
| `CW-FREEFALL-001` | `SCENARIO_PASS` | All 97 frames match the exact canonical gravity recurrence. Both repeats produce trajectory root `e9ab0e40aff19f919802c7bd59d0e31f623197ebdf46b570f3545f4334f374f6`; final energy residual is `5,450,667 ppb` and momentum residual is `0 ppb`. | Raw report SHA-256 `df7ee0eea3b76aa8caa46ade0993a302352e73a3acfc9dbfc23787aaaac81681`. |
| `CW-HYDRO-001` | `FAILED` | The first density solve reaches iteration 20 at `74,482,699 ppb`, above the inclusive `100,000 ppb` threshold. Stable cause: `WATER_DENSITY_NONCONVERGENCE`. | Raw report SHA-256 `b664d83093e4ce1f8502f07c3248f401692fd599a2064cbff2a5a0448d146dd7`. |

The hydro run accepts only step 0. Its last accepted frame root is
`85fdd55b00b56023ae2f16b7df18270e19f734fb333e4c1d81b0922e2c1f8f7c`.
The reconstructed initial density-ratio percentiles are:

- p50: `999,972,466 ppb`;
- p95: `1,544,669,732 ppb`;
- p99: `1,734,263,219 ppb`.

The exact initial centre of mass is `(500000, 375000, 500000) µm`, so the
failure precedes publication of the first integrated frame and is not a
centre-of-mass or canonicalization mismatch.

## Conclusion and stop decision

W1 code exists and its non-interacting analytical control passes, but the W1
exit criterion is not met. The full nominal corpus and external SPlisHSPlasH
comparison were not run after the first interacting nominal scenario failed.
`CONTINUUM-WATER-REF-P1` therefore remains `NOT_RUN`; neither `PASS` nor a
production/integration claim is admissible.

The result reopens the W0B/W1 numerical boundary. The leading possibilities
are an implementation mismatch in density/boundary reconstruction or a frozen
profile incompatibility among the initial lattice, Akinci boundary quadrature,
density tolerance and 20-iteration ceiling. The current evidence does not
choose between them.

The smallest next discriminator is an independent row-level audit of the
initial `CW-HYDRO-001` density, factor and first/last Jacobi iterations for
interior, face, edge and corner samples. If that audit differs, repair W1
without changing W0B roots. If it matches, revise and re-close W0B explicitly;
do not tune thresholds inside W1, move to W2, or hide the failure with
parallelism, PhysX or GPU execution.
