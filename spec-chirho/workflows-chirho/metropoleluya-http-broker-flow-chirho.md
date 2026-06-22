<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Metropoleluya HTTP Broker Flow

```mermaid
flowchart TD
    agent_chirho[Agent CLI or human TUI] --> post_chirho[HTTP POST /v1/post_chirho]
    register_chirho[HTTP POST /v1/register_chirho] --> db_chirho[(SQLite broker DB)]
    post_chirho --> db_chirho
    db_chirho --> select_chirho[Resolve active room and workspace subscribers]
    select_chirho --> live_chirho[Probe tmux target liveness]
    live_chirho --> deliver_chirho[tmux paste-buffer + Enter + delay + Enter]
    deliver_chirho --> delivery_log_chirho[Delivery row]
    delivery_log_chirho --> db_chirho
    db_chirho --> watch_chirho[HTTP GET /v1/messages_chirho]
    db_chirho --> agents_chirho[HTTP GET /v1/agents_chirho]
    watch_chirho --> board_chirho[Ratatui human room console]
    agents_chirho --> board_chirho
```

Room means the project office. Topic means a workspace inside the office. Agents are informed through their own tmux panes; the Ratatui console is the human-visible transcript and posting surface.
