# NSR3-B4C3A -- level-local canonical staging transaction

Status: `PASS / JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE / B4C3T_DESIGN_AUTHORIZED`

Parent B4C2T selects `JOINT_PRESSURE_B4B2_CONTROLLER_CANDIDATE`; semantic
SHA-256 is `00b67a5506dd2a69fa477c14db0c730140e1747e26fc66736d734ee8cbb44606`
and JSON-without-final-LF SHA-256 must equal
`f33093415620deb819d93c52fec9f969e802cf1341bc1d0a17cbfbaf46512d0a`.

## Identity and roots

```text
identity  joint-pressure-canonical-stage-r0
profile   345eb8876aec66fc5a94a8ea1c6cf0f148c7868327bf5d6bafe96058a30d4087
P1        71fd23dd299bc892254007dc7dcaa25898794ff456b0c0dfdbec0271ac3e9884
P2        0bfc8b62d52e479b724824b2c0e5886a6b89faec8891986e13f2c1382c7a7f87
```

Use the passing NPR1-A `publish_frame`, frame-root and trajectory-root
arithmetic without modification. Link it into the isolated formula-reclosure
target; do not change or resume the stopped NPR1 solver.

## One-frame transactions

Run:

- P1 frame zero at the selected coarse/fine `21/42` substeps;
- P2 frame zero at inactive `1/2` substeps.

Each level starts from the same frozen canonical fixture. After every passing
joint KKT solve, publish exactly one staged frame containing all fluid samples,
then replace position and velocity with exact integer decode before the next
solve. Require sample count/IDs and step numbers exact and maximum publication
error `<=0.5e-6 m` and `<=0.5e-6 m/s` per component.

Measure the publication bound between the exact binary64 solver input promoted
to `long double` and the canonical integer divided by the exact decimal scale
in `long double`. Do not subtract the re-decoded binary64 state: `1/1,000,000`
is not binary64-exact and that comparison can report a false excess at a true
half-unit tie. The canonical quantizer and the frozen half-unit bound are not
changed by this measurement rule.

The unchanged embedded gate must select the fine level. Commit exactly the
fine staged roots as one atomic batch and compute a trajectory root from them.
No coarse root may be appended to the committed batch. The committed frame
count must equal fine substeps and the final committed state must equal the
last decoded fine frame exactly.

Against the corresponding binary64 fine interval report:

- RMS position difference `<=100e-6 m`;
- RMS velocity difference `<=1e-3 m/s`;
- contact feature set exact;
- contact time error no greater than one coarse substep plus 64 epsilon.

These are frozen discriminator bounds, not full-trajectory authority.

## Exact identity and failure gates

Two executions and identity/reverse/coprime-affine publication input orders
must have exact frame roots, trajectory root and decoded state. Input order is
canonicalized by stable `SampleId`; solver order remains the B4C2T order.

Inject separately:

- forced solver failure after two staged substeps;
- nonfinite publication value;
- position-range overflow;
- duplicate `SampleId`.

Each returns the exact typed error, commits zero frame roots and leaves the
pre-transaction state/root unchanged. A failed coarse or fine level cannot
advance committed step numbering.

## Exit

Two reports must be byte-identical. NPR1-A self-test and B4C2T/B4C2Q/B4C1 raw
reports remain exact.

PASS selects `JOINT_PRESSURE_CANONICAL_STAGE_CANDIDATE` and authorizes only
B4C3T full canonical controller physical-bound design. FAIL preserves B4C2T.

No full canonical trajectory, nominal run, CUDA, runtime, schema or production
authority is granted.
