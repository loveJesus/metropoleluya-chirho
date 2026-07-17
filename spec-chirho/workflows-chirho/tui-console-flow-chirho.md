<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# TUI console flow — compose editing + timestamp display

How a key press moves through the Ratatui console: the dirty-flag loop repaints
only on an event or a data refresh, keys route by mode then focus, and the
compose box is a full [`ratatui-textarea`] line editor. Timestamps everywhere in
the view are localized to America/New_York.

```mermaid
flowchart TD
    poll["event::poll (100ms)"] -->|event or refresh| handle["handle_event_chirho"]
    poll -->|idle: no repaint| poll

    handle --> key["handle_key_event_chirho"]
    key -->|Ctrl-C| quit["quit"]
    key -->|mode != Normal| modal["handle_modal_key_chirho\n(membership popups)"]
    key -->|Tab| focus["toggle_focus_chirho"]
    key -->|focus = Listeners| listeners["handle_listeners_key_chirho\n(select / remove / add)"]

    key -->|focus = Compose| compose{compose key?}
    compose -->|Enter| submit["submit_input_chirho"]
    compose -->|Alt+Enter| newline["input_area_chirho.input → newline"]
    compose -->|PageUp / PageDown| scroll["transcript scrollback"]
    compose -->|everything else| edit["input_area_chirho.input\n(Ctrl-A/E/K/U/W, arrows, Home/End, word ops)"]

    submit -->|/topic /clear /quit| local["local command"]
    submit -->|message body| post["post_tui_message_chirho → POST /v1/post_chirho"]
    submit --> reset["input_area_chirho = new_compose_area_chirho()"]

    handle --> render["render_tui_chirho"]
    render --> input["render_input_chirho\n(status · editor · help)"]
    render --> transcript["render_transcript_chirho"]
    transcript --> ts["format_timestamp_chirho\n(America/New_York, DST-correct via jiff)"]
    post -.->|broker echoes at_text_chirho| ts
```

## Participating functions (commented to point here)

- `new_compose_area_chirho` (`src/tui_chirho.rs`) — builds the empty editor
  (block cursor, placeholder); reused after every send so styling is consistent.
- `handle_key_event_chirho` (`src/tui_chirho.rs`) — mode → focus → key routing;
  the compose branch feeds all non-command keys to the editor.
- `submit_input_chirho` (`src/tui_chirho.rs`) — joins the editor's lines, runs
  `/`-commands or posts, then resets the editor.
- `format_timestamp_chirho` (`src/main.rs`) — the single display formatter; epoch
  storage stays UTC, display localizes to Eastern (EST/EDT by season).

Notes: `Home`/`End` move the cursor (line editing) — transcript paging is on
`PageUp`/`PageDown`. Message bodies may be multi-line (`Alt+Enter`), matching the
broker's multiline support. The SQL `*_with_time_chirho` views deliberately keep
UTC text so tests stay hermetic across machine time zones.

[`ratatui-textarea`]: https://crates.io/crates/ratatui-textarea
