# NSR3-B4EP10SIRDIREA evaluation buffer liveness evidence -- 2026-08-22

Status: `PASS / EVALUATION_BUFFER_REUSE_CONTRACT_RESEARCH_AUTHORIZED`

## Result

Both fresh serialized audit processes are byte-identical at stdout SHA-256
`dd965972...cc51`, exit zero and emit no stderr. They reproduce identity
`dfc1bb3d...7e1d`, B4EP10SIRDI result `b4f847cb...777e9`, B4EP10SII result,
correspondence, five physics roots and all query/work/reuse/retention counts.
The duration-free audit result is `a60d0770...c556` in both processes.

Every requested slot is explicitly written before publication:

| Role | Accumulated slots | Maximum call slots | Element bytes |
|---|---:|---:|---:|
| gradient | 2,672,224 | 11,824 | 24 |
| density | 1,356,000 | 6,000 | 8 |
| radius | 85,716,150 | 380,511 | 8 |
| compression | 1,356,000 | 6,000 | 8 |
| HVP gradient | 85,716,150 | 380,511 | 8 |
| HVP second | 85,716,150 | 380,511 | 8 |
| density contribution | 85,716,150 | 380,511 | 8 |

The shadow checks observe 150,845,996 density-contribution reads only after
pair writes and 1,356,000 density reads only after center writes. The missing-
write corruption rejects before publication.

## Lifetime and projection

The exact returned-workspace lifetime is 226 acquires, 226 releases, maximum
two simultaneously live receipts and zero final live receipts. Accepted moves,
rejected trials and 42 retained diagnostic transfers acquire no extra receipt.
The builder-local density-contribution lane records 226 acquires/releases,
maximum one live and zero final live. The duplicate-release corruption rejects.

The unchanged path value-initializes 2,828,746,176 bytes across the seven roles
over one nominal macro. Exact two-workspace plus one-ephemeral high-water growth
projects to 22,067,592 bytes, ratio `0.0078011919864810096`. Maximum shadow
payload is 1,545,868 bytes. Both remain far below their frozen 64 MiB and 8 MiB
caps. The pair-role maximum is 380,511; the earlier 670,229 high-water belongs
to directed slots and must not size these buffers.

This is a structural work projection, not a speedup. The audit deliberately
adds shadow traffic and admits no timing.

## Build and artifacts

- implementation commit: `ecc9ab903b9919099d855a531b304506a4161184`;
- executable SHA-256: `8d30e0dc...ae71`, size 4,348,944 bytes;
- Build ID: `defea5f156723f064008be187077ac3e07545335`;
- `compile_commands.json` SHA-256: `78defa02...a66`;
- raw metrics SHA-256: `8d65a362...b4e3`.

Source SHA-256 values are `17afc61c...34ce` for `boundary_reference.cpp`,
`fc66fe7e...e0f1` for `boundary_reference.hpp` and `c23403bb...9042` for
`formula_reclosure_main.cpp`. Raw reports remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdirea.pL7BUC`.
The Release build passes `-Werror`. The old B4EP10SIRDI stdout remains exact at
`539f1ec5...e7e7`; B4EP10SIRDIRE remains at `f445395e...18a6`.

## Decision

Close B4EP10SIRDIREA as PASS. Research and freeze one opt-in implementation/A-B
contract for a two-lane retained workspace buffer pool plus one transaction-
local density-contribution scratch lane. Preserve SIRDI as rollback, require
exact receipt behavior and grant speed credit only from balanced external A/B.
B4E2, broad corpus, runtime/GPU/schema and production remain blocked.
