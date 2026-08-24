# NSR3-B4E2D7R19R23 within-epoch outer-9 grant evidence

Date: `2026-08-24`

Status: `PASS / OUTER9_GRANT_CANDIDATE / SHADOW ONLY`

## Outcome

The exact non-admissible R22 successor issues one zero-work outer-9 grant
inside epoch 1. All canonical roots match the independently frozen targets.
The source owner is consumed once and the active owner remains unconsumed.

```text
source state     dc983c93075040fc2f23080072807f8a1b691adade4f3d7a950be3c9f44b36d9
owner before     1d7364cd688d8be1cfc0cc3f12fa58975cd8fe318c702da1201c81cbfbb9d7c0
owner after      f4bb27171627bffbbfdd47551ec84eb2b842d17b29e0f2ff1464c34c01febd3b
grant            4dd8744547f991b40e5871a4c1823c3e073952dc7b51d6c28acabd92cebf5d18
receipt          16b216ea8c7748ee80a5551f3baf5ee8f3afedea13fe5b88c6283cf07f7de285
active owner     bd3b2356eb10bf422ae07c16ba9433a63a63da1ccd7e567dcdd3dd3bc4fcee81
committed state  b9b8a2d3f0827d9a66a54aeb24f70c674015b8d968d8aaae16189e4adc72ef82
```

Epoch remains `1`; next outer is `9`; slice/cumulative HVP remain `182/705`.
The complete ledger remains
`9,3,32,1,182,705,46,28,679,26,2,28,0`.

## Atomicity and scope

All `14/14` valid, rejection, abort and duplicate-replay routes pass at corpus
root `276b751d2929c70484ae0231c6795f49cb5634d260aea1bf9473ad985dd89b5b`.
Every failure rolls back byte-exactly and duplicate replay is idempotent.

```text
new HVP                 0
new workspaces          0
new precision audits    0
new outer updates       0
outer 9 executed        false
public/world commit     false
timing admitted         false
runtime/production      false
```

This result authorizes only research/freeze of the separate outer-9 execution
candidate/oracle pair. It does not authorize a cap change, another substep,
macro, trajectory, public state or production use.

## Reproducibility

Contract/research commit: `606a0e7c`.

Implementation commit: `465ae3c6`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r23-a.FgVmbu`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r23-b.uPHH2K`.

Both binaries are `6,635,816` bytes, have SHA-256
`b77f560090b58670fcfd5248c6a8f1889cc07c12d2eba97f221638922a107dc4`
and GNU build ID `cf6a41e379d30ded284f1703fe06744c5c85f0e8`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r23-a.FcVA2i`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r23-b.K5P5fn`.

Both exit `0`, emit empty stderr and reproduce `1,411` stdout bytes:

```text
stdout SHA-256  c1aca629743893dc973112f9ced8262728e641084cc172e09bfe8ce5bf467a01
semantic        27e37a5a05689e8483956038cbc63ce24ded6d0307c41b7e9e86a44c0c42bfa8
route           OUTER9_GRANT_CANDIDATE
```

## Next action

Research/freeze one outer-9 slice candidate at `182/512` and one independent
unsliced oracle at cumulative `705/8704`, with the unchanged per-trust cap 34.
Do not execute outer 9 before that contract is frozen.
