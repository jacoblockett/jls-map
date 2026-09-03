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
| `create intent|question|decision|idea|fact ...` | Create a node. |
| `relate <SOURCE> <TARGET>... [--dependent]` | Add inferred relationships. |
| `unrelate <SOURCE> <TARGET>... [--dependent]` | Remove inferred relationships. |
| `set ...` | Set Map or node properties. |
| `replace <OLD> <NEW> --reason <TEXT> [--in-place]` | Replace a node, preserving history. |
| `abandon <ID> --by user|assistant --reason <TEXT>` | Abandon a node. |
| `delete <ID>... [--force]` | Delete nodes. |
| `get intents|questions|decisions|ideas|facts ...` | List matching current IDs. |
| `show <ID>...` | Show node data. |
| `context <ID>` | Show related graph context. |
| `status` | Show Map/runtime status. |
| `validate` | Validate graph invariants. |
| `search <QUERY> [--limit N] [--include-history]` | Search nodes. |
| `history <ID> [--limit N]` | Show replacement history. |
| `session init|summary|exchange|pending|end ...` | Manage recovery state. |

## Output

Successful commands emit JSON to stdout. Errors use `map: <error>` on stderr and exit non-zero.

`validate` reports graph validity in JSON via `ok` and `errors`.

Node IDs are opaque 20-character lowercase alphanumeric strings. Historical IDs generally resolve to their current replacement target.

`.map/db` is runtime-managed; use the CLI rather than editing it directly.
