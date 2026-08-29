# NSR3-B4E2D7R19R17 atomic owner/epoch transition evidence

Date: `2026-08-24`

Status: `PASS / OWNER_EPOCH_TRANSITION_CANDIDATE / SHADOW ONLY`

## Outcome

R17 preserves the exact R16 suspension envelope, consumes its private trusted
source owner once and emits a separately rooted epoch grant, transition
receipt and unconsumed active owner through one copy-on-write composite-state
replacement.

All precommit failures preserve their exact input state. Replaying the source
after success returns duplicate and preserves the exact committed state.

## Parent and identity closure

```text
R17 identity SHA-256  22d564118bbf04dd49865629bb4ad700df2f1bfadc91b9510de2acd7edf51e73
R16 stdout SHA-256    d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f
R16 semantic          e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd
transition policy     12f7a0956a2a7b911530f348834363db43d2ad0119fc5d6a19f9aa5145a06a6c
```

R16, R15, R14 and all transitive parents remain exact. The immutable R16
envelope remains `540` bytes at root
`069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9`;
position and dual payload roots are unchanged.

## Canonical target closure

Every object decodes into a fresh value, matches its independently derived
semantics and re-encodes byte-exactly:

```text
object                 bytes  SHA-256
source owner before       60  174c0609ed0dc2411fbf99fcda920c7c0c37d71ad51799e5facc397ebd2ca9db
source owner after        60  51dc71281dc2c7b77e793e2fa040015764a71a941559195c68594bc61721bb58
epoch grant              268  266f8ac864d9a83e7bf71fa1df0b4dc33fe964d6064ced058287f04e61f4d475
transition receipt       132  485b0cb29256f8ba2f0e6f8701925fa292cd409dc802b7189650009933431875
active owner             124  cc6b9e854ca39d940f826899f02827765f293b96bbf9429f7b5b7caceddae9b2
owner state before       180  c28e4b7d7ce948a2358d50626af5ac93c354584af350b4c5d3bf37290c8225d7
owner state after        180  d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54
```

No native layout, padding, locale or host endianness participates in these
roots.

## Ledger transition

The successful transition changes only:

```text
source owner consumed   false -> true
budget epoch                0 -> 1
slice HVP                 523 -> 0
```

Cumulative HVP stays `523`; all other `13` used counters, all seven limits,
history, payload, policy and `next_outer=6` remain exact. The new active owner
is unconsumed and points to the epoch-1 grant.

## Negative and atomicity corpus

All `15/15` fixed cases select the frozen first route. Corpus receipt root is
`b41bf1040bae4d92626ad677fc251f6b1676d800c382ce2fb6becdd3e272e00f`.

Route counts in precedence order are:

```text
source owner duplicate stale overflow resource grant receipt active abort valid
   1     1      2       1      1        1      1      1      1     4     1
```

The four injected abort points are before preparation, after grant, after
receipt and immediately before commit. Grant, receipt and active-owner
corruption also fails before commit. Every negative preserves its exact input
state; duplicate replay preserves the exact successful after-state.

## Work and authority

R17 runs one exact R16 control parent, then performs canonical metadata
encoding, decoding, validation, hashing and one local state assignment only:

```text
new workspace/HVP/model/trial/precision/outer  0/0/0/0/0/0
resume / outer 6                               false / false
physics/public commit                          false / false
```

This is observational atomicity in a private single-process shadow
projection. It is not concurrent or durable CAS. No public schema, live
budget-code, production policy, substep, macro, trajectory or timing changes.

## Reproducibility

Contract/research commit:
`b3f49858` (`docs: freeze D7R19R17 owner epoch transition`).

Implementation commit:
`125f1526` (`research: add atomic owner epoch transition`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r17-a.ugDf1y`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r17-b.vtjshs`.

Both binaries are `6,372,160` bytes, have SHA-256
`83db7532aede869d4d6282a16b4023fe9ef19735ad76ff10be111f37be53d68e`
and GNU build ID `a5ec781faee5fb9873b8f4f4a3450a6e6c159b41`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r17-a.mLfXdJ`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r17-b.reIvWt`.

Both exit `0`, emit empty stderr and reproduce the same `4,803` stdout bytes:

```text
stdout SHA-256  d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447
semantic        c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173
route           OWNER_EPOCH_TRANSITION_CANDIDATE
```

## Disposition

R17 closes the private ownership/epoch barrier. It does not make the solver
resumable or production-ready. Research/freeze R18 next as one bounded shadow
outer-6 resume with active-owner consume, exact unsliced-oracle equivalence,
explicit work ceiling and rollback. Do not implement resume before that
contract is frozen.
