# NSR3-B4E2D7R19R21 within-epoch outer-8 grant research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question

How should the exact non-admissible R20 successor authorize outer 8 without
resetting the still-active epoch or allowing duplicate budget consumption?

## Decision

Reuse the R19 within-epoch one-use grant protocol with a new source identity.
R20 consumed the outer-7 owner and emitted an exact successor state, receipt
and history. A separate source owner for that exact state is consumed once and
issues an unconsumed outer-8 owner.

Epoch remains `1`. Slice/cumulative HVP remain `102/625`, leaving 410 HVP in
the active slice. Advancing to epoch 2 here would erase valid remaining
capacity and is rejected.

## Bound source and canonical targets

```text
R20 state             1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f
R20 receipt           ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd
R20 history           7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d
used                  8,2,27,1,102,625,41,25,602,23,2,25,0
owner before          138f23969d4cc130e02ddd54cbe1fbbb76dd75b0beee0f53312de8f3116b0c7b
owner after           ca301305414a08ca75cc8acda791d0c4a93152988d184f63316658ac278032d1
grant                 6df9d41926767b1cca9df69f576170253e2eb6682435850372bbf4420b7d0b43
grant receipt         0675f2ab94286aee2b85afe6cf496bcef8fd615efc514143ecf978c51e2bc0f3
active owner          1987820aa99c22ac2d49abbb894994370a2f85231e9b4792bfc518e3d0eb4c54
committed state       64f90f969b76efcddb331705a653c8d31858e6d502555615d6eea2071c78093f
```

The objects retain the R19 schema/magic and policy root
`d951d089d3de3c3d3581dab82b8c7a5ed90b7b2af5fe4af191ed675be890709e`.
All roots above were independently pre-derived from exact fixed-order
little-endian bytes before implementation.

## Atomicity and controls

The transaction validates R20 parent/source, owner root, epoch, complete
ledger, next outer and all canonical artifacts before one copy-on-write
commit. Source/owner/duplicate/stale, epoch/slice/cumulative/next-outer drift,
artifact corruption and injected abort must preserve exact bytes. Duplicate
replay after success rejects prework and preserves the committed state.

## Scope

R21 performs metadata encoding/hashing only. It may issue one private outer-8
grant, but may not consume it, execute outer 8, reset the epoch, run another
substep/macro/trajectory/timing, publish world state or change live budget and
production policy.

The executable gate is frozen by the
[R21 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r21-within-epoch-outer8-grant-contract.md).
