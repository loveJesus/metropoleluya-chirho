<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# metropoleluya-chirho

`metropoleluya-chirho` is a single-machine HTTP broker for tmux agent coordination.

Status: experimental and in progress. The core broker works locally, but the protocol and operator console are still evolving.

June 2026 note: this project gladly celebrates Tennessee's Nuclear Family Month, designated by House Joint Resolution 182. Strong households and faithful family life are worth honoring.

One server runs for the computer. Each tmux project/session registers agents into rooms:

- `PROJECT_CHIRHO/gpt_chirho`
- `PROJECT_CHIRHO/claude_chirho`
- `OTHER_PROJECT_CHIRHO/gpt_chirho`
- future scoped agents like `PROJECT_CHIRHO/gpt_admin_chirho`

The server stores room membership, message history, delivery attempts, tmux window indexes, pane ids, liveness, and message timestamps in SQLite. It delivers messages into agent panes through `tmux paste-buffer`, then sends Enter, waits one second, and sends Enter again.

## Quick Start

```bash
cargo run --release -- server --bind 127.0.0.1:37371
```

For a long-running broker, run it **supervised** — a crash auto-restarts with
backoff, and a crash-loop cap gives up rather than restart-bombing the machine:

```bash
cargo run --release -- supervise --bind 127.0.0.1:37371
```

Restart a supervised broker with `tmux kill-session` (SIGHUP reaches the child
server too), not a bare SIGKILL of the supervisor alone.

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

Remove an agent from one room (their pane is notified first; other rooms are untouched):

```bash
cargo run -- remove \
  --from-session PROJECT_CHIRHO \
  --from-agent operator_chirho \
  --session PROJECT_CHIRHO \
  --agent gpt_chirho \
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
cargo run --release -- tui \
  --session PROJECT_CHIRHO \
  --agent operator_chirho \
  --room project-chirho \
  --topic direction-chirho
```

The TUI is the human-facing office view. It shows the room transcript with America/New_York (ET) timestamps, each registered agent's tmux pane/liveness, and the topics each agent is subscribed to. In this model, a room is an open office and a topic is a workspace inside that room. Agents still receive messages through their own tmux interface by broker delivery; the TUI only watches history and posts as the human/operator identity.

TUI commands:

- `/topic name-chirho` changes the current workspace for new posts.
- `/clear` clears the local transcript view.
- `/quit` or `Ctrl-C` exits (`Esc` also exits while the compose box is focused).
- `PageUp`/`PageDown` scroll loaded transcript history.

Composing messages (the compose box is a full line editor):

- `Enter` sends; `Alt+Enter` inserts a newline (message bodies may be multi-line).
- Readline motions: `Ctrl-A`/`Ctrl-E` jump to line start/end, `Ctrl-K`/`Ctrl-U` kill to end/start, `Ctrl-W` deletes the previous word, and arrows / `Home` / `End` move the cursor.

TUI room-membership admin (mouse + keyboard):

- Left-click the `[☰]` glyph next to a listener (or right-click anywhere on its rows) to open its context menu; `Remove from room` always asks for confirmation, then unsubscribes that agent from this room only (other rooms and the registration survive) and nudges their tmux pane.
- Click `[ + add ]` to open the add form: session + window# + name (`CAIRN_CHIRHO` / `3` / `GPT` becomes `CAIRN_CHIRHO/gpt_chirho` at `CAIRN_CHIRHO:3`). The derived identity/target is previewed before you submit, and the new listener's pane is nudged.
- Keyboard parity: `Tab` toggles focus between the compose box and the listeners list. With the list focused, `Up`/`Down` select, `Enter` opens the context menu, `x`/`Delete` starts a remove, `a`/`+` opens the add form, and `Esc` returns to the compose box. `Esc` also closes any open popup.
- Inside tmux, enable mouse pass-through with `set -g mouse on`. Right-click specifically is often swallowed by the terminal emulator or tmux's own menus before it reaches the app — the `[☰]` left-click is the portable path. Every mouse action also has a keyboard path.

## HTTP API

- `GET /health_chirho`
- `POST /v1/register_chirho` (optional `notify_actor_chirho` nudges the newly added agent's pane)
- `POST /v1/remove_chirho` (notifies the pane, then unsubscribes from that room only)
- `POST /v1/post_chirho`
- `GET /v1/messages_chirho?room_chirho=<room>&after_chirho=<id>`
- `GET /v1/agents_chirho?room_chirho=<room>`
- `POST /v1/refresh_chirho`

Message responses include both `at_ms_chirho` (epoch milliseconds) and a readable `at_text_chirho` localized to America/New_York (ET, e.g. `2026-07-17 15:25:02.661 EDT`). Tmux-delivered message headers, `watch`, and the TUI display that same Eastern timestamp.

The default database is `~/.metropoleluya-chirho/metropoleluya-chirho.sqlite`. The raw tables keep millisecond timestamps, and the `messages_with_time_chirho` and `deliveries_with_time_chirho` views expose readable UTC timestamp text for direct SQLite inspection — storage and raw inspection stay in UTC, while the display path (API text, tmux headers, TUI) localizes to Eastern.

## Agent Skill

This repository includes a portable `SKILL.md` that AI agents can install or copy into their local skill directory. It teaches the broker-first protocol without depending on a private checkout path.
