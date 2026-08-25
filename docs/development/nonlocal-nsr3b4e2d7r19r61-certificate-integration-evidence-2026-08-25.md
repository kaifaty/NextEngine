# NSR3-B4E2D7R19R61 certificate-integration evidence

Date: `2026-08-25`

Status: `PASS / TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `f89f738f`.

Frozen identity SHA-256:
`ab51a986d1931551526becef8d38abd23a352b4d4ade994badb49c8b9533f504`.

## Result

R61 adds a new reusable binary64 audit owner without modifying any transitive
legacy audit. The owner owns the moved workspace and witness, validates the
flat topology, executes one fresh directed JVP and computes legacy shadow and
row-local candidate in one stable row scan.

### Owned topology and shared residual

```text
topology exact       true
minimum degree         44
maximum degree        113
distinct degrees       53
rows below maximum   5992
image root       857037364d5986384955430c5fb3c11c2b0388c4c46bab2d70512199ff24ae33
raw root         b0775bebb4c556a9031fe7e99c86042587dbaae0cf6876b47f5e240561ea61d7
```

Offsets start at zero, are monotone, end exactly at the directed-slot count,
all slots reference valid pairs and their derived maximum equals the workspace
maximum. No caller degree vector exists.

### Legacy shadow reproduction

```text
positive rows          234
maximum upper          1.3678206846699582e-24
maximum row            2522
upper root       9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8
active root      cda544504d01d5a92254b6c9cd657151a8e3cb5f4f7a3cc09ad9fcb38fb6f8fc
```

The shadow is bit-identical to the R58 terminal audit.

### Row-local candidate reproduction

```text
positive rows            0
maximum upper           -1.9354174334860891e-23
maximum row              4930
upper root       78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf
active root      82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3
comparison root  5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de
```

The candidate reproduces R59 exactly. Candidate upper exceeds shadow on zero
rows; candidate-active/shadow-inactive failures are zero. The shared image/raw
identity prevents the paths from silently certifying different residuals.

The five retained degree controls pass, the dense dual-path case exercises
strict lower-degree improvement and maximum-degree equality, and a truncated
`offsets.back()` topology is rejected. Controls root:
`cfe8b6104f7dedda4ca115868414ae420b61d551e661689e259d86944a7a2c8b`.

```text
route root  8429ba15d91b064e93ec730dce4e7581ef69e44ae8fa2220b904b826eebe8a0a
semantic    18794b59910842019f4d99ed468cb55ffa6b925bbb3c7c0b4ccfbe46c0377436
```

New work is one directed JVP, one owner row scan and six dense controls. One
workspace is built and released; rollback is exact. Binary128 is absent from
the integrated owner.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r61-final-a.EkWRdu
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r61-final-b.u4qZbt
binary SHA-256 bb006fb783b48a1ec80a6dabc19eb92bd7e521a45a37f0b6ea0397dd968d3776
size           8407168
ELF build-id   241aee378b3c96a892047764a917c22bdfba11d2
stdout bytes   2289
stdout SHA-256 cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4
```

Both clean Release binaries and outputs are byte-exact; both processes exit
zero. Wall time is not performance evidence. R60 remains exact internally at
stdout SHA-256
`6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7`.

## Consequence

The linearized restoration-compatibility certificate now has a validated,
topology-owned binary64 integration candidate. R61 does not replace the legacy
owner or apply the witness. Research R62 as a separate restoration promotion
transaction: define immutable source/target ownership, map the dimensionless
witness into the intended state, rebuild fresh nonlinear topology, recheck
contact/trust/constraint merit and commit atomically only on complete success.
Any failure must preserve the exact R43/R58/R61 source state.
