# NSR3-B4BK1 -- contact-face symmetry repair contract

Status: `EXECUTED_PASS / BOX_CONTACT_KKT_CANDIDATE / B4B_R1_DESIGN_AUTHORIZED`

Parent B4BK is exact FAIL with semantic SHA-256
`7cb256e5b1c62db865c7230a03a94a9553b07f1112a7e9552c78bd6b273a9f93`
and JSON-without-final-LF SHA-256
`b110585a9e8894126666c4f5c941b447361b15e3ea771deebdf0e760afbec716`.

## Identity and sole repair

```text
box-contact-kkt-discriminator-r1-face-symmetry
```

Replay the complete B4BK implementation, three split states, constrained
solver actions, thresholds, work caps and detached P2 negative unchanged.
Replace only r0's false `no x/z or upper multiplier` assertion.

For each P1 `48/96/192` constrained state require exact active face counts in
feature order `[x-,x+,y-,y+,z-,z+]`:

```text
[12, 12, 16, 0, 12, 12].
```

Report the nonnegative multiplier sum and signed fluid contact impulse for
each face. Require:

- `y+` count and multiplier sum are exactly zero;
- `|J_x- + J_x+| <= 1e-12 N s` and
  `|J_z- + J_z+| <= 1e-12 N s`, with the upper-face impulse carrying the
  negative axis sign;
- the aggregate lateral contact impulse equals the sum of reported face
  impulses within `1e-12 N s`;
- lower-y count is 16 and its fluid impulse is nonnegative.

No equality of opposite multiplier magnitudes is required beyond the signed
impulse closure: binary64 traversal may distribute roundoff differently while
the complete vector ledger remains authoritative.

## Repeatability and exit

Two r1 reports must be byte-identical. B4BK r0, B4B, B4A, B3R, D5, original
B3 and B2 raw outputs remain exact.

PASS selects `BOX_CONTACT_KKT_CANDIDATE` and authorizes design of a separately
frozen `tiny-pressure-water-corpus-r1-contact-kkt`. FAIL rejects this one
geometry correction and opens the new contact-potential formula lineage.
No full trajectory, neighborhood, nominal water, viscosity, surface tension,
CUDA, runtime or production authority is granted.

Execution evidence is recorded in the
[dated B4BK1 report](../../development/nonlocal-nsr3b4bk1-contact-face-evidence-2026-08-21.md).
