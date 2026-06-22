<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# HTTP Tmux Broker MVP Tasklist

## Objective

Create a standalone `metropoleluya-chirho` Rust project for a centralized localhost HTTP broker that coordinates tmux agents by room/topic, stores history in SQLite, and delivers messages into subscribed tmux panes.

## Checklist

- [x] Create project scaffold outside Cairn.
- [x] Add SQLite-backed server state for agents, subscriptions, messages, and deliveries.
- [x] Add HTTP endpoints for register, post, messages, agents, refresh, and health.
- [x] Add CLI commands for server, register, post, watch, console, agents, and refresh.
- [x] Deliver messages with tmux paste-buffer plus delayed double-enter.
- [x] Store tmux liveness, window index, and pane id for registered agents.
- [x] Run format, tests, clippy, and a local smoke.
- [x] Add a Ratatui human room console that shows transcript history, listeners, tmux panes, and topic subscriptions.
