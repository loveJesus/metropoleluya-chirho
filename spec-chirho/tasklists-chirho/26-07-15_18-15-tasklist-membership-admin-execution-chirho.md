<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Execution checklist — room-membership admin (claude2_chirho)

Executes spec `26-07-15_18-01-tasklist-room-membership-admin-chirho.md`. Owner: `METROLELUYA/claude2_chirho`.

## Pre-work (L.J. mid-flight instructions)

- [x] Commit handed-off dirty work by name (`bbd69db`): tui CPU dirty-flag fix + `--release` docs + spec.
- [x] `.gitignore`: add `.env` (was NOT ignored), exempt committed progress DB (`dab1933`).
- [x] Swap METROLELUYA window 1 to opencode for today (tokens exhausted on window-1 Claude).
- [x] Create `spec-chirho/log_step_chirho.ts` + progress DB (first logged step).

## Architecture call (announce in room at brick 1 — spec C3)

- [x] New module `src/tui_membership_chirho.rs`: modal state machine, hit-testing, add-form,
  popup rendering, broker wiring + tests. `tui_chirho.rs` keeps loop/fetch/core render and
  delegates. Reason: tui_chirho.rs is 635 lines and would cross ~1k.

## Part A — broker (`src/main.rs`)

- [x] A1 `RemoveRequestChirho` struct (serde + `validate_token_chirho` on all five fields).
- [x] A2 `remove_agent_chirho`: notify-first pane nudge, then this-room-only DELETE; returns
  `{ ok_chirho, identity_chirho, removed_count_chirho, notified_chirho }`.
- [x] A3 route `("POST", "/v1/remove_chirho")`.
- [x] A4 CLI `remove` subcommand → `run_remove_cli_chirho` (+ usage text).
- [x] A5 register gains optional `notify_actor_chirho`; best-effort add-nudge, `notified_chirho` in response.
- [x] A6 tests: this-room-only delete (other room + agent row survive); non-member remove is a no-op.

## Part B — TUI (`src/tui_chirho.rs` + new `src/tui_membership_chirho.rs`)

- [x] B1 mouse capture on entry, released in `TerminalRestoreChirho::drop`.
- [x] B2 `TuiModeChirho` (Normal / ContextMenu / ConfirmRemove / AddForm) + `AddFormStateChirho`
  with name normalization (`GPT` → `gpt_chirho`) and derived identity/target preview.
- [x] B3 hit-test geometry recorded at render (`agent_hit_rows_chirho`, `add_button_rect_chirho`).
- [x] B4 mouse events: right-click row → menu; left-click `[ + add ]` → form; click item → activate;
  click elsewhere → close.
- [x] B5 popup rendering: context menu / confirm / add form (Clear + bordered Block, centered).
- [x] B6 keyboard parity: Tab focus toggle, Up/Down, Enter menu, `x`/Delete remove, `a`/`+` add,
  Esc closes; help line updated.
- [x] B7 wire confirm→`/v1/remove_chirho`, add→`/v1/register_chirho`+`notify_actor_chirho`;
  status line + forced roster refresh; every branch sets `needs_render_chirho`.
- [x] B8 tests: form derivation + normalization; hit-test math; mode transitions.

## Part C — docs

- [x] C1 README + SKILL TUI-controls sections (incl. tmux `mouse on` note).
- [x] C2 mermaid DAG `spec-chirho/workflows-chirho/` + function comments pointing at it.
- [x] C3 split executed (announced first).

## Gates

- [x] `cargo build --release` clean.
- [x] `cargo clippy --release -- -D warnings` clean.
- [x] `cargo test --release` green (existing 10 + new).
- [x] Manual E2E on scratch broker/db/tmux session: remove nudges + unsubscribes this room only
  (SQLite-verified); add registers + nudges; keyboard path reaches both; terminal clean after
  `/quit`; idle CPU ~0% with popup open.
- [x] John 3:16 header on new files; Chirho suffixes; file sizes within limits.
- [x] Commit only files I changed, by name. Log step end. Room pings at gates + completion.

## Outcome notes (2026-07-15)

- Mid-task collision: window-1 opencode picked up my room status post as a task and half-implemented
  the same spec into the shared tree. Interrupted it, snapshotted all of its work to the session
  scratchpad (`*-opencode-collision-snapshot.rs`), reset the collided files to the committed
  baseline, sent it a stand-down (broker message #3886), re-implemented cleanly.
- `main.rs` crossed the 1.5k hard cap with Part A, so its inline test module moved to
  `src/main_tests_chirho.rs` (matches the tests-in-own-files preference).
- clippy `enum_variant_names` fires on Chirho-suffixed variants; the naming convention wins — two
  targeted, commented `#[allow]`s on `AddFieldChirho` / `TuiModeChirho`.
- Mouse E2E caveat: tmux `send-keys` (hex or literal) does not deliver the SGR mouse sequence
  atomically enough for crossterm, so synthetic right-click injection was not reproducible. Mouse
  handlers are unit-tested with real `MouseEvent` values; the capture handshake was verified live
  (`#{mouse_any_flag}` = 1 while running, 0 after `/quit`). Live right-click check pending L.J.'s
  hand with tmux `set -g mouse on`.
- Deploy note: the running broker on 37371 and the three live `lj_chirho` TUIs still run the old
  binary; they need a restart from this build to serve `/v1/remove_chirho` and the TUI admin.
