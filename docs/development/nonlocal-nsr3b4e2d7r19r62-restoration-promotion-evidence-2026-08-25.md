# NSR3-B4E2D7R19R62 restoration-exit transaction evidence

Date: `2026-08-25`

Status: `PASS / RESTORATION_EXIT_TRANSACTION_CANDIDATE / PRIVATE PAYLOAD ONLY`.

Implementation commit: `1b972180`.

Frozen identity SHA-256:
`aebba856f1088d41440d18d0948636941fa9e3fa3c21d93fc776a4e518776422`.

## Result

R62 closes restoration at the private research transaction boundary. The
promoted iterate is bit-exact R43; the exact R58 witness is retained as the
dimensionless normal step for the next TRQP and is not applied to particle
position.

```text
promoted position  0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50
cached normal      040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd
normal applied     false
next phase         ordinary-trqp-ready
```

The passive R61 capture preserves its public stdout byte-for-byte at
`cdfe69bfccba8161708a92636a256313a3efbb27a8f6e7bb1bfa915a04047cb4`.

## Fresh nonlinear owner

A fresh superset is anchored at R43 and its exact filtered support equals the
new workspace topology:

```text
superset root   355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01
workspace root  cfcc7ebbeac9ce8d54093d3af109367e5b68b4aceadf11ed516dbfddf5c2d73f
coverage root   e5521e2832617a271d356d78b9c530d2645896b1dcd1757a3b1f996ee2ca99fb
coverage exact  true
```

The superset pair root happens to remain equal to the prior R42 superset, but
R62 rebuilds and owns it from R43 rather than inheriting that equality.

Fresh nonlinear coordinates reproduce R46 exactly:

```text
f             8.0152688022051136e-13
h             3.7024372773750754e-8
psi           6.8540208964482797e-16
active rows   464
gamma         0.5
h slack       3.5456028079875901e-9
filter root   19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19
```

The conservative filter decision passes by feasibility. Composite AL merit is
not used, consistent with the R45 negative result and filter-SQP semantics.

## Contact, trust and certificate

Both independent geometry traversals pass:

```text
source -> R43 tests               36000
new / worsened                        0 / 0
R43 -> diagnostic normal tests    36000
new / worsened                        0 / 0
normal endpoint root  ab463d5781bebceb53993ce11df1b880480f182ec80a3bc01db78607baabdc7e
```

The second endpoint is diagnostic only. It is never published as position.

```text
next radius       0.0625
normal reserve    0.03125
witness norm      5.0437438283215812e-7
active faces      1290
interval audit    exact
```

The unchanged R61 binary64 owner, evaluated on the same fresh R43 workspace
and cached normal, reproduces the exact zero-positive certificate:

```text
positive rows       0
maximum upper      -1.9354174334860891e-23
upper root    78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf
active root   82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3
```

Binary128 is absent.

## Atomic publication

The valid copy-on-write payload publishes exactly once. Nine controls reject
repeat publication, corrupted position, cached normal, filter, radius,
topology, certificate and publication before all gates.

```text
payload root   5995a2cca7c24b99b328d4c66af7f9c7a710cfe4e684d0af305f81f1e162b41c
controls root  32fc2e02d2ab7ab729cedf47e4337fb8bb8a77172196da08ec953c5f0e57cc14
publish count  1
route root     af19870d606976b5a465c875993974a9303c106c80bed1676a2733283d1e7d2f
semantic       61de28c308a1f9199c954a4c785fe88e767cabd96e71223d3503b0111145da43
```

New work is one R43 workspace lifecycle, one directed JVP, one owner row scan
and 72,000 contact tests. All transitive position, witness, certificate and
old-superset roots roll back exactly. No cached-normal consumption,
tangential solve, multiplier/Hessian update, following outer or runtime state
mutation occurs.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r62-final-a.2izSad
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r62-final-b.s6MaGV
binary SHA-256 b37244bc9fae60ff3de50f79689b34f9a930a9857444479fe76ca29a18f84c55
size           8458544
ELF build-id   9015024fa086f683b9b2b6139e074ea112c54154
stdout bytes   2619
stdout SHA-256 acbfa4befbc91a1c59e0ab75937665e7747112fd118a66eea6753f1206195fae
```

Both clean binaries and outputs are byte-exact; both processes exit zero.
Wall time is not performance evidence.

## Consequence

Preserve R62 as the complete private restoration-exit candidate. R63 may now
research the ordinary next TRQP: consume the cached normal exactly once,
construct a contact-feasible tangential step, apply switching/filter
globalization and define success/rejection rollback. R63 must not rerun
restoration, apply the normal by itself, mutate runtime state or claim
production readiness.
