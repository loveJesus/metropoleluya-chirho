<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Tasklist — TUI room-membership admin (add / remove listeners)

**Assigned by:** L.J. (via `METROLELUYA/claude_chirho`) → **owner:** `METROLELUYA/claude2_chirho`
**Date:** 2026-07-15 18:01
**Repo:** `metropoleluya-chirho` (this checkout — shared working tree)

## Goal

Let the operator manage who is in a room directly from the Ratatui TUI:

- **Right-click a listener → popup → "Remove from room"** → unsubscribe them from **this room only**, and **notify their tmux pane** that they were removed.
- **`+` affordance (click) or `a` key → add form** → register someone into the current room by
  **session + window# + name** (e.g. typing `CAIRN_CHIRHO` / `3` / `GPT` ⇒ identity `CAIRN_CHIRHO/gpt_chirho`,
  tmux target `CAIRN_CHIRHO:3`), and **notify their pane** that they were added.

## Locked decisions (from L.J.)

1. **Remove scope = this room only.** Delete the subscription rows for `(identity, room)`; keep the
   `agents_chirho` row and any other room subscriptions. Reversible via the add flow.
2. **Input = mouse + keyboard both.** Right-click popup and clickable `[ + add ]`, **plus** full keyboard
   parity so native workflows survive.
3. **No silent removes.** Removal always goes through a visible confirm step in the popup — honors the
   project rule "no popups or silent defaults for operator decisions" (this is an operator-initiated
   action surface in the operator's own console, and every destructive step is confirmed).
4. **Notifications are direct pane nudges**, not room broadcasts: reuse `send_tmux_message_chirho` to
   paste a one-line operator-attributed notice into the target pane. They are **not** written to
   `messages_chirho` (no transcript spam). If L.J. later wants an audit line in the transcript, that is a
   follow-up, not this task.

## ⚠️ Shared-tree state — read before you touch anything

This tree has **uncommitted** work you must build **on top of**, not revert:

- `src/tui_chirho.rs` — the CPU dirty-flag fix (`run_tui_loop_chirho` only repaints when
  `needs_render_chirho` is set by a data change or an input event). **Preserve this pattern**: every new
  mouse/menu/form event must set `needs_render_chirho = true`, and an open popup must **not** cause
  continuous repaint (idle stays ~0% CPU).
- `README.md`, `SKILL.md` — `--release` doc edits.
- `.env` is untracked and **gitignored** — never stage or commit it.

Multi-agent etiquette: commit **only files you change, by name**; never `git stash` / `reset` / `checkout`
my uncommitted work. Ping `METROLELUYA/claude_chirho` if you need me to commit the CPU fix first for a
cleaner base.

---

## Part A — Broker (`src/main.rs`): minimal, boring surface

Keep the broker boring (localhost HTTP, SQLite, tmux delivery). One new endpoint + optional notify
fields on register. Reuse existing helpers — do not reinvent tmux delivery.

Reusable anchors already in `main.rs`:
- `identity_chirho(session, agent)` (~122), `validate_token_chirho(name, value)` (~126)
- `probe_tmux_target_chirho(target)` (~611) → gives window/pane/alive
- `send_tmux_message_chirho(target, text)` (~674) → the pane-paste delivery you reuse for notices
- `register_agent_chirho(conn, request)` (~228), route match `route_http_chirho` (~852),
  CLI dispatch (~1221: `"server"|"register"|"post"|"tui"`)
- `subscriptions_chirho(identity_chirho, room_chirho, topic_chirho, active_chirho)` schema

- [ ] **A1. `RemoveRequestChirho`** struct: `{ from_session_chirho, from_agent_chirho, session_chirho,
  agent_chirho, room_chirho }` (serde, all validated with `validate_token_chirho`).
- [ ] **A2. `remove_agent_chirho(conn, request) -> Result<Value, String>`**:
  1. Compute `identity_chirho`; look up the target's `tmux_target_chirho` from `agents_chirho`.
  2. **Notify first** (while still subscribed): `send_tmux_message_chirho(target, notice)` where the
     notice is operator-attributed, e.g.
     `"metropoleluya: you were removed from room <room> by <from_identity>."` Record `notified_chirho`
     (best-effort; a dead pane just means `notified_chirho=false`, still proceed).
  3. `DELETE FROM subscriptions_chirho WHERE identity_chirho=?1 AND room_chirho=?2` — **only this room**.
     Capture `removed_count_chirho` from `changes()`.
  4. Return `{ ok_chirho, identity_chirho, removed_count_chirho, notified_chirho }`.
- [ ] **A3. Route:** add `("POST", "/v1/remove_chirho")` arm in `route_http_chirho` (mirror
  `/v1/register_chirho`).
- [ ] **A4. CLI:** add `"remove" => run_remove_cli_chirho(&args_chirho)` and implement
  `run_remove_cli_chirho` (mirror `run_register_cli_chirho`: parse `--from-session/--from-agent/--session/
  --agent/--room`, POST `/v1/remove_chirho`). Useful for tests + manual ops.
- [ ] **A5. Add-notify:** extend register so the add path can nudge the newcomer **without** a second
  endpoint. Add optional `notify_actor_chirho: Option<String>` to the register request; when present,
  after the upsert+subscribe, `send_tmux_message_chirho(target, "metropoleluya: you were added to room
  <room> by <notify_actor>.")`. Keep it best-effort; include `notified_chirho` in the response.
- [ ] **A6. Tests** (`#[cfg(test)]` in `main.rs`, in-memory SQLite like `in_memory_db_registers_agent_and_room_chirho`):
  remove deletes only the targeted room's subscription and leaves other rooms + the agent row intact;
  remove of a non-member is a no-op (`removed_count_chirho == 0`).

## Part B — TUI (`src/tui_chirho.rs`): mouse + modal + keyboard parity

- [ ] **B1. Mouse capture lifecycle.** Add `EnableMouseCapture` to the `execute!` after
  `EnterAlternateScreenChirho` in `run_tui_cli_chirho`, and `DisableMouseCapture` in
  `TerminalRestoreChirho::drop` (import both from `crossterm::event`). Verify the terminal is clean after
  `/quit` (no stuck mouse mode). Note in README that tmux needs `mouse on` for right-click to reach the app.
- [ ] **B2. Modal state.** Add `mode_chirho: TuiModeChirho` to `TuiStateChirho`:
  - `NormalChirho`
  - `ContextMenuChirho { agent_index_chirho: usize, selection_chirho: usize }`
  - `ConfirmRemoveChirho { agent_index_chirho: usize }`
  - `AddFormChirho(AddFormStateChirho)` where `AddFormStateChirho { session_chirho, window_chirho,
    agent_chirho: String, focus_chirho: AddFieldChirho }`.
  Derive on submit: `tmux_target = format!("{session}:{window}")`,
  `identity = format!("{session}/{agent}")`. Normalize the agent name to the fleet convention
  (lowercase + ensure `_chirho`, e.g. `GPT` → `gpt_chirho`); show the derived identity/target in the form
  so the operator sees what will be registered before submitting.
- [ ] **B3. Hit-testing.** Ratatui has no built-in hit-testing — record the rendered geometry during
  `render_agents_chirho`: each agent's row-range and the `[ + add ]` rect. Store them on `TuiStateChirho`
  (e.g. `agent_hit_rows_chirho: Vec<(u16 /*y0*/, u16 /*y1*/, usize /*idx*/)>`,
  `add_button_rect_chirho: Option<Rect>`), refreshed every render. Mouse events test against the
  last-rendered geometry (a one-frame lag is fine). Keep the math in one small helper so render and the
  handler agree.
- [ ] **B4. Mouse events** in `handle_event_chirho` (`CrosstermEventChirho::Mouse`):
  - Right-button-down over an agent row → `ContextMenuChirho { agent_index, selection: 0 }`.
  - Left-button-down over `[ + add ]` → open `AddFormChirho` (empty, session defaults to current session).
  - Left-button-down on a menu/form item → activate it. Anywhere else with a popup open → close to Normal.
- [ ] **B5. Popup rendering.** Centered overlay via `Clear` + a small bordered `Block`:
  - Context menu: title = the agent identity; items = `Remove from room`, `Cancel`.
  - Confirm-remove: `Remove <identity> from <room>?  [Enter] yes  [Esc] no`.
  - Add form: three labeled fields (session / window# / name) + derived `identity → target` preview +
    `[Enter] add  [Esc] cancel`; highlight the focused field.
- [ ] **B6. Keyboard parity (because "mouse + keyboard both").** The compose box currently eats all
  `Char` keys, so gate list controls behind focus: **`Tab` toggles focus** between the compose box
  (default, types as today) and the listeners list. When the listeners list is focused: `Up/Down` move the
  highlighted agent, `Enter` opens its context menu, `x`/`Delete` → confirm-remove, `a`/`+` → add form.
  `Esc` closes any popup/mode back to `NormalChirho`. Show the focus + these keys in the help line.
- [ ] **B7. Wire actions to the broker.** On confirm-remove → POST `/v1/remove_chirho` (from = the TUI's
  own `session_chirho`/`agent_chirho`). On add submit → POST `/v1/register_chirho` with
  `notify_actor_chirho` = the TUI identity. After either, set `status_chirho` from the response
  (`"removed X (notified)"` / `"added X (notified)"`) and force a roster refresh
  (`last_refresh_chirho = Instant::now() - REFRESH_INTERVAL_CHIRHO`). Every branch sets
  `needs_render_chirho`.
- [ ] **B8. Tests:** add-form field → `(identity, tmux_target)` derivation incl. name normalization;
  hit-test row math (`agent_index_at`); mode transitions (right-click→menu→confirm→Normal).

## Part C — Docs & housekeeping

- [ ] **C1.** Update the **TUI commands** sections of `README.md` and `SKILL.md` with the new controls
  (right-click / `[ + add ]` / `Tab` focus / `Up`·`Down` / `x` remove / `a` add / `Esc`).
- [ ] **C2.** Add a small mermaid DAG for the add/remove membership flow under
  `spec-chirho/workflows-chirho/` and comment the participating functions to point at it (repo convention).
- [ ] **C3.** If `tui_chirho.rs` heads past ~1k lines (it's ~640 now; the modal machine + mouse handling
  will grow it), **split at brick 1** — e.g. move the membership/modal logic into
  `src/tui_membership_chirho.rs` — and say so in the room before doing it. New files start with the John
  3:16 header.

## Definition of done (gates — run them, report failures plainly)

- [ ] `cargo build --release` clean; `cargo clippy --release -- -D warnings` clean (fix, don't suppress).
- [ ] `cargo test --release` green (existing 10 + the new Part A/B tests).
- [ ] Manual: right-click remove notifies + unsubscribes (this room only, verified in SQLite); `+`/`a`
  add registers + notifies; keyboard path reaches both; terminal clean after `/quit`; idle CPU still ~0%
  (dirty-flag preserved — an open popup does not spin).
- [ ] John 3:16 header on any new file; **Chirho suffix** on every new identifier (correct casing);
  files/dirs within size limits.
- [ ] Docs updated (C1). Commit only the files you changed, by name — do not sweep my uncommitted work.
