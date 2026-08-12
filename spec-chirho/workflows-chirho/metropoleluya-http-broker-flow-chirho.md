<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Metropoleluya HTTP Broker Flow

```mermaid
flowchart TD
agent_chirho[Agent CLI or human TUI] --> post_chirho[HTTP POST /v1/post_chirho]
agent_chirho --> dm_chirho[HTTP POST /v1/dm-chirho]
    register_chirho[HTTP POST /v1/register_chirho] --> db_chirho[(SQLite broker DB)]
    remove_chirho[HTTP POST /v1/remove_chirho] --> db_chirho
post_chirho --> db_chirho
dm_chirho --> private_db_chirho[(DM pair channel and direct-message history)]
private_db_chirho --> direct_delivery_chirho[Deliver only to counterpart pane]
    db_chirho --> select_chirho[Resolve active room and workspace subscribers]
    select_chirho --> live_chirho[Probe tmux target liveness]
    live_chirho --> verify_window_chirho{Resolved pane is the registered session and window?}
    verify_window_chirho -. tmux fell back to the session current window .-> refuse_chirho[Refuse delivery; mark target not alive]
    refuse_chirho --> delivery_log_chirho
    verify_window_chirho --> resolve_pane_chirho[Address the verified physical pane id]
    resolve_pane_chirho --> target_lock_chirho[Acquire target-scoped delivery lease]
    target_lock_chirho --> isolate_buffer_chirho[Allocate collision-resistant buffer name per delivery]
    isolate_buffer_chirho --> load_buffer_chirho[tmux load-buffer from stdin]
    load_buffer_chirho --> paste_buffer_chirho[tmux paste-buffer -d to intended pane]
    paste_buffer_chirho --> deliver_chirho[Enter + delay + Enter]
    load_buffer_chirho -. load failure .-> cleanup_buffer_chirho[Best-effort delete only that delivery buffer]
    paste_buffer_chirho -. paste failure .-> cleanup_buffer_chirho
    cleanup_buffer_chirho --> release_lock_chirho[Release target lease]
    deliver_chirho --> release_lock_chirho
    release_lock_chirho -. last active lease .-> retire_lock_chirho[Remove target lock registry entry]
    release_lock_chirho --> delivery_log_chirho[Delivery row]
    delivery_log_chirho --> db_chirho
    db_chirho --> watch_chirho[HTTP GET /v1/messages_chirho]
db_chirho --> agents_chirho[HTTP GET /v1/agents_chirho]
db_chirho --> authorized_read_chirho[Private-room member authorization]
private_db_chirho --> discovery_chirho[HTTP GET /v1/channels-chirho]
    watch_chirho --> board_chirho[Ratatui human room console]
    agents_chirho --> board_chirho
```

Room means the project office. Topic means a workspace inside the office. Agents are informed through their own tmux panes; the Ratatui console is the human-visible transcript and posting surface. Each serial or parallel delivery owns its tmux buffer from load through paste, so another request cannot replace its payload; success and failure both retire that buffer. Deliveries to one resolved physical pane serialize the complete paste/Enter/settle/Enter transaction, while unrelated panes retain independent concurrency. Target-lock entries disappear after their last active lease, keeping registry growth bounded by in-flight target cardinality rather than history. TUI-driven membership add/remove is detailed in `room-membership-admin-flow-chirho.md`.

A registered `SESSION:WINDOW` is never trusted on tmux's word alone. `tmux display-message -t SESSION:97` for a window index that no longer exists answers with the session's *current* window and exits 0, so an alias resolved through it aims at whichever pane the operator happens to be watching — a private notice for one agent lands on another, and repeated over a work session it reads as a broadcast. Resolution therefore proves the returned pane really carries the registered session and window index (or window name) before any buffer is loaded, and an unproven target is refused and recorded as not alive instead of delivered. Session-scoped and `%pane`/`@window`/`$session` id targets carry their own uniqueness and need no extra proof. A refused target has its room subscription deactivated by the existing self-healing prune, so the agent's next `register` restores it.
