<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Room-Membership Admin Flow (TUI add / remove)

```mermaid
flowchart TD
    operator_chirho[Operator in Ratatui TUI] -->|left-click its menu glyph / right-click listener, or Tab + Enter / x| menu_chirho[Context menu popup]
    menu_chirho -->|Remove from room| confirm_chirho[Confirm popup - no silent removes]
    confirm_chirho -->|Enter or click yes| remove_http_chirho[HTTP POST /v1/remove_chirho]
    remove_http_chirho --> remove_fn_chirho[remove_agent_chirho]
    remove_fn_chirho --> notify_removed_chirho[send_tmux_message_chirho pane nudge, best effort]
    notify_removed_chirho --> delete_sub_chirho[(DELETE subscriptions_chirho rows for identity + THIS room only)]
    delete_sub_chirho --> remove_status_chirho[TUI status line + forced roster refresh]

    operator_chirho -->|click add button, or a with list focused| form_chirho[Add form: session + window# + name]
    form_chirho -->|derive_identity_target_chirho preview| register_http_chirho[HTTP POST /v1/register_chirho with notify_actor_chirho]
    register_http_chirho --> register_fn_chirho[register_agent_chirho upsert + subscribe]
    register_fn_chirho --> notify_added_chirho[send_tmux_message_chirho pane nudge, best effort]
    notify_added_chirho --> add_status_chirho[TUI status line + forced roster refresh]
```

Locked decisions (L.J., 2026-07-15): remove is scoped to the one room (the
`agents_chirho` row and other subscriptions survive, so the add flow reverses
it); mouse and keyboard both reach every step; every remove passes through a
visible confirm popup. Notifications are direct pane nudges via
`send_tmux_message_chirho`, not transcript messages.

Participating functions (each carries a comment pointing here):

- `src/main.rs`: `register_agent_chirho` (add + nudge), `remove_agent_chirho` (notify-then-unsubscribe)
- `src/tui_chirho.rs`: `handle_key_event_chirho` (routing), `render_agents_chirho` (hit-test geometry)
- `src/tui_membership_chirho.rs`: the modal machine — mouse/modal/listeners handlers, `submit_remove_chirho`, `submit_add_chirho`, popup renderers
