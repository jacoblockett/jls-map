# Map CLI reference

This document describes the Map command-line interface.

Examples use `map` as the executable name. When installed through JLS, use the installer-provisioned Map CLI path for the active platform.

## Synopsis

```text
map [--path PATH] [--config PATH] <command>
```

Top-level commands:

```text
init
create
relate
unrelate
set
replace
abandon
delete
get
show
context
status
validate
search
history
session
```

Built-in help:

```text
map --help
map <command> --help
map create <kind> --help
map get <kind> --help
map session <command> --help
```

Version:

```text
map --version
```

## Output and errors

Successful runtime commands emit pretty-printed JSON to standard output. Help and version output are plain text.

Runtime failures are written to standard error:

```text
map: <error>
```

Failures exit with a non-zero status.

`map validate` is different from most failures. Graph validation errors are returned as JSON:

```json
{
  "ok": false,
  "errors": [
    "..."
  ]
}
```

Inspect the `ok` field to determine whether validation passed.

## Map location

Global location options:

```text
--path PATH
--config PATH
```

Examples:

```text
map --path /path/to/project status
map --path /path/to/project/.map status
map --config /path/to/.maprc status
```

`--path` may point to either a project root or its `.map` directory.

For commands other than `init`, the resolved location must contain an existing `.map` directory.

### Location precedence

For an existing Map, path selection follows this order:

1. `--path`.
2. `path` from an explicit `--config` file or directory.
3. `path` from `.maprc` in the current working directory.
4. The current working directory.

If an explicitly selected location does not contain a Map, the command fails rather than falling back to another location.

### `.maprc`

Map configuration uses TOML:

```toml
path = "../project"
schema = "../map-runtime/schema.surql"
```

Both fields are optional.

Relative paths are resolved relative to the directory containing the config file. `~`, `~/...`, and `~\...` are expanded using the user's home directory when available.

`--config` may point directly to a config file or to a directory containing `.maprc`.

## Initialization

```text
map [--path PATH] [--config PATH] init [--schema PATH]
```

Examples:

```text
map init
map --path /path/to/project init
map --path /path/to/project init --schema /path/to/schema.surql
```

The target may be an existing project directory or a `.map` path whose parent already exists. Initialization creates the `.map` directory but does not create missing project parent directories.

Initialization fails if the target `.map` already exists.

### Schema selection

Schema resolution order:

1. `init --schema PATH`.
2. `schema` from explicit `--config`.
3. `schema` from cwd `.maprc`.
4. The schema packaged with the installed runtime.

Initial Map metadata:

```text
depth:  mvp
stance: normal
```

Successful initialization returns JSON containing:

```json
{
  "ok": true,
  "path": ".../.map",
  "schema": ".../schema.surql",
  "schemaVersion": "2",
  "runtimeVersion": "...",
  "projectId": "..."
}
```

## On-disk files

A Map stores its project identity at:

```text
.map/project.json
```

The embedded database is stored under:

```text
.map/db
```

The project ID is created during initialization and preserved when the Map directory is moved or copied. Normal runtime commands require a valid project identity.

`.map/db` is runtime-managed and should not be edited directly.

## IDs

New graph nodes receive opaque 20-character IDs containing lowercase ASCII letters and digits.

Example shape:

```text
7k2h8s4m1p9x3q6v0abc
```

Most commands use bare IDs.

### Replacement-aware IDs

Commands that resolve a current node generally accept a historical replacement ID and resolve it to the current replacement target.

For example, after replacing `OLD` with `NEW`:

```text
map show OLD
```

returns the current `NEW` node.

`history` exposes the replacement chain itself. `delete` operates on stored records and replacement metadata rather than treating historical IDs only as aliases.

## Semantic model

Node kinds:

| Kind | Meaning |
| --- | --- |
| `intent` | Goal, desired outcome, or scoped objective |
| `question` | Unresolved decision or clarification |
| `decision` | Settled or provisional answer or choice |
| `idea` | Possible concept that is not itself a settled decision |
| `fact` | Contextual evidence or established information |

Relationship kinds:

| Relationship | Meaning |
| --- | --- |
| `contains` | Intent contains an intent, question, or decision |
| `answers` | Decision answers a question |
| `depends_on` | Intent depends on an intent, or question depends on a question |
| `fact_context` | Node is associated with a fact |
| `idea_context` | Node is associated with an idea |

`relate` and `unrelate` infer relationship kinds from the source and target node kinds.

## Creating nodes

### Intent

```text
map create intent <INTENT> [--context TEXT] [--depth mvp|thorough] [--stance normal|adversarial]
```

Example:

```text
map create intent "Add account recovery"
```

New intents start with:

```text
explored = false
closed   = false
```

`--depth` and `--stance` are optional intent-level overrides. Without an override, the intent inherits the Map-level value.

### Question

```text
map create question <QUESTION> --intent <INTENT_ID> [--reason TEXT]
```

Example:

```text
map create question "How long should reset links remain valid?" \
  --intent 7k2h8s4m1p9x3q6v0abc \
  --reason "Expiry affects security and recovery UX"
```

The parent must be a current, non-abandoned intent.

Creating a question also creates a `contains` relationship from the parent intent. New questions start with `asked = false`.

Adding an unresolved question beneath a closed intent can reopen that intent.

### Decision

```text
map create decision <DECISION> \
  [--question <QUESTION_ID>] \
  [--source user|assistant] \
  [--assistant-reasoning TEXT] \
  [--notes TEXT] \
  [--soft]
```

`--source` defaults to `user`.

If `--question` is supplied, the decision is linked as that question's answer. A question may have only one current answer.

Assistant-authored decisions require `--assistant-reasoning`:

```text
map create decision "Use a 30 minute expiry" \
  --source assistant \
  --assistant-reasoning "Derived from the established security requirements"
```

`--assistant-reasoning` is invalid with `--source user`.

`--soft` marks the decision as provisional. A non-abandoned soft decision prevents affected intent scope from closing.

### Idea

```text
map create idea <IDEA>
```

Example:

```text
map create idea "Consider passkey-based recovery later"
```

### Fact

```text
map create fact <FACT> [--made-by user|assistant]
```

`--made-by` defaults to `user`.

Example:

```text
map create fact "The current auth service already supports one-time tokens" --made-by assistant
```

Successful `create` commands return the new node ID:

```json
{
  "id": "..."
}
```

## Relationships

### Relate

```text
map relate <SOURCE_ID> <TARGET_ID>... [--dependent]
```

### Unrelate

```text
map unrelate <SOURCE_ID> <TARGET_ID>... [--dependent]
```

Multiple targets may be supplied in one command.

Without `--dependent`:

| Source | Target | Relationship |
| --- | --- | --- |
| intent | intent | `contains` |
| intent | question | `contains` |
| intent | decision | `contains` |
| question | decision | `answers` |
| any node | fact | `fact_context` |
| any node | idea | `idea_context` |

With `--dependent`:

| Source | Target | Relationship |
| --- | --- | --- |
| intent | intent | `depends_on` |
| question | question | `depends_on` |

Question-to-question relationships require `--dependent`.

The runtime rejects illegal relationship shapes, duplicate relationships, removal of relationships that do not exist, multiple current answers to one question, dependency cycles, containment cycles, and other graph-invariant violations.

Successful output:

```json
{
  "ok": true,
  "operation": "relate",
  "relationships": [
    {
      "kind": "depends_on",
      "source": "...",
      "target": "..."
    }
  ]
}
```

Removing or adding relationships may reopen affected intents when prior closure is no longer valid.

## Setting properties

`set` supports Map-level and node-level properties.

Map-level form:

```text
map set <PROPERTY> <VALUE>
```

Node-level form:

```text
map set <ID> <PROPERTY> <VALUE>
```

### Map-level properties

```text
map set depth mvp
map set depth thorough
map set stance normal
map set stance adversarial
```

Changing inherited depth from `mvp` to `thorough`, or inherited stance from `normal` to `adversarial`, can reopen closed intents whose effective setting becomes more rigorous. Intent-level overrides are respected.

### Intent properties

```text
map set <INTENT_ID> explored true|false
map set <INTENT_ID> close true|false
map set <INTENT_ID> depth mvp|thorough|null
map set <INTENT_ID> stance normal|adversarial|null
```

`null` removes the intent-level depth or stance override and restores inheritance from Map-level metadata.

`close true` fails when the intent is not currently closable. See [Intent closure](#intent-closure).

### Question properties

```text
map set <QUESTION_ID> asked true|false
```

### Decision properties

```text
map set <DECISION_ID> soft true|false
```

Setting `soft = true` can reopen affected intents.

### Keywords

All node kinds support keywords:

```text
map set <ID> keywords '<JSON_STRING_ARRAY>'
```

Example:

```text
map set 7k2h8s4m1p9x3q6v0abc keywords '["auth","recovery","password reset"]'
```

The value must be a valid JSON array of strings. Shell quoting rules vary by platform.

## Replacing nodes

```text
map replace <OLD_ID> <NEW_ID> --reason <TEXT> [--in-place]
```

Requirements:

- old and new must resolve to different current nodes;
- both nodes must have the same kind;
- both must be non-abandoned;
- the new node cannot already have replacement history that would merge histories;
- `--reason` must be non-empty.

The old node's graph relationships are transferred to the new node and replacement metadata is recorded.

### Normal replacement

Without `--in-place`, the old node remains stored as historical state. Current-node resolution points to the replacement target.

### In-place replacement

With `--in-place`, the old node record is removed while the replacement event remains in history.

Replacement can reopen intents if the resulting graph invalidates prior closure.

Successful output includes:

```text
ok
old
new
mode
reason
reopened
```

## Abandoning nodes

```text
map abandon <ID> --by user|assistant --reason <TEXT>
```

All node kinds may be abandoned. `--reason` must be non-empty.

Abandonment records:

```text
abandoned = true
abandonedBy
abandonedReason
```

Abandoned nodes remain stored and inspectable.

Abandoning an answer decision can make its question unresolved again and reopen affected intents. Abandoning an intent used as a dependency can reopen dependent intents.

## Deleting nodes

```text
map delete <ID>... [--force]
```

Deletion removes stored nodes.

If a selected node participates in semantic relationships or replacement history, deletion fails unless `--force` is supplied. The error reports affected relationships.

Forced deletion still enforces replacement-history safety. Deleting a replacement target cannot silently revive a historical predecessor; the runtime may require related predecessor nodes to be included in the same forced deletion.

Deletion can reopen surviving intents if the resulting graph invalidates prior closure.

## Reading collections with `get`

`get` returns sorted JSON arrays of node IDs.

Several flags are inclusion switches rather than exclusive selectors. For example:

```text
map get intents
```

returns current open, non-abandoned intents.

```text
map get intents --closed
```

includes closed intents in addition to open intents.

Likewise, `--abandoned` generally includes abandoned nodes, and `get questions --answered` includes answered questions in addition to unresolved ready questions.

### Get intents

```text
map get intents \
  [--id <ID>...] \
  [--explored | --unexplored] \
  [--closed] \
  [--abandoned] \
  [--limit N]
```

Default behavior:

- current IDs only;
- excludes abandoned intents;
- excludes closed intents.

Options:

- `--id`: restrict to supplied IDs after current replacement resolution.
- `--explored`: restrict to explored intents.
- `--unexplored`: restrict to intents not marked explored.
- `--closed`: include closed intents.
- `--abandoned`: include abandoned intents.
- `--limit`: truncate the sorted result.

There is no dedicated closed-only flag. To obtain only closed intents, include `--closed` and filter the returned nodes by their `closed` field after `show`.

### Get questions

```text
map get questions \
  [--id <ID>...] \
  [--intent <ID>...] \
  [--asked | --unasked] \
  [--answered] \
  [--abandoned] \
  [--include-blocked] \
  [--limit N]
```

Default behavior returns current, non-abandoned, unanswered questions that are ready.

Options:

- `--id`: restrict by question ID.
- `--intent`: restrict to questions directly contained by supplied intents.
- `--asked`: restrict to questions marked asked.
- `--unasked`: restrict to questions not marked asked.
- `--answered`: include answered questions.
- `--abandoned`: include abandoned questions.
- `--include-blocked`: include unresolved questions whose dependencies currently block them.
- `--limit`: truncate the sorted result.

To enumerate current non-abandoned questions for an intent, including answered and blocked questions:

```text
map get questions --intent <ID> --answered --include-blocked
```

See [Question readiness](#question-readiness) for the readiness rules.

### Get decisions

```text
map get decisions \
  [--id <ID>...] \
  [--question <ID>...] \
  [--intent <ID>...] \
  [--soft] \
  [--decided-by user|assistant] \
  [--abandoned] \
  [--limit N]
```

Default behavior returns all current, non-abandoned decisions.

Options:

- `--id`: restrict by decision ID.
- `--question`: restrict to current answers for supplied questions.
- `--intent`: restrict to decisions directly associated with supplied intents. This includes decisions directly contained by the intent and answers to questions directly contained by it.
- `--soft`: restrict to soft decisions.
- `--decided-by`: restrict by decision source.
- `--abandoned`: include abandoned decisions.
- `--limit`: truncate the sorted result.

`--intent` is direct rather than recursive. `context <INTENT_ID>` provides recursive `scopeDecisions` for the intent subtree.

### Get ideas

```text
map get ideas [--id <ID>...] [--abandoned] [--limit N]
```

Default behavior returns current, non-abandoned ideas.

Options:

- `--id`: restrict by idea ID.
- `--abandoned`: include abandoned ideas.
- `--limit`: truncate the sorted result.

### Get facts

```text
map get facts \
  [--id <ID>...] \
  [--made-by user|assistant] \
  [--abandoned] \
  [--limit N]
```

Default behavior returns current, non-abandoned facts.

Options:

- `--id`: restrict by fact ID.
- `--made-by`: restrict by fact provenance.
- `--abandoned`: include abandoned facts.
- `--limit`: truncate the sorted result.

## Showing nodes

```text
map show <ID>...
```

One ID returns one JSON object. Multiple IDs return a JSON array.

Historical replacement IDs resolve to their current node.

Common node fields:

```text
id
kind
text
keywords
abandoned
abandonedReason   when abandoned
abandonedBy       when abandoned
```

Intent fields may include:

```text
context
explored
closed
depth
stance
effectiveDepth
effectiveStance
```

`effectiveDepth` and `effectiveStance` include inherited Map-level values when the intent has no override.

Question fields may include:

```text
reason
asked
answered
ready
```

`answered` and `ready` are computed from current graph state.

Decision fields may include:

```text
source
assistantReasoning
notes
soft
```

Fact-specific output includes:

```text
madeBy
```

## Context

```text
map context <ID>
```

`context` returns the current semantic neighborhood around a node. It rejects an abandoned focused node.

Payload fields:

```text
node
parents
children
dependencies
dependents
facts
ideas
```

For an answered question:

```text
answer
```

For an intent:

```text
scopeQuestions
scopeDecisions
```

Field meanings:

- `node`: full `show`-style representation of the focused node.
- `parents`: direct containment parents; for an answer decision, also the answered question.
- `children`: direct `contains` children.
- `dependencies`: outgoing dependency targets with a computed `satisfied` boolean.
- `dependents`: nodes that directly depend on the focused node.
- `facts`: outgoing `fact_context` target IDs.
- `ideas`: outgoing `idea_context` target IDs.
- `answer`: current answer decision ID for an answered question.
- `scopeQuestions`: current non-abandoned questions anywhere in the focused intent subtree.
- `scopeDecisions`: current non-abandoned decisions associated anywhere in the focused intent subtree.

## Status

```text
map status
```

Returned fields include:

```text
path
runtimeVersion
schemaVersion
depth
stance
nodes
historicalReplacements
session
```

`nodes` contains counts for each node kind split between current non-abandoned nodes and abandoned current nodes.

Example:

```json
{
  "path": ".../.map",
  "runtimeVersion": "...",
  "schemaVersion": "2",
  "depth": "thorough",
  "stance": "normal",
  "nodes": {
    "intent": {
      "current": 3,
      "abandoned": 0
    }
  },
  "historicalReplacements": 2,
  "session": false
}
```

`session` reports whether recovery session state currently exists.

## Validation

```text
map validate
```

Validation checks runtime and semantic graph invariants, including:

- schema/runtime compatibility;
- node-kind field validity;
- abandonment metadata consistency;
- relationship endpoint validity;
- duplicate or illegal relationship shapes;
- historical nodes participating in current relationships;
- self dependencies;
- intent containment cycles;
- intent dependency cycles;
- question dependency cycles;
- multiple current answers to a question;
- replacement-chain consistency;
- validity of closed intents.

Successful validation:

```json
{
  "ok": true,
  "errors": []
}
```

Validation failures are returned in the `errors` array with `ok: false`.

## Search

```text
map search <QUERY> [--limit N] [--include-history]
```

`--limit` defaults to `10`.

Search returns a ranked JSON array of node IDs.

Search normalizes case and whitespace and considers:

- node text;
- intent context;
- question reason;
- assistant decision reasoning;
- decision notes;
- keywords.

Exact normalized text and exact keyword matches receive the strongest weighting, followed by phrase and token overlap.

Abandoned nodes are excluded. Historical replacement nodes are excluded unless `--include-history` is supplied.

## Replacement history

```text
map history <ID> [--limit N]
```

Returned fields:

```text
root
current
nodes
events
```

- `root`: earliest ID in the discovered replacement chain.
- `current`: current resolved node ID when resolution succeeds.
- `nodes`: IDs in chain order with node data when the stored record still exists.
- `events`: replacement metadata, including old ID, new ID, reason, mode, and creation time.

For an in-place replacement, a historical entry may contain `node: null` because the old node record was removed while the replacement event remained.

## Recovery sessions

### Initialize

```text
map session init
```

Fails if a recovery session already exists.

Initial values:

```text
summary:   empty
depth:     6
exchanges: empty
pending:   none
```

### Summary

Read:

```text
map session summary
```

Set:

```text
map session summary <TEXT>
```

Summary whitespace is normalized. Maximum length is 2200 Unicode characters.

### Exchanges

Read:

```text
map session exchange
```

Append a user message:

```text
map session exchange -u <TEXT>
map session exchange --user <TEXT>
```

Append an assistant message:

```text
map session exchange -a <TEXT>
map session exchange --assistant <TEXT>
```

Set retention depth:

```text
map session exchange --depth N
```

`--user` and `--assistant` conflict with each other. Exchange depth must be at least `2`. When the retained exchange count exceeds the configured depth, the oldest exchanges are removed.

### Pending

Read:

```text
map session pending
```

Set:

```text
map session pending <TEXT>
```

Clear:

```text
map session pending --clear
```

### End

```text
map session end [--force]
```

A session with pending state cannot end without `--force`.

## Depth and stance

Depth values:

```text
mvp
thorough
```

Stance values:

```text
normal
adversarial
```

Intents may override either Map-level value. `show` reports stored overrides and computed effective values.

## Question readiness

Question readiness is computed by the runtime and exposed through `show` and the default behavior of `get questions`.

A question is ready when:

1. it is current;
2. it is not abandoned;
3. it is unanswered;
4. it belongs to at least one current intent;
5. each question dependency is answered or abandoned;
6. relevant ancestor intent dependencies are satisfied.

For an intent dependency to be satisfied, the prerequisite intent must be non-abandoned and closed.

## Intent closure

`closed = true` is runtime-enforced.

A current, non-abandoned intent can close only when:

1. the intent is marked explored;
2. every non-abandoned child intent in its recursive scope is closed;
3. relevant intent dependencies are satisfied;
4. every non-abandoned question in scope is answered;
5. every non-abandoned decision in scope is not soft.

Changes that invalidate existing closure can automatically reopen affected intents or ancestors. These include adding unresolved questions, adding dependencies, making decisions soft, abandoning answers, increasing effective depth or stance, replacing nodes, changing relationships, and deleting state.

## Command classification

Read-only graph commands:

```text
get
show
context
status
validate
search
history
```

Mutation commands:

```text
init
create
relate
unrelate
set
replace
abandon
delete
```

Recovery-session commands are grouped under:

```text
session
```
