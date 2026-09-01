# Original Nonlocal GPU capacity-160 performance gate — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ3` revision 1 |
| Status | `FROZEN / PERFORMANCE_DISCRIMINATOR / TOOL_ONLY` |
| Parent evidence | NGQ2 revision 2 dynamic visual PASS |

## Question

Does the original fused-owner/compact-u16 Nonlocal GPU route still fit the
game budget at 48,000 particles after adopting the `160`-neighbor allocation
required by the accepted 4k/16k dynamic visual corpus?

This is a performance discriminator, not a new physics experiment. It changes
allocation headroom only. The actual graph membership, pair work, five fixed
iterations, H3 coefficients and analytic GPU box contact remain unchanged.

## Frozen profile

The only admitted performance profile is:

```text
profile id                 nuv-basin-48k-analytic-contact-game-cap160.v6
dynamic samples            48,000 (80 x 15 x 40)
neighbor capacity          160 per sample
directed-pair capacity     7,680,000
neighbor encoding          compact u16
fixed iterations           5
horizon                    0.15 m
time step                  1/240 s
kappa / lambda             9196.875 / 360
static ghosts              0
contact                    analytic_box_clamp_gpu_v1, timed
```

The version-5 `N*123` profile remains retained evidence and is not silently
rewritten. Revision 6 must also be used by the visual corpus so the combined
quality-and-budget statement names one capacity/profile identity. Different
fixture geometry between the 48k timing lattice and the visual falling-dam
lanes is expected and must be reported.

## Execution and gates

Run two independent processes. Each process performs:

1. the existing exact P2 correctness check;
2. retained-u32 versus compact-u16 seed/trace correspondence;
3. 256 conditioning executions;
4. 64 warmup executions;
5. 512 measured executions.

The process passes only if all of the following are true:

- P2 correctness passes;
- trace and compact-memory correspondence are exact;
- all 512 measurements are valid;
- CUDA total `p95 <= 4.0 ms`;
- CUDA total `p99 <= 6.0 ms`;
- analytic contact is included in the primary CUDA interval;
- profile, input, trace, output and logical-CSR roots are published;
- maximum degree is at most `160` and directed pairs do not exceed
  `7,680,000`.

Timing samples may differ between processes. Semantic/correctness roots must
match. A command exit code alone is not the decision: the JSON
`decision.decision_gate` is authoritative.

## Stop and interpretation

- Do not lower capacity, reduce iterations, remove contact or weaken either
  timing threshold after observing the result.
- If capacity or correctness fails, classify the route
  `APPARATUS_INCONCLUSIVE` and stop.
- If correctness passes but either timing gate fails, classify the exact
  configuration `GAME_BUDGET_REFUTED_BOUNDED`.
- If both processes pass, classify it
  `QUALITY_AND_BUDGET_SUPPORTED_BOUNDED` when combined with the separately
  passing NGQ2 visual corpus on the same version-6 capacity profile.

The claim ceiling is the exact RTX 3080, CUDA tool and finite fixtures. It does
not authorize a shipping backend, renderer integration, a corrected-research
solver claim, or laboratory fidelity. CPU DFSPH remains the product fallback.

## Next action after PASS

Prototype presentation-only surface smoothing/extraction over the accepted
frames. The observer and smoothing path must not feed positions, velocities or
contacts back into the simulation.
