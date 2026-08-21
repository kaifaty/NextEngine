# NSR3-B4EP3 -- canonical superset feasibility-audit contract

Status: `FROZEN / IMPLEMENTATION_AND_TWO_RUNS_AUTHORIZED / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep3-canonical-superset-audit|v1|parent=3268d59c30c11f892e45c5d781989fb085d54b7d5924069285fa4b56e8d0196d:314bc306099a28f46d1856704145eddb90f9410fe94050cadc9dffcbf3557693|oracle=470a0f4ec9b51ac57f1ecb69e937bb9d2756db41299a0dc01fa427ae63a78e99:25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04:4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112|states=227;skin=0.04h=0.006;superset=0.156;cell=h;reach=2;certificate=4*d2<=s2*(1-2e-12)|order=lexicographic-fluid-participant;filter=norm<=h;flat-csr=exact;evaluation+tape=bit-exact|capacity=max-degree<=160;candidate-active<=1.25;reused>=114;candidate-work<full-cell-tests|negatives=certificate-overflow;pair-removal;order-swap|runs=2-byte-exact|reference=closed|credit=b4ep3i-design-only-or-hvp-route
```

Identity SHA-256:
`19c2c1f27c1bec7444743c1d67ce1bb5d69b8acc22804513ef5e8a6cc21e2552`.

## Implementation boundary

Add only the research command:

```text
nonlocal-formula-reclosure --nominal-hydro-topology-reuse-audit
```

It must execute the same B4EP1 parent/transaction once with
`StaticSupportWorkTrace::capture_fluid_states=true`, retain work-only only for
the transaction, and reproduce all frozen B4EP1 physical roots/counters. It
then analyses the captured states offline. Do not route the solver through the
superset candidate and do not time it as an optimization.

Implement an audit-local superset builder with:

- `skin = 0.04 * HORIZON`, list radius `HORIZON + skin`;
- existing `HORIZON` cell edge and search reach two in each axis;
- exact binary64 `norm <= list_radius` membership;
- final lexicographic `(fluid, participant)` pair sort;
- the existing maximum degree 160 and pair-capacity admission;
- rebuild on first state or failed
  `4*d_max_squared <= skin_squared*(1-2e-12)` certificate;
- unchanged exact `norm <= HORIZON` filter and existing flat-CSR convention.

## Exact gate

Require:

1. B4EP1 physical/work decisions and roots exactly, including 14/28 selected
   substeps, 42/28/14 attempted/accepted/discarded, 221/0 outer/rejected,
   411/48 nonlinear/spectral HVP and the frozen frame/aggregate/ledger roots.
2. Exactly 227 captured, canonical and candidate states; parent hash policy
   `1/0`, transaction policy `0/226`, and no final ownership.
3. At every state, exact fluid/support positions, active pair vector/order,
   pair counts, maximum active degree, flat offsets/indices, evaluation and
   pressure tape versus the canonical full builder.
4. Maximum superset degree `<=160`, candidate/active pair-visit ratio
   `<=1.25`, at least 114 certified reuses, and
   `superset_build_cell_tests + filtered_candidate_checks`
   strictly below canonical full cell-distance tests.
5. Certificate-overflow, removed-active-pair and swapped-order negatives all
   reject through their intended predicates.

Emit a deterministic corpus root, candidate topology root, work receipt and
result root. Run two fresh Release processes and require byte-identical JSON.
Timing/RSS may be captured externally but cannot gate or enter the report.

## Exit

PASS selects `CANONICAL_SUPERSET_FEASIBLE` and authorizes only B4EP3I cache
implementation/A-B design. Any failure records its first exact state/predicate,
stops this topology family without skin/capacity tuning and authorizes only
HVP design. B4E2, references, runtime/CUDA and production remain blocked.
