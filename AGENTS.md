## Map

If `.map/` exists, treat it as authoritative durable state for user intent, questions, decisions, ideas, facts, dependencies, and semantic history. Session data inside Map is recovery state only.

Do not create or initialize `.map/` merely because these instructions are present.

Before making choices that may depend on established user intent, query Map rather than reconstructing that intent from chat history or guessing. Prefer the smallest relevant read, usually `context`, `show`, or `search`.

Use the Map CLI at `{{MAP_CLI}}`. Invoke that path using the active shell's normal executable syntax. Do not read or modify `.map/db` directly and do not execute SurrealQL as a substitute for the CLI.

Common read-only commands:

```text
{{MAP_CLI}} status
{{MAP_CLI}} get intents
{{MAP_CLI}} get questions
{{MAP_CLI}} show <id>
{{MAP_CLI}} context <id>
{{MAP_CLI}} search "<query>"
{{MAP_CLI}} validate
```

If the user explicitly invokes `$map` or asks to start/resume Map clarification, follow the `map` skill workflow. Map requires the named specialists defined by that workflow. The parent should invoke the exact named specialist for each required stage; the specialist definition owns its semantic contract. Do not replace a required specialist with a generic child or improvised parent-thread judgment; if a required named specialist cannot run, report that stage as blocked.

Outside an explicit Map workflow, ordinary agents may query Map as a read-only primitive. Do not silently create questions, decisions, intents, replacements, abandonment, forced deletion, or closure merely because Map exists.

For additional commands and exact flags:

```text
{{MAP_CLI}} --help
{{MAP_CLI}} <command> --help
```
