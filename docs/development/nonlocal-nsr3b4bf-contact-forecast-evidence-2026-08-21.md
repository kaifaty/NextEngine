# NSR3-B4BF contact-onset forecast evidence -- 2026-08-21

Status: `PASS / CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE`

## Reproduction

```text
nonlocal-formula-reclosure --contact-onset-forecast-self-test
```

Three reports are byte-identical:

```text
raw JSON plus LF  309c99aec8299e3c03afff45d7cc6e2ec0036730aaa7f26408693d7eeaf0df13
JSON without LF   dfd6b39d8c12da4e494ff2ed4de5590d7a08b2be598c2d411c076ad6f8f1b550
semantic result   c6d53131786bcfacdae1bbb1a4ee2076846019d037b62b167b2a76b4d28e146f
```

## First-frame curve

All independent P1 KKT intervals `1..192` complete. Adjacent pairs alone are
misleading at small counts: every adjacent gate passes, while fixed-192
kinetic comparison fails at `n=1,2,4` with errors `30.93%`, `24.46%` and
`17.09%`. It first passes at `n=8` with `10.52%`.

This confirms that adding a second adjacent-pair rule would not identify the
non-asymptotic onset by itself.

## Derived forecast

The feasible clamped macro predictor activates 16 pressure centres. Its
48-HVP spectrum is:

```text
lambda_max = 70346.222687210058
omega_max  = 750.17983277190308 rad/s
n_forecast = ceil((1/240)*omega_max/0.15) = 21.
```

The immutable `21/42` pair passes:

| Observable | embedded 21/42 | fine-42 vs fixed-192 | Gate |
|---|---:|---:|---:|
| position | `2.326462267833048e-5 dx` | `1.8301026748496891e-5 dx` | `<=0.05dx` |
| velocity | `4.6575840733823729e-6 c` | `4.0051904909091139e-6 c` | `<=0.001c` |
| kinetic | `2.4146016129171496%` | `2.0526603852240567%` | `<=15%` |

Detached P2 remains pressure-inactive, selects `n=1`, charges zero spectral
HVPs and is bit-identical to exact free flight with zero contact/support
reaction.

## Decision

Select `CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE`. Freeze B4B2 before a full
retry. Do not replace the reference gate with adjacent convergence, hard-code
`n=8/21`, or run the forecast on detached inactive frames unnecessarily.
