# NSR3-B4E2D7R20R63ZM fixed binary trace boundary — revision 5

Revision: `FROZEN / FORMAL_REREVIEW_PENDING / NO_IMPLEMENTATION_AUTHORITY`.

This contract replaces the lost diagnostic-JSON direction. It is staged under an
ignored `target/` directory because the active research worktree is read-only
to the current sandbox. It does not reopen R63ZL and cannot authorize a
recurrence consumer.

## Frozen question

Can one fixed-fixture producer read the exact parent cache once, authenticate
all selected tangent semantics, compute the six direct-index binary128
products, publish a fixed-size canonical artifact with one terminal seal, and
have a separately implemented fixed-capacity checker reconstruct every
selected semantic, component, work span, event and route without invoking any
R63ZL/core authority?

## Trust and work boundary

Package-owned domain work is finite and sealed. It includes:

- every input byte read and hashed;
- every structural length/predicate/skip;
- every selected scalar/vector view and output component write;
- every input/payload/selected/work/product/event/body/result/audit hash call
  and byte;
- every arithmetic term and fixed result-buffer write;
- every artifact field parse and semantic/component/event comparison.
- every producer-model parse/reconstruction operation used to predict work;
- every unused event/root/work/value-slot zero scan on rejection routes.

The implementation uses fixed-capacity storage for the exact profile:

- producer cache slab: exactly `1033625` bytes, plus a reusable `4096`-byte
  streaming scratch for rejected oversize inputs; selected values remain
  immutable views and are not copied into a parallel slab;
- producer result slab: `6 * (2 * 315 + 2 * 102) = 5004` components;
- published output slab: `6 * 102 = 612` components;
- producer event roots: at most `9`;
- checker audit: exactly `420` bytes for every route;
- artifact bytes: exactly `12916` in revision 3 below.

No package-owned `vector`, `string`, stream, JSON object or runtime-sized
container is permitted. POSIX file read/write, the fixed byte-oriented SHA-256
primitive, binary128 arithmetic primitives, terminal tree/result sealing and
the final byte writer form the finite TCB. They are not recursively logged.
The outer mutation harness and process launcher are test infrastructure, not
producer/checker work.

This revision deliberately does **not** claim to count libc/C++ runtime internal
allocator behavior. Instead it prohibits package-owned dynamic allocation in
the claimed path and seals all domain reads/writes/hashes. If pre-freeze review
requires total process allocation accounting, the package stops: that property
is toolchain/runtime-specific and is not silently substituted for domain-work
completeness.

## Canonical raw binary128 identities

A selected scalar is the cache's 16 little-endian bytes reversed to canonical
big-endian order. A vector root is:

```text
SHA256(
  TLV(0x01, "nextengine.nonlocal.r63zm.binary128-vector.v1") ||
  TLV(0x02, count_u64_be) ||
  TLV(0x03, concat(component_binary128_be))
)
```

TLV is `tag_u8 || length_u64_be || value`. There is no `%Qa`, locale,
decimal/hex text or `ostringstream` in the authority path.

The exact parent cache has SHA-256
`23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84`,
size `1033625`, dimension `102`, columns `315` and scalar encodings:

- inverse: `4000624bd73cb7ab659062c1b0cefe41`;
- projected scale: `4000624bd73cb7ab659062c1b0cefe40`;
- sigma: `3ffd71f345eb5c7b206dccda825423fa`.

Raw-byte payload roots in tangent/projected/original/baseline0/baseline1/
baseline2/common2 order are:

1. `03e18bacbb09013618182985941b23496d9e9bab0ca95cb3f9edfe22193e1ada`
2. `2cb58eea5f6b17bc666a5d4b1c0c9ef644b2c10f154f5d651bf24e446ad6a729`
3. `1cabfd31c413d1a36fc299955739196bb50924bebe744ae57c1c99f31a9fc5b9`
4. `950cd25039eafe48f7bb5d5cbdd8b049fe6d14cd0a178338fd50c9d797710a5b`
5. `1573eeeb09ca81f104de3d56e4bf268ac0e9839ce83525472bb9c51d803bf622`
6. `4d59cf5bcaaea440e3f061d00cbb05872d4be75d408e9927050a3ffd1efff392`
7. `1b5581ab5871dcd60b5ff1ce180d0572fcb9725dffe40570595c70fda866bc49`.

The selected-semantic material uses domain
`nextengine.nonlocal.r63zm.selected-binary.v1`, dimensions, all three scalar
bytes, tangent count/root, six `(role,count,root)` triples and the three
declared identities in the same order as the earlier PoC. Its exact size is
`847` bytes and root is
`dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5`.

The prior `%Qa` roots remain external regression oracles only. They are not
fields or authority in the v3 artifact.

## Admission and arithmetic

Admission evaluates and stops at the first failed predicate:

1. parser remains valid and consumes exactly `1033625` bytes;
2. dimension `102`;
3. columns `315`;
4. tangent count `32130`;
5. declared tangent identity equals its frozen legacy identity;
6. inverse bytes equal frozen bytes;
7. projected-scale bytes equal frozen bytes;
8. declared projected-RHS identity equals its frozen legacy identity;
9. declared original-RHS identity equals its frozen legacy identity;
10. sigma positive;
11. sigma finite;
12. sigma exactly equals `1 / inverse`;
13–18. each role count is `102` in role order;
19. raw tangent root equals its frozen root;
20–25. each role payload root equals its frozen root;
26. selected-semantic root equals its frozen root;
27. full source cache hash equals the frozen cache.

Putting the full-source predicate last preserves a first-specific semantic
failure while still catching mutation of every skipped/non-selected byte.

Success creates no new allocation or component copy: it retains immutable
spans into the fixed cache slab. The bundle root binds the selected root and
role mapping.

For each role the producer executes the frozen direct-index two-product/
two-sum schedule:

- `315` inner dots and `102` outer dots;
- `32130` inner, `32130` outer and `32130` separately owned bound-propagation
  terms;
- `102` scale products;
- four fixed result spans;
- four raw-byte vector roots and one product root.

Each product seals this 17-field work tuple, in order:

```text
kernel_calls,input_guard_checks,inner_dots,outer_dots,inner_terms,outer_terms,
propagation_terms,scale_products,fixed_result_spans,result_component_writes,
vector_root_calls,vector_root_metadata_bytes,vector_root_payload_bytes,
product_root_calls,product_root_bytes,temporary_operand_copies,
package_allocations
= 1,1,315,102,32130,32130,32130,102,4,834,4,320,13344,1,466,0,0
```

The product root is the `466`-byte SHA-256 material
`domain,role,exact,no_underflow,rows,columns,input_root,work_tuple,
four_vector_roots`, each field encoded as a TLV. Across six roles the newly
explicit propagation count is `192780`; the other arithmetic totals are six
kernels, `1890/612` dots, `192780/192780` dot terms and `612` scale products.
All `612` values are canonical big-endian binary128 and are independently
recomputed and compared by the checker.

Frozen revision-5 raw identities are:

| Role | intermediate | intermediate bound | value | value bound | product |
|---:|---|---|---|---|---|
| 0 | `3606fee3d39b3e2ec2503075a19fcbaee77df6dc824e1d015d1067ce5391a130` | `8a60a078dc77fd82b47b9e858c6d6aa17a81bcd59017ab3f81d0f9004dc52bbc` | `3afb5864ab94f0ef3838a1e5e39fb5b94ef47744ae2c690accaa5b73a9300fae` | `fb4967e19f1ee83de2f55ee6ec38212fd10fcd00c58941ff95a1f78144268cb9` | `30bdfce6ecb17c6c0760c31b0b2361a6528759f0a15957baa573604a5f50d602` |
| 1 | `60232121203ec24ea854015de9f6e318285f7854f056e193f72dbb34609cf65c` | `a2168f384f268f2d5d8bb24e2241bd118e3b7e1ce30a3ead1c82a30b558c0ab0` | `cbe3032547c1ff03b3f8cc2301f65898d70787d725e8f7f0617b1c8aa3bad5ad` | `f45d565df6ce96e4effd8fb58a6d621812499d3ef09f2b2eedcc077c0715d883` | `d1592b4a806cd9986b3cfc99b128d90eb623eb80d7233f9a55e538a20523ea01` |
| 2 | `175110276598e6091c59feeea5eadacae6f5bddab42ee94855b3e8df9b1451c0` | `834a264bad329ed6b2121b959c7dc88144e35bdfa304bea097ef89261e7ff892` | `8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce` | `1f298596bd5b2627f4687e7682ce240753fdd29fe3c4e7107b1c7d85edf12b0e` | `0844663f1044c00b9397dcb57157e2c60b959458c40f96ba87fd08998bccc1d8` |
| 3 | `07900d4495b5b5d5fe39f74c99a696dd067c30576966c7dbd6f5fa7226696f37` | `9d462d74c3e1765cf9d06b57419c75e9b0f01bb7dc9903d3d4b1362ab61e6971` | `fed92ef46915e7918d7a0419dd8f8cd0f7a7d79db91ce26a272e3c333940216f` | `1b5d3f3f92ed7c0a9f08ddd4791c58504b3e22922d8f06631e2802b689d508f4` | `53806b58536bad09f389ecc07e5fcb2f0aee4132f53e66068993531d41297df9` |
| 4 | `637ff22e5b4f1cd5a519045df7c32ada5a692d6e022706320d0f633233e5b354` | `881d31e1adc3c322405a4c23cc4c83a2c37dcfe8358ce0b9059fa9b17300a955` | `3a327d762f0e0ad869a7bb8e3cd49dcbc4339ae21a20d780ee714b2c3f409701` | `82279c57f7952cf569090a86ffd5c9c0f16729241935cecf6b1347ac38dfab62` | `ae6974942b501204aac8c10f9fd2fdb84e7d00bc8f6bf5bbe4f080db93145c13` |
| 5 | `48e8df3a30a22b6442d3305d19ce53d7d394e9b0003bfacd1052836ad6960201` | `8611c4b4b0e62eaeb08c83c4a492e0b384a2a12f8491f17dc07a36bc3523c459` | `d1e48cc6d4667092ff61dc71a6bfa28f62444073e845453c141939076cae6ca0` | `56e6b2ce0bcad157ea26cb236555d662b5eb5d3aa69c7547dbeb511e98d17c4c` | `f8c208e8c8463685cbd6a0e265fc15990c9600dc25cca8211f9f158f70ddaaa1` |

The product-set root is
`6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b`.
The old ordered `%Qa` product roots all match and remain external
non-regression oracles; they are not artifact fields or authority.

## Fixed artifact layout

All integers are unsigned big-endian. Every unused slot and reserved byte must
be zero. Unknown route/version, nonzero padding, short/long input or trailing
byte is malformed. Valid and rejected artifacts have the same `12916` bytes.
The producer and checker stream-hash every finite input file with fixed memory;
short, trailing, oversize and missing files therefore have distinct size/root
identities and exact read receipts.

| Offset contribution | Field | Bytes |
|---:|---|---:|
| `0` | magic `NER63ZM1` | 8 |
| `8` | version `3` | 4 |
| `12` | total bytes `12916` | 8 |
| `20` | source byte count | 8 |
| `28` | source cache root | 32 |
| `60` | producer route | 4 |
| `64` | selected-semantic root | 32 |
| `96` | bundle root or zero | 32 |
| `128` | product-set root or zero | 32 |
| `160` | event count | 4 |
| `164` | nine event-root slots | 288 |
| `452` | trace root | 32 |
| `484` | terminal result root | 32 |
| `516` | 26 named read/semantic work counters | 208 |
| `724` | admission exact + seven zero pad bytes + 18 counters | 152 |
| `876` | 16 producer seal-work counters | 128 |
| `1004` | seven raw payload roots | 224 |
| `1228` | product count + four reserved zero bytes | 8 |
| `1236` | six fixed product records, `1944` bytes each | 11664 |
| `12900` | product-set hash count and bytes | 16 |

Each product record is:

| Relative offset | Field | Bytes |
|---:|---|---:|
| `0` | exact, no-underflow, role, five zero pad bytes | 8 |
| `8` | seventeen named producer-work counters | 136 |
| `144` | four raw vector roots | 128 |
| `272` | product root | 32 |
| `304` | value count | 8 |
| `312` | 102 canonical binary128 value components | 1632 |

Producer and checker each compile-time assert the complete header chain from
offset `0` through `12916`, every product-relative boundary
`0/8/144/272/304/312/1944`, and the complete audit chain
`0/8/12/20/24/28/36/244/252/284/292/324/356/388/420`.

Failure routes zero every unavailable root/product/value slot. Checker parses
all fixed fields but stops semantic reconstruction at the first unavailable
producer stage.

## Ordered producer transcript

The valid event schedule is exactly:

1. `ReadSpan` — source root plus read-work root;
2. `AdmissionSpan` — selected root plus admission-work root;
3. `BundleSpan` — bundle root plus selected root;
4–9. `ProductSpan[0..5]` — product root plus its authenticated input root.

Every event root uses the same `159`-byte typed TLV material. The trace root
binds route, count and all ordered event roots in `446` bytes; route selection
is not hidden as a tenth event.

Rejected routes are implemented. `READ_REJECTED` publishes one `ReadSpan`;
`ADMISSION_REJECTED` publishes `ReadSpan,AdmissionSpan`; `CANDIDATE` and the
defensive `PRODUCT_REJECTED` route publish all nine spans. Every unused event,
root, product and value slot is zero. The checker reconstructs the route from
the exact cache input before it interprets the producer route.

Producer routes are `0 CANDIDATE`, `1 READ_REJECTED`,
`2 ADMISSION_REJECTED`, `3 PRODUCT_REJECTED`. No other producer value exists.

The serializer first writes the complete fixed artifact with the 32-byte result
slot zero and proves the final offset. A `12983`-byte domain-separated body
hash then binds every artifact byte, including version, padding, unused slots,
work and output components. The terminal result is a `153`-byte typed TLV over
artifact version, route, exact source size, body root and total artifact size.
It is patched into the reserved slot once. Main then performs only the fixed
byte write and mechanical exit mapping; no semantic or layout decision follows
the terminal seal.

The valid baseline work tuples are:

```text
read[26] =
1,1,253,1033625,1,1033625,1,1033625,269,1033625,16,4,170,46,
15,51,87,269,7,560,523872,1,847,0,1,271

admission[18] =
27,0,9,288,3,192,3,48,2,1,1,1,1,429,1,212,0,1

seal[16] =
1,379,9,1431,1,446,1,12983,1,153,6,11664,612,9792,1,0
```

Read fields are open/stat/read/bytes/close/source-size/source-hash calls/bytes,
ten parser counters, payload-root calls/metadata/payload bytes, selected-root
calls/bytes, allocations and work-root calls/bytes. Admission fields are
predicate/failure ordinal, digest/text/scalar comparisons and bytes, scalar
loads, positivity/finite/division work, bundle-root work, work-root work,
allocations and route decisions. Seal fields are product-set/event/trace/body/
result hash calls and bytes, product records and bytes, output components and
bytes, route decisions and allocations. Rejected tuples are path-derived and
are independently reconstructed by the checker; the control report records
their exact values.

The frozen valid baseline seals are bundle
`e1ec9d48e74630ea1698bc7ea4cccd84ecd3fd65e300048f47055e505e72970c`,
trace `8b6e3d1ebd261767aa6722440b9e61975ef49a00cd272a1b75dfc01364b93ce6`,
result `7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3`
and complete artifact
`ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87`.
The nine event roots are, in order:

```text
653cad279282080b5f184d6168fa76c54ab6c840fb587c34bef3b0c503a387c7
a2a4495d53d8298e6ecd2eadede3a0aff454ca24d34739e1b770fa1ed2e8c99e
8c8579a08ee17b5002b07d05c50782adbc096afdd05d7f38461da38c1870da29
d2509f3a36bb4e4ee8553c1f21728737da1448d96e0a97417cea599e2bc05f52
c28fe6e47fd83206219216425a9f5e2986d5c55b5998c4e994237d473e8b8667
5fd9f211e7d463d056c93c2ac042988643f0264414277d264be3948f637a4bf6
9561802aabc41a3d958c4c5718b9142fb105b7db37aa0c6f5f64b20ddefd4a26
af3a53d69d38f9ae629f938b9a331b1edc6bcea5c7d1ffa618fa471a956e3020
759b0e2e6925877d38db5cedcfa827ea0fa4ec6b29dba025b1ab4420028b9357
```

## Independent checker

The checker is a separate executable and shares only the fixed SHA primitive,
binary format constants and binary128 platform primitives. It has its own
cache reader, role table, admission code, arithmetic loops, event schedule and
route classifier. It does not include or link any formula-probe/core/R63ZL
reader, bundle, kernel, validator, classifier, receipt or root helper.

For every cache input it independently parses and executes the ordered
admission gates before consulting the producer-model result. The independent
failure ordinal and route are authority; the separately instrumented producer
model exists only to reconstruct the producer work tuple, and disagreement is
a checker semantic failure. For a valid artifact it additionally performs:

- one exact source-cache parse/hash;
- seven raw payload roots and one selected root;
- all 27 ordered admission predicates;
- six arithmetic products and their raw roots;
- `612` raw component comparisons;
- all nine producer event comparisons, observed body/result reconstruction and
  a separately streamed expected-artifact body/result reconstruction;
- one input-bound fixed checker audit over exact cache/artifact roots and both
  producer and independently reconstructed result roots.

The frozen accepted checker schedule is exactly:

```text
253,4,6,1033625,12916,346,303,52,612,164,62,6,192780,192780,
192780,68,2212278,2,0,530,269,1033625,389,108,0,0
```

in the order cache/artifact reads, file-metadata calls, bytes,
cache/artifact parse checks, semantic/component/work/root comparisons,
kernels and three arithmetic term classes, hash calls/bytes, route checks,
package allocations, checker-audit bytes, producer-model take calls/bytes,
producer-model typed operations, producer-model reconstruction operations and
unused-slot checks/bytes. Its checker root is
`b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4`;
the fixed `420`-byte audit version 3 has SHA-256
`fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80`.

Checker routes are `0 CANDIDATE_ACCEPTED`, `1 READ_REJECTED_VERIFIED`,
`2 ADMISSION_REJECTED_VERIFIED`, `3 PRODUCT_REJECTED_VERIFIED`,
`4 MALFORMED`, `5 SEMANTIC`, `6 WORK`, `7 SEAL`, `8 UNKNOWN`. Routes `0..3`
mean the producer outcome is independently verified and return success;
routes `4..8` reject the artifact. Every invocation, including every control,
returns its own fixed-size closed audit. No control is executed inside the
baseline checker process. The audit layout is magic `0`, version `8`, size
`12`, route `20`, flags `24`, component count `28`, 26 checker-work counters
at `36`, cache size/root at `244/252`, artifact size/root at `284/292`,
producer result at `324`, independently reconstructed result at `356` and
checker root at `388`; the terminal offset is `420`.

## Control corpus

The outer harness owns only byte mutation and process invocation. Cache cases
run the real producer and then the real checker; artifact-adversary cases start
from the real baseline artifact and run the real checker:

- missing file; truncated header, truncated selected vector, near-end
  truncation, one trailing byte and two same-size distinct oversize inputs;
- serialized dimension `102 -> 103` at offset `90`;
- each selected scalar mutated independently;
- one tangent component and one component in each of six roles;
- one selected vector length `102 -> 101`;
- stale cache version;
- invalid early-certificate, profile and late-certificate booleans;
- invalid skipped string length, both certificate collection counts and one
  skipped profile-vector length;
- selected/source/payload/product/product-set root drift;
- product semantic, work, output-value and output-root drift;
- deleted, duplicated and reordered event roots;
- route, event-count, trace and terminal-result drift;
- nonzero reserved/padding byte and unknown route.

Artifact controls recompute every public dependent root they are intended to
reseal before checker invocation. A control earns credit only when the checker
selects the declared first-specific rejection from the full public entrypoint.

Revision 5 has `56/56` controls passing. All cache cases run the real
producer and then the real checker; artifact-only adversarial cases run the
real checker. The corpus includes a complete independent Python reseal of the
valid artifact, fully resealed component/semantic/work/root/event/route drift,
and real producer read/admission rejection paths. All `57` receipts, including
baseline, have distinct audit SHA-256 identities; paired same-size oversize
cache and artifact inputs explicitly prove that input content, not only route
or size, is bound. Producer and checker allocation probes report `0 calls / 0
bytes` for baseline and every applicable control.

## Revision-4 formal NO-GO and batched repair

The revision-4 formal review reproduced the baseline but rejected the package:
mutating certificate/profile booleans at cache offsets `5690`, `775040` or
`1032659` incorrectly failed admission predicate 27 instead of parser
predicate 1; producer-model authority hid the independent parser result on
rejection routes; model reconstruction and unused-slot scans were absent from
checker work; header/product/audit assertions were incomplete.

Revision 5 validates all skipped booleans, derives route and first failure from
the independent parser/admission on every cache-readable path, requires the
producer model to agree, seals six explicit model/unused-scan counters, adds
seven real typed-field controls and proves every layout offset. This is the
single batched repair allowed by the revision-4 protocol.

## Claim ceiling and stop rules

Even a reviewed `GO` supports only the exact fixed-cache tangent boundary:
selected raw semantics, six bit-exact direct-index products, complete domain
work, fixed binary transport and independent replay. It grants no recurrence,
representation family, dynamic builder, corpus/generalization, timing,
runtime, Rust, GPU, cross-target or production authority.

The freeze manifest records completion of these pre-freeze requirements:

1. independent critique must accept the domain-work/finite-TCB boundary;
2. compute and record exact raw product/vector/product-set/event/result roots;
3. implement producer and checker without package-owned dynamic allocation;
4. prove binary size/offset tables with compile-time assertions;
5. execute the complete control manifest, Dev and two clean Release repeats;
6. record exact parent/snapshot/diff/source/cache/contract identities;
7. only then request the single formal revision-5 re-review allowed after the
   batched revision-4 repair.

A remaining semantic omission, hidden package-owned domain operation,
post-seal decision, shared producer/checker authority or route ambiguity closes
R63ZM as `INCONCLUSIVE`. Recurrence work remains forbidden until formal `GO`.
