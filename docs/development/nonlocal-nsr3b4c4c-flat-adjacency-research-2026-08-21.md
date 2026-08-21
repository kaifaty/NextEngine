# B4C4C flat-adjacency ownership research

Status: `DESIGN FROZEN / EXECUTION AUTHORIZED`

Date: `2026-08-21`

## Question

Can each retained/static-index workspace construct the canonical pressure
adjacency only once, without the per-centre `vector<vector<u32>>`, while
preserving every pair, participant order, binary64 operation and durable root?

B4C4B1 is the selected baseline. It removes repeated immutable-support
sorting, but every workspace still constructs sorted nested participant rows.
`build_joint_pressure_tape` immediately reconstructs the same rows as CSR
pair indices, validates them against the nested rows and keeps both layouts
alive for the remainder of the workspace lifetime.

## Complete consumer audit

The nested rows have only three semantic consumers:

1. `evaluate_joint` walks each active centre in participant order;
2. the untaped reference HVP walks the same order twice;
3. `build_joint_pressure_tape` reconstructs CSR pair indices, sorts each CSR
   row by participant and compares it to the nested row.

After the tape is built, production-path research queries use stored
evaluation plus taped HVP. No accepted, rejected, retained, diagnostic,
publication or ledger path reads nested adjacency again. Negative controls
only inspect that a failed builder has published no adjacency.

## Ordering proof

Canonical unique pairs sort by `(fluid, participant)`, with fluid-fluid pairs
stored only as `i < j` and support participants numbered after all fluid.
For a centre `c`, a single scan of that sorted pair array encounters:

```text
reverse fluid neighbours i < c, in increasing i
forward fluid neighbours j > c, in increasing j
support neighbours F+b, in increasing F+b
```

These three ranges are disjoint and already ordered by the combined
participant id. Filling CSR cursors during that scan therefore produces the
exact legacy row without a per-row sort. Each CSR entry stores the pair index;
the participant is recovered through the already proven pair relation.

## Candidate lifetime

The opt-in candidate adds temporary flat adjacency ownership to the research
neighborhood:

- `F+1` checked `u32` offsets;
- one checked `u32` pair index per directed adjacency record;
- no nested row objects and no participant-record copy.

`evaluate_joint` and the untaped HVP read that CSR in the identical participant
order. Once evaluation is complete, the pressure-tape builder validates the
flat layout, creates only radii/compression, and moves the two CSR allocations
into the tape. The neighborhood then owns neither nested nor flat adjacency;
the workspace has exactly one live CSR owner.

The legacy builder and tape remain unchanged. The candidate is explicit and
research-only; no runtime or public schema changes are authorized.

## Predeclared structural effect

The isolated retained/static-index one-macro lanes build `264` P1 workspaces
with `F=48`, and `9` P2 workspaces with `F=27`. Therefore the candidate must
remove exactly:

| Work | P1 | P2 |
|---|---:|---:|
| nested row objects constructed | `12,672` | `243` |
| nested row-sort calls | `12,672` | `243` |
| duplicate tape row-sort calls | `12,672` | `243` |
| CSR ownership transfers | `264` | `9` |

The number of directed records depends on the evolving state and is not
fitted into the design. The gate records it from the legacy trace, requires
the candidate count to be identical and requires all of those nested
participant insertions and duplicate tape CSR insertions to become zero.
Candidate flat offset and pair-index records must equal the final legacy tape
records exactly. No timing threshold is attached to this structural gate.

## Failure boundaries

The flat source must reject malformed offset size/start/end/monotonicity,
out-of-range or duplicate pair indices, a pair unrelated to the row centre,
non-increasing participant order, directed-capacity overflow and payload
overflow before transferring ownership or publishing a pressure tape.
Synthetic corruption must leave the source CSR owned by the neighborhood;
success must leave it empty and the tape as sole owner.

## Rejected alternatives

- Keeping both nested and flat layouts removes no peak workspace duplication.
- Storing participants rather than pair indices would still require a second
  participant-to-pair lookup or tape reconstruction.
- Pointing the tape at neighborhood memory creates lifetime coupling across
  workspace moves and rejected trials; ownership transfer is explicit.
- Combining support views, static indexing or solver arithmetic with this
  layout change would make failures non-local and invalidate the isolated
  discriminator.

## Decision

Execute the frozen
[B4C4C contract](../plans/nonlocal-nonlinear-solver-research/03b4c4c-flat-adjacency-contract.md)
first on the two retained/static-index one-macro lanes. A PASS may authorize a
separate interleaved construction benchmark and complete-lane rollout. B4D,
nominal corpus, CUDA, runtime/schema and production remain blocked.
