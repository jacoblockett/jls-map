# Map CLI reference

```text
map [--path PATH] [--config PATH] <command>
```

Use `map --help`, `map <command> --help`, or `map --version` for exact syntax.

## Global options

- `--path PATH` — project root or `.map` directory.
- `--config PATH` — `.maprc` file or directory containing one.

Path precedence: `--path` → explicit config → cwd `.maprc` → cwd.

Example:

```text
map --path /path/to/project status
```

## Commands

| Command | Purpose |
| --- | --- |
| `init [--schema PATH]` | Create a `.map`. |
| `create intent <TEXT> [--context TEXT] [--depth ...] [--stance ...]` | Create an intent. |
| `create question <TEXT> --intent <ID> [--reason TEXT]` | Create a question. |
| `create decision <TEXT> [--question ID] [--source ...] [--assistant-reasoning TEXT] [--notes TEXT] [--soft]` | Create a decision. |
| `create idea <TEXT>` / `create fact <TEXT> [--made-by ...]` | Create an idea or fact. |
| `relate <SOURCE> <TARGET>... [--dependent]` | Add inferred relationships. |
| `unrelate <SOURCE> <TARGET>... [--dependent]` | Remove inferred relationships. |
| `set ...` | Set Map or node properties. |
| `replace <OLD> <NEW> --reason <TEXT> [--in-place]` | Replace a node, preserving history. |
| `abandon <ID> --by <ACTOR> --reason <TEXT>` | Abandon a node. `ACTOR`: `user` or `assistant`. |
| `delete <ID>... [--force]` | Delete nodes. |
| `get <KIND> ...` | List matching current IDs. `KIND`: `intents`, `questions`, `decisions`, `ideas`, or `facts`. |
| `show <ID>...` | Show node data. |
| `context <ID>` | Show related graph context. |
| `status` | Show Map/runtime status. |
| `validate` | Validate graph invariants. |
| `search <QUERY> [--limit N] [--include-history]` | Search nodes. |
| `history <ID> [--limit N]` | Show replacement history. |
| `session <ACTION> ...` | Manage recovery state. `ACTION`: `init`, `summary`, `exchange`, `pending`, or `end`. |

Common `set` forms:

```text
map set depth mvp|thorough
map set stance normal|adversarial
map set <ID> <PROPERTY> <VALUE>
```

Use subcommand help for supported node properties and `get` filters.

## Notes

Successful commands emit JSON to stdout. Errors use `map: <error>` on stderr and exit non-zero.

`validate` reports graph validity in JSON via `ok` and `errors`; inspect `ok` rather than treating validation errors as a normal command failure.

Several `get` flags are inclusion switches. For example, `--closed`, `--answered`, and `--abandoned` include those states in addition to the default result set rather than selecting only those states.

Node IDs are opaque 20-character lowercase alphanumeric strings. Historical IDs generally resolve to their current replacement target; use `history` to inspect replacement chains.

`.map/db` is runtime-managed; use the CLI rather than editing it directly.
