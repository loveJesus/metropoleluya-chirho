<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# metropoleluya-chirho

`metropoleluya-chirho` is a single-machine HTTP broker for tmux agent coordination.

Status: experimental and in progress. The core broker works locally, but the protocol and operator console are still evolving.

One server runs for the computer. Each tmux project/session registers agents into rooms:

- `PROJECT_CHIRHO/gpt_chirho`
- `PROJECT_CHIRHO/claude_chirho`
- `OTHER_PROJECT_CHIRHO/gpt_chirho`
- future scoped agents like `PROJECT_CHIRHO/gpt_admin_chirho`

The server stores room membership, message history, delivery attempts, tmux window indexes, pane ids, and liveness in SQLite. It delivers messages into agent panes through `tmux paste-buffer`, then sends Enter, waits one second, and sends Enter again.

## Quick Start

```bash
cargo run -- server --bind 127.0.0.1:37371
```

Register agents:

```bash
cargo run -- register \
  --session PROJECT_CHIRHO \
  --agent gpt_chirho \
  --tmux-target PROJECT_CHIRHO:3 \
  --room project-chirho

cargo run -- register \
  --session PROJECT_CHIRHO \
  --agent claude_chirho \
  --tmux-target PROJECT_CHIRHO:1 \
  --room project-chirho
```

Post to a room:

```bash
cargo run -- post \
  --from-session PROJECT_CHIRHO \
  --from-agent gpt_chirho \
  --room project-chirho \
  --topic audit-chirho \
  --body "Please audit the current diff."
```

Watch a room:

```bash
cargo run -- watch --room project-chirho
```

Open a simple talk console:

```bash
cargo run -- console \
  --session PROJECT_CHIRHO \
  --agent operator_chirho \
  --room project-chirho \
  --topic direction-chirho
```

Open the human room TUI:

```bash
cargo run -- tui \
  --session PROJECT_CHIRHO \
  --agent operator_chirho \
  --room project-chirho \
  --topic direction-chirho
```

The TUI is the human-facing office view. It shows the room transcript, each registered agent's tmux pane/liveness, and the topics each agent is subscribed to. In this model, a room is an open office and a topic is a workspace inside that room. Agents still receive messages through their own tmux interface by broker delivery; the TUI only watches history and posts as the human/operator identity.

TUI commands:

- `/topic name-chirho` changes the current workspace for new posts.
- `/clear` clears the local transcript view.
- `/quit`, `Esc`, or `Ctrl-C` exits.
- `PageUp`, `PageDown`, `Home`, and `End` navigate loaded transcript history.

## HTTP API

- `GET /health_chirho`
- `POST /v1/register_chirho`
- `POST /v1/post_chirho`
- `GET /v1/messages_chirho?room_chirho=<room>&after_chirho=<id>`
- `GET /v1/agents_chirho?room_chirho=<room>`
- `POST /v1/refresh_chirho`

The default database is `~/.metropoleluya-chirho/metropoleluya-chirho.sqlite`.

## Agent Skill

This repository includes a portable `SKILL.md` that AI agents can install or copy into their local skill directory. It teaches the broker-first protocol without depending on a private checkout path.
