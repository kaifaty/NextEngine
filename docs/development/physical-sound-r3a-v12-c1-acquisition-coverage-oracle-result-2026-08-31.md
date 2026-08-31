# Physical Sound R3A V12-C1 acquisition-coverage oracle — exact result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `PASS_KNOWN_TRUTH_FRF` |
| Next authorized step | `C3_SOURCE_ROLE_FREEZE_IF_C2_PASSED` |
| Protocol | [V12-C1 protocol](physical-sound-r3a-v12-c1-acquisition-coverage-oracle-protocol-2026-08-31.md) |
| Research basis | [Coverage and source research](physical-sound-r3a-v12-c1-coverage-and-source-research-2026-08-31.md) |
| Product authority | None; synthetic external research only |

## Claim

The frozen coverage-certified common-pole estimator passes one fresh
known-truth revision. It recovers every mode that the force ensemble certifies,
rejects intentionally unsupported acquisition domains, and keeps object-model
validity separate from the relevance of one query force.

This does **not** establish real-object, material-family, audible-quality,
validator-release or runtime credit. It only closes the synthetic
identifiability prerequisite for the V12 real-data path.

## Frozen identity

- Freeze manifest SHA-256:
  `67b2c4723835d02f6163ca1410a06c0a5cc9f4fd69aeb82ea090f76c6a5ca7b2`.
- Paired zero-sample preflight report SHA-256:
  `fc686c5d9cff647fdfd17e33e951ee3e78e7248e368cb93e0303dee415b113c2`.
- Protocol SHA-256:
  `9b5887da7c05918d5d7da171201a9f2e261f3724891390af6ed7821e4db1c7b5`.
- Runner SHA-256:
  `4e864f6ed15488a2c31defffe39b220b5858c7cb424cd0ea44a700db36c51e69`.
- Frozen implementation commit: `5def5030`.
- Runtime: Python `3.11`, NumPy `1.26.4`, SciPy `1.11.4`.
- Network requests, real payload bytes and parent holdout samples read: `0`.

The force certificate is created before truth and response access. Its role
order is exactly `force_certificate -> truth_fixture_and_fit_response ->
development -> holdout`. Holdout exists because all development gates passed.

## Result

### Acquisition coverage

- Certified band: `200…9,500 Hz`, supported fraction `1.0`.
- Minimum profile support over the band: `3`.
- All seven truth neighborhoods are certified.
- Minimum support in the seven neighborhoods: `5, 5, 5, 5, 4, 4, 4`.
- All five leave-one-profile-out fits still recover `7/7` modes with zero
  false positives.

### Modal identification and held response

| Metric | Development | Holdout | Gate |
| --- | ---: | ---: | ---: |
| Supported truth modes | `7/7` | `7/7` | `7/7` |
| False positives | `0` | `0` | `0` |
| Mean candidate NRMSE | `0.004977` | `0.004961` | `<= 0.08` |
| Maximum candidate NRMSE | `0.005609` | `0.005635` | `<= 0.15` |
| Mean log-spectrum RMSE | `0.024494 dB` | `0.021430 dB` | `<= 2 dB` |
| Maximum frequency error | `0.069002 Hz` | `0.069002 Hz` | `<= 1 Hz` |
| Maximum decay error | `0.415307 s^-1` | `0.415307 s^-1` | `<= 1.5 s^-1` |
| Maximum relative decay error | `1.012944%` | `1.012944%` | `<= 20%` |
| Minimum supporting contacts/residues | `6 / 6` | `6 / 6` | `>= 3 / >= 3` |

### Calibrated controls

- Acquisition notch at `6,000 Hz`: only modes `0,1,2,3,5,6` are supported and
  recovered; decision `OOD_ACQUISITION_HOLE`, zero false positives.
- The same notch used only as a query force keeps all seven object modes;
  decision `VALID_QUERY`, mean NRMSE `0.005489`, maximum target-mode energy
  ratio `0.011157` against the `0.02` ceiling.
- Weak force family: `OOD_WEAK_EXCITATION`, zero admitted modes.
- Dynamic unmeasured interference: `OOD_LOW_COHERENCE`, median reconstruction
  NRMSE `0.600882` on holdout.
- Missing impact: `OOD_MODEL_MISMATCH`, median reconstruction NRMSE `0.734800`,
  zero admitted modes.

## Reproducibility

Run A and Run B were executed into independent empty external roots. All twelve
canonical files are byte-identical. Important hashes are:

- `report.json`: `33371827f283dfe596dbcee05e001fb5cb2348f462030b78ccfeacc98767e85f`;
- `certificate.json`: `80d4bbaa7a187510e77f5831308b75e81e6b534fa4b29adf4f12ed04849c3e61`;
- `model.json`: `11f75af9137b35930ab8f80a4a90bc2155261d817d4eb6a7ca00e9008a49ca6c`;
- `modal_transfer.npy`: `1e789809e2881aa3ea66dddbadf896ccd0fe411433a053c320dfcf83b0271a71`;
- `modal_reconstruction.npy`:
  `5bf6e727a12a542e9abfdd023634cabadcef5a54daa511a45082f1d0326da1e0`.

The external evidence roots are:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c1-acquisition-coverage-freeze`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c1-acquisition-coverage-preflight-a` and `-b`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c1-acquisition-coverage-run-a` and `-b`.

No generated array, dataset, WAV or report is added to Git.

## Decision and remaining risk

C1 is complete. The previous B1R3 failure was caused by acquisition coverage,
not by an inherent inability of this bounded estimator to identify the seven
known poles. The V12 split between acquisition coverage, identifiable object
model and query relevance is therefore retained.

C3 is still blocked. C2 must first prove, without decoding waveform payloads,
that at least one internet source has stable paired force and microphone data,
lineage, object/trial identity and sufficiently explicit axes. Real PCM, ML,
validator release, atlas cooking and runtime integration remain closed; authored
clips remain mandatory.
