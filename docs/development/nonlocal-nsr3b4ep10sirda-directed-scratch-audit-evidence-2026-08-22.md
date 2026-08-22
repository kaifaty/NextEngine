# NSR3-B4EP10SIRDA directed scratch audit evidence -- 2026-08-22

Status: `PASS / DIRECTED_SCRATCH_REUSE_RESEARCH_AUTHORIZED`

## Result

Two fresh serialized audit processes are byte-identical at stdout SHA-256
`fef9a997...0423`. Both preserve B4EP10SII result `f7b1542f...25fb2`,
correspondence `1e4bedbb...08a35d` and all five physics roots. The audit
semantic result is `944f00de...089a`.

Across 226 evaluation plans and 459 HVP calls:

- 454,936,226 full directed slots are currently value-initialized;
- 374,945,086 active slots are overwritten by arithmetic;
- those active slots receive exactly 749,890,172 valid target reads;
- every inactive slot is unread and every target is assigned;
- one never-shrunk buffer grows to only 670,229 slots;
- 454,265,997 repeated initialization slots are structurally removable;
- high-water/full ratio is `0.0014732372620508791`, below `0.01`;
- projected buffer payload is 16,085,496 bytes; shadow audit peak is
  1,340,458 bytes, both below 64 MiB;
- maximum scratch depth is one and final depth is zero;
- missing-write and duplicate-read mutations both reject;
- audit failures are zero.

This proves liveness and capacity only. It does not prove a wall-time benefit.

## Build and regressions

- implementation commit: `28e0fcf25999124c54a34f956e00ed6f6c6f5565`;
- executable SHA-256: `172c37d4...8b0d`, size 4,296,880 bytes;
- Build ID: `919318801a508eb1d35017d424b3ae4fc4b4d9d7`;
- `compile_commands.json` SHA-256: `30e9925f...a0bc`;
- B4EP10SII stdout remains `31990f6f...17ef`;
- B4EP10SIR remains PASS at result `62a2205d...10aa3`.

Source SHA-256 values are `2236cb82...67c0` for `boundary_reference.cpp`,
`5d92084e...9f96` for `boundary_reference.hpp` and `6f708837...bbb6` for
`formula_reclosure_main.cpp`. Compilation passes the repository's `-Werror`
policy and `git diff --check`.

## Decision

Close B4EP10SIRDA as PASS. Research and freeze one opt-in transaction-local
directed-scratch reuse implementation/A-B contract. Keep all arithmetic,
source traversal, target fold and B4EP10I rollback unchanged. B4E2, broad
corpus, runtime, CUDA/GPU, schema and production remain blocked.
