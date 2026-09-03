---
name: map
description: Define, clarify, and persist durable user intent in the local Map graph. Use when explicitly invoked as $map; ordinary agents may query Map without invoking this workflow.
---

# Map

Map is the interactive editor for durable intent. It never implements the mapped work.
Use this workflow when the user explicitly invokes `$map` or asks to start/resume Map clarification.
The selected `.map` graph is authoritative semantic state; session state is recovery only. Never read or modify `.map/db` directly or use SurrealQL instead of the CLI.

## Runtime invariants

1. Preserve user intent. Never invent requirements, decisions, facts, or rationale.
2. Ask only decisions that can materially affect the requested outcome at the effective depth/stance. Prefer safe inference and downstream freedom over interrogation.
3. Five questions is a cap, never a quota.
4. Facts are contextual evidence; decisions are choices. Do not ask the user for externally knowable facts when they can be established safely.
5. Children run serially. Consume and close each child before spawning another. Children never spawn children.
6. Use subagents at semantic transaction boundaries, not for CLI clerical work.
7. Spawn prompts contain only dynamic arguments/evidence; the installed agent definition owns its semantic contract.
8. One semantic repair cycle maximum per reviewed transaction. No reviewer/worker ping-pong.
9. Persist recovery state before exposing resumable work. Clear pending only after its semantic consequence is durably verified.
10. `explored` and `closed` are explicit. Never infer either from question count.
11. Do not implement, plan implementation, create tasks, export to Beads/Jira/etc., or turn Map into a task tracker.
12. Map state is authoritative assistant memory, not assumed user working memory. Before presenting a question or clarification, ensure it contains enough local context to understand without recalling prior graph state or Map-internal terminology.

While Map owns the workflow, invoke another skill only if the user explicitly requested it or this file permits it.

## Required specialists

`JLS` installs eight native specialists: `map-state-writer`, `map-state-reviewer`, `map-source-extractor`, `map-discovery`, `map-discovery-reviewer`, `map-linguist`, `map-context`, and `map-completion-auditor`.
Spawn a fresh child using the exact registered name for each required stage. Its installed definition owns the semantic contract; supply only dynamic arguments/evidence, never a generic substitute or inlined/paraphrased contract. Close it after consuming the result.
If a required specialist cannot run, fail that stage closed. Do not replace its semantic judgment with the parent thread.
Normal fresh flow without external context should normally use State Writer, Discovery, then Discovery Reviewer before questions; Linguist runs only when needed.

## CLI

Use the installer-provisioned CLI:

```text
{{JLS_MAP_CLI}} [--path PATH] [--config PATH] <command>
```

Use `status`, `context`, `show`, `get`, `search`, `history`, and `validate` for reads. Use `--help` for exact command flags. Normal commands require an existing Map; installation alone never initializes one.

## Start or resume

If the user explicitly asks Map to implement, execute, or otherwise perform mapped work, stop before initialization or mutation. Briefly clarify Map's non-implementation role and offer to map/clarify the requested outcome instead; do not silently reinterpret the request as Map work.

1. Resolve the intended Map and run `status` when one exists.
2. If a recovery session exists, inspect pending/exchange plus authoritative graph state before unrelated new Map work. Never blindly replay pending work.
3. Determine the requested outcome. Do not restate clear direction merely for confirmation; ask only when a material interpretation is ambiguous.
4. Existing Map: use its stored Map-level `mvp|thorough` depth and `normal|adversarial` stance without reconfirming them merely for a new request/session.
5. No Map: infer a recommended project configuration from the request. Briefly explain `mvp`, `thorough`, `normal`, and `adversarial`, state the recommended combination, and ask the user to confirm or change only those settings. Do not initialize or semantically mutate before confirmation.
6. After new-Map settings are confirmed, run `init` at the resolved intended path, then explicitly `set depth` and `set stance` to the confirmed Map-level values before continuing. Do not initialize another path by inference.
7. Ensure a session exists for substantive Map conversation and record exact exchanges per the session-first persistence invariant.

`mvp + normal` are fallback recommendations, not permission to skip new-Map settings confirmation. Intent-level depth/stance overrides remain valid when explicitly established.
If the user is only querying an existing Map, answer from read-only Map state without starting the full discovery workflow.

## Optional context transaction

Use `map-context` only when repository/environment/external context materially affects the focused intent. Pass the exact authorized scope and focus; reuse its compact report for the current semantic transaction. Skip it when Map state and user evidence suffice.
Context evidence may establish facts. It never silently becomes user intent.

## Existing-source migration

Use this path when the user explicitly asks to bootstrap or consolidate Map from authorized existing goal/spec/note material. It feeds normal Map semantics; after migration, continue normal Discovery.

1. Mechanically inventory the authorized corpus into bounded deterministic batches and checkpoint scope/progress in session recovery state. Do not rely on thread memory for coverage.
2. For each batch, spawn `map-source-extractor` with exact source paths, focus, and the user's authority statement. Require every supplied source to be accounted for.
3. Spawn `map-state-writer` with `MODE: MIGRATION`, Map path, source paths/authority, and extractor report.
4. Close Writer; spawn `map-state-reviewer` with `MODE: MIGRATION`, the same packet, and Writer result. Reviewer may inspect the source batch directly.
5. On PASS, `validate`, verify affected context, and advance the checkpoint. On FAIL, allow one fresh Writer+Reviewer repair using exact deficiencies; otherwise stop with checkpoint intact.
6. Never choose precedence for ambiguous duplicates/conflicts without evidence. After all sources are accounted for, complete the checkpoint and continue Discovery.

## Baseline semantic transaction

Before Discovery, concrete direction must exist as durable Map semantic state.
For a new direction or material direction correction:

1. Persist the recovery checkpoint first.
2. Spawn `map-state-writer` with `MODE: BASELINE` or `MODE: DIRECTION_CORRECTION`, Map path, source evidence, and optional context report.
3. Close it; run `validate` and inspect affected intent/context.
4. If mechanical validation fails, allow one State Writer repair with the exact defect. If it still fails, stop and report the blocker.

Normal baseline does not use State Reviewer. Concrete direction must produce at least one durable intent unless that would require inventing the user's objective.

## Discovery batch

For each focused non-closed intent needing exploration:

1. Read its compact `context` and effective depth/stance.
2. Spawn `map-discovery` once with Map path, focus intent IDs, optional context report, and optional audit focus; close it.
3. Spawn `map-discovery-reviewer` once with the whole candidate batch and exhaustion claim; close it.
4. Persist only reviewer-approved questions using their exact approved semantic decision and reason. Reject duplicates/already-settled candidates.
5. If Reviewer requests wording help, run `map-linguist` once for that subset and use its wording exactly.
6. Mark the focused intent `explored=true` after it has actually been examined.
7. Before presenting questions, append the exact assistant message to session exchange and set pending to the question IDs/text.
8. Mark each presented question `asked=true`, then present the approved batch together.

Discovery standards:
- material ambiguity only; no arbitrary quota
- semantic duplicates do not become new questions
- dependent questions wait for prerequisites
- optional capabilities default out once direction is concrete
- postponable implementation choices stay open
- adversarial stance challenges assumptions rather than merely increasing question count
- `EXHAUSTED` means no additional worthwhile decision belongs in the current pass; fewer than five is correct

If zero candidates are approved and Reviewer rejects exhaustion, allow one focused Discovery+Reviewer retry. If the same gap remains, surface it instead of looping.

## Answers and semantic persistence

When the user answers:

1. Append the exact user message to session exchange; keep pending intact.
2. Preserve the approved question, material clarification, and user answer without strengthening/generalizing them.
3. Spawn `map-state-writer` with `MODE: DECISIONS`, exact evidence, Map path, and optional context report; close it.
4. Spawn `map-state-reviewer` once with the same evidence plus Writer's affected IDs/result; close it.
5. On PASS, `validate`, verify affected context, update session summary, and clear only pending work whose semantic consequence is durable.
6. On FAIL, allow one fresh Writer+Reviewer repair using original evidence plus exact deficiencies. If it fails again, stop with pending intact.
7. Return to Discovery using refreshed Map state.

User decisions use user provenance. Assistant decisions require explicit assistant reasoning and must be soft when deliberately revisit-worthy. Never hide assistant inference as a user decision.

## Completion

When reviewed Discovery is exhausted and no current unanswered material question remains, spawn `map-completion-auditor` once.
- PASS: mechanically attempt `map set <intent> close true` for each auditor-approved focus intent; runtime closure invariants remain authoritative.
- FAIL: allow one focused Discovery batch using the auditor's unresolved areas. If the same blocker remains, surface it.
Closing an intent is not permanent; later requirements may reopen affected state through normal Map semantics.

## Parent-owned mechanics

The parent owns non-semantic CLI bookkeeping: path/status resolution; session init/exchange/summary/pending/end and migration checkpoints; exact persistence of reviewer-approved questions; returned IDs/results; `asked`, `explored`, and auditor-approved `close`; validation/readback; retry counts; serial child lifecycle.
The parent must not replace Discovery, Reviewer, Linguist, State Writer/Reviewer, Source Extractor, Context, or Completion Auditor semantic work with its own substitute.

## Recovery invariant

For substantive Map conversation:

```text
A. Persist recovery state first.
B. Apply and verify semantic mutations.
C. Clear pending only after B is durable.
```

A crash between mutation and pending clear recovers by comparing pending/exchange against authoritative graph state, never by blind replay.

## Completion boundary

Map clarification is complete when effective depth/stance has been genuinely explored, no material current question remains unresolved, runtime closure invariants hold, and `closed=true` succeeds.
Sufficient definition is the target, not maximal specification.