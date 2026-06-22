<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# metropoleluya-chirho

`metropoleluya-chirho` is a single-machine HTTP broker for tmux agent coordination.

One server runs for the computer. Each tmux project/session registers agents into rooms:

- `CAIRN_CHIRHO/gpt_chirho`
- `CAIRN_CHIRHO/claude_chirho`
- `CBNETCHIRHO/gpt_chirho`
- future scoped agents like `CAIRN_CHIRHO/gpt_admin_chirho`

The server stores room membership, message history, delivery attempts, tmux window indexes, pane ids, and liveness in SQLite. It delivers messages into agent panes through `tmux paste-buffer`, then sends Enter, waits one second, and sends Enter again.

## Quick Start

```bash
cargo run -- server --bind 127.0.0.1:37371
```

Register agents:

```bash
cargo run -- register \
  --session CAIRN_CHIRHO \
  --agent gpt_chirho \
  --tmux-target CAIRN_CHIRHO:3 \
  --room cairn-chirho

cargo run -- register \
  --session CAIRN_CHIRHO \
  --agent claude_chirho \
  --tmux-target CAIRN_CHIRHO:1 \
  --room cairn-chirho
```

Post to a room:

```bash
cargo run -- post \
  --from-session CAIRN_CHIRHO \
  --from-agent gpt_chirho \
  --room cairn-chirho \
  --topic audit-chirho \
  --body "Please audit the current diff."
```

Watch a room:

```bash
cargo run -- watch --room cairn-chirho
```

Open a simple talk console:

```bash
cargo run -- console \
  --session CAIRN_CHIRHO \
  --agent lj_chirho \
  --room cairn-chirho \
  --topic direction-chirho
```

## HTTP API

- `GET /health_chirho`
- `POST /v1/register_chirho`
- `POST /v1/post_chirho`
- `GET /v1/messages_chirho?room_chirho=<room>&after_chirho=<id>`
- `GET /v1/agents_chirho?room_chirho=<room>`
- `POST /v1/refresh_chirho`

The default database is `~/.metropoleluya-chirho/metropoleluya-chirho.sqlite`.

