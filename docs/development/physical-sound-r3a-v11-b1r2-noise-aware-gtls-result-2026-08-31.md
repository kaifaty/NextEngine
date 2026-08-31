# Physical sound R3A V11 B1R2 — noise-aware GTLS result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Protocol | [B1R2 noise-aware GTLS](physical-sound-r3a-v11-b1r2-noise-aware-gtls-protocol-2026-08-31.md) |
| Status | `REPEAT_EXACT_REJECT / DEVELOPMENT_FAILED / HOLDOUT_UNOPENED` |
| Decision | `REJECT_NOISE_AWARE_FRF` |
| Product effect | None; authored clips remain authoritative |

## Outcome

B1R2 fixes the B1R noise-floor defect: the weak-force control now has exactly
zero high-band force-valid coverage and returns `OOD_WEAK_EXCITATION`. The
noise-aware GTLS field also predicts twelve unseen development responses at
`0.030031` mean NRMSE, indistinguishable from corrected H1/H2 and substantially
better than impulse or raw-output-modal controls.

The complete revision still rejects on development:

- broad `200…10,000 Hz` force-valid coverage is `0.661254`, below `0.98`;
- only `5/7` poles are retained because the two highest modal neighborhoods do
  not satisfy that broad force-SNR mask;
- the low-coherence corruption is invalidly admitted because its additional
  interference was included in the separately retained response-noise channel
  and therefore subtracted as calibrated noise.

The runner stops before holdout. No real source, quality claim, validator,
atlas, ML model or runtime work is authorized.

## Exact identity and accounting

Two complete runs are byte-identical for every JSON and NPY output.

| Artifact | SHA-256 |
| --- | --- |
| Frozen manifest | `eb56de37e1c693c9e593fca35f2bf630854dba695ed0dc41320184d9e29efe14` |
| Repeated preflight report | `7ee00447a015bbbc506cc68564fd0d94e18c30ba41b732a7da87727bb0c80bc1` |
| Model | `f0bc2707793d4d99c62b70b2727b8a791cddb3c2c06d494efdd6014b7c2edf4d` |
| Run report | `717caabeb0fc3badcdc4d25d3ab9a0a0cbbc72188e29a6fcfa47c76b99b2f0ac` |
| GTLS transfer | `f0fbdf81750e86b7ede53a448e5f4853719d4b4ae702c26382cf258700818f7e` |
| Modal transfer | `0906ab4cf4cda737f0827813d0e31287efd9104ffee5da11434bc6bfb666508e` |
| Input SNR | `4d4c81c876f807830ab3a77cbea123588a0a12efe723cf9ee259c98f8b8bf4e0` |
| Corrected coherence | `42ffe5038459fd3487eaa5ad3bc2530a29504fb2d49ef357029f806c8d9748db` |
| Force-valid mask | `246cc2d32c97cfdd9cc758466a7275e1a7c92b47f38f5c21034d9b495edf9147` |
| Modal reconstruction | `9cf9d91f467fa516bcba61b10fb3272b5ae557a2717c6289348aa98f5396a4dd` |

External roots are `r3a-v11-b1r2-gtls-freeze`, paired preflights and
`r3a-v11-b1r2-gtls-run-a/-b` under the external physical-sound store.

The run evaluates `96` fit and `12` development trials, separately accounts
for `9,216,000` force-noise and `9,216,000` response-noise samples, generates
zero holdout trials, reads zero parent-holdout/real samples and makes zero
network requests.

## Measurements

| Endpoint | Gate | Observed | Result |
| --- | ---: | ---: | --- |
| Noiseless identity NRMSE | `<= 1e-11` | `0` | pass |
| Minimum force-valid coverage | `>= 0.98` | `0.661254` | **fail** |
| Truth modes / false positives | `7 / 0` | `5 / 0` | **fail** |
| Maximum frequency error | `<= 1 Hz` | `0.032524 Hz` | pass |
| Maximum absolute damping error | `<= 1.5/s` | `0.117613/s` | pass |
| Maximum relative damping error | `<= 0.20` | `0.010635` | pass |
| Mean / maximum modal NRMSE | `<= 0.08 / 0.15` | `0.032197 / 0.051644` | pass |
| Mean spectrum RMSE | `<= 2 dB` | `1.980979 dB` | pass |
| Modal / impulse NRMSE | `<= 0.70` | `0.070440` | pass |
| Modal / raw-output-modal NRMSE | `<= 0.80` | `0.038656` | pass |
| Modal / raw-H1 NRMSE | `<= 1.50` | `1.075459` | pass |
| GTLS / best corrected H1/H2 | `<= 1.05` | `1.000009` | pass |

The five retained modes are extremely accurate. The missing bands are a source
admission issue, not a pole-estimator drift:

| Truth mode | Median input SNR across contacts | Valid fraction in `±20 Hz` |
| ---: | ---: | ---: |
| `557…4,051 Hz` | `1,735…20,538` | `1.0` |
| `6,451 Hz` | `145…171` | `0.854…0.932` |
| `9,769 Hz` | `23.4…26.2` | `0…0.005` |

Requiring SNR `>=100` in almost every broad-band bin removes the final mode even
though its local force SNR remains about `14 dB`. This does not authorize
lowering the opened threshold; it identifies the next estimator question.

## OOD interpretation

- Weak force: high-band force-valid coverage `0`, decision
  `OOD_WEAK_EXCITATION`, zero modes. The independent noise floor fixes B1R's
  false observability.
- Low coherence: corrected coherence and coherent coverage both become `1.0`,
  decision `INVALIDLY_ADMITTED`. The implementation classified the injected
  interference as a known calibration component. That exactly subtracts the
  counterexample the gate intended to detect.

The latter is a frozen experiment-design failure. Reclassifying the same bytes
after seeing the result is not allowed.

## Bounded conclusion and next discriminator

Noise-aware GTLS is supported as a stable transfer diagnostic, not yet as the
accepted oracle. It matches corrected H1/H2, preserves unseen-force response and
produces accurate poles wherever the force is admitted. The remaining problem
is the evidence policy around modal neighborhoods, not waveform reconstruction.

Any successor must be a fresh revision and must:

1. define source observability per discovered local modal neighborhood, with a
   separately frozen minimum SNR and partial-band/OOD result, rather than demand
   near-complete broad-band coverage;
2. keep calibration noise limited to pre-impact/stationary sensor noise;
   injected unmeasured interference must remain outside that calibration and be
   detected by residual/coherence;
3. preserve GTLS/H1/H2 controls, fresh phases/seeds, explicit shared poles,
   contact residue uncertainty and the staged development-before-holdout rule;
4. either recover every supported truth mode or explicitly emit
   `FallbackOutOfDomain` for an unsupported modal band—never invent a mode.

B2 remains blocked until a repeat-exact known-truth pass under that revised
local-support policy.
