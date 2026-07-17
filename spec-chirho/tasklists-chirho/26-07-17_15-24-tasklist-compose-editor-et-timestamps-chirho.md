<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Tasklist — compose-box readline editing + America/New_York timestamps

**Owner:** `METROLELUYA/claude_chirho` · **Requested by:** L.J. · **Date:** 2026-07-17 15:24 EDT
**Repo:** `metropoleluya-chirho` (shared working tree — claude2 owns recent main.rs / tui_chirho.rs commits)

## Goals (two independent asks)

1. **Compose box gets real line editing** — cursor, `CTRL-A/E/K/U/W`, arrows, `Home`/`End`, multi-line —
   via the **`ratatui-textarea`** crate (L.J.'s pick; canonical `tui-textarea` is pinned to ratatui 0.29
   and incompatible with our 0.30.2).
2. **Timestamps display in America/New_York** (ET, DST-correct) instead of `…Z`, via the **`jiff`** crate.

## Locked decisions (from L.J.)

- Editor = `ratatui-textarea` v0.9.2 (fork on ratatui 0.30 modular crates; pinned in committed `Cargo.lock`).
- Timezone = `jiff` v0.2.32, explicit IANA `America/New_York` (not a hard-coded offset — DST stays correct).
- Store `at_ms_chirho` as epoch (unchanged); localize only the **display** text.

## Deps — DONE (brick 1)

- [x] Verified `ratatui-textarea` 0.9.2 default `crossterm` feature matches our crossterm 0.29 / ratatui 0.30.2.
- [x] `cargo tree -d`: no duplicate `ratatui-core`/`crossterm` (only harmless foldhash/hashbrown). Builds clean.
- [x] Added `ratatui-textarea` + `jiff` to Cargo.toml; heads-up posted to claude2 (broker msg #4670).

## Part A — Timezone (jiff), `src/main.rs`

- [x] **A1.** Rewrite `format_timestamp_chirho(at_ms)` to convert epoch-ms → `America/New_York` via jiff and
      render `YYYY-MM-DD HH:MM:SS.mmm EDT/EST` (zone abbrev, no more `Z`). Keep `at_ms_chirho` as epoch.
- [x] **A2.** Reconcile the two SQLite views (`main.rs:~217`, `~234`) that build `at_text_chirho` via
      `datetime(...,'unixepoch') || '…Z'`. Trace consumers first; prefer routing display through the Rust
      formatter (single source) or, if the views must stay self-contained, localize + drop the `Z`.
- [x] **A3.** Update timezone tests in `src/main_tests_chirho.rs` (`timestamp_formatter_uses_utc_text_chirho`
      @24, `sqlite_messages_view…` @61): epoch 0 → `1969-12-31 19:00:00.000 EST` (winter −5), and add a
      **summer** case (July → `EDT −4`) to prove DST. Rename the test to reflect ET.

## Part B — Line editor (ratatui-textarea), `src/tui_chirho.rs`

- [x] **B1.** Replace `input_chirho: String` with `input_area_chirho: TextArea<'static>` on `TuiStateChirho`
      (field Chirho-named; `TextArea` is external). Placeholder text + block cursor styling.
- [x] **B2.** Route keys in the compose-focus branch of `handle_key_event_chirho`:
      - `Enter` → `submit_input_chirho` (send). `Alt+Enter` → insert newline (broker supports multiline).
      - `PageUp`/`PageDown` → keep transcript scrollback.
      - everything else → `input_area_chirho.input(key_chirho)` (gives CTRL-A/E/K/U/W + arrows + Home/End free).
      - **Trade-off to document:** `Home`/`End` now = line-start/end (editor), replacing the old
        oldest/live-tail scrollback jumps; paging stays on `PageUp`/`PageDown`.
- [x] **B3.** `submit_input_chirho`: read `input_area_chirho.lines().join("\n")`, then clear the textarea;
      keep `/quit` `/clear` `/topic` handling; post supports multi-line body.
- [x] **B4.** `render_input_chirho`: render the `TextArea` widget in the input region (status line above,
      help line below). Show cursor only when compose is focused; dim otherwise.
- [x] **B5.** Confirm the CPU dirty-flag still holds — every edit arrives as an event, and the loop sets
      `needs_render_chirho` on events; an idle open editor must not spin.
- [x] **B6.** Tests: submit extracts multi-line text + clears; command parsing intact; `.input()` mutates
      lines. (Don't re-test the crate's own keymap.)

## Part C — Docs & housekeeping

- [x] **C1.** README + SKILL TUI-controls: new editing keys (`Home`/`End`/arrows/`CTRL-A/E/K/U/W`,
      `Alt+Enter` newline, `Enter` send) + note timestamps now show in America/New_York (ET).
- [x] **C2.** Update/annotate the relevant `spec-chirho/workflows-chirho/` DAG (compose→submit path);
      comment participating functions.
- [x] **C3.** Size gate: `tui_chirho.rs` (766) — textarea *replaces* manual char handling, net small;
      `main.rs` (1358) — timezone change small. Stay < 1.5k.

## Gates (run, report plainly)

- [x] `cargo build --release` + `cargo clippy --release -- -D warnings` clean; `cargo test --release` green.
- [x] Chirho suffix on new identifiers; John 3:16 header on any new file.
- [x] Manual: editor keys work, `Enter` sends, `Alt+Enter` newlines, timestamps read ET, terminal clean
      after `/quit`, idle CPU ~0%.
- [x] Commit only files I changed, by name. Log end row in progress DB.
