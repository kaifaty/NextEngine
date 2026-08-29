# NSR3-B4E2D7R20R63Q full-operator materialization evidence

Status: `PASS / TANGENT_GRAM_FULL_OPERATOR_CANDIDATE`.

Claim status: `SUPPORTED_EXACT` for the complete captured operator. Evidence
classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `e7da2737`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-full-operator-materialization
```

Two independent executions are byte-identical:

```text
stdout sha256   8a64c841bb8b2f139f73701f1615b00d6a77840e26c4d8a86d00e546c0316c59
semantic sha256 48ec75fe0a173dabbfae012372dd43c39d8d8c34ed5f2dddc39fea39ebd0e345
route           TANGENT_GRAM_FULL_OPERATOR_CANDIDATE
controls root   7db4944134972d9b8485b9fbe8048afb9cdb5bc32f912a7fda33b6ef7f183857
matrix root     1e935dbe8a29210c3601038812c7b6b18bcb73b7d82c3f64bbfaf628c982998a
```

Platform, controls, exact R63P reconstruction, all conversions, symmetry,
work and lifecycle gates pass.

## Full operator result

All 10,404 ordered entries of common exact
`H*=sigma A P_d A^T`, stored dense `H` and exact stored-tangent Gram were
audited by integer/exponent arithmetic. No rounded division entered a gate.

```text
metric                         stored dense H      stored tangent Gram
strict entry wins              3,505               6,899
maximum entry error            1.8959075e-36       7.1320848e-37
infinity error norm            1.1217250e-35       5.8675910e-36
squared Frobenius error         1.4730294e-70       3.8318454e-71
```

There are no ties. Both exact global gates are strict: tangent Gram has the
smaller infinity norm and the smaller squared Frobenius norm. The maximum
entry error is also smaller, although that metric was diagnostic rather than
an acceptance threshold.

The common operator infinity norm is `2.5357661e-1`. The exact full semantic
defect caused by `P_d^2-P_d` has infinity norm only `4.9464564e-49`, over
thirteen orders below tangent Gram's finite coefficient/materialization error.
It is therefore globally negligible at this frozen state, consistent with the
R63P weak-direction rejection.

## Exact roots and work

```text
oracle numerator root  221b475869ab34ee919d1d3add4aa993ab361e2dd914be308722f03726c41266
tangent matrix root    1875cca6c378cefcd99d7c2d35a1ae28a25aa74fd15eb89791b5798e94abdca9
semantic defect root   1f96cfbb8890754c656c674b435783d32b830e3639b22b0157cd0837726e92a2
dense error root       4c62725e1b055294221886d1ce6a8812ded4b11aafd94e2ca68cf9657cc6abae
tangent error root     f16c4fda7bf0d0aee0c7df4086c8848ef0992c1102b185494e772f2ade2c146e
audit root             b24da3fed653f7583b2b49a47a698c3421aa09ba68ae2342f60a1d061c7eae09
```

The upper-triangle construction executes exactly 914,022 free-source products
and 1,654,695 tangent products, mirrors 5,151 off-diagonal entries and audits
all ordered errors/norms. There are zero RHS/inverse/factor/triangular solves,
PCG updates, sparse constructions, timing samples or state changes.

## Meaning

R63Q upgrades R63P from a witness result to a complete operator result:
tangent Gram is the better finite representation of the captured physical
derivative under two exact global norms. Stored dense `H` remains a valid
historical numerical system, so R63O is unchanged, but it is no longer the
scientific correctness oracle for future structured solves.

This is research authority, not runtime authority. It does not prove an RHS
solution or justify production integration.

## Next discriminator

R63R should build an approximate inverse from the original binary128 tangent
QR and certify `||I-ZH*||_inf<1` directly against the exact common operator.
This replaces the dense inverse as a verifier without yet solving the immutable
RHS. Only a contractive common-operator inverse can support a later R63S
structured RHS/PCG sign certificate.

R63B--R63P stdout files remain byte-identical to all frozen hashes. Build
passed. No CPU/wall performance comparison, runtime state, GPU or production
inference occurred.

