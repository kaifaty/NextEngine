# NSR3-B4E2D2 -- reference-binary64 topology contract

Status: `FROZEN / NOT_RUN / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d2-reference-binary64-topology|v1|parent=c3fb3522dba71768a7933c293523ebbbbaf08a0b5ee408a57b27a2300a2e22d8:2c340f6a9cb570d57a70c3cceed9ab81d8e6a9fb11d278ec3d8d7ad2aefda9ce:5a9d2f67550b73169c7405c1c46fb6ee5eeebeb539915d296620e1c407922c07|selected=micrometre-division:0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;addition=b7063e2b024c90acce762533ca612382321a677eee46443942e0eea1a7e28dc4|alignment=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a:8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09:a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8|topology=decoded:fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13:342502:611520:120;addition:c330a0aecb913d92e95478dc3325f9bc326d5e8d3057485723dd62eef1493889:335814:596256:117|audit=pair-delta-only-near-horizon<=64epsh;active-centers0;pressure-energy-gradient-zero;density-finite;capacity-pairs<=960000;degree<=160|controls=decoded-position-bit;pair-root-change|runs=2-processes;byte-exact|trajectory=none|timing=none|credit=b4e2d3-pilot-repair-contract-research-only
```

Identity SHA-256:
`6ad559bfb28a79468f2810ff30615f1f1493cc13dad66f64ba4ecb0015a63e22`.

## Gate

Add `--nominal-dam-reference-binary64-topology`. Generate the addition-built
B4E0 state and the independently decoded micrometre-division state. Require
the selected external raw-bit identity, scenario/static roots, and exact old
and decoded pair roots/counts/degrees frozen above.

Compare unique sorted pairs by their canonical participant IDs. Publish
common, addition-only and decoded-only counts plus maximum
`abs(radius-h) / (epsilon*h)` over every one-sided pair, evaluated in its
own admitted neighborhood. Require no duplicate pair, full CSR consistency,
and maximum normalized horizon distance at most 64.

For both neighborhoods require finite density/energy/gradient, zero active
centres, exactly zero pressure energy and every gradient component exactly
zero. The decoded candidate must remain at most 960,000 pairs and degree 160.

Flip one least-significant position bit in a private decoded sample and
require either the pair root or exact topology counts to change. Restore the
state before reporting.

Run two fresh processes from one Release build and require exit zero, empty
stderr and byte-identical stdout. No macro transaction, solver, trajectory,
timing or speed claim.

PASS authorizes only B4E2D3 pilot repair contract research: safe empty-prefix
failure reporting plus decoded-frame-zero topology expectations. No physical
rerun occurs until B4E2D3 freezes.

