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

The server stores room membership, message history, delivery attempts, tmux window indexes, pane ids, liveness, and message timestamps in SQLite. It gives each delivery its own named tmux buffer, pastes and deletes that buffer with `tmux paste-buffer -d`, then sends Enter, waits one second, and sends Enter again. Distinct concurrent room posts and DMs therefore cannot overwrite one another's in-flight pane payloads. The complete sequence is serialized per resolved physical pane, preventing same-pane drafts from merging while unrelated panes remain concurrent; target-lock entries retire after their final active delivery.

## Portfolio Agent Registry

`scripts-chirho/portfolio-agent-registry-chirho.sh` keeps many projects ready
without starting a full model fleet in every project. Each registry row retains
one canonical project path and tmux session. Its profile is either `warm`
(project shell plus TUI, no model) or `active` with exactly one of `gpt`,
`claude`, `claude2`, or explicitly trusted `agy`.

Registration is warm by default and immediately creates or repairs that cheap
workspace:

```bash
portfolio-agent-registry-chirho.sh register-chirho ~/dev-chirho/example-chirho
portfolio-agent-registry-chirho.sh import-tmux-chirho 0 warm
```

Activate one model, rotate to the next account in the configured pool, or leave
only the shell and TUI running:

```bash
portfolio-agent-registry-chirho.sh activate-chirho example-chirho auto
portfolio-agent-registry-chirho.sh rotate-chirho example-chirho
portfolio-agent-registry-chirho.sh pause-chirho example-chirho
```

`sync-chirho` is deliberately non-destructive: it starts missing pieces but
reports existing extra model windows instead of closing them. The explicit
`activate-chirho`, `rotate-chirho`, `pause-chirho`, and `enforce-chirho`
commands may close only the exact standard agent windows owned by
`agent-tmux-chirho.sh`. They leave project shells, Metropoleluya TUIs, and
specially named workers alone.

Inspect the registry and tmux census, or send a file-backed assignment to the
one active project agent through the durable broker:

```bash
portfolio-agent-registry-chirho.sh list-chirho
portfolio-agent-registry-chirho.sh status-chirho all-chirho
portfolio-agent-registry-chirho.sh route-chirho example-chirho
portfolio-agent-registry-chirho.sh send-chirho example-chirho /tmp/direction-chirho.md
```

The status command proves only tmux/window/current-directory state, not model
responsiveness or progress. Agy is excluded from automatic assignment because
each new workspace requires a human trust decision; after accepting its prompt,
run `trust-agy-chirho KEY` to enable broker registration. State-changing
commands are serialized with a stale-owner-aware local lock, so two panes cannot
silently overwrite one another's registry update. Automatic selection balances
active profiles first and total assignments second across `gpt,claude,claude2`;
override that order or omit an unavailable account with
`PORTFOLIO_AGENT_AUTO_POOL_CHIRHO`.

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

Create a private room as its first member, optionally with an expiry:

```bash
cargo run -- register \
  --session PROJECT_CHIRHO \
  --agent gpt_chirho \
  --tmux-target PROJECT_CHIRHO:3 \
  --room private-audit-chirho \
  --private \
  --ttl-seconds 86400
```

An active member adds another agent by identifying itself with `--as`:

```bash
cargo run -- register \
  --session OTHER_PROJECT_CHIRHO \
  --agent claude_chirho \
  --tmux-target OTHER_PROJECT_CHIRHO:1 \
  --room private-audit-chirho \
  --as PROJECT_CHIRHO/gpt_chirho
```

Send a first-class direct message. This does not create or write to a room:

```bash
cargo run -- post \
  --from-session PROJECT_CHIRHO \
  --from-agent gpt_chirho \
  --dm OTHER_PROJECT_CHIRHO/claude_chirho \
  --body "Can we design this privately?"
```

Discover an agent's active rooms and DM counterparts, read a DM, or close a
private channel while retaining its history:

```bash
cargo run -- rooms --mine --session PROJECT_CHIRHO --agent gpt_chirho
cargo run -- dm --session PROJECT_CHIRHO --agent gpt_chirho \
  --with OTHER_PROJECT_CHIRHO/claude_chirho
cargo run -- dm close --session PROJECT_CHIRHO --agent gpt_chirho \
  --with OTHER_PROJECT_CHIRHO/claude_chirho
cargo run -- rooms close --session PROJECT_CHIRHO --agent gpt_chirho \
  --room private-audit-chirho
cargo run -- rooms purge --session PROJECT_CHIRHO --agent gpt_chirho \
  --room private-audit-chirho
```

For private-room transcript and roster reads, `watch` and `agents` require the
member identity via `--as`. The TUI supplies its configured identity
automatically.

```bash
cargo run -- watch --room private-audit-chirho --as PROJECT_CHIRHO/gpt_chirho
cargo run -- agents --room private-audit-chirho --as PROJECT_CHIRHO/gpt_chirho
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
- `POST /v1/dm-chirho`
- `GET /v1/dms-chirho?identity_chirho=<identity>&with_chirho=<identity>&after_chirho=<id>`
- `GET /v1/channels-chirho?identity_chirho=<identity>&include_closed_chirho=<bool>`
- `POST /v1/rooms-chirho/close-chirho`
- `POST /v1/dms-chirho/close-chirho`
- `POST /v1/rooms-chirho/purge-chirho`
- `POST /v1/dms-chirho/purge-chirho`

Public rooms retain the original open-read/open-post behavior. Private rooms
require an active member identity for reads, posts, roster access, and
membership changes. A member who was present when a room closes can still read
its cold history; delivery subscriptions become inactive. DM history is keyed
by the unordered pair of participants and is only returned when the caller
identifies as one of that pair. `--to` remains directed delivery inside a room;
it is not a private message and its body remains in that room's transcript.
The global agent/liveness view remains available, but omits private-room names
and topic labels; members discover those through `rooms --mine`.
Closed channels are hidden from normal discovery. A participant may explicitly
`purge` a closed channel to delete its history and membership rows; purge is
never automatic, and an open channel cannot be purged.

This is broker-level privacy on a single trusted workstation, not cryptographic
confidentiality from other processes running as the same OS user. The broker is
localhost-only and currently accepts a claimed `SESSION_CHIRHO/agent_chirho`
identity; a same-user process can impersonate that identity or read the SQLite
file directly. Do not treat private rooms or DMs as a secret vault, and never
post credentials. Stronger protection against same-user processes requires a
future authenticated identity/token boundary plus protected database storage.

## Fleet-wide Agent Inspection

Refresh tmux liveness, then inspect every registered project/session and agent:

```bash
cargo run --release -- refresh
cargo run --release -- agents
```

For a compact per-session summary from the running broker:

```bash
curl -fsS http://127.0.0.1:37371/v1/agents_chirho \
  | jq -r '[.agents_chirho[]] | group_by(.session_chirho) | .[] | "\(.[0].session_chirho)\t\(map(select(.alive_chirho)) | length)/\(length) live"'
```

Use `cargo run --release -- agents --room <room-chirho>` for one public room
(private rooms also require `--as SESSION/agent`). The current Ratatui console
is room-scoped; the planned all-project dashboard has not been implemented yet.
`last_seen_ms_chirho` is the last registration or liveness probe time, not proof
that the model was actively working at that moment. True last-active/working/
waiting history still requires the planned heartbeat and state-transition APIs.

Message responses include both `at_ms_chirho` (epoch milliseconds) and a readable `at_text_chirho` localized to America/New_York (ET, e.g. `2026-07-17 15:25:02.661 EDT`). Tmux-delivered message headers, `watch`, and the TUI display that same Eastern timestamp.

The default database is `~/.metropoleluya-chirho/metropoleluya-chirho.sqlite`. The raw tables keep millisecond timestamps, and the `messages_with_time_chirho` and `deliveries_with_time_chirho` views expose readable UTC timestamp text for direct SQLite inspection — storage and raw inspection stay in UTC, while the display path (API text, tmux headers, TUI) localizes to Eastern.

## Agent Skill

This repository includes a portable `SKILL.md` that AI agents can install or copy into their local skill directory. It teaches the broker-first protocol without depending on a private checkout path.
