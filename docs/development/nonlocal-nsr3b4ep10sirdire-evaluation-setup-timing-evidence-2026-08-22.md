# NSR3-B4EP10SIRDIRE evaluation setup timing evidence -- 2026-08-22

Status: `PASS / BUFFER_LIVENESS_AUDIT_SELECTED_FOR_RESEARCH`

## Result

All three fresh serialized processes reproduce identity
`a018e4d4...3974`, parent B4EP10SIRDIR result `1f66ab3c...5992e`,
B4EP10SIRDI result `b4f847cb...777e9`, B4EP10SII result, correspondence,
five physics roots, work/reuse counts and phase/executor identities exactly.
The new duration-free result is `f445395e...18a6` in all three processes.
Each process records exactly 226 positive validation, capacity and buffer
segments; checked accounting reconstructs `evaluation_setup` exactly.

| Run | Setup ns | Validation share | Capacity share | Buffer share | Residual share |
|---:|---:|---:|---:|---:|---:|
| 1 | 609,152,117 | 0.097860 | 0.0000177 | 0.901978 | 0.0001440 |
| 2 | 638,245,479 | 0.106651 | 0.0000129 | 0.893193 | 0.0001433 |
| 3 | 646,878,763 | 0.104898 | 0.0000133 | 0.894957 | 0.0001314 |

Median buffer share is `0.89495693182928004`; median validation share is
`0.10489834244256988`. Buffer preparation leads validation by
`8.531659423686836x`. Their absolute share ranges are respectively
`0.008784942229152382` and `0.008790566315664147`; capacity and residual
ranges are also far below the frozen `0.05` gate.

The measured median instrumented transaction is 4.648177664 s. The containing
processes take 4.83/4.92/4.98 s and emit no stderr. These values grant no
speed credit and do not replace the uninstrumented SIRDI result.

## Interpretation

The selected `evaluation_setup` cost is overwhelmingly vector preparation,
not structural validation or checked capacity arithmetic. In the current
implementation the timed region value-initializes seven arrays on every one
of 226 evaluations: gradient, density, radius, compression, two HVP
coefficient arrays and the temporary density-contribution array. Timing alone
does not prove that any initialization is removable: returned workspaces can
coexist, retained tape arrays feed later HVP calls, and rejected trials must
not leak state.

The frozen route therefore authorizes one timing-free structural audit only.
It must prove write-before-read for each initialized range, identify exact
allocation/ownership lifetime events, bound simultaneous live workspaces and
derive the minimum safe high-water buffer count before any reuse candidate is
designed. No broad buffer pooling or initialization removal is authorized by
this result.

## Build and artifacts

- implementation commit: `7f5bc93d118f0dab33e62d3584c9362b1e2f2ce0`;
- executable SHA-256: `501ec616...a9f0`, size 4,331,208 bytes;
- Build ID: `2f9ab9f46727687bd37bb0e232c436c764402ca8`;
- `compile_commands.json` SHA-256: `78defa02...a66`;
- raw metrics SHA-256: `0bb2eff0...751d`.

Source SHA-256 values are `c38c069d...cfd2` for `boundary_reference.cpp`,
`b6070752...4606` for `boundary_reference.hpp` and `2a911a5d...7162` for
`formula_reclosure_main.cpp`. Raw reports remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10sirdire.FZHVO2`.
The Release build passes the repository `-Werror` policy. The old B4EP10SIRDI
stdout remains byte-exact at `539f1ec5...e7e7`, and the old B4EP10SIRDIR
semantic result remains `1f66ab3c...5992e`.

## Decision

Close B4EP10SIRDIRE as PASS and select exactly one buffer write-before-read,
lifetime and high-water structural audit for research. Do not claim an
optimization or implement reuse yet. B4E2, broad corpus, runtime/GPU/schema
and production remain blocked.
