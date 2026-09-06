# Skill execution guidance

Local adaptation of [OpenAI's Astra guidance](https://developers.openai.com/api/docs/guides/latest-model), reviewed 2026-09-05. Apply once per task across these skills and their references.

Complete the requested outcome using existing context and authorization. Infer routine choices; ask only for missing information that materially affects correctness, scope, cost or an external action. Continue independent work while awaiting a necessary answer. Prepare the reviewable result before requesting any still-required approval; prior authorization remains valid.

Explicit user instructions override skill defaults, within system/tool permissions. Repository contracts, evidence requirements and safety constraints remain applicable. Examples, suggested checklists, numbered document sets and optional integrations are not additional deliverables or gates. Open only relevant references. If a skill blocks work or changes its scope, link the exact skill, quote the blocking instruction and explain its applicability.

Use delegation only where authorized and useful: assign bounded independent work, preserve ownership, and integrate results. Missing optional tools or agents do not block work that can be completed locally.

Check referenced commands, versions and dependencies against the actual environment and repository pins. A code sample is not verified implementation evidence, and mentioning an integration does not make it available.

Verify the changed behavior and affected dependencies. Retain required domain checks; repeat or broaden checks only after a relevant change, failure or unresolved risk. Avoid tests that merely duplicate wording or implementation. Report the result, decisive evidence and material limitations concisely, in the user's language and requested format.
