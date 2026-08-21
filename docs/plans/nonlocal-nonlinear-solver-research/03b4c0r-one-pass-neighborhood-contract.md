# NSR3-B4C0R -- one-pass pre-admitted joint neighborhood

Status: `PASS / JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE / B4C1_DESIGN_AUTHORIZED`

Parent B4C0 is exact FAIL with semantic SHA-256
`44aec304e5fd7d3a54a3d74d9512630d4a76595b5d437fda42751fc4187e8a89`
and JSON-without-final-LF SHA-256
`d811c8d55fd3ed1dad803d489b70eb9f2b1ca690296d4b4420c42ad44908f761`.
B4B2 remains the physical parent and B4C0 remains negative evidence.

## Sole repair

Retain B4C0's canonical IDs, `H=0.15 m` cells, binary64 cutoff, pair/adjacency
order, capacities and all positive/negative fixtures. Replace only:

```text
exact membership count -> exact-size allocation -> membership fill
```

with:

```text
admit and reserve maximum pair payload before query
  -> one exact membership scan
  -> reject and clear all private output on overflow
  -> sort admitted pairs and build exact adjacency
```

Use `u32` fluid/participant indices. Pair payload capacity is
`160*N_fluid*sizeof(u32[2]) <= 64,000,000 bytes`; degree-bounded adjacency
payload is `<=32,000,000 bytes`. Report both plus actual pair count. Nested-row
container overhead is diagnostic and grants no nominal memory credit.

## Gates

Every B4C0 case must retain exact pair digest, all-pairs membership,
density/energy/gradient, both HVPs, repeat and storage permutations. Every
typed failure must return zero pair and adjacency sizes.

Executed cell distance tests must now be strictly below all-pairs candidate
checks for P1 initial, P1 feasible forecast and P2 detached initial. Report
ratios; do not count a hypothetical unexecuted pass.

## Repeatability and exit

Two reports must be byte-identical. B4C0, B4B2 and all earlier raw reports
remain exact.

PASS selects `JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE` and authorizes only B4C1
compact pressure-tape/CSR contract design. FAIL ends this cell-construction
retry and requires a new bounded algorithm study.

No solver substitution, canonical continuation, nominal run, CUDA,
performance, runtime, schema or production authority is granted.
