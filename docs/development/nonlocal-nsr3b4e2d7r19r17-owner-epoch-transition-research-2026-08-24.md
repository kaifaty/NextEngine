# NSR3-B4E2D7R19R17 atomic owner/epoch transition research

Date: `2026-08-24`

Status: `PASS / OWNER EPOCH TRANSITION CANDIDATE / OUTER6 SHADOW RESEARCH NEXT`

## Question

How can the exact R16 suspension envelope become single-use continuation
authority without modifying its bytes, losing the cumulative work ledger or
allowing a partially applied epoch transition?

R17 is an ownership transaction only. It does not resume the nonlinear
solver, execute outer 6 or mutate a physical payload.

## Finding

The R16 v2 envelope is an immutable suspension receipt. Reissuing it with a
new epoch would conflate two facts:

1. what state and work were captured at suspension;
2. who may perform the next bounded continuation.

R17 therefore keeps the exact `540` R16 bytes and root
`069f8bdd...15b8f9` unchanged. A successful transaction consumes their
trusted source owner and creates four separate private canonical objects:

```text
immutable R16 envelope + unconsumed source owner
                       |
                       v
             validate and prepare locally
                       |
                       v
    consumed source owner + epoch grant + receipt + active owner
```

The resulting active owner points to a grant, not to a rewritten physics
snapshot. A future shadow-resume must present both the original envelope and
this grant chain.

## Frozen transition policy

The exact policy projection is:

```text
nextengine.nonlocal.al-owner-epoch-transition-policy|v1|source-consume-once|epoch-increment-one|slice-reset-zero|cumulative-preserve|copy-on-write|validate-before-commit
```

Its SHA-256 is
`12f7a0956a2a7b911530f348834363db43d2ad0119fc5d6a19f9aa5145a06a6c`.

On the exact R16 source, the only ledger changes are:

```text
source owner consumed     false -> true
budget epoch                  0 -> 1
slice HVP                   523 -> 0
```

`cumulative_hvp=523`, all other used counters, all seven limits,
`next_outer=6`, the history root and the entire source envelope remain exact.
The source epoch must not be `UINT64_MAX`, and no grant is issued when the
cumulative ceiling is already exhausted.

## Canonical private objects

All headers are eight-byte magic, little-endian `schema=1` and little-endian
body length. Digests are 32 raw bytes. There is no native padding.

### Source owner

```text
magic  "NEALSOW1"
body   flags u32, expected_source_root[32], expected_epoch u64
size   44 body / 60 total
```

Flag bit 0 is `consumed`. Exact roots are:

- before: `174c0609ed0dc2411fbf99fcda920c7c0c37d71ad51799e5facc397ebd2ca9db`;
- after: `51dc71281dc2c7b77e793e2fa040015764a71a941559195c68594bc61721bb58`.

### Epoch grant

```text
magic  "NEALEPG1"
body   flags u32
       source_epoch u64, active_epoch u64, next_outer u64
       source_envelope_root[32], history_root[32]
       seven u64 limits, thirteen u64 used counters
size   252 body / 268 total
```

Flag bit 0 is `active`. The successor used ledger is exactly:

```text
6,2,25,1,0,523,33,21,504,19,2,21,0
```

Its root is
`266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475`.

### Transition receipt

```text
magic  "NEALORC1"
body   flags u32, source_epoch u64, active_epoch u64
       source_envelope_root[32], grant_root[32], policy_root[32]
size   116 body / 132 total
```

Flags are `3`: source consumed and active owner issued. Its root is
`485b0cb29256f8ba2f0e6f8701925fa292cd409dc802b7189650009933431875`.

### Active owner

```text
magic  "NEALAOW1"
body   flags u32, expected_grant_root[32], source_envelope_root[32]
       expected_epoch u64, transition_receipt_root[32]
size   108 body / 124 total
```

Flag bit 0 is active; the consumed bit is clear. Its root is
`cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2`.

### Composite owner state

```text
magic  "NEALOST1"
body   flags u32, source_envelope_root[32], source_owner_root[32]
       grant_root[32], receipt_root[32], active_owner_root[32]
size   164 body / 180 total
```

The before-state has flags `0` and zero grant/receipt/active roots. The
after-state has flags `7`. Their exact roots are:

- before: `c28e4b7d7ce948a2358d50626af5ac93c354584af350b4c5d3bf37290c8225d7`;
- after: `d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54`.

## Atomicity and idempotence model

The selected research implementation is copy-on-write:

1. validate source bytes and trusted owner;
2. reject duplicate, stale, overflow and exhausted-resource inputs;
3. prepare grant, receipt, consumed source owner and active owner in a local
   candidate;
4. decode/re-encode and semantically validate every candidate object;
5. replace the composite state once.

Injected aborts before preparation, after grant, after receipt and immediately
before commit must leave the canonical before-state bytes exact. Corrupt
grant, receipt and active-owner candidates also fail before commit. Replaying
the source after success observes the consumed owner, returns duplicate and
leaves the exact after-state unchanged.

This proves observational atomicity for the private single-process shadow
projection. It is not a durable database transaction, lock-free primitive or
concurrent CAS. Those require a separately selected storage owner and remain
outside current authority.

## Negative corpus and route order

The fixed corpus has one valid transition and controls for corrupt source,
wrong owner root, already-consumed owner, stale epoch, epoch overflow,
exhausted cumulative resource, four abort points, corrupt grant, corrupt
receipt, corrupt active owner and duplicate replay.

First-failure precedence is source, owner, duplicate, stale, overflow,
resource, grant, receipt, active owner, injected abort, candidate. Every
failure must preserve the exact input state and perform zero solver work.

## Authority boundary and next stage

R17 may construct private canonical metadata and atomically replace its local
shadow owner state. It may not publish a save/runtime schema, change live
budget code, execute resume/outer 6, issue a solve/trial/HVP, commit physics,
start a second substep/macro/trajectory or run timing.

If the frozen corpus passes twice from clean Release builds, R18 may research
one bounded shadow outer-6 resume. That later stage must independently freeze
the active-owner consume, unsliced oracle comparison, exact work ceiling and
rollback rules before implementation.

The executable gate is frozen by the
[D7R19R17 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r17-owner-epoch-transition-contract.md).

The transition passes as recorded in the
[D7R19R17 evidence](nonlocal-nsr3b4e2d7r19r17-owner-epoch-transition-evidence-2026-08-24.md).
