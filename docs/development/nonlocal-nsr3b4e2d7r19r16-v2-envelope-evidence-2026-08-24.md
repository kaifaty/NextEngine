# NSR3-B4E2D7R19R16 canonical v2 continuation-envelope evidence

Date: `2026-08-24`

Status: `PASS / TOKEN_V2_VALIDATION_CANDIDATE / READ ONLY`

## Outcome

R16 builds the exact predeclared `540`-byte private v2 envelope, reproduces
its independently derived token root, decodes/re-encodes it exactly and
classifies one valid plus 23 fixed negative controls at their expected first
validation layer.

The trusted owner remains unchanged. No epoch transition or resume executes.

## Parent and canonical closure

```text
R16 identity SHA-256  700629514111fa5630d4463abadaedaa55709704fb588cb6db9d748e91badec2
R15 stdout SHA-256    257244d44f3ec4460889968221cf95f0f4ca72090ec6406c7b7349a0710b81b4
R15 semantic          ed209bfa1c6a52411aef21b9f5036fabe46baf68bd2acf95e3b46a424782f0bf
body bytes            524
envelope bytes        540
v2 token root         069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9
```

R15, R14 and all transitive parents remain exact. Magic/schema/length/flags,
fixed little-endian fields and raw digest order round-trip exactly without
native-layout or locale dependence.

## Complete context binding

The envelope binds `6000` particles, outer `6`, provisional `-1`, previous
primal bits `0x3e5c40ff44000000`, theta bits `0x3fc5cccccccccccd`, all
position/dual/outer/predicted/static roots and these policy/history roots:

```text
formula     676900335dcf77e2f8536e69e92597d608baf1bb7c4e514cfe071070e6c88895
solver      193016ce6c55a4b3052d3c980a38364de72cc9b168653bcb7e3a5b9790347697
completion  c6bb447392499aad19c49b69857d43b061c73c9468fe64d097a98ce5ecf99034
history     9754a0bb714011fe4eb637060c6c1eee930e7eb4a1c7db991f664ae40011af7b
```

Every structural limit and used counter from R16's frozen `7/13` field order
is exact, including derived cumulative HVP ceiling `8704` and current
epoch/slice/cumulative HVP `0/523/523`.

## Validation corpus

All `24/24` cases match their expected first route. Corpus receipt root is
`403e00e7dd5d0e43033a8aca95597bf44cc5c6d39b76a68008f2af215209589f`.

Route counts in frozen precedence order are:

```text
canonical  integrity  owner  duplicate  stale  payload
    5          1        1        1        1       3

context  policy  resource  history  valid
   4       3        3         1       1
```

Category controls carry recomputed matching envelope/owner roots. Payload,
context, policy, resource and history mismatches are therefore rejected by
their independent checks rather than by the earlier integrity check alone.

The valid case selects `TOKEN_V2_VALIDATION_CANDIDATE`, proving the validator
does not merely reject every input.

## Work and authority

R16 runs one exact R15 control parent, then performs encoding, decoding and
hash validation only:

```text
new workspace/HVP/model/trial/precision/outer  0/0/0/0/0/0
owner consumed                                 false
epoch transitioned                             false
resume / outer 6                               false / false
```

Payload, parent and owner rollback are exact. No public schema, physics,
budget-code, substep, macro, trajectory, timing or production-policy change
occurs.

## Reproducibility

Contract/research commit:
`2eae5c59` (`research: freeze canonical v2 continuation envelope`).

Implementation commit:
`5c2689c6` (`research: validate canonical v2 continuation envelope`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r16-a.4caFu5`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r16-b.S6JABK`.

Both binaries are `6,282,664` bytes, have SHA-256
`889210c3ce3c239dbed8244552cc41bf600866d45288260615013ebbaca3cb2f`
and GNU build ID `abcc5556af62cfdd4f67d51c5ba2e3558f614b50`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r16-a.XUdsL7`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r16-b.HU8Yyr`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  4,824
stdout SHA-256        d493c68da969911afe7228e26dbfcb32434ebdf3c028e4aaef91a9ca30af689f
semantic result       e0ed3a9bf7f804eb2ee77ee65d9ebdfe10c71745e24a82bc341eec55047cbdcd
route                 TOKEN_V2_VALIDATION_CANDIDATE
```

## Decision

Retain v2 as the private validation candidate and keep v1 permanently
non-resumable. R16 still grants no resume authority.

Research D7R19R17 as one atomic owner-consume and epoch-transition projection:

- compare expected token root/epoch and `consumed=false`;
- consume the old owner exactly once;
- increment epoch `0 -> 1` and reset only `slice_hvp 523 -> 0`;
- preserve cumulative HVP `523`, every substep-lifetime resource/history
  field and the physical payload exactly;
- create one active-epoch state root;
- reject duplicate, stale, wrong-root, overflow and partial-transition
  controls idempotently before work.

R17 may not execute outer 6 or any solver operation.
