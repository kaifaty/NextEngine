# NSR3-B4E2D7R19R56 joint-witness certificate evidence

Date: `2026-08-25`

Status: `PASS / JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED / ROLLBACK ONLY`.

Implementation commit: `7c4a6b30`.

Frozen identity SHA-256:
`d7fd1d53edb0f1467c99fd6b76e42c96c1775cda9ab5ea95821c89301acfef6e`.

## Result

R56 reproduces the exact R55 cycle-64 witness and compares its residual in
fresh pair-once binary64, captured directed binary64, compensated binary128
term-fold and full frozen-coefficient binary128 representations.

The fresh pair-once and captured directed binary64 images are bit-identical.
All 308 binary64 raw-positive rows remain strictly positive in full binary128;
all other 5692 rows are resolved negative and no row has unresolved sign:

```text
representation                         positive rows
fresh pair-once binary64                         308
captured directed raw binary64                   308
compensated binary128 term fold                  308
full binary128 recomputation                     308
full binary128 resolved negative                5692
full binary128 unresolved                           0
pair/full128 sign disagreement                      0
directed bound-only positive                     168
```

The current directed certificate therefore has 476 positive upper rows: 308
with a true positive raw residual and another 168 positive only because of the
unchanged enclosure. The selected frozen route is
`JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED`.

## Worst row and numerical attribution

The maximum is row 182 in every raw representation:

```text
pair/direct binary64 raw       3.8207058903518072e-20
compensated term-fold raw      3.8206790159947496e-20
full binary128 raw             3.8206759623566776e-20
full binary128 lower bound     0x1.68da46c132e21e22852be33077c4p-65
full binary128 upper bound     0x1.68da46c132e21e2651c41ccf883cp-65
current directed upper         4.1425104588214889e-20
```

The lower bound is strictly positive, so this is neither binary64 cancellation
nor an unresolved enclosure sign. Pair-once and directed binary64 roots are
identical and their maximum difference is exactly zero, rejecting operator
alignment as the cause. The largest term-fold/full-recompute difference is
`6.6985431309868752e-25`, without a sign disagreement.

R55 is consequently an extremely accurate contact-feasible approximation, but
not a certified feasible point. The remaining raw residual is about six orders
of magnitude below R52-64 and is still mathematically real.

## Work, roots and rollback

New work is exactly one moved-workspace build/release, one fresh pair-once JVP
and one compensated binary128 directed-row traversal. The captured directed
image is reused. There is no new Dykstra cycle, density sweep, box block,
projection, basis/Gram construction, HVP, nonlinear trial or outer.

```text
pair/direct  98996adbbc3648b74616197e416f4873b3291d42c75bf7dad00237ac1d75f72e
term128      aedd247b65e8a20492605d15e9e3433065eaae43bb8cab25d2885e7cd499c605
full128      c8fd14c0a32e6c79557d407bfc2ec8d2b45b8f5d2bcc4a1f23fa92ce464ffd93
comparison   2fbef7b796b3af18cbae1e032b4ce8330be93deb160a06893ac8237f356806f9
dense        4ecee1823c3f8f57325ac25f6ea2a3249a943f52f4946d37ce66826420d07e85
routes       ee1c12c1d8ec20f5ecc52cacdecbe5a572eef1b0a6692b661d82a1706a989905
semantic     625f8db0a1be011dd36737743487ad9ee54e629bfb8926c0fb130120c5b9f7ca
```

R55 remains byte-exact at stdout SHA-256
`fce9bec7ef370caed4904cedf61851f0fb87bf539960134fabefbb9dc94d6db8`.
All parent state, witness, correction, box dual, topology, filter, trust and
restoration state roll back exactly. Binary128 remains an offline oracle;
runtime and production authority remain false.

## Pre-execution control repair

The first executable attempt stopped at `JOINT_CERTIFICATE_WORKSPACE_REJECTED`
before either diagnostic pass. The frozen identity text was unchanged, but its
documented digest had been computed incorrectly. Replacing only that digest
with the actual no-newline SHA-256 `d7fd1d53...acfef6e` admitted the frozen
identity. Algorithm, physical source, work budget and classification precedence
did not change. The pre-execution attempt receives no scientific credit.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r56-final-a.VvRpqg
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r56-final-b.ltC6oe
binary SHA-256 74ae6451c94c4288c2992b1f03acf337f45ba85bb5645120b989c4469d9b4eec
size           8192656
ELF build-id   787981ea7c6f429caa704be6d1d42467f5f52e4f
stdout bytes   2809
stdout SHA-256 e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa
```

Both independent Release binaries and concurrent outputs are byte-exact. Wall
time is not performance evidence.

## Consequence

Preserve grouped-box Dykstra and the exact cycle-64 state. Do not weaken the
certificate, fit a zero tolerance or promote the witness. The next research
must determine the smallest bounded terminal correction that removes the true
raw residual while respecting the box and then re-audits the unchanged
directed enclosure. A finite continuation-depth probe and a face-aware
primal-dual/active-set polish are candidates; neither is authorized until its
cost, state ownership and outcome-independent selection are frozen.
