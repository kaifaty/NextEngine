# NSR3-B4E2D7R19R16 canonical v2 continuation-envelope research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / V2 VALIDATION PASS / OWNER TRANSITION NEXT`

## Question

Can one private, canonical continuation envelope bind the complete R15 resume
context and reject corruption, context drift, stale ownership and duplicate
use before any solver work?

R16 validates a v2 candidate only. It does not consume ownership, advance the
budget epoch or execute outer 6.

## Selected representation

V2 uses a fixed-width canonical binary metadata envelope, not a delimiter-
based string and not a public save schema. Integer fields are little-endian;
floating-point values are stored as exact IEEE-754 binary64 bit patterns;
digests are decoded to 32 raw bytes. No native struct layout, padding, locale
or host endianness enters the root.

```text
header, 16 bytes
  magic[8]       = "NEALCTN2"
  schema u32     = 2
  body_bytes u32 = 524

body, 524 bytes
  flags u32
  next_outer u64
  provisional_index i64
  previous_primal_bits u64
  theta_bits u64
  particle_count u64
  ten 32-byte roots
  seven u64 limits
  thirteen u64 used/receipt counters
```

Total canonical size is exactly `540` bytes. Flags are `13`: suspended,
outer-complete and inner-complete are set; previous-admissible is clear. No
unknown flag is accepted.

## Root order

The ten roots are ordered and mandatory:

1. R14 source-suspension root;
2. position payload root;
3. dual payload root;
4. completed outer-state root;
5. predicted-position root;
6. static-support identity;
7. formula identity root;
8. solver identity root;
9. completion-policy identity root;
10. history-receipt root.

The selected policy identities are:

```text
formula     676900335dcf77e2f8536e69e92597d608baf1bb7c4e514cfe071070e6c88895
solver      193016ce6c55a4b3052d3c980a38364de72cc9b168653bcb7e3a5b9790347697
completion  c6bb447392499aad19c49b69857d43b061c73c9468fe64d097a98ce5ecf99034
```

Each is SHA-256 of the exact identity string selected in R15.

```text
formula     nuv-variational-fcr2+split-static-boundary-r0
solver      nuv-newton-krylov-r0+outer-state-hessian-tape-v1
completion  tiered-grace-residual-model-v1
```

The history receipt is SHA-256 of the frozen domain, R10 transaction root,
R12 accepted-trial root, R13 outer-state root and the completed
outer/accepted/rejected/recurrence/model/residual/workspace/precision/HVP
ledger. Its exact root is
`9754a0bb714011fe4eb637060c6c1eee930e7eb4a1c7db991f664ae40011af7b`.

Its exact input bytes, without a final line feed, are:

```text
nextengine.nonlocal.al-continuation-history|v1|c6a03d43a04ba00cef80b618bb983ab23f59e27d79ad073f327f119b145660cf|58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8|5dd9a07d60cbfe851bdc4c383944a1eb8042f178a821f4a546da33101dd0a19a|6:21:0:504:19:2:33:21:523
```

## Bound resource policy and ledger

Limits, in order:

```text
outer updates              16
inner trials/update        16
HVP/trial                  34
soft HVP/epoch            512
workspace builds          288
precision audits           64
cumulative HVP ceiling   8,704
```

`8,704 = 16 * 16 * 34` is a derived absolute ceiling. It prevents indefinite
epoch renewal without choosing a new physical tuning parameter.

Used/receipt counters, in order:

```text
outer updates               6
inner trials in outer 5     2
last-trial HVP             25
budget epoch                0
slice HVP                 523
cumulative HVP            523
workspace builds           33
precision audits           21
recurrence HVP            504
direct-model HVP           19
residual-model uses         2
accepted trials            21
rejected trials             0
```

The envelope also binds `next_outer=6`, provisional index `-1`, previous
primal bits `0x3e5c40ff44000000`, theta bits `0x3fc5cccccccccccd` and particle
count `6000`.

The resulting canonical envelope SHA-256 is
`069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9`.

## Ownership model

Single-use safety cannot be provided by a self-contained hash: a valid token
can be copied. Validation therefore receives a trusted owner record:

```text
expected_token_root
expected_budget_epoch
consumed
```

R16 only reads it. A later atomic-admission stage must compare-and-consume the
expected root and epoch. Duplicate (`consumed=true`), stale epoch and wrong
expected-root controls must fail before work.

The hash provides deterministic integrity, not authenticity against a
malicious editor. Persisted/untrusted transport still requires a trusted
envelope or MAC and remains outside scope.

## Validation layers and negative controls

Validation order is frozen:

1. canonical magic/schema/length/flags and exact decoding;
2. envelope root and trusted expected-root integrity;
3. duplicate owner state, then stale epoch;
4. payload count and independently recomputed position/dual roots;
5. outer/predicted/theta/static context;
6. formula/solver/completion policy identity;
7. resource limits, used counters, additive ledgers and hard invariants;
8. independently reconstructed history receipt;
9. validation candidate.

The bounded corpus contains one exact valid envelope plus fixed controls for
bad magic, schema, length, truncation, flags, token/owner root, duplicate,
stale epoch, position, dual, particle count, outer context, predicted, theta,
static identity, all three policy roots, limit, used counter, cumulative
invariant and history receipt. Category controls carry a recomputed matching
envelope/owner root so they cannot be rejected merely by the integrity layer.

Every case must report its exact first route and zero new workspace/HVP/model/
trial/precision/outer work.

## Authority boundary and next stages

R16 may reproduce R15, construct/decode v2 bytes and run the validation
corpus. It may not publish a schema, mutate the live budget, consume the owner,
increment epoch, resume, execute outer 6, start another solve/substep/macro/
trajectory or run timing.

If R16 passes, R17 may research an atomic owner-consume and epoch-transition
projection. Only after that passes may a separately frozen stage consider one
shadow outer-6 continuation and compare it against an unsliced oracle.

The executable validator is frozen by the
[D7R19R16 v2 envelope contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r16-v2-envelope-contract.md).

The canonical envelope and validation corpus pass as recorded in the
[D7R19R16 evidence](nonlocal-nsr3b4e2d7r19r16-v2-envelope-evidence-2026-08-24.md).
