# ADR-084: Public creator run and project-package vertical

| Field | Value |
|---|---|
| ID | ADR-084 |
| Status | Accepted |
| Version | 1.1 |
| Decision date | 2026-08-18 |
| Last verified | 2026-08-18 |
| Normative dependencies | [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](047-simple-application-session-and-save-on-close.md), [ADR-048](048-direct-exact-project-lock.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md), [ADR-083](083-public-creator-project-cli-vertical.md) |
| Supersedes | Narrowly supersedes ADR-083's statement that creator run/package have no current consumer. It adds two commands and two report contracts without changing Creator Command Report V1 or the R6a validate/cook semantics. |
| Superseded by | Narrowly [ADR-085](085-public-creator-project-inspect-and-diff-vertical.md) for the later read-only inspect/diff consumer; run/package commands and report/package semantics remain unchanged |

## Context

R6a proved that a data-only project outside engine Rust can be validated,
cooked, atomically published and activated. It did not prove that the project
can enter the application/runtime lifecycle or leave the repository as a
validated distribution artifact. The existing external-project option in
`next_headless` was insufficient: activation selected the caller's project,
but execution still constructed `ReferenceGameDriverV2` and therefore required
reference-alpha roles, identities and presentation content.

R6b needs a real second-project consumer without freezing the later inspector,
scenario, template or full SDK surfaces. A native standalone Windows/Linux
game bundle is also too broad: target binaries, renderer dependencies and
paired release evidence remain the R7 package boundary.

## Decision

### Generic project runtime cut

`ApplicationCoordinator::run_project_headless` is the current project-neutral
startup path. It accepts the already activated exact project package, derives
world identity and RNG material from the project lock, installs that project's
RPG definitions and policy hashes, activates its authored initial population
placement, world streaming, optional routine, population, activity and
cognition owners, and executes exactly one production runtime tick.

The cut is intentionally headless and non-interactive. It creates no player,
reference-game aggregate, presentation object or privileged creator command.
After the tick it materializes the owner-complete authoritative root and uses
the ordinary Application Session transition plus save-on-close journal. The
current creator command owns an isolated state directory, so interrupted
creator runs do not create a public resume promise.

Exactly one tick is part of this bounded contract. It proves runtime
construction, schedule/physics advancement, owner closure and final-save
publication without inventing a creator gameplay scenario. Multi-tick
world-service behavior, interactive composition and scenario authoring remain
later R6 consumers.

### Current public commands and reports

ADR-083's commands remain byte-compatible and keep Creator Command Report V1:

```text
next project validate --project <project-directory>
next project cook --project <project-directory> --output <content-store-directory>
```

R6b adds:

```text
next project run --project <project-directory>
next project run --package <creator-package-directory>
next project package --project <project-directory> --output <new-directory>
```

The two run spellings are mutually exclusive. Authoring run cooks and
publishes into isolated storage before launch. Package run validates the
complete package and then launches its embedded published bytes. Both emit one
Creator Run Report V1 JSON object containing project identity, source kind and
the deterministic application run/final-save proof.

Package emits one Creator Package Report V1 JSON object containing exact
project identity, package format/hash, bounded inventory totals, required
notices and the same run proof. Failures retain the one-object, path-free
stable diagnostic shape. The three report families are separately versioned;
R6b does not add fields or commands to Creator Command Report V1.

### Creator Project Package V1

`nextengine.creator-project-package.v1` is a current-only content distribution
envelope for users who already have a compatible `next` tool/runtime. It is
not a standalone native game package and makes no target ABI, glibc, MSVC,
renderer or launcher claim.

The envelope contains:

- one canonical `creator-package.manifest.jcs` with exact project roots,
  sorted file inventory and the production run proof;
- one complete immutable ContentStore publication beneath the package-owned
  project area; its internal generation layout remains private;
- the project's nonempty root `NOTICE` required for redistribution.

Every inventory path is normalized, relative and bounded. Links, traversal,
unknown files, duplicate/unsorted entries, oversized files, missing notices,
hash/size mismatch, unsupported format/version, activation mismatch or run
proof mismatch reject the whole package before success. `project package`
builds in a private sibling staging directory, validates and reruns the staged
bytes, then publishes only to a previously absent output directory. It never
merges with or overwrites caller content.

Package identity and run evidence grant no runtime capability. Scripts and
mechanics retain the exact project lock, capability ceiling and validation
paths defined by ADR-014/048.

## Product impact

An author can now go from editable project data to a deterministic runtime
startup and a redistributable, self-verifying project package using only the
public `next` binary. A recipient can run the packaged bytes without the
authoring source tree, while exact roots and NOTICE remain inspectable and
tampering fails closed.

This completes the R6b run/package vertical and materially reduces B-09. At
this decision boundary it does not complete R6: diff/inspect, templates,
scenarios/minimization, replay inspection and the broader SDK/documented
cold-authoring exercise remain open. ADR-085 later closes only diff/inspect.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| focused creator CLI | Repeat authoring run, build two packages, run packaged bytes | Identical reports/manifests and one-tick/final-save roots; no reference-game bootstrap | Stable failure and complete temporary cleanup |
| creator package negative matrix | Existing/symlink output, changed inventory byte, missing/linked NOTICE, malformed/retired manifest | Reject before launch/publication; caller content remains unchanged | No overwrite, migration or tolerant decode |
| `content-package` | Run creator authoring, build/validate package and run its published bytes | Authoring/package runtime proofs and exact project lock match | Fail the product check |
| `play` / `persistence-replay` | Shared application/runtime/save regressions | Existing reference gameplay and persistence roots remain valid | Keep R6b incomplete |
| Linux `platform` | Shared launch/session path on the active host | Existing Linux platform smoke remains valid | Report the Linux failure; Windows remains deferred |

## Considered alternatives

- **Make creator-smoke satisfy `ReferenceGameDriverV2`.** Rejected: matching
  reference-only roles and fixed identities would preserve the hidden
  first-party bootstrap R6 is meant to remove.
- **Call activation itself a run.** Rejected: R6a already proved activation;
  R6b must cross Runtime, Application Session and final-save boundaries.
- **Package source files only.** Rejected: recipients would depend on local
  cooker behavior and would not receive the exact validated generation.
- **Build a native standalone game from `next project package`.** Rejected for
  this increment: target runtime inventory, desktop dependencies and paired
  Windows/Linux release evidence belong to R7.
- **Overwrite an existing package directory atomically.** Rejected: a public
  creator command cannot infer ownership of arbitrary caller content. A fresh
  destination is the bounded safe rule.

## Consequences and next boundary

- The public creator surface now has four operations and three independent
  report families: R6a validate/cook, R6b run and R6b package.
- Project-neutral startup no longer depends on the reference-game crate's
  world roles or aggregate constructor.
- The current package format is exact-current under ADR-046. A future format
  is rejected until an explicit consumer-backed successor exists.
- The next bounded R6 increment should add read-only project diff/inspect over
  this same creator/package fixture before templates and scenarios widen the
  mutation surface; ADR-085 later fulfills that increment.
- Native Linux remains the active development host. Windows execution and
  paired shipping evidence remain `NotRun(WindowsHostDeferred)` under ADR-082.

## Supersession

Changing one-tick runtime semantics, report fields, package inventory rules,
NOTICE requirements, existing-output policy, package format/version handling
or granting creator tools gameplay mutation authority requires an explicit
successor decision. Adding a consumer-backed read-only inspector or scenario
contract may use a later ADR without changing this run/package boundary.
