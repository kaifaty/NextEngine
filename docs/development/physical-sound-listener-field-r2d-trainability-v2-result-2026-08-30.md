# Physical sound R2D context trainability V2 result

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Scope | Query-free optimizer-only successor over the unchanged 420-row Green Goblet context field |
| Decision | `R2D_TRAINABILITY_GATE_PASS / N0.3E_RESEARCH_AUTHORIZED` |
| Public/runtime authority | None; no spatial query quality was measured, SPEC-45 remains `Proposed`, and clips remain mandatory |

## Question and answer

R2D V2 asked one preregistered question: was the last V1 failure caused by a
non-decaying terminal AdamW step rather than the frozen basis, objective,
tasks, cooker or thresholds?

Yes for the context-only training substrate. Replacing fixed `0.05` with one
inclusive-endpoint half-cosine schedule from `0.05` to `0.00001`, while
retaining every other V1 input and gate, makes all three tasks pass. Runs A and
B repeat exactly at the normalized report, checkpoint and prediction
boundaries. The immutable decision is `R2DTrainabilityGatePass`; one separately
frozen N0.3E coordinate-to-coefficient field is now authorized without query
feedback.

This is not a held-listener quality result. It proves only that the
representation, objective, optimizer and real cooker can preserve the context
signal.

## Frozen optimizer-only revision

V2 references the exact V1 manifest instead of rebuilding its factorization or
controls. The V2 manifest records the only permitted change and lists basis,
objective, initialization, tasks, steps, Adam hyperparameters, clipping,
cooker, metrics, gates and context rows as unchanged.

| Artifact or observation | Exact value |
| --- | --- |
| V1 manifest SHA-256 | `3fe4129584da5395b1502c0f55e6061603c530ed9e214c48bfd13f3166c05f28` |
| V2 manifest SHA-256 | `47920fce0dac93035d19a3b69ddbbd7c6eda4b5db2996887942a0765b683f6d1` |
| V2 freeze report SHA-256 | `c428dec386a68a70257fc0fbcb668f9cdc680c204b1bbdf83f355187d8be9a40` |
| Schedule | half-cosine, no warmup, applied before each step |
| Initial/final learning rate | `0.05 / 0.00001` |
| Steps | `800 / 1,200 / 1,600`, unchanged |
| Rank-96 retained total energy | `0.9963964439551598`, inherited unchanged |
| Query/method-holdout/shadow reads | `0 / 0 / 0` bytes |
| Optimizer steps before freeze | `0` |

The endpoint schedule is exact for each task. Its first step uses `0.05`, its
last step uses `0.00001`, and progress is the optimizer step index divided by
`total_steps - 1`.

## Repeated optimizer result

Runs A and B produce byte-identical normalized `run-report.json` with SHA-256
`898ee201018c4f9375f838e2f6de9e62a05f8b59700258571ed9ee7a5100875c`.
All three coefficient checkpoints and all 37 emitted prediction WAVs also
match byte-for-byte. MLflow databases, run IDs and raw Rust metric wrappers
remain path-local lineage; the normalized metric rows in the deterministic
report are identical.

| Task | Coefficient RMSE | Raw NMSE | Mean abs log-energy | Clip fraction | Oracle aggregate delta max | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| One row, 800 steps | `0` | `0` | `0` | `0.00125` | `0 dB` | PASS |
| Eight-row block, 1,200 steps | `0` | `0` | `0` | `0.000833` | `0 dB` | PASS |
| Full context, 1,600 steps | `3.86e-8` | `1.64e-14` | `4.54e-9` | `0` | `1.37e-5 dB` | PASS |

The unchanged gates are coefficient RMSE at most `0.005`, raw NMSE at most
`1e-4`, mean absolute log-energy error at most `0.005`, objective/zero at most
`0.01`, clipping fraction at most `0.01`, oracle aggregate delta at most
`0.15 dB`, and strict improvement over both zero and global mean on every one
of the five primary cooker endpoints. Every check is true for every task; no
cook or non-finite failure occurs.

Checkpoint SHA-256 values match across both runs:

- one row: `e08ec433373de5bbaeb9a46d3ffc7ea6fcb7cb1227341485dce56dda8d93f22e`;
- eight-row block: `e846e100059396a0f3c2e99527c4befbd18422802150dbf2cf02af2048517b8e`;
- full context: `341c7a61cd96a1b997c8900f289e8b410dec9681a778515d030d0ed060418837`.

## Consequence and next boundary

V2 falsifies the fixed-terminal-step explanation and removes the trainability
blocker. It does not show that coordinates generalize to held listeners.

The next and only authorized candidate is N0.3E:

1. reuse the passed context-only rank-96 basis and targets;
2. freeze one data-only network mapping published listener coordinates to
   normalized complex coefficients;
3. train and repeat it without query reads or query-driven checkpoint choice;
4. open the 180 grouped queries once for the frozen candidate;
5. require strict improvement over every unchanged classical control on all
   five primary aggregates, otherwise return
   `REJECT_LOW_RANK_COEFFICIENT_FIELD` and stop R2.

Physics loss, architecture grids, threshold changes and a second query-tuned
candidate remain unauthorized. This result creates no admission, public
schema or runtime authority.
