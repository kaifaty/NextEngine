# ADR-097: Linux v1 distribution closure

| Field | Value |
|---|---|
| ID | ADR-097 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-24 |
| Last verified | 2026-08-28 |
| Normative dependencies | [SPEC-04](../04-rendering-and-platform.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](001-product-repository-license-and-platforms.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md) |
| Supersedes | Replaces PackageManifest V5 as the current native distribution format and narrows the R7e Linux version, dependency/license, documentation and protected-data boundary; V5 packages remain strict historical R7b evidence |

## Context

R7b proved that two PackageManifest V5 trees from one exact Linux commit are
byte-identical and that copied game, headless and public tool binaries work from
isolated package state. V5 intentionally stopped at that install/runtime
boundary. It records direct ELF libraries and four repository notices, but it
does not identify the selected statically linked Cargo closure, carry the
corresponding upstream license files, bind a release version, ship user start
and troubleshooting guidance or publish a content-level protected-data scan.

The accepted R7b ELF also exposes absolute builder paths from Rust crate source
locations. Those strings do not affect gameplay, but they leak local-machine
identity and make otherwise identical source builds depend on checkout/cache
location. R7e cannot claim protected-data absence or cross-root reproducibility
while retaining them.

## Decision

### Version and current target

The current product/workspace release version is `1.0.0`. Native v1
distribution remains only `x86_64-unknown-linux-gnu` under ADR-090. The
current packager emits and accepts strict `PackageManifestV6`; old schemas are
rejected before nested use and are not migrated in place.

Package report schema 3 exposes the distribution receipt to the Linux native
gate. The target report may continue to summarize the package by its exact
manifest hash because that hash binds the complete V6 distribution structure.

### Exact distribution contents

V6 preserves every V5 invariant and additionally requires:

- root `GETTING_STARTED.md` and `TROUBLESHOOTING.md` documents;
- the exact workspace `Cargo.lock` and its SHA-256;
- canonical `DEPENDENCY_INVENTORY.jcs` schema 1;
- a `THIRD_PARTY_LICENSES/<name>-<version>/...` tree containing the bounded
  license, notice and copyright files taken from each selected registry crate;
- release name/version, selected dependency/license counts and a protected-data
  scan receipt in the manifest.

The dependency inventory is generated offline from locked Cargo metadata for
`next_game` with `desktop-sdl-ash`, `next_headless` and `next_cli`. Traversal
excludes dev-only edges and conservatively retains build dependencies because
source-build wrappers such as static SDL contribute code to the shipped ELF.
Every external record requires exact name, version, crates.io source, Cargo.lock
checksum, nonempty license expression and at least one copied license/notice or
copyright file. Unknown sources, absent checksums/licenses/files and any
inventory/hash drift fail before atomic publication.

### Builder-path and protected-data closure

The release build sets deterministic Rust path remapping for the repository and
builder user root. It must not embed the original checkout, Cargo registry or
user-home prefix in a packaged binary.

After every content, document, dependency/license and binary file is staged,
the packager scans every inventoried byte before executing copied-root smoke.
The post-smoke inventory must be byte-identical to the scanned inventory.
Scanner `nextengine-protected-data-v1` rejects high-confidence private-key
headers, bounded credential-token forms, AWS secret assignments and absolute
Linux/macOS/Windows user-home paths. Package path validation additionally
rejects protected legacy-game asset extensions. The receipt binds scanner ID,
`PASS`, exact scanned file count and byte count; installed-package validation
reruns the same scan and requires equality.

The scanner is a distribution backstop, not a general secret-classification or
legal-rights oracle. Provenance is also constrained structurally: shipped
content comes only from the exact neutral reference source/cooker, binaries
come from the locked selected build and all remaining files come from explicit
repository or registry-license allowlists.

## Consequences

- A V5 artifact remains valid R7b history but cannot be called an R7e/v1
  distribution.
- Package creation now needs the locked Cargo source cache but performs no
  network lookup.
- Adding or changing a selected dependency changes the inventory and copied
  license tree visibly; missing redistribution evidence blocks publication.
- Release binaries no longer reveal builder-local source locations.
- User launch, system prerequisite and recovery guidance travels inside the
  exact package rather than depending on repository access.
- Windows package generation stays dormant/out of scope and cannot be inferred
  from retained historical enum/adapter code.

## Product checks

| Check | Expected |
|---|---|
| Focused package tests | V6 strict round-trip; retired/unknown fields fail; selected offline graph has checksums/licenses/files; inventory/license/document tamper and protected markers fail before smoke/publication |
| `content-package` and package pipeline | Neutral source/provenance remains exact and no protected/imported path or byte is accepted |
| Reproducible `v1-package` A/B | Complete V6 trees, modes and canonical manifest hashes are identical; schema-3 reports expose version, dependency/license counts and scan PASS |
| Linux native gate | Installed package validation reruns V6 inventory/runtime/source/distribution/scan controls and copied game/headless/tool receipts; final target remains release-ready only when every scheduled Linux check passes |

## Rejected alternatives

- **Extend V5 with optional fields.** Rejected because an old validator would
  accept an incomplete distribution under the same current format identity.
- **Treat direct ELF imports as the dependency inventory.** Rejected because
  statically linked Rust/SDL/Luau/Wasm code is absent from that list.
- **Package only Cargo.lock.** Rejected because it does not select the shipped
  closure or directly expose license expressions and copied notices.
- **Repository grep as the protected-data check.** Rejected because the
  observed leak exists only in generated release ELFs.
- **Online license/SBOM lookup during packaging.** Rejected because it makes
  package publication depend on mutable remote state.
- **Waive builder paths as harmless debug text.** Rejected because the paths
  expose local identity and vary across otherwise equivalent build roots.
