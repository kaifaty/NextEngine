# Physical sound V34 T0 terminal-path proof result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / ALL_TERMINALS_ATOMIC / ZERO_OFFICIAL_TARGET_MODEL_ACCESS` |
| Owner commit | `dd3fddb6ea22bd436fdff0ae247d628d4f0afc86` |
| Terminal publisher SHA-256 | `60c54b392e20f1024a1cfbb3c2f69d3aed925e341495219787b6379fe1268f8d` |
| T0 orchestrator SHA-256 | `e3df18e5570ed07fce65dce2bfc8e2a3aada9b82d0f516c4691596631af76c88` |
| F0 profile SHA-256 | `7be7e96a2899d0e4ef7aad668368669886d7f45adac34ec6676afbbe6b45e429` |
| Claim | `DISCARDED_FULL_SHAPE_TERMINAL_PUBLICATION_PROOF_ONLY / NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen T0 orchestrator ran in two fresh CPU processes. Both return
`T0_TERMINAL_PATH_PROOF_PASS`, publish the same `28` files / `1,188,178` bytes
and authorize only `V34-D0-owner-freeze-and-development`. Every artifact and
stdout byte matches; stderr differs only in diagnostic wall/RSS fields.

T0 exercises the reusable terminal publisher that the future D0 owner must
import unchanged. It uses discarded zero-valued payloads with the official
shape: `10,800 x 3` targets, `4,320 x 3` development predictions, `1,811`
candidate parameters and `1,491` raw-control parameters. These bytes have no
scientific or quality authority.

## Terminal matrix

| Forced path | Published outcome | Candidate freeze | Atomic closure |
| --- | --- | --- | --- |
| Pass | `Pass` | present, discarded-only | complete |
| Metric failure | `MetricReject` | absent | complete |
| Hard-gate failure | `HardGateReject` | absent | complete |
| Resource failure | `ResourceReject` | absent | bounded reject receipt |
| Invalid pre-access contract | `ContractReject` | absent | no output directory |

The focused suite additionally injects failures after the first payload and
after evidence publication. Both leave neither the requested output nor a
staging directory. Freeze/outcome mismatches reject before publication. This
directly closes V33's failure mode in which a hard-gate exception escaped after
development access and bypassed the terminal artifact contract.

## Access and identity closure

All official train/development/method-holdout target, feature and model counts
remain zero, as do oracle, real/protected signal and network counters. The
`10,800` discarded rows are separately marked `post-access` and cannot be
mistaken for official evidence. Pre-access rejection records exact zero rows.

The full canonical tree-manifest SHA-256 is
`1a06de8756bd7095a7b5487e9be1ede8008fbdf2dbf20054016cb544ebef7423`.
Top-level evidence/report hashes are:

| Artifact | SHA-256 |
| --- | --- |
| `evidence.json` | `8e0e693c30bc0d1cf0d5ca8a8390fb2920bd0a9584af9b09b1e3a91ff8aa7c85` |
| `report.json` | `adaa81152beab1654d8c6a15783dba88f1b887935d2ede61d3cdebf2fab946e2` |

Repeated stdout SHA-256 is
`cd6f19f6b4a4fcf5df9025727f8df56355ab68868dc8d04621301ccf42a35bff`.
Both outer runs use `0.07 s`; maximum RSS is `33,616 / 33,876 KiB`.

## Conclusion and next boundary

All V34 pre-training gates are now complete. A new official D0 owner may be
implemented and frozen, but it must:

- bind the exact F0/C0/T0 results and terminal-publisher SHA;
- build only fresh V34 train/development roles;
- convert every post-access hard/metric/resource result into the shared atomic
  publisher rather than raising;
- keep method holdout inaccessible and all official values unopened until its
  owner commit is frozen.

D0 remains one-shot: any failure closes V34 before holdout. SPEC-45 remains
`Proposed`; T0 creates no model-quality, real-material, validator, admission,
cooker, demo, runtime or ProductCheck authority.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V34 T0/terminal-publisher Python suite: `PASS`, `5/5` tests;
- official process A/B: `PASS / REPEAT_EXACT`, `28/28` files and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
