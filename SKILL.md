---
name: map
description: Define and persist a software project's intended end state by interrogating the user, maintaining durable goals, and compiling approved goals to Beads. Use only when explicitly invoked as $map.
---

# Map

Map defines product goals. It never implements the product.
Use only when the user explicitly invokes `$map`.

## Runtime invariants

1. Preserve user intent; never invent requirements.
2. Ask only decisions that materially affect the requested end state. Prefer safe inference and engineering freedom over interrogation.
3. Product files contain durable product truth only. Process state belongs only in `PENDING.md`, `.discovery`, and `.tmp/`.
4. Product files must pass the deletion test: deleting all Map/process state and history must not make their statements stale, process-relative, or unintelligible.
5. Active goals are direct children of `goals/`: `goals/<name>.md`. Never create `goals/active/` or another active-goal subtree. Only deferred work may live under `goals/deferred/`.
6. Five questions is a cap, never a quota.
7. Children run serially. Consume and close each child before spawning another. Children never spawn children.
8. Use subagents at semantic transaction boundaries, not for filesystem clerical work.
9. Ordinary fresh flow with empty context whitelist should normally use 3 children before questions: Goal Writer, Discovery, Discovery Reviewer. Linguist is a fourth child only when needed.
10. One semantic repair cycle maximum per reviewed transaction. No adversarial ping-pong.
11. `.discovery` persists across stops/resumes until successful finalization.
12. Finalization compiles approved goals to Beads and stops. Never transition into implementation.

While Map owns the workflow, invoke another skill only if the user explicitly requested it or this file explicitly permits it. For compact `SUMMARY.md` or finalized Beads content, use maximally terse 文言文-style compressed prose, targeting roughly 80–90% character reduction. Prefer classical sentence patterns, verbs before objects, omitted subjects where clear, and particles such as 之/乃/為/其, while preserving all technical substance and exact technical terms.

## Agents

Use these global custom agents as functions; their TOMLs own their semantic contracts:

- `map_goal_writer`
- `map_goal_reviewer`
- `map_discovery`
- `map_discovery_reviewer`
- `map_linguist`
- `map_context`
- `map_completion_auditor`
- `map_beads_writer`
- `map_beads_reviewer`

Spawn prompts contain only dynamic arguments/evidence. Do not restate or override an agent's role.

## Workspace

Durable state:

```text
goals/
├── README.md
├── SUMMARY.md
├── .context_whitelist
├── .discovery              # procedural memory while Map is active
├── PENDING.md              # only while decisions await persistence
├── <active-goal>.md        # active goals are root-level
├── deferred/
└── .tmp/
```

Authority:

- active root-level goal files: authoritative current requirements
- deferred files: authoritative postponed product intent, not current scope
- README: human product overview/index
- SUMMARY: compact settled product state for agents
- PENDING: unanswered or answered-but-unpersisted user decisions
- `.discovery`: durable Discovery/reviewer memory, never product authority

Do not duplicate PENDING or `.discovery` concerns into product files.

Use bundled `scripts/state.py` for deterministic state operations when applicable. Resolve it relative to this `SKILL.md`.
If state is malformed or the script cannot safely operate, read `references/state-contract.md` and repair conservatively rather than guessing.

## Start or resume

1. Run `state.py bootstrap <GOALS_ROOT>`.
2. Read README, SUMMARY, PENDING if present, `.discovery` if present, `.context_whitelist`, and active/deferred inventory as needed.
3. Briefly tell the user your understanding of the current direction and ask for confirmation/correction before new semantic work. Do this on fresh and resumed invocations.
4. If the user supplied answers to existing PENDING questions, record them as `answered_unpersisted` and persist those decisions before new Discovery.
5. If `.discovery` says the current batch is exhausted and its Pending IDs still match unanswered PENDING entries, present/resume those questions instead of rediscovering.

## Optional repository context

If `.context_whitelist` has no active patterns, do not inspect implementation context.
If it has patterns, spawn `map_context` once with PROJECT_ROOT, GOALS_ROOT, and confirmed direction. Reuse its compact report for the current semantic transaction. Never broaden beyond the whitelist.

## Baseline goal transaction

Before Discovery, settled concrete product intent must exist in at least one authoritative active goal.
README/SUMMARY never substitute for a goal.
Zero active goals is valid only when the direction is genuinely exploratory and no concrete product outcome can yet be written without invention.

Run baseline when active goals are absent for concrete direction or confirmed direction materially invalidates current goal state:

1. Spawn `map_goal_writer` with `MODE: BASELINE`, GOALS_ROOT, TMP_DIR, confirmed direction/source evidence, and optional context report.
2. Close it.
3. Mechanically validate staged targets and commit only complete staged files.
4. Run `state.py validate-baseline <GOALS_ROOT> --require-goal` when direction is concrete.
5. If mechanical validation fails, discard stale `.tmp` candidates and retry Goal Writer once with the exact mechanical defect. If it fails again, stop and report the blocker.

Normal baseline does not use Goal Reviewer.

## Discovery batch

If no current exhausted pending batch:

1. Verify the baseline invariant above.
2. Spawn `map_discovery` once with GOALS_ROOT and optional AUDIT_FOCUS. It reads compact settled/pending/discovery state and returns up to 5 candidates plus an exhaustion claim.
3. If it reports `BASELINE_GOAL_MISSING`, repair baseline before continuing.
4. Close Discovery.
5. Spawn `map_discovery_reviewer` once with GOALS_ROOT, the whole candidate batch, exhaustion claim, and optional AUDIT_FOCUS.
6. Close Reviewer.
7. Apply reviewer-approved `.discovery` actions. Persist accepted/recovered questions to PENDING with stable IDs from `state.py qid`.
8. If any approved questions need wording help, run `map_linguist` once for that subset.
9. Present all approved questions together in a compact numbered list.

Discovery/review semantics live in their agent TOMLs. Globally enforce only these standards:

- unresolved ambiguity deserves user attention only when it can materially affect correctness, usability, relevant safety/privacy/performance, internal coherence, or objective verification
- optional capabilities and safely postponable implementation choices default out of scope once direction is concrete
- compare semantic decisions, not wording, against SUMMARY/goals, PENDING, `.discovery`, and the same batch
- a compound candidate must not lose a necessary constituent merely because another constituent is unnecessary; Reviewer must preserve independently necessary decisions
- `EXHAUSTED` means no additional worthwhile decision belongs in the current batch; pending questions need not be answered and fewer than 5 is correct

If Discovery returns zero candidates with exhaustion claimed and Reviewer rejects exhaustion, allow exactly one focused Discovery+Reviewer retry. If still unresolved, surface the audit focus instead of looping.

## Answers and persistence

When the user answers:

1. Associate answers with stable PENDING IDs and preserve materially relevant wording/clarifications.
2. Mark answered entries `answered_unpersisted` before semantic persistence. Ask only the smallest direct clarification if an answer cannot be persisted without guessing.
3. Build a Decision Packet containing approved question text, user answer, material clarification, current SUMMARY, and optional context report.
4. Spawn `map_goal_writer` with `MODE: DECISIONS`; it stages affected goals/deferred files plus refreshed README/SUMMARY as one transaction.
5. Close Writer. Spawn `map_goal_reviewer` once with the Decision Packet and staged envelope. Close Reviewer.
6. On PASS, commit mechanically, remove persisted PENDING entries, reopen `.discovery` batch while retaining valid findings, and clean `.tmp`.
7. On FAIL, discard stale candidates and allow one fresh Writer+Reviewer repair using the original Decision Packet plus exact deficiencies. If it fails again, stop and report defects.
8. Return to Discovery with refreshed durable state.

Use the same writer path with `MODE: DIRECTION_CORRECTION` when confirmed direction changes materially.

## Completion

When Discovery returns no approved questions, exhaustion passes, and PENDING is empty, spawn `map_completion_auditor` once.

- PASS: tell the user goals appear complete and ask explicit authorization to finalize.
- FAIL: allow at most one focused Discovery batch using the auditor's unresolved areas as AUDIT_FOCUS. If the same blocker remains, surface it.

If the user adds/changes requirements instead of approving, persist them and continue Map.

## Finalization

Only after explicit user approval:

1. Spawn `map_beads_writer`; close it.
2. Spawn `map_beads_reviewer`; close it.
3. On reviewer failure, allow one writer correction and one final review. Then stop on any remaining defect.
4. On success, active goals remain authoritative; Beads map back to source goals; deferred work stays excluded.
5. Run `state.py finalize-cleanup <GOALS_ROOT>` and stop.

Do not implement product code.

## Parent-owned mechanics

The parent owns PENDING/.discovery bookkeeping and staging commits. Use `state.py` for bootstrap, stable question IDs, baseline/path validation, temp cleanup, and final cleanup. Do not spawn agents for filesystem clerical work.

All semantic writers stage complete candidate files directly in `goals/.tmp/`; `.tmp/` never mirrors final directories. Delete stale candidates before retries and used candidates immediately after commit.

## Completion boundary

Map is complete when a fresh capable implementation agent can read the finalized repository and `/goals` package, determine the complete required end state, and objectively judge whether each active goal is satisfied without the Map conversation.

Sufficient definition is the target, not maximal specification.
