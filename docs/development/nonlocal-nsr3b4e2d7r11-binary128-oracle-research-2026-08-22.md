# NSR3-B4E2D7R11 binary128 accepted-sign oracle research

Date: `2026-08-22`

Status: `CLOSED / PASS / OFFLINE_ACCEPTED_SIGN_CERTIFICATE`

## Question

Is the tight D7R10 candidate acceptance truly energy-decreasing, or did both
the binary64 divided formula and quadratic model agree on the wrong sign?

Only three D7R10 accepted current/trial pairs need inspection. Two already
have resolved-positive long-double signs; the third is positive in both
long-double sums but below their conservative resolution threshold. A wider
offline oracle is the smallest discriminator before deriving runtime error
bounds or integrating an outer AL step.

## Selected oracle

Use GCC `__float128` plus libquadmath on the frozen Linux x86-64 research
profile. GCC documents `__float128` as the supported 128-bit floating type on
x86-64, and libquadmath exposes its mantissa constants, square root,
`nextafter` and deterministic formatting facilities. See the official
[GCC floating-types documentation](https://gcc.gnu.org/onlinedocs/gcc-14.3.0/gcc/Floating-Types.html),
[libquadmath type/constants](https://gcc.gnu.org/onlinedocs/gcc-8.1.0/libquadmath/Typedef-and-constants.html),
[math routines](https://gcc.gnu.org/onlinedocs/gcc-13.2.0/libquadmath/Math-Library-Routines.html)
and [`quadmath_snprintf`](https://gcc.gnu.org/onlinedocs/gcc-9.1.0/libquadmath/quadmath_005fsnprintf.html).

The exact admitted profile is:

```text
Linux x86-64
GCC __float128
sizeof(__float128) = 16
FLT_RADIX = 2
FLT128_MANT_DIG = 113
libquadmath header and library available
```

This is an offline evidence dependency only. The runtime candidate remains
binary64; CUDA, GPU kernels and particle state may not use or require
`__float128` from this result.

## Independence

Promote the exact binary64 current/trial positions, predictions, multipliers
and frozen binary64 kernel scale. Independently recompute in binary128:

```text
pair displacement and sqrtq radius
piecewise cubic kernel
density
PHR active coefficient and energy
inertia
total current and trial energy
```

Evaluate both fixed-order and compensated binary128 sums. Compare binary64
and binary128 pair membership. The oracle sign resolves only if both sums
agree and each reduction magnitude is at least 4096 binary128 ULPs of its
total-energy scale. This is far stricter in absolute precision than D7R8's
80-bit test while retaining the same independence pattern.

The binary64 divided reduction must match every resolved oracle sign and stay
within 5% relative magnitude. That threshold is frozen before execution and
is already looser than the maximum 3.99153% D7R9R1 error against resolved
long-double cases.

## Routes

1. `BINARY128_SIGN_CONTRADICTION`: any accepted pair resolves negative.
2. `OFFLINE_ACCEPTED_SIGN_CERTIFICATE`: all three resolve positive, pair
   membership agrees and each divided reduction is within 5%.
3. `STRONGER_ORACLE_REQUIRED`: controls pass but at least one sign or magnitude
   remains unresolved.

A certificate validates only the three exact private acceptances. It is not a
general runtime forward-error bound. The following complete private-outer
stage must audit every newly accepted near-floor step; production integration
will still need either a derived binary64 certificate, a bounded fallback or
a justified stationarity/noise floor.

## Closure

D7R11 passes; see the
[dated evidence](nonlocal-nsr3b4e2d7r11-binary128-oracle-evidence-2026-08-22.md).
All three reductions resolve positive at least `3.77e17` binary128 ULPs from
zero, pair membership is exact and the maximum candidate relative error is
`3.23e-5`. This selects a three-pair offline certificate, not wider runtime
precision. Proceed only to a rollback-only complete private-outer D7R12.
