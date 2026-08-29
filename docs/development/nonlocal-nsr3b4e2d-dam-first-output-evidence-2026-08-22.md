# NSR3-B4E2D Dam first-output evidence

Date: `2026-08-22`

Status: `FAIL / PREFLIGHT_REPORT_EXCEPTION / NO_TRAJECTORY`

## Result

The first B4E2D Release process exits `1` before entering the four-step loop.
One exact identity/alignment predicate is false, so the loop is gated off.
The new failure-report path then calls the canonical trajectory-root function
with an empty frame list; that API rejects counts outside unsigned 32-bit
positive range and the top-level exception handler writes:

```text
nonlocal-formula-reclosure: trajectory frame-root count is outside u32 range
```

This is a pilot-harness observability failure, not a Nonlocal physical or
reference-comparison result. No macro transaction, KKT solve or canonical
step publication starts. Per the stop-first contract, build/process B was not
run and no retry or tolerance/profile change was made.

## Exact evidence

Raw root:
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d.rhPjQy`

| Fact | Value |
|---|---|
| source implementation | `22aa521b3acea2b9776d8b4a9eb8ebc468fb9463` |
| executable SHA-256 / bytes | `2eebe88e021b75629e0c75bdcc786fba25f97a519ae8064b8008bc3351b49cbb` / 4,467,224 |
| GNU Build ID | `505b9f2359eb960693fac48afc978f81124149e9` |
| exit | `1` |
| stdout | 0 bytes / `e3b0c442...b855` |
| stderr | 77 bytes / `b469c0897d9303242bdc6eda802047f4207d6d37251dd934727e45c5b8f808af` |
| trajectory started | `false` |
| second process | not run |

The embedded B4E2D projection extracted from the executable hashes to the
expected `282b6ee1...671e`; identity-byte drift is already excluded. The
remaining preflight predicates need explicit observability before any repair
or physical rerun.

## Disposition

Preserve B4E2D as FAIL. Freeze B4E2D0 as a no-trajectory diagnostic that
publishes every identity/alignment predicate and always completes a bounded
report without computing an empty trajectory root. It may select only the
first exact reclosure target; it cannot authorize B4E2D physics by itself.

