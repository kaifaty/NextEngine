# NSR3-B4E2D7R19R25 within-epoch outer-10 grant evidence

Date: `2026-08-24`

Status: `PASS / OUTER10_GRANT_CANDIDATE / SHADOW ONLY`

## Outcome

The exact non-admissible R24 successor issues one zero-work outer-10 grant
inside epoch 1. All canonical roots match the independently frozen targets.
The source owner is consumed once and the active owner remains unconsumed.

```text
source state     749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98
owner before     3519d3b80578ac1dfc2b89b68340f1fe748775ab265a9535e2eb7bb5d75039bb
owner after      11bf48f76ee1c521e2fa29d43e2b0a3f9cd1a03c56e4ee344497099e01e4991f
grant            f825ede7b3fa52c93b47806f92e40f34eefb93daca54d0b2d3092bf0870eede7
receipt          cc39098c863c2fa614086e3702029b0c04182bdfd501ffbf94753d9bd410a8fa
active owner     2f6aea4168bdc513f21099f363a9aa28cc9171620f4a7c558b2cc340d6a6f6c6
committed state  b82cd881e7b50794de029e6829e8575b86630d50b0f4caaf74a7526a3bb66153
```

Epoch remains `1`; next outer is `10`; slice/cumulative HVP remain `231/754`.
The complete ledger remains `10,2,24,1,231,754,50,30,726,28,2,30,0`.

## Atomicity and scope

All `14/14` valid, rejection, abort and duplicate-replay routes pass at corpus
root `49bdd2305eacebde6bb071d103c1160cae70269be9a97986a0b6ffe84fdafd0f`.
Every failure rolls back byte-exactly and duplicate replay is idempotent.

```text
new HVP                 0
new workspaces          0
new precision audits    0
new outer updates       0
outer 10 executed       false
public/world commit     false
timing admitted         false
runtime/production      false
```

The exact R24 stationarity observation is identity-bound but not interpreted
by the grant. This result authorizes only research/freeze of a separate
outer-10 candidate/oracle pair. It does not authorize a cap or policy change.

## Reproducibility

Contract/research commit: `b790c107`.

Implementation commit: `e84381c4`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r25-a.8p9lA2`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r25-b.hzNpj1`.

Both binaries are `6,719,832` bytes, have SHA-256
`2958d9ff0c4f0410cf7224e223c9991c2b07a2ef6e74d77767f24de77df69154`
and GNU build ID `64bc32e4e5c39d588ba35d1509a58922d1b8bb2a`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r25-a.FahtEq`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r25-b.BnusLd`.

Both exit `0`, emit empty stderr and reproduce `1,415` stdout bytes:

```text
stdout SHA-256  b20fffc6ac40ca1f16feb91e412bf7ed7a5be1430502c8921827662ec206cd69
semantic        fc8e0182409785c966c4f1c7f391c910e6a30d3d942335a6eb53b1b3e6fc18a9
route           OUTER10_GRANT_CANDIDATE
```

## Next action

Research/freeze one outer-10 slice candidate at `231/512` and one independent
unsliced oracle at cumulative `754/8704`, with cap 34 unchanged. Preserve and
report the stationarity trend; do not execute outer 10 before freeze.
