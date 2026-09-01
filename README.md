# Map

Map is a durable local intent graph for AI agents. It preserves user intent, unresolved questions, decisions, facts, ideas, and dependencies so long-running work can survive context loss and resume without reconstructing the project from chat history.

Map runs locally through a Rust CLI backed by embedded SurrealKV. Project state lives under `.map/`; no database server or external service is required. Native agent integrations are provided for OpenAI Codex and Claude Code.

## Install

The recommended installation path is [JLS](https://github.com/jacoblockett/jls), which installs the correct Map package for the current platform and manages its agent integration, updates, and removal.

Release packages are available from [Releases](https://github.com/jacoblockett/jls-map/releases). Nightly is a rolling prerelease from `main`; versioned releases are stable.

Installing Map does not create project state. A `.map/` directory is created only when Map is explicitly initialized for a project.

## Development

Map requires Rust 1.89 or newer.

```bash
cargo test --locked
cargo build --release --locked
```

## License

This project is licensed under the [MIT License](LICENSE).

Copyright © 2026 Jacob Lockett.
