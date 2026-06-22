<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Metropoleluya HTTP Broker Flow

```mermaid
flowchart TD
    agent_chirho[Agent or human console] --> post_chirho[HTTP POST /v1/post_chirho]
    register_chirho[HTTP POST /v1/register_chirho] --> db_chirho[(SQLite broker DB)]
    post_chirho --> db_chirho
    db_chirho --> select_chirho[Resolve active room/topic subscribers]
    select_chirho --> live_chirho[Probe tmux target liveness]
    live_chirho --> deliver_chirho[tmux paste-buffer + Enter + delay + Enter]
    deliver_chirho --> delivery_log_chirho[Delivery row]
    delivery_log_chirho --> db_chirho
    db_chirho --> watch_chirho[HTTP GET /v1/messages_chirho]
    watch_chirho --> board_chirho[Visible tmux room window]
```

