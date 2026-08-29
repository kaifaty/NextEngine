# NSR3-B4E2D7R20R63U exact residual-image certificate evidence

Status: `PASS / TWO_LANE_EXACT_RESIDUAL_IMAGE_CANDIDATE`.

Claim status: `SUPPORTED_EXACT` for both immutable R63S iteration-8
candidates under exact common-operator semantics.
Evidence classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `1448f958`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-exact-residual-image
```

Two independent executions are byte-identical:

```text
stdout sha256   b4d0b5ce0032c81f17d9277bde6ecf8570425d5d9f8475cf6a01fffa92027aa9
semantic sha256 832aa624e33ad947ca288c1507efb6fcf752cb38b6a58f87c3a23966b357d1de
route           TWO_LANE_EXACT_RESIDUAL_IMAGE_CANDIDATE
controls root   b6359e921575171f7af25755af9d49824b43bf620d8a49f0ec1723781f4e78fe
inverse root    9aa68591d9a13ea82cd8e42815865b91e9ffd63c31ca6982d18ecb77070a815a
```

Platform, controls, exact R63S reconstruction, all inverse conversions,
residual/image products, sign comparisons, fixed work and lifecycle gates
pass.

## Exact result

The two unchanged R63S final candidates certify independently:

```text
lane       exact residual inf   exact image inf   certified error
retained   5.4079562458e-19     4.4739959720e-4   4.5149079088e-4
exported   4.8831084575e-19     1.0928340419e-3   1.1028273359e-3
```

Both resolve exactly `24 positive / 78 negative / 0 unresolved` and produce
the same independent sign root
`89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094`.
No expected sign counts or reference vector entered the certificate.

For comparison, the R63S product-norm bounds on the same candidates were
`5.67937e14/5.12818e14`. Evaluating the actual residual image rather than
substituting `||Z|| ||r||` tightens the rigorous radius by approximately
17--18 orders without changing one candidate bit.

Certificate roots:

```text
retained  38bd22c1b779801d807b525acb8a50a4515b7afb16dc84cf0301619358987adf
exported  f1937129df5e6daf0c6b5c2ecd810c8dbf1d52688e7c701ce108f573e18fe29f
```

## Meaning

R63S remains a valid rejection of the coarse product-norm certificate, and
R63T remains a valid rejection of binary128 center refinement past its update
floor. R63U proves that neither rejection is a correctness failure of the
stationary direct-PCG candidates. The candidates already enclose the unique
common solution tightly enough to establish all component signs.

This closes the mathematical correctness gap for one immutable dimension-102
RHS: tangent Gram is the selected finite operator, R63R verifies exact
invertibility, and exact residual-image certification proves the structured
solutions independently of stored dense `H/X`.

## Work and next gate

The stage converts 10,404 inverse entries once and performs 20,808 exact
residual plus 20,808 exact image products. It performs zero candidate updates,
rounded residual uses, R63T center uses, stored dense `H/X` uses, legacy sign
uses, factor corrections, sparse constructions or timing samples.

The next gate should replay the immutable initial state and all eight R63S
updates with the same exact image certificate to locate the earliest stable
passing iterate. After that, research must replace dense exact image
evaluation with a bounded finite/runtime-plausible enclosure before any
sparse or performance work.

R63B--R63T stdout files remain byte-identical to all frozen hashes. Build
passed. No runtime, GPU or production inference occurred.
