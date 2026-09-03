# Map CLI reference

```text
map [--path PATH] [--config PATH] <command>
```

Use `map --help`, `map <command> --help`, or `map --version` for exact syntax.

## Global options

- `--path PATH` — project root or `.map` directory.
- `--config PATH` — `.maprc` file or directory containing one.

Path precedence: `--path` → explicit config → cwd `.maprc` → cwd.

## Commands

| Command | Purpose |
| --- | --- |
| `init [--schema PATH]` | Create a `.map`. |
| `create intent <TEXT> ...` | Create an intent. |
| `create question <TEXT> --intent <ID> ...` | Create a question. |
| `create decision <TEXT> ...` | Create a decision. |
| `create idea <TEXT>` / `create fact <TEXT> ...` | Create an idea or fact. |
| `relate <SOURCE> <TARGET>... [--dependent]` | Add inferred relationships. |
| `unrelate <SOURCE> <TARGET>... [--dependent]` | Remove inferred relationships. |
| `set ...` | Set Map or node properties. |
| `replace <OLD> <NEW> --reason <TEXT> [--in-place]` | Replace a node, preserving history. |
| `abandon <ID> --by <ACTOR> --reason <TEXT>` | Abandon a node. |
| `delete <ID>... [--force]` | Delete nodes. |
| `get <KIND> ...` | List matching current IDs. |
| `show <ID>...` | Show node data. |
| `context <ID>` | Show related graph context. |
| `status` | Show Map/runtime status. |
| `validate` | Validate graph invariants. |
| `search <QUERY> [--limit N] [--include-history]` | Search nodes. |
| `history <ID> [--limit N]` | Show replacement history. |
| `export [-f FORMAT] [--include-history] [--include-abandoned] [-o PATH]` | Export the current graph. |
| `session <ACTION> ...` | Manage recovery state. |

`ACTOR`: `user|assistant`. `KIND`: `intents|questions|decisions|ideas|facts`.

Common `set` forms:

```text
map set depth mvp|thorough
map set stance normal|adversarial
map set <ID> <PROPERTY> <VALUE>
```

## Export

`export` writes JSON to stdout by default. `--format`/`-f` supports `json`, `yaml`, or `toml`. `--output`/`-o` writes to a file; the target is checked for writability before the Map is opened.

Exports contain Map settings, validation results, current nodes, and normalized relationships. Abandoned nodes are omitted unless `--include-abandoned` is set. Replacement history is omitted unless `--include-history` is set. Session/recovery state is never exported. References remain node IDs.

## Notes

Successful commands emit JSON to stdout except `export` when another format or output file is selected. Errors use `map: <error>` on stderr and exit non-zero.

`validate` reports validity in JSON via `ok` and `errors`; inspect `ok`.

Several `get` flags are inclusion switches: `--closed`, `--answered`, and `--abandoned` add those states to the default result set.

Node IDs are opaque 20-character lowercase alphanumeric strings. Historical IDs generally resolve to their current replacement target; use `history` to inspect replacement chains.

`.map/db` is runtime-managed; use the CLI rather than editing it directly.
