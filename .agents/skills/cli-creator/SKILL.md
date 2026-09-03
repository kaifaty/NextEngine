---
name: cli-creator
description: Build a durable CLI when the user explicitly needs a command usable across repositories or repeated sessions. Do not use for a one-off repository script, a single shell invocation, or when extending an existing command is smaller.
---

# CLI Creator

Build the smallest installed command that completes the user's repeated jobs.
A CLI is the primary artifact; documentation, discovery layers and companion
skills are optional support.

## Scope first

1. Name the requested jobs and the smallest command that completes the first one.
2. Check for an existing command or repository tool before scaffolding.
3. Use the project's existing language/runtime when practical. Prefer Rust for
   a standalone binary, TypeScript when an official SDK is decisive, and Python
   for data/file tooling. Do not introduce a language for stylistic preference.
4. If a local script solves the stated need, write that instead and stop.

Read [references/agent-cli-patterns.md](references/agent-cli-patterns.md) only
when designing a multi-command agent-facing interface.

## Minimum command contract

The first vertical slice needs only:

- useful `--help`;
- the requested command with bounded inputs and stable exit status;
- machine-readable output when Codex or another tool will consume it;
- errors that omit credentials;
- one focused success test and one relevant failure test.

Add these only when the requested jobs demonstrate the need:

- `doctor` for non-trivial auth/configuration;
- discovery or name-to-ID resolution when callers cannot supply stable IDs;
- pagination when listing a real collection;
- a raw request escape hatch when the supported API surface is intentionally
  incomplete;
- global PATH installation when cross-repository use was requested;
- README/reference material beyond concise usage examples;
- a companion skill after repeated use is established or the user asks for it.

Do not scaffold all of them by default.

## Auth and writes

Reuse the service's standard environment variable or existing provider config.
Never print secrets. Add a new config file only when the existing auth path is
insufficient.

For live writes, expose one narrow verb. Prefer preview, draft or `--dry-run`
when the service supports it, and request authorization immediately before a
material external mutation. A broad command such as `fix` or `auto` must not
hide unrelated writes.

## Build and verify

1. Inspect source material only far enough to implement the first requested job.
2. Implement that end-to-end command before adding infrastructure.
3. Test the installed invocation only if installation is in scope; otherwise
   test the repository-native command.
4. Run format/build and focused tests for request construction, output shape and
   the relevant failure path.
5. Show the working command and result. Expand the surface only for the next
   demonstrated job.

If the source is a web application or captured curl, sanitize credentials and
customer data before saving fixtures. Screenshots can establish vocabulary but
do not prove an API contract.

## Companion skill

Create one only when explicitly requested or when the completed CLI is a
confirmed repeated tool. Keep it short: existence check, auth source, common
read/write commands, safety boundary and a few examples. API documentation
belongs with the CLI, not duplicated in the skill.
