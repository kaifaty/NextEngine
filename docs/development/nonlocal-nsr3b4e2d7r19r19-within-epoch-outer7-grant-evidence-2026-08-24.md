# NSR3-B4E2D7R19R19 within-epoch outer-7 grant evidence

Date: `2026-08-24`

Status: `PASS / OUTER7_GRANT_CANDIDATE / SHADOW ONLY`

## Result

The frozen zero-work transaction passes. It reproduces the exact R18 report,
binds the non-admissible outer-6 successor, consumes its separate source owner
once and creates an unconsumed outer-7 owner without changing epoch or any
resource counter.

```text
route                  OUTER7_GRANT_CANDIDATE
parent stdout SHA-256  3047e85dc42f22afe287b47b0bc3aaa2e9de3bb767b01b6d4c516497c51841e8
stdout SHA-256         0b002ee449562c08b3be84bc94eff219c72d3c2afdf41490d7b65a1fddaffb24
semantic result        e49c1fa1a3f8621c289bb75c7bc470a72338d3433ab94f49750f87d15d421b97
epoch                  1
slice/cumulative HVP   50/573
next outer             7
new solver work        0
```

R18 remains non-admissible. R19 does not execute outer 7, start another
substep, publish world state or create runtime/production authority.

## Canonical objects

All frozen canonical roots reproduce exactly:

| Object | Bytes | SHA-256 |
|---|---:|---|
| source owner before | 60 | `6ead6c878eb810ab6ed7046014e02afd8ea6aeba1d8c1bd57b2f2e33f8bc37a1` |
| source owner after | 60 | `ea9f947b8778e3147f3ba83f3fbf6a2f04f703cb1b3a45def70739b2c71ca0e4` |
| outer grant | 292 | `cb1b13d53754e7b1408d66eeb0a00f7dc02a697ccd338d8d77834a6f8b1a6776` |
| grant receipt | 132 | `28b5b27673dc9ed247957c159f7c4ac071a074902c918a273327a0d0a40b513a` |
| active outer owner | 124 | `f88ea265a2e2544b76ae335770bb548da6aef76c7a3883ff7ea87e62e6f92ac4` |
| committed grant state | 180 | `37d1556efa98ceb68db566a6f553d82295fc6b233fcbfb0aa6d393a96a4ef6b5` |

The policy root is
`d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e`.
The complete identity projection remains exact at
`1b16ea2def4561ff4f2c92879c40b1868c8894c0f53d900eafbc9778c7b50ec0`.

## Atomicity and work

All `14/14` valid/negative routes match their frozen precedence. Their corpus
root is
`fd49fc1ca51317257fc266e6b7658dcfd33146120507fbe554d4783e325f1709`.
Every failing route compares the complete serialized transaction state before
and after and obtains byte equality. The successful route equals the sole
preconstructed committed state; duplicate replay rejects prework and preserves
that committed state exactly.

R19 adds zero HVPs, workspaces, model evaluations, trials, precision audits or
outer updates. The one inherited parent substep is attestation work only.
Timing was neither collected nor admitted.

## Reproducibility

Contract/research commit:
`071a856c` (`docs: freeze D7R19R19 within-epoch outer7 grant`).

Implementation commit:
`e6b08dbf` (`research(nonlocal): implement within-epoch outer7 grant`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r19-a.5L1nbM`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r19-b.YhFSFd`.

Both binaries are `6,475,712` bytes, have SHA-256
`09dbfaae5e1c459e6809aeb283be81bca8f5b79bd94ddd83f126d0b02220969e`
and GNU build ID `4df9808e7f79525aa11315ecce8b1a1efefb29b8`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r19-a.DJcYwB`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r19-b.A4SjAB`.

Both exit `0`, emit empty stderr and reproduce the same `1,410` stdout bytes.

## Next action

Research and freeze R20 as exactly one shadow outer-7 update inside epoch 1.
Compare the slice-budget candidate, starting from `50/512`, with an independent
unsliced cumulative oracle starting from `573/8704`. Preserve every physical,
solver, topology, precision and history identity. Do not execute outer 7 until
that new contract is frozen; do not start a substep, macro, trajectory, timing
run or public/world commit.
