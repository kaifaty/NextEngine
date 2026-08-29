# NSR3-B4E2D7R20R63ZN fixed-artifact initial recurrence research

Status: `REVISION_2_CONTRACT_FROZEN / EARLY_REJECTION_MATRIX_PASS /
LATE_ARITHMETIC_REJECTIONS_AND_FULL_CONTROLS_PENDING`.

## Decision

Use R63ZM as an immutable parent product boundary for exactly the initial
original-input binary128 recurrence prefix. The next package will independently
solve `x0`, prove correspondence with R63ZM role 2, consume that role's value
as `H*x0`, and independently derive `r0`, `z0` and a certified-positive
Dot2 enclosure for `rho0`.

Do not attempt a complete recurrence from the six parent products. The parent
contains products of projected RHS, original RHS, baseline states 0/1/2 and
common-projected state 2. It does not contain products of the dynamic search
directions `p0` and `p1`.

The complete frozen boundary is the
[R63ZN revision-2 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zn-fixed-artifact-initial-recurrence-contract.md).

## Competing hypotheses and belief update

| Hypothesis | Discriminator | Result |
|---|---|---|
| H1: all three recurrence operator calls can be read directly from R63ZM | compare R63ZM role mapping with the frozen `H*x0/H*p0/H*p1` schedule | falsified: only role 2 can correspond to `H*x0`; no role is `p0` or `p1` |
| H2: later direction products can be recovered bit-exactly from state-product differences | test whether rounded binary128 state updates preserve exact `x1-x0=alpha*p0` | falsified by a one-scalar exact counterexample |
| H3: role 2 can drive a non-circular initial prefix | derive `x0` from factor/RHS before reading role 2, compare all components, then consume only its product value | selected; smallest causal consumer |
| H4: immediately rebuild the complete dynamic recurrence | compare required trust/work surface with H3 | deferred until H3 passes; it would add two dynamic products, two updates and certificates at once |

Before inspection, H1/H2 were plausible because the mathematical operator is
linear and R63ZM transports products of all three baseline states. After the
role trace and rounding counterexample, their probability is effectively zero
for a bit-exact checker. H3 is now the only bounded path that consumes rather
than recomputes a parent operator result without trusting a cached state as the
producer of `x0`.

## Revision-2 arithmetic trace correction

After revision 1 froze but before any R63ZN source existed, direct inspection
of the reviewed R63ZC endpoint showed that its triangular solves use the
frozen compensated binary128 accumulator, residual components use two-term
Dot2, `rho0` uses 102-term Dot2, and positivity is
`rho0.value - rho0.bound > 0`. Ordinary subtraction/summation would define a
different recurrence even if it happened to produce nearby values.

Revision 2 corrects only that numerical schedule. It keeps the same parent,
role mapping, prefix boundary, independent candidate/checker requirement and
claim ceiling. This correction was committed before implementation so no
observed R63ZN endpoint influenced the method.

## Author preflight observation

A bounded external preflight implemented the revision-2 arithmetic directly
over the exact cache and R63ZM artifact. It is not the contracted package: it
uses frozen offsets after external whole-file identity checks and publishes no
receipt, independent checker or controls. Its only purpose was to determine
whether the selected prefix is numerically reachable before building the full
trust/work boundary.

Observed twice-identical stdout:

```text
permutation_exact=1
start_exact=1
x0_matches=102
residual_exact=1
residual_no_underflow=1
residual_products=204
residual_sums=102
preconditioned_exact=1
rho_exact=1
rho_no_underflow=1
rho_products=102
rho_sums=101
rho_positive=1
rho_value=+0x1.f4d792f082e81eb9febf63958eed00000000p+3
rho_bound=+0x1.f4d792f082e8a8eadc3d21ccdada00000000p-109
rho_lower=+0x1.f4d792f082e81eb9febf63958eeb00000000p+3
```

Exact identities:

| Item | SHA-256 |
|---|---|
| preflight source | `1ff6553e822999c1c75e05e93a5c5f306445c6864592fe265eb47e423b7ff576` |
| strict Release-style binary | `606994dba3726b94299e4f22179741c5662a9d528ca264fe859ff1febfd6b861` |
| stdout, run 1 and run 2 | `dccaf8c156cd0d310185eada4f79e13c6bb071122cf8d999ac6245644140fc5f` |
| cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| R63ZM artifact | `ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87` |

This establishes reachability only: the independently coded factor solve
matches all `102` role-2 input components, the consumed product produces an
exact/no-underflow residual, the second solve is exact, and the Dot2 lower
bound is positive by a very wide margin. It does not establish parser,
identity, work, receipt, route or checker correctness and therefore grants no
R63ZN scientific claim.

## Standalone candidate checkpoint

The first tracked implementation replaces the offset-only diagnostic on the
accepted path. It independently:

- reads and hashes the exact cache, R63ZM artifact and R63ZM audit;
- verifies the named parent formats, routes, roots and role-2 record;
- traverses the entire cache with its own fixed-capacity reader (`168` reads,
  `1033625` bytes, `87` structural predicates);
- selects factor/permutation/inverse/RHS/baseline-0 from parsed spans rather
  than frozen cache offsets;
- reproduces the revision-2 arithmetic; and
- writes a fixed `1120`-byte `NER63ZN1` revision-2 receipt with `36` work
  counters, seven ordered events, one trace root and one terminal result root.

Dev and two Release invocations write no stdout and produce the same receipt:

| Item | SHA-256 |
|---|---|
| candidate source | `71887cabc7d89db460825a211c72f229a5323f9ba415825f2a7d0b3390ebfbc9` |
| Release binary | `2bfa5e4d417571d7851ac312e488db784b898999b1cafb0fa4ed2987d403acc0` |
| Dev binary | `a0a8a5f49cb3ed0defa1ab71d74e214f47e45fb04f1144bd1468f2ed5933c042` |
| Dev and both Release receipts | `91810c72cefd6c9cd877d88b8aed1641af17c05b45a10d267360e45ea3dfc245` |
| terminal result | `6de1eca2c37017a705be8813930e947f58f332a8d7044fcbe4c2ba65ad0a4283` |

A one-byte parent-audit flag mutation returns the current defensive identity
failure exit `66`. Rejection receipts are not implemented yet, so this is not
a contract control and receives no review credit.

This checkpoint is author evidence only. The work tuple has not yet been
independently reconstructed, rejection routes are incomplete, no separate
checker exists, and the full mutation corpus has not run. The candidate must
not be described as reviewed, correspondence-closed or production-ready.

## Accepted-path independent checker checkpoint

The first independent work reconstruction rejected the initial author receipt
as incomplete before any review request was frozen. The candidate hashed all
three inputs twice while reporting three hashes, omitted the `parent-set` root,
under-counted the trace hash by nine bytes, and its 36-field tuple had no room
to own all required input/domain checks, residual Dot2 calls and receipt
construction. The `91810c72...c245` receipt remains preserved above as
superseded author evidence and receives no correspondence credit.

The repaired candidate hashes every input once and reuses the digest, checks
all selected factor/binary128 values and permutation properties, owns both
solves and all finite/exact/underflow checks, and publishes a 48-field work
tuple in a fixed 1,216-byte receipt. The separately compiled checker shares
only the fixed SHA primitive. It uses a distinct cursor, factor solve, Dot2,
root construction, work model and route classifier, rebuilds every accepted
receipt byte, and emits a fixed 548-byte checker audit.

Two Release candidate/checker pairs are byte-identical and allocation-free:

| Item | SHA-256 |
|---|---|
| candidate source | `c5005d5e2721c471b984f85e1bf350ee1daff6b0bef2fa0e91ade0677876ed80` |
| checker source | `4ba3acb89e255b1ddaca55543f17772df48523d0b0b6dbeecc1ec4464fac8673` |
| Release candidate binary | `40b5d12309f40bc8a35fe7d1fa325b91bf4fecda0e674b2bbb82635f6b58bd97` |
| Release checker binary | `ae22bcc791cc3b34b30401a1ed79ab4337fad2cc7e945cde41eaddfac052c497` |
| both candidate receipts | `6b6db341431221b9eba199d8b818bd0809bafedfc99cbbd46b9dc622441a8a06` |
| both checker audits | `e7934c3a0a106dc556286d73e12f5f615ce0ec40a96edac5f4205932f19f385a` |
| trace root | `2a6d3ccb5c27594b2aa526b7bc29a375565029e84a7aa178567bfef90461cba6` |
| terminal result root | `3e3cb36341fb09f6e36834e01b3782ef990baea8857d2fb529510ca4d105902b` |
| checker root | `90aa280005078f8a81e967db642e835e9bc50d67a52aba01108acfc26209412f` |

Both stdout files are empty and both allocation probes report zero calls and
zero bytes. Six bounded smoke mutations produce distinct sealed checker
audits and the first-specific routes: exact-input drift `2`, malformed receipt
`3`, semantic drift `5`, work drift `6`, event drift `7` and seal drift `8`.
These are checker-discrimination smoke controls, not the contract's complete
mutation corpus. This accepted-only checkpoint is superseded by the early
rejection checkpoint below.

## Early rejection receipt checkpoint

The candidate now publishes fixed 1,216-byte route-specific receipts rather
than process-only errors for the first four frozen classifier branches. It
stops input work in first-failure order, records observed sizes and only roots
files read at their exact size, zeros all unavailable semantic/event slots,
and seals route-specific work, a zero-event trace and the terminal result. The
checker derives the expected route before consulting the receipt and
independently reconstructs every rejection byte.

The complete early-input matrix passes through the public Release entrypoints:
missing, short, trailing and same-size-mutated cache, parent artifact and
parent audit. All 12 candidate invocations exit zero with routes `0/1/2/3` as
applicable; all checker invocations exit zero with verified routes `1/2/3/4`;
all 12 checker audits are distinct. Five separate malformed/semantic/work/
event/seal receipt mutations reject at checker routes `6/8/9/10/11`.

Two accepted Release pairs remain byte-identical and allocation-free:

| Item | SHA-256 |
|---|---|
| candidate source | `955f12c1a5e515e5ab011c8b95da806dc7d38a5ddae36c43774d400868cd9019` |
| checker source | `4adc0163e849fed98c47eb65276135ab8f5fb7c6e8867f1b91d4496afad5663e` |
| Release candidate binary | `4b3784c1c345ac03c12c8b7b788f46654339cd4896db6eedd64cfd5624327a4e` |
| Release checker binary | `273ecc6656cd41426a07192e29d8dc7450da705adefb02c73142697567055532` |
| both accepted receipts | `4229e5509a2b153dfd1bb6ce9f94642a9bef62d55e36d784911b577f051a296c` |
| both accepted audits | `16875a04bd8e44228686ee90117e576dd7f26687240bf054e0a9a17b4bf7dfb2` |
| accepted trace root | `b99c8d707cd1333ba482ab63c5d28319d7d32660cf896e62fa4947451e888246` |
| accepted terminal result | `166da96ca4ba7dbb5845a5074b8b81349609011f315123ad17f2344b6d20ff4f` |
| accepted checker root | `5cebbd94855b6296ad5c2e4691a44b5aa6c1c0a851234da9e6678951b6ba3924` |

Both stdout files remain empty and every baseline/control allocation probe
reports zero calls and bytes. This is still author evidence. Routes 4--6 need
independently reconstructed `x0` mismatch, nonfinite/underflow prefix and
nonpositive-lower-bound controls; every work counter still needs individual
resealed mutation; event deletion/duplication/reordering and fresh independent
review remain pending. R63ZN grants no admitted recurrence result.

## Complete author package checkpoint

The final author package supersedes the accepted-only and early-rejection
checkpoints above. The candidate now has all eight frozen terminal routes:
input, parent-artifact, parent-audit and cache-semantic rejection; `x0`
correspondence rejection; prefix exactness/underflow rejection; nonpositive
`rho0` rejection; and accepted prefix. Selector parsing occurs only after the
three exact inputs are admitted, so no late control adds work to an early
route.

The fixed revision-3 receipt is `1368` bytes. Its `67`-field candidate ledger
owns rounding mode, logical and physical file operations, exact bytes read and
hashed, cursor work including the terminal predicate, all selected-field
decodes/checks, two solves and compensated dots, parent-domain role-2
comparison, every semantic/structural/event/trace/result root, classifier
decisions, control-only arithmetic, receipt construction and the zero-allocation
claim. The independently coded checker emits a `676`-byte audit with a separate
`48`-field ledger, including receipt padding scans and audit I/O. It derives the
candidate route before reading the published route and uses its own cursor,
solve, Dot2, roots, work model and final classifier.

The external mutator and Python runner are test apparatus only. They are not
linked into or invoked by candidate/checker authority. The runner exercises
`116/116` cases:

- baseline plus all missing/short/trailing/same-size input failures;
- typed cache and parent-artifact mutations;
- `x0`, underflow, nonpositive-`rho0`, independent 3-by-3 solve and
  state-difference entrypoint controls;
- all six mutable semantic roots and every one of the 67 work fields under
  complete resealing;
- event deletion, duplication and reordering, route, malformed receipt and
  terminal seal mutation; and
- three wrong-selector/receipt pairings.

All `116` checker audits are distinct. Two Release runs and two independent
clean Release builds produce the same `41698`-byte report
`745eafba951678bb584e79bf569b554468355c17ff564fdd3e553f10bc264a02`.
The Dev report has different executable identities, but its complete control
record array is byte-for-byte equal to Release. Every candidate and checker
invocation has empty stdout and the exact 50-byte allocation receipt with zero
calls and bytes.

| Item | SHA-256 |
|---|---|
| candidate source | `e782332da862d0b0347af96b227e1e59bc9209de315ac249c92f3fd924d642c4` |
| checker source | `10ddc3b0ef4414d33d40a8da6340d512d5a0872a689b770d99cabb82b40a62cb` |
| receipt mutator source | `4259ade66c0984ff08869e41c34302632eeac0fa9ce70988ae97376ebc46f707` |
| control runner source | `592075606615df76d5c28b89206c5158fabce74f38036de9d3e2f3e8c00b5b49` |
| Dev candidate binary | `725ab44cbff15d730a77d1b022a5edd233911150be30e3aa9b137f633f59fcc5` |
| Dev checker binary | `46a9c387e46cd6bac72c728da913321d214e8257e12d8c8c3f56c4ef36dca1f2` |
| Dev mutator binary | `7e9b3759fad60b89776093834864808bb6420c843b1c67f5dca5b9c879be5a41` |
| clean Release A/B candidate binary | `ea497b3306d60a1ff9dab6cf08f59285bb5190f4145581fa61e9fc8815499f9e` |
| clean Release A/B checker binary | `e3d5a729593c71c6edb5e9e34e4bba838f65d870133332a7d1bffbdfd7ae355b` |
| clean Release A/B mutator binary | `bc2ef0f22337806ddaa8efb611bf93b77540aed123bc457435f822d138a7d125` |
| accepted receipt | `e0015763eec6f77854568505da47d5874de389ecd505c91c5537e6b12a9af8da` |
| accepted checker audit | `d0a98ac66b06847278469bf36f7781dd2eaf886bd157cc8fbbb3cf9e739d2cc3` |

The accepted trace is
`6aa6f881294eb5b68b20e778ed608a4ad4499a12207de7474a6832a68ebad908`,
the terminal result is
`3303938b7b2dca351324302e1da0ddb43f3059ac7b6ad2238dfde943fec49963`,
and the checker root is
`000de639269409d6edc6862bbab167316e030c3480b7492987b345e924d82b32`.
ASan/UBSan with leak detection disabled passes the same `116/116` corpus;
LeakSanitizer is not claimed. A fresh R63ZM checker run reproduces its exact
420-byte audit `fd4bcf00...1f80`, with empty stdout and zero allocations.

This is hash-closed author evidence, not an independent verdict. Until a fresh
read-only review returns `GO`, R63ZN is not reviewed and grants no recurrence,
certificate, portable-representation or production authority.

## Exact structural evidence

R63ZM's raw probe maps cache vectors as:

```text
role 0 = projected RHS
role 1 = original RHS
role 2 = baseline solution 0
role 3 = baseline solution 1
role 4 = baseline solution 2
role 5 = common-projected solution 2
```

The frozen recurrence schedule independently computes the factor solve
`x0`, applies the first operator to that result, forms `r0`, solves `z0`, and
then applies later products to `p0=z0` and derived `p1`. Thus role 2 has a
well-defined admission use only after the consumer's `x0` equals its input.
Roles 3/4 authenticate `H*x1/H*x2`, not `H*p0/H*p1`.

## Binary128 counterexample to state-difference reconstruction

The bounded control used GCC binary128 with strict flags and:

```text
x0 = 1
p = 2^-113
alpha = 1
x1 = fl(x0 + alpha*p)
```

Observed output:

```text
x1_equals_x0=1
state_difference=+0x0.000000000000000000000000000000000000p+0
direct_direction=+0x1.000000000000000000000000000000000000p-113
equal=0
```

Exact identities:

| Item | SHA-256 |
|---|---|
| source | `8bea012b86e32507cfff3817895edfc1d5b503ee15f0a3c02022d2a02609d3b5` |
| Release-style binary | `3ae0a7560870079dace67167420367b215e6c0d240362e103c2a2f85a91a8dbc` |
| stdout | `875f5ea6cfb3e4294b14c99c3645ccda4a4a3c1f97a6777fc9ca7fd1abbba0ea` |

This does not claim the frozen 102-dimensional update hits this exact tie. It
is a constructive refutation of the general identity required to make
state-product subtraction a bit-exact replacement for a direction product.
Using it in the checker would therefore require a new error-bound contract,
which R63ZN explicitly forbids.

## Architecture and claim boundary

The work remains offline serial numerical research. SPEC-38 and ADR-076 are
still `Proposed`; ADR-081 guardrails remain binding. The package is not a
public contract or production consumer, uses no runtime state, and cannot
authorize a portable representation or wider roadmap stage.

The smallest next action is a fresh read-only review of the exact frozen
candidate, checker, mutator, runner, contract, input identities, binaries and
control report. If review finds shared authority, an unowned semantic or work
path, an ambiguous first-failure route, a resealed mutation that passes, or an
identity mismatch, repair the frozen package before interpreting the prefix.
Do not import later cached states or widen the claim.
