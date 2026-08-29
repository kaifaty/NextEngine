# NSR3-B4C4C -- flat adjacency with single-owner tape transfer

Status: `PASS / ONE_MACRO CANDIDATE SELECTED`

Parent B4C4B1 passes with JSON-without-final-LF SHA-256
`7d93828a21473fd6af6534107199e844387f41b8b3d86ef18b7141a854cd74b9`
and semantic SHA-256
`05c4b74a330fc23f598bbae687e68e84403da3f69ee0bf9bbdb05b70add818c8`.

## Identity

```text
sha256  b8711c6a643680c9a778555480337762f28dfd7cefd7b871381275c2a255b771
text    nextengine.nonlocal.flat-adjacency-transfer|v1|parent=b4c4b1|layout=csr-pair-index|owner=neighborhood-until-evaluation-then-pressure-tape|one-macro=p1:264x48,p2:9x27|thresholds=none
```

## Candidate

Retain B4C4A workspace ownership and B4C4B immutable support indexing. Add an
opt-in one-macro layout that builds adjacency directly as CSR pair indices.
Construct rows by a single scan of canonical sorted pairs. Do not sort a row:
the B4C4C ordering proof must be checked against the legacy participant rows.

Before tape publication, evaluate density/gradient and the independent
untaped HVP through the flat rows in identical participant order. Build tape
radii and compression without reconstructing CSR; after all validation,
move offsets/indices from neighborhood to tape. On success the tape is the
only CSR owner and the neighborhood has no nested or flat adjacency payload.

Legacy nested construction/tape reconstruction remain the default oracle.

## Exact gates

Run legacy B4C4B1-layout, candidate and repeated candidate one-macro P1/P2
transactions. Require bit-exact:

- canonical fluid/support points, pairs, pair hash and pair classification;
- row participant sequence before transfer and final tape offsets/indices;
- density, gradient, energy, active set, radius, compression and all HVPs;
- workspace/query-chain hashes, KKT decisions and accepted/rejected trials;
- physical diagnostics, adaptive schedule, topology, publication and ledgers;
- trajectory, policy/legacy roots and retained ownership receipts;
- B4C4B/B4C4B1 parent hashes and static-index work receipts.

Legacy/candidate/repeat workspace and tape counts remain `264` P1 and `9` P2.
Static index work remains exactly B4C4B. Two B4C4C reports must be
byte-identical.

## Structural work obligations

| Counter | P1 legacy / candidate | P2 legacy / candidate |
|---|---:|---:|
| nested row objects | `12,672 / 0` | `243 / 0` |
| nested row sorts | `12,672 / 0` | `243 / 0` |
| duplicate tape row sorts | `12,672 / 0` | `243 / 0` |
| CSR ownership transfers | `0 / 264` | `0 / 9` |

For each case, legacy nested participant records, candidate flat pair-index
records and final directed tape records must have one identical measured
count. Legacy tape CSR allocation/fill counts must equal that count plus its
`F+1` offsets per workspace; candidate duplicate tape CSR allocation/fill
counts must be zero. No wall-time threshold or speed claim applies.

## Negative controls

Corrupt one property at a time and require `PRESSURE_TAPE_ADJACENCY` or the
existing capacity class with zero tape publication and no ownership transfer:

1. offset count, non-zero start, decreasing/end-out-of-range offset;
2. pair index out of range or duplicate within a row;
3. pair not incident to the row centre;
4. non-increasing participant order;
5. directed limit and total payload limit.

The valid source transfers exactly once. Reuse after transfer must fail as a
missing adjacency source. Permuted fluid/support input must canonicalize to
the same pairs, CSR, evaluation and tape.

## Authority

PASS selects only `ONE_MACRO_FLAT_ADJACENCY_TRANSFER_CANDIDATE` and may
authorize a separately frozen timing discriminator and complete-lane design.
B4D, nominal corpus, CUDA, runtime/schema and production remain blocked.
