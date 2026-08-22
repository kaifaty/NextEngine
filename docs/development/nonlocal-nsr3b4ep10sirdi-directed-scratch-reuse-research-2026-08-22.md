# NSR3-B4EP10SIRDI directed scratch reuse research -- 2026-08-22

Status: `COMPLETE / OPT_IN_REUSE_SELECTED`

## Goal

B4EP10SIRDA proves that evaluation/HVP directed temporaries have complete
write-before-read liveness and that one high-water buffer replaces 454.3M
repeated value-initializations structurally. The remaining question is whether
that reduction improves the exact eight-worker candidate in wall time without
moving memory into persistent engine state.

## Selected ownership

Use one research-only transaction-local `Vec3` buffer owned beside the
parallel executor trace:

- grow only when the current directed count exceeds the retained size;
- never shrink during the macro transaction;
- expose only the current prefix to evaluation/HVP source-row kernels;
- overwrite all active source slots before the unchanged target fold;
- release storage on every transaction exit, including failures;
- retain only scalar counters in the returned trace.

Evaluation and HVP share this buffer sequentially. Compression-direction and
target-output vectors remain unchanged, as do all formulas, source traversal,
64-partition scheduling and canonical target addition order. The SIRDA shadow
bitmap is not run in the timed candidate; its closed evidence is the structural
precondition, while lightweight size/capacity checks remain fail-closed.

## Measurement

Compare B4EP10SII with one opt-in reuse command in the same Release executable.
Use one warmup per command, then serialized `AB`, `BA`, `AB` pairs pinned to
CPUs `0..7`. Require exact physics/work, 685 reuse calls, 670,229 growth slots,
zero live scratch at exit and three candidate wins. Retain the established
`1.05x` median paired speed gate rather than accepting a statistically visible
but architecturally marginal change. Also reject unstable wall measurements,
more than 16 MiB RSS growth or more than 2% median total-CPU regression.

## Decision

Freeze B4EP10SIRDI as an opt-in implementation/A-B contract. Failure retains
B4EP10SII and the SIRDA structural result; PASS selects reuse only for nominal
research and must be followed by residual attribution. No broad-corpus,
runtime, GPU or production claim follows.
