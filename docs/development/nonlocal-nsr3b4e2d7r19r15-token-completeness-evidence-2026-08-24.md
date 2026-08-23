# NSR3-B4E2D7R19R15 continuation-token completeness evidence

Date: `2026-08-24`

Status: `PASS / TOKEN_V1_RESOURCE_LEDGER_COLLISION / V1 NOT RESUMABLE`

## Outcome

R15 proves that the R14 version-1 token is deterministic and sensitive to all
16 field groups it contains, but is not a complete resume authority. Distinct
locally-valid resource and solver-policy contexts retain the exact same v1
token root while changing the amount or semantics of future work.

This closes v1 fail-closed. R14's soft-cap suspension result remains valid;
only its token candidate is rejected for resume.

## Parent and bound-field closure

```text
R15 identity SHA-256  4431b8854b5b1734fce5340587fee30bfa082d1952352bdad7db146e14e2e00f
R14 stdout SHA-256    16b357b173d5c36f123dbbad31cca0c777229ff38772d3847a85357c9a0b4625
R14 semantic          34fdc84cb5bddcc89337f7b1cc961cbc18f5d5eda7cfe008ac6ae0af89d08751
v1 token root         c06dbfeeac346d4413d114082d18b4e4725edfac81896ed72cbabfacf3f188b5
bound mutations       16
changed roots         16
mutation root         a8d05e1ee8bac1605d7bd776a6287e6bca9ebf0fff73c30dbb17f4c1394d35ae
```

The typed reconstruction exactly reproduces the R14 projection and token
root. Mutating schema, phase/state roots, convergence state, predicted/theta/
static identity or HVP epoch/ledger fields changes the v1 root in every fixed
control. R13 and all transitive parents remain exact.

## Exact resource context

The projected suspension ledger is:

```text
limits: outer 16, inner/update 16, HVP/trial 34, soft HVP 512,
        workspaces 288, precision audits 64
used:   outer 6, inner 2, last-trial HVP 25, slice/cumulative HVP 523/523,
        workspaces 33, precision audits 21, accepted/rejected 21/0
```

Its complete research context root is
`c0adedf143687fd46510b30efeb67acfdfa62033421cf7c89ca7e3724077e944`.

The fixed resource twin is also locally valid but records workspaces `32`,
precision audits `20`, accepted trials `20` and outer limit `17`. Its complete
context root is
`0d8a339b6b8ec64fb4d753d73c68d5b6c5e50b01c0082e1816fcfb25820cba90`,
while its v1 token root remains exactly `c06dbfee...8b5`.

The twin would silently grant additional remaining resources and lose one
accepted-trial receipt. This is the selected
`TOKEN_V1_RESOURCE_LEDGER_COLLISION` route.

## Policy identity collision

Changing only completion policy from
`tiered-grace-residual-model-v1` to `direct-model-hvp-v1` gives complete
context root
`3bc76d4b24eb71bfd975250564f54d61928cd0810015900a4c3937340cc05505`,
again with the unchanged v1 token root. The policy collision is recorded even
though the resource collision has earlier route precedence.

Both are projection omissions, not cryptographic SHA-256 collisions.

## Work and authority

R15 runs one exact R14 control parent, then performs hash projections only:

```text
new workspaces / HVP / model HVP  0 / 0 / 0
new trials / precision / outer     0 / 0 / 0
resume / outer 6                   false / false
```

Rollback is exact. No budget code, physics, public schema, substep, macro,
trajectory, timing or production policy changes.

## Reproducibility

Contract/research commit:
`c6970b74` (`research: freeze continuation token completeness`).

Implementation commit:
`0852ea94` (`research: classify continuation token completeness`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r15-a.A6cvnr`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r15-b.9duTCJ`.

Both binaries are `6,218,264` bytes, have SHA-256
`a72d70145b36c136eed390cf411c7d15d499a658f8f4f4761f0cd571e1ea5868`
and GNU build ID `71f9046d2c1490887f36a61da50fc6287c8e76d4`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r15-a.7l1L3L`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r15-b.AY7UIk`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  2,246
stdout SHA-256        257244d44f3ec4460889968221cf95f0f4ca72090ec6406c7b7349a0710b81b4
semantic result       ed209bfa1c6a52411aef21b9f5036fabe46baf68bd2acf95e3b46a424782f0bf
route                 TOKEN_V1_RESOURCE_LEDGER_COLLISION
```

## Decision

Never resume from the v1 token. Preserve it only as exact R14/R15 research
evidence.

Research D7R19R16 as a canonical v2 continuation envelope. It must bind the
full policy identity, payload and history roots, every structural limit and
used counter, monotone epoch/slice/cumulative ledgers and expected-token
ownership. It must reject fixed stale, duplicate and one-field mismatch
controls before work. R16 still may not execute outer 6 or resume.
