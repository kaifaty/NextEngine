# Linux-only v1 release policy — task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / POLICY_PROMOTED` |
| Updated | 2026-08-20 |
| Task key | `linux-only-v1-release-policy` |
| Scope | Remove Windows from the current v1/R7 boundary indefinitely and define the bounded Linux-only R7 release path |
| Definition of done | Accepted ADR, core SPEC/guidance, roadmap, blocker register and dormant backlog agree that Windows is not a v1/R7 target and R7 can begin with Linux release-closure implementation |
| Authority | Working context only; ADR-090, affected Accepted SPECs and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Windows is `OUT_OF_SCOPE / INDEFINITELY_DEFERRED` and
  no longer blocks R7 or v1. Linux x86_64 GNU/Vulkan is the sole v1 shipping
  and release target.
- **Why:** The product owner explicitly removed unavailable Windows work from
  the current scope for an undefined period. ADR-082's pre-R7 bring-up is
  therefore obsolete rather than merely delayed again.
- **Next action:** Implement R7a by versioning release aggregation so one exact
  Linux native bundle can produce an honest v1 release-ready result without a
  Windows slot.
- **Current blocker:** None for starting R7a. Current `v1-closure` semantics are
  an implementation gap to change, not a Windows prerequisite.
- **Do not retry:** Do not schedule Windows/THOTH/paired runs, maintain a live
  Windows backlog or treat historical Windows PASS as current support.
- **Reconsider when:** A future explicit product request provides a supported
  Windows outcome, an available native host/owner and a separate roadmap slot.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| Product-owner decision, 2026-08-20 | `WINDOWS_OUT_OF_SCOPE_INDEFINITE` | Existing pre-R7 Windows bring-up policy must be superseded |
| [ADR-090](../../architecture/adr/090-linux-only-v1-and-indefinitely-deferred-windows.md) | `ACCEPTED` | Linux-only v1/R7 is the normative platform boundary |
| Existing Linux native gate at `d15c11a…` | `HISTORICAL PASS` | Linux implementation/package path exists; it is not final R7 exact-commit evidence |
| Current `v1-closure`/native comparator | `IMPLEMENTED / OLD TWO-TARGET SEMANTICS` | R7a must version the release report; documentation cannot relabel the old output |

## Decisions that still constrain the work

### D-001 — R7 is Linux product closure, not Windows bring-up

- **Observation:** ADR-082 made unavailable Windows execution the only entry
  condition for R7 even though all active development and current hardware
  evidence are Linux-native.
- **Evidence:** ADR-082, roadmap R7/B-01/B-12 and the product-owner decision.
- **Decision:** Start R7 with Linux-only release-authority implementation;
  archive Windows work outside the current roadmap.
- **Rejected alternatives:** Indefinite pre-R7 wait, synthetic Windows PASS or
  automatic promotion of Linux report-only timing.
- **Consequences:** R7a changes aggregate release semantics; R7c separately
  establishes measured Linux release performance.
- **Uncertainty:** Exact report-version shape and accepted Linux performance
  fingerprint remain R7a/R7c implementation decisions.
- **Reconsider when:** A bounded R7a design proves a smaller compatible report
  change, or Windows is explicitly returned by a future product decision.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: existing single-target native reports can remain unchanged while only the aggregate closure advances | Target reports already separate target-local evidence from comparator state | `v1-closure` may embed a fixed two-target descriptor shape | R7a source/schema audit before editing contracts |

## Required context

1. `AGENTS.md`, architecture routing, SPEC-00/04/09/12/15 and ADR-090
2. `docs/roadmap.md`, especially R7, B-01 and B-12
3. current `v1-closure`, `native-gate-run`, `native-gate-compare` and package report schemas/tests
4. historical ADR-036/063/082 only for retained evidence semantics, not active policy

## Next action

1. Audit current release report/closure schemas and their consumers.
2. Choose the smallest versioned Linux-only aggregate that never fabricates a
   target result and preserves existing target-local reports where possible.
3. Prove focused failure behavior, Linux native closure and old-version
   rejection before claiming R7a complete.

## Do not retry

- Explicit pre-R7 Windows bring-up — removed from current scope; reconsider
  only through the ADR-090 re-entry rule.
- Relabel current two-target `shipping_ready = false` output — schema meaning is
  exact and must advance through implementation rather than prose.
- Inherit THOTH budgets on Linux — host identity and baseline are incompatible;
  R7c must establish a Linux authority explicitly.

## Handoff

- **Workspace state:** Documentation-only policy change; no executable report,
  package or runtime behavior is changed.
- **Checks:** `git diff --check` and direct link/path/ID validation `PASS`;
  Cargo/ProductChecks are `NOT_RUN / NoExecutableChange`.
- **Remaining risk:** R7a report-version design and R7c Linux performance
  profile are intentionally open implementation work.
- **Promotion needed:** Complete in ADR-090, affected SPEC/guidance and roadmap.
