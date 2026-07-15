// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

//! Room-membership admin for the TUI: right-click / keyboard remove, and the
//! `[ + add ]` / `a` add form. Flow doc:
//! spec-chirho/workflows-chirho/room-membership-admin-flow-chirho.md

use crate::http_client_chirho;
use crate::tui_chirho::{TuiStateChirho, REFRESH_INTERVAL_CHIRHO};
use crossterm::event::{
    KeyCode as KeyCodeChirho, KeyEvent as KeyEventChirho, MouseButton as MouseButtonChirho,
    MouseEvent as MouseEventChirho, MouseEventKind as MouseEventKindChirho,
};
use ratatui::layout::{Position as PositionChirho, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use serde_json::{json, Value};
use std::time::Instant;

pub(crate) const ADD_BUTTON_LABEL_CHIRHO: &str = "[ + add ]";
const CONTEXT_MENU_ITEMS_CHIRHO: [&str; 2] = ["Remove from room", "Cancel"];

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TuiFocusChirho {
    ComposeChirho,
    ListenersChirho,
}

// The fleet naming convention requires the Chirho suffix on every identifier,
// variants included, so the shared postfix these lints flag is intentional.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AddFieldChirho {
    SessionChirho,
    WindowChirho,
    AgentChirho,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AddFormStateChirho {
    pub(crate) session_chirho: String,
    pub(crate) window_chirho: String,
    pub(crate) agent_chirho: String,
    pub(crate) focus_chirho: AddFieldChirho,
}

// Chirho-suffixed variants are the fleet naming convention (see above).
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TuiModeChirho {
    NormalChirho,
    ContextMenuChirho {
        agent_index_chirho: usize,
        selection_chirho: usize,
    },
    ConfirmRemoveChirho {
        agent_index_chirho: usize,
    },
    AddFormChirho(AddFormStateChirho),
}

/// Geometry recorded at render time so mouse events can be tested against
/// what is actually on screen (a one-frame lag is acceptable).
#[derive(Debug, Clone, Default)]
pub(crate) struct MembershipHitsChirho {
    pub(crate) listeners_rect_chirho: Option<Rect>,
    pub(crate) add_button_rect_chirho: Option<Rect>,
    pub(crate) agent_rows_chirho: Vec<AgentRowHitChirho>,
    pub(crate) popup_rect_chirho: Option<Rect>,
    pub(crate) popup_item_rows_chirho: Vec<PopupItemRowChirho>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AgentRowHitChirho {
    pub(crate) row_start_chirho: u16,
    pub(crate) row_end_chirho: u16,
    pub(crate) agent_index_chirho: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PopupItemRowChirho {
    pub(crate) row_chirho: u16,
    pub(crate) item_index_chirho: usize,
}

/// Normalizes an operator-typed agent name to the fleet convention:
/// lowercase with a `_chirho` suffix (`GPT` becomes `gpt_chirho`).
pub(crate) fn normalize_agent_name_chirho(raw_chirho: &str) -> String {
    let lower_chirho = raw_chirho.trim().to_lowercase();
    if lower_chirho.is_empty() || lower_chirho.ends_with("_chirho") {
        lower_chirho
    } else {
        format!("{lower_chirho}_chirho")
    }
}

/// Derives what will actually be registered from the add form:
/// `(identity, tmux target)`, e.g. `CAIRN_CHIRHO / 3 / GPT` becomes
/// `("CAIRN_CHIRHO/gpt_chirho", "CAIRN_CHIRHO:3")`.
pub(crate) fn derive_identity_target_chirho(form_chirho: &AddFormStateChirho) -> (String, String) {
    let session_chirho = form_chirho.session_chirho.trim();
    let window_chirho = form_chirho.window_chirho.trim();
    let agent_chirho = normalize_agent_name_chirho(&form_chirho.agent_chirho);
    (
        format!("{session_chirho}/{agent_chirho}"),
        format!("{session_chirho}:{window_chirho}"),
    )
}

/// Maps a mouse position to a listener index using the last-rendered rows.
pub(crate) fn agent_index_at_chirho(
    hits_chirho: &MembershipHitsChirho,
    column_chirho: u16,
    row_chirho: u16,
) -> Option<usize> {
    let listeners_rect_chirho = hits_chirho.listeners_rect_chirho?;
    if !listeners_rect_chirho.contains(PositionChirho {
        x: column_chirho,
        y: row_chirho,
    }) {
        return None;
    }
    hits_chirho
        .agent_rows_chirho
        .iter()
        .find(|hit_chirho| {
            row_chirho >= hit_chirho.row_start_chirho && row_chirho <= hit_chirho.row_end_chirho
        })
        .map(|hit_chirho| hit_chirho.agent_index_chirho)
}

fn rect_contains_chirho(rect_chirho: Option<Rect>, column_chirho: u16, row_chirho: u16) -> bool {
    rect_chirho.is_some_and(|rect_chirho| {
        rect_chirho.contains(PositionChirho {
            x: column_chirho,
            y: row_chirho,
        })
    })
}

fn popup_item_at_chirho(
    hits_chirho: &MembershipHitsChirho,
    column_chirho: u16,
    row_chirho: u16,
) -> Option<usize> {
    if !rect_contains_chirho(hits_chirho.popup_rect_chirho, column_chirho, row_chirho) {
        return None;
    }
    hits_chirho
        .popup_item_rows_chirho
        .iter()
        .find(|item_chirho| item_chirho.row_chirho == row_chirho)
        .map(|item_chirho| item_chirho.item_index_chirho)
}

pub(crate) fn toggle_focus_chirho(state_chirho: &mut TuiStateChirho) {
    state_chirho.focus_chirho = match state_chirho.focus_chirho {
        TuiFocusChirho::ComposeChirho => {
            state_chirho.status_chirho =
                "listeners focused: Up/Down select, Enter menu, x remove, a add, Tab back"
                    .to_string();
            TuiFocusChirho::ListenersChirho
        }
        TuiFocusChirho::ListenersChirho => {
            state_chirho.status_chirho = "compose focused: type to speak".to_string();
            TuiFocusChirho::ComposeChirho
        }
    };
}

pub(crate) fn open_add_form_chirho(state_chirho: &mut TuiStateChirho) {
    state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(AddFormStateChirho {
        session_chirho: state_chirho.session_chirho.clone(),
        window_chirho: String::new(),
        agent_chirho: String::new(),
        focus_chirho: AddFieldChirho::SessionChirho,
    });
}

fn close_popup_chirho(state_chirho: &mut TuiStateChirho) {
    state_chirho.mode_chirho = TuiModeChirho::NormalChirho;
    state_chirho.hits_chirho.popup_rect_chirho = None;
    state_chirho.hits_chirho.popup_item_rows_chirho.clear();
}

/// Keys for the listeners list while no popup is open (list focused via Tab).
pub(crate) fn handle_listeners_key_chirho(
    state_chirho: &mut TuiStateChirho,
    key_chirho: KeyEventChirho,
) {
    let agent_count_chirho = state_chirho.agents_chirho.len();
    match key_chirho.code {
        KeyCodeChirho::Up => {
            state_chirho.selected_agent_chirho =
                state_chirho.selected_agent_chirho.saturating_sub(1);
        }
        KeyCodeChirho::Down => {
            if agent_count_chirho > 0
                && state_chirho.selected_agent_chirho + 1 < agent_count_chirho
            {
                state_chirho.selected_agent_chirho += 1;
            }
        }
        KeyCodeChirho::Enter => {
            if agent_count_chirho > 0 {
                state_chirho.mode_chirho = TuiModeChirho::ContextMenuChirho {
                    agent_index_chirho: state_chirho
                        .selected_agent_chirho
                        .min(agent_count_chirho - 1),
                    selection_chirho: 0,
                };
            }
        }
        KeyCodeChirho::Char('x') | KeyCodeChirho::Delete => {
            if agent_count_chirho > 0 {
                state_chirho.mode_chirho = TuiModeChirho::ConfirmRemoveChirho {
                    agent_index_chirho: state_chirho
                        .selected_agent_chirho
                        .min(agent_count_chirho - 1),
                };
            }
        }
        KeyCodeChirho::Char('a') | KeyCodeChirho::Char('+') => open_add_form_chirho(state_chirho),
        KeyCodeChirho::Esc => {
            state_chirho.focus_chirho = TuiFocusChirho::ComposeChirho;
            state_chirho.status_chirho = "compose focused: type to speak".to_string();
        }
        _ => {}
    }
}

/// Keys while a popup is open. Esc always returns to Normal; a remove only
/// ever happens from the confirm step (no silent removes).
pub(crate) fn handle_modal_key_chirho(
    state_chirho: &mut TuiStateChirho,
    key_chirho: KeyEventChirho,
) {
    match state_chirho.mode_chirho.clone() {
        TuiModeChirho::NormalChirho => {}
        TuiModeChirho::ContextMenuChirho {
            agent_index_chirho,
            selection_chirho,
        } => match key_chirho.code {
            KeyCodeChirho::Up | KeyCodeChirho::Down => {
                state_chirho.mode_chirho = TuiModeChirho::ContextMenuChirho {
                    agent_index_chirho,
                    selection_chirho: selection_chirho ^ 1,
                };
            }
            KeyCodeChirho::Enter => {
                if selection_chirho == 0 {
                    state_chirho.mode_chirho = TuiModeChirho::ConfirmRemoveChirho {
                        agent_index_chirho,
                    };
                } else {
                    close_popup_chirho(state_chirho);
                }
            }
            KeyCodeChirho::Esc => close_popup_chirho(state_chirho),
            _ => {}
        },
        TuiModeChirho::ConfirmRemoveChirho { agent_index_chirho } => match key_chirho.code {
            KeyCodeChirho::Enter => submit_remove_chirho(state_chirho, agent_index_chirho),
            KeyCodeChirho::Esc => close_popup_chirho(state_chirho),
            _ => {}
        },
        TuiModeChirho::AddFormChirho(mut form_chirho) => match key_chirho.code {
            KeyCodeChirho::Esc => close_popup_chirho(state_chirho),
            KeyCodeChirho::Enter => submit_add_chirho(state_chirho, &form_chirho),
            KeyCodeChirho::Tab | KeyCodeChirho::Down => {
                form_chirho.focus_chirho = next_add_field_chirho(&form_chirho.focus_chirho);
                state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
            }
            KeyCodeChirho::BackTab | KeyCodeChirho::Up => {
                form_chirho.focus_chirho = previous_add_field_chirho(&form_chirho.focus_chirho);
                state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
            }
            KeyCodeChirho::Backspace => {
                add_form_field_mut_chirho(&mut form_chirho).pop();
                state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
            }
            KeyCodeChirho::Char(value_chirho) => {
                if form_chirho.focus_chirho != AddFieldChirho::WindowChirho
                    || value_chirho.is_ascii_digit()
                {
                    add_form_field_mut_chirho(&mut form_chirho).push(value_chirho);
                }
                state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
            }
            _ => {}
        },
    }
}

/// Mouse handling: right-click a listener opens its menu, left-click
/// `[ + add ]` opens the add form, clicks on popup items activate them, and
/// any other click while a popup is open closes it.
pub(crate) fn handle_mouse_event_chirho(
    state_chirho: &mut TuiStateChirho,
    mouse_chirho: MouseEventChirho,
) {
    let MouseEventKindChirho::Down(button_chirho) = mouse_chirho.kind else {
        return;
    };
    let column_chirho = mouse_chirho.column;
    let row_chirho = mouse_chirho.row;
    match state_chirho.mode_chirho.clone() {
        TuiModeChirho::NormalChirho => match button_chirho {
            MouseButtonChirho::Right => {
                if let Some(agent_index_chirho) =
                    agent_index_at_chirho(&state_chirho.hits_chirho, column_chirho, row_chirho)
                {
                    state_chirho.selected_agent_chirho = agent_index_chirho;
                    state_chirho.mode_chirho = TuiModeChirho::ContextMenuChirho {
                        agent_index_chirho,
                        selection_chirho: 0,
                    };
                }
            }
            MouseButtonChirho::Left => {
                if rect_contains_chirho(
                    state_chirho.hits_chirho.add_button_rect_chirho,
                    column_chirho,
                    row_chirho,
                ) {
                    open_add_form_chirho(state_chirho);
                } else if let Some(agent_index_chirho) =
                    agent_index_at_chirho(&state_chirho.hits_chirho, column_chirho, row_chirho)
                {
                    state_chirho.selected_agent_chirho = agent_index_chirho;
                    state_chirho.focus_chirho = TuiFocusChirho::ListenersChirho;
                }
            }
            MouseButtonChirho::Middle => {}
        },
        TuiModeChirho::ContextMenuChirho {
            agent_index_chirho, ..
        } => match popup_item_at_chirho(&state_chirho.hits_chirho, column_chirho, row_chirho) {
            Some(0) => {
                state_chirho.mode_chirho = TuiModeChirho::ConfirmRemoveChirho {
                    agent_index_chirho,
                };
            }
            _ => close_popup_chirho(state_chirho),
        },
        TuiModeChirho::ConfirmRemoveChirho { agent_index_chirho } => {
            match popup_item_at_chirho(&state_chirho.hits_chirho, column_chirho, row_chirho) {
                Some(0) => submit_remove_chirho(state_chirho, agent_index_chirho),
                _ => close_popup_chirho(state_chirho),
            }
        }
        TuiModeChirho::AddFormChirho(mut form_chirho) => {
            match popup_item_at_chirho(&state_chirho.hits_chirho, column_chirho, row_chirho) {
                Some(0) => {
                    form_chirho.focus_chirho = AddFieldChirho::SessionChirho;
                    state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
                }
                Some(1) => {
                    form_chirho.focus_chirho = AddFieldChirho::WindowChirho;
                    state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
                }
                Some(2) => {
                    form_chirho.focus_chirho = AddFieldChirho::AgentChirho;
                    state_chirho.mode_chirho = TuiModeChirho::AddFormChirho(form_chirho);
                }
                Some(_) => submit_add_chirho(state_chirho, &form_chirho),
                None => close_popup_chirho(state_chirho),
            }
        }
    }
}

fn next_add_field_chirho(field_chirho: &AddFieldChirho) -> AddFieldChirho {
    match field_chirho {
        AddFieldChirho::SessionChirho => AddFieldChirho::WindowChirho,
        AddFieldChirho::WindowChirho => AddFieldChirho::AgentChirho,
        AddFieldChirho::AgentChirho => AddFieldChirho::SessionChirho,
    }
}

fn previous_add_field_chirho(field_chirho: &AddFieldChirho) -> AddFieldChirho {
    match field_chirho {
        AddFieldChirho::SessionChirho => AddFieldChirho::AgentChirho,
        AddFieldChirho::WindowChirho => AddFieldChirho::SessionChirho,
        AddFieldChirho::AgentChirho => AddFieldChirho::WindowChirho,
    }
}

fn add_form_field_mut_chirho(form_chirho: &mut AddFormStateChirho) -> &mut String {
    match form_chirho.focus_chirho {
        AddFieldChirho::SessionChirho => &mut form_chirho.session_chirho,
        AddFieldChirho::WindowChirho => &mut form_chirho.window_chirho,
        AddFieldChirho::AgentChirho => &mut form_chirho.agent_chirho,
    }
}

fn tui_identity_chirho(state_chirho: &TuiStateChirho) -> String {
    format!(
        "{}/{}",
        state_chirho.session_chirho, state_chirho.agent_chirho
    )
}

fn force_roster_refresh_chirho(state_chirho: &mut TuiStateChirho) {
    state_chirho.last_refresh_chirho = Instant::now() - REFRESH_INTERVAL_CHIRHO;
}

/// Confirmed remove: POST /v1/remove_chirho (the broker notifies the pane
/// first, then unsubscribes this room only). Broker errors land in the status
/// line instead of tearing the console down.
pub(crate) fn submit_remove_chirho(state_chirho: &mut TuiStateChirho, agent_index_chirho: usize) {
    close_popup_chirho(state_chirho);
    let identity_chirho = match state_chirho.agents_chirho.get(agent_index_chirho) {
        Some(agent_chirho) => agent_chirho.identity_chirho.clone(),
        None => {
            state_chirho.status_chirho = "listener list changed; remove cancelled".to_string();
            return;
        }
    };
    let Some((session_chirho, agent_name_chirho)) = identity_chirho.split_once('/') else {
        state_chirho.status_chirho = format!("cannot parse identity {identity_chirho}");
        return;
    };
    let body_chirho = json!({
        "from_session_chirho": state_chirho.session_chirho,
        "from_agent_chirho": state_chirho.agent_chirho,
        "session_chirho": session_chirho,
        "agent_chirho": agent_name_chirho,
        "room_chirho": state_chirho.room_chirho,
    });
    match http_client_chirho(
        &state_chirho.server_chirho,
        "POST",
        "/v1/remove_chirho",
        Some(&body_chirho),
    ) {
        Ok(response_chirho) => {
            let removed_chirho = response_chirho
                .get("removed_count_chirho")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let notified_chirho = response_chirho
                .get("notified_chirho")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            state_chirho.status_chirho = if removed_chirho > 0 {
                format!(
                    "removed {identity_chirho} from {} ({})",
                    state_chirho.room_chirho,
                    if notified_chirho {
                        "pane notified"
                    } else {
                        "pane not notified"
                    }
                )
            } else {
                format!(
                    "{identity_chirho} was not a member of {}",
                    state_chirho.room_chirho
                )
            };
            force_roster_refresh_chirho(state_chirho);
        }
        Err(err_chirho) => {
            state_chirho.status_chirho = format!("remove failed: {err_chirho}");
        }
    }
}

/// Add submit: POST /v1/register_chirho with notify_actor_chirho so the
/// broker nudges the newcomer's pane. Incomplete forms stay open.
pub(crate) fn submit_add_chirho(
    state_chirho: &mut TuiStateChirho,
    form_chirho: &AddFormStateChirho,
) {
    let session_chirho = form_chirho.session_chirho.trim().to_string();
    let window_chirho = form_chirho.window_chirho.trim().to_string();
    let agent_name_chirho = normalize_agent_name_chirho(&form_chirho.agent_chirho);
    if session_chirho.is_empty() || window_chirho.is_empty() || agent_name_chirho.is_empty() {
        state_chirho.status_chirho = "add needs session, window#, and name".to_string();
        return;
    }
    let (identity_chirho, tmux_target_chirho) = derive_identity_target_chirho(form_chirho);
    close_popup_chirho(state_chirho);
    let body_chirho = json!({
        "session_chirho": session_chirho,
        "agent_chirho": agent_name_chirho,
        "tmux_target_chirho": tmux_target_chirho,
        "rooms_chirho": [state_chirho.room_chirho],
        "topics_chirho": [],
        "notify_actor_chirho": tui_identity_chirho(state_chirho),
    });
    match http_client_chirho(
        &state_chirho.server_chirho,
        "POST",
        "/v1/register_chirho",
        Some(&body_chirho),
    ) {
        Ok(response_chirho) => {
            let alive_chirho = response_chirho
                .get("alive_chirho")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let notified_chirho = response_chirho
                .get("notified_chirho")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            state_chirho.status_chirho = format!(
                "added {identity_chirho} @ {tmux_target_chirho} ({}, {})",
                if alive_chirho {
                    "pane alive"
                } else {
                    "pane not found"
                },
                if notified_chirho {
                    "notified"
                } else {
                    "not notified"
                }
            );
            force_roster_refresh_chirho(state_chirho);
        }
        Err(err_chirho) => {
            state_chirho.status_chirho = format!("add failed: {err_chirho}");
        }
    }
}

/// Draws whichever membership popup is open (and records its hit geometry).
/// Called last from render_tui_chirho so the overlay sits on top.
pub(crate) fn render_membership_overlay_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &mut TuiStateChirho,
) {
    state_chirho.hits_chirho.popup_rect_chirho = None;
    state_chirho.hits_chirho.popup_item_rows_chirho.clear();
    match state_chirho.mode_chirho.clone() {
        TuiModeChirho::NormalChirho => {}
        TuiModeChirho::ContextMenuChirho {
            agent_index_chirho,
            selection_chirho,
        } => render_context_menu_chirho(
            frame_chirho,
            area_chirho,
            state_chirho,
            agent_index_chirho,
            selection_chirho,
        ),
        TuiModeChirho::ConfirmRemoveChirho { agent_index_chirho } => {
            render_confirm_remove_chirho(frame_chirho, area_chirho, state_chirho, agent_index_chirho)
        }
        TuiModeChirho::AddFormChirho(form_chirho) => {
            render_add_form_chirho(frame_chirho, area_chirho, state_chirho, &form_chirho)
        }
    }
}

fn centered_rect_chirho(area_chirho: Rect, width_chirho: u16, height_chirho: u16) -> Rect {
    let width_chirho = width_chirho.min(area_chirho.width);
    let height_chirho = height_chirho.min(area_chirho.height);
    Rect {
        x: area_chirho.x + (area_chirho.width - width_chirho) / 2,
        y: area_chirho.y + (area_chirho.height - height_chirho) / 2,
        width: width_chirho,
        height: height_chirho,
    }
}

fn listener_identity_chirho(state_chirho: &TuiStateChirho, agent_index_chirho: usize) -> String {
    state_chirho
        .agents_chirho
        .get(agent_index_chirho)
        .map(|agent_chirho| agent_chirho.identity_chirho.clone())
        .unwrap_or_else(|| "unknown listener".to_string())
}

fn render_context_menu_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &mut TuiStateChirho,
    agent_index_chirho: usize,
    selection_chirho: usize,
) {
    let identity_chirho = listener_identity_chirho(state_chirho, agent_index_chirho);
    let width_chirho = (identity_chirho.len() as u16 + 4).max(26);
    let popup_chirho = centered_rect_chirho(area_chirho, width_chirho, 4);
    frame_chirho.render_widget(Clear, popup_chirho);
    let mut lines_chirho = Vec::new();
    for (item_index_chirho, label_chirho) in CONTEXT_MENU_ITEMS_CHIRHO.iter().enumerate() {
        let style_chirho = if item_index_chirho == selection_chirho {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines_chirho.push(Line::from(Span::styled(*label_chirho, style_chirho)));
        state_chirho
            .hits_chirho
            .popup_item_rows_chirho
            .push(PopupItemRowChirho {
                row_chirho: popup_chirho.y + 1 + item_index_chirho as u16,
                item_index_chirho,
            });
    }
    state_chirho.hits_chirho.popup_rect_chirho = Some(popup_chirho);
    let menu_chirho = Paragraph::new(lines_chirho).block(
        Block::default()
            .borders(Borders::ALL)
            .title(identity_chirho),
    );
    frame_chirho.render_widget(menu_chirho, popup_chirho);
}

fn render_confirm_remove_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &mut TuiStateChirho,
    agent_index_chirho: usize,
) {
    let identity_chirho = listener_identity_chirho(state_chirho, agent_index_chirho);
    let question_chirho = format!(
        "Remove {identity_chirho} from {}?",
        state_chirho.room_chirho
    );
    let yes_chirho = "[Enter] yes - remove from this room only";
    let no_chirho = "[Esc] no - keep listening";
    let width_chirho = (question_chirho.len().max(yes_chirho.len()) as u16 + 4).max(30);
    let popup_chirho = centered_rect_chirho(area_chirho, width_chirho, 5);
    frame_chirho.render_widget(Clear, popup_chirho);
    let lines_chirho = vec![
        Line::from(question_chirho.clone()),
        Line::from(Span::styled(
            yes_chirho,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(no_chirho),
    ];
    state_chirho
        .hits_chirho
        .popup_item_rows_chirho
        .push(PopupItemRowChirho {
            row_chirho: popup_chirho.y + 2,
            item_index_chirho: 0,
        });
    state_chirho
        .hits_chirho
        .popup_item_rows_chirho
        .push(PopupItemRowChirho {
            row_chirho: popup_chirho.y + 3,
            item_index_chirho: 1,
        });
    state_chirho.hits_chirho.popup_rect_chirho = Some(popup_chirho);
    let confirm_chirho = Paragraph::new(lines_chirho).block(
        Block::default()
            .borders(Borders::ALL)
            .title("confirm-remove-chirho"),
    );
    frame_chirho.render_widget(confirm_chirho, popup_chirho);
}

fn render_add_form_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &mut TuiStateChirho,
    form_chirho: &AddFormStateChirho,
) {
    let (identity_chirho, tmux_target_chirho) = derive_identity_target_chirho(form_chirho);
    let preview_chirho = format!("-> {identity_chirho} @ {tmux_target_chirho}");
    let action_chirho = "[Enter] add   [Esc] cancel";
    let field_rows_chirho = [
        ("session:", &form_chirho.session_chirho, AddFieldChirho::SessionChirho),
        ("window#:", &form_chirho.window_chirho, AddFieldChirho::WindowChirho),
        ("name:   ", &form_chirho.agent_chirho, AddFieldChirho::AgentChirho),
    ];
    let width_chirho = (preview_chirho.len() as u16 + 4).max(44);
    let popup_chirho = centered_rect_chirho(area_chirho, width_chirho, 7);
    frame_chirho.render_widget(Clear, popup_chirho);
    let mut lines_chirho = Vec::new();
    for (item_index_chirho, (label_chirho, value_chirho, field_chirho)) in
        field_rows_chirho.iter().enumerate()
    {
        let style_chirho = if form_chirho.focus_chirho == *field_chirho {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines_chirho.push(Line::from(vec![
            Span::raw(format!("{label_chirho} ")),
            Span::styled(format!("{value_chirho} "), style_chirho),
        ]));
        state_chirho
            .hits_chirho
            .popup_item_rows_chirho
            .push(PopupItemRowChirho {
                row_chirho: popup_chirho.y + 1 + item_index_chirho as u16,
                item_index_chirho,
            });
    }
    lines_chirho.push(Line::from(Span::styled(
        preview_chirho,
        Style::default().fg(Color::Green),
    )));
    lines_chirho.push(Line::from(Span::styled(
        action_chirho,
        Style::default().add_modifier(Modifier::BOLD),
    )));
    state_chirho
        .hits_chirho
        .popup_item_rows_chirho
        .push(PopupItemRowChirho {
            row_chirho: popup_chirho.y + 5,
            item_index_chirho: 3,
        });
    state_chirho.hits_chirho.popup_rect_chirho = Some(popup_chirho);
    let form_widget_chirho = Paragraph::new(lines_chirho).block(
        Block::default()
            .borders(Borders::ALL)
            .title("add-listener-chirho"),
    );
    frame_chirho.render_widget(form_widget_chirho, popup_chirho);
}

#[cfg(test)]
mod tests_chirho {
    use super::*;
    use crate::tui_chirho::TuiAgentChirho;
    use crossterm::event::{KeyModifiers as KeyModifiersChirho, MouseEventKind};

    fn test_state_chirho() -> TuiStateChirho {
        let mut state_chirho = TuiStateChirho::new_chirho(
            "http://127.0.0.1:1".to_string(),
            "METROLELUYA".to_string(),
            "operator_chirho".to_string(),
            "metroleluya-chirho".to_string(),
            "general-chirho".to_string(),
            0,
        );
        state_chirho.agents_chirho = vec![
            TuiAgentChirho {
                identity_chirho: "CAIRN_CHIRHO/gpt_chirho".to_string(),
                alive_chirho: true,
                window_index_chirho: Some("3".to_string()),
                pane_id_chirho: Some("%9".to_string()),
                topics_chirho: vec!["*".to_string()],
            },
            TuiAgentChirho {
                identity_chirho: "METROLELUYA/claude2_chirho".to_string(),
                alive_chirho: true,
                window_index_chirho: Some("5".to_string()),
                pane_id_chirho: Some("%12".to_string()),
                topics_chirho: vec!["*".to_string()],
            },
        ];
        state_chirho
    }

    fn key_chirho(code_chirho: KeyCodeChirho) -> KeyEventChirho {
        KeyEventChirho::new(code_chirho, KeyModifiersChirho::empty())
    }

    fn mouse_down_chirho(
        button_chirho: MouseButtonChirho,
        column_chirho: u16,
        row_chirho: u16,
    ) -> MouseEventChirho {
        MouseEventChirho {
            kind: MouseEventKind::Down(button_chirho),
            column: column_chirho,
            row: row_chirho,
            modifiers: KeyModifiersChirho::empty(),
        }
    }

    fn seeded_hits_chirho() -> MembershipHitsChirho {
        MembershipHitsChirho {
            listeners_rect_chirho: Some(Rect::new(80, 3, 30, 20)),
            add_button_rect_chirho: Some(Rect::new(81, 5, 9, 1)),
            agent_rows_chirho: vec![
                AgentRowHitChirho {
                    row_start_chirho: 6,
                    row_end_chirho: 8,
                    agent_index_chirho: 0,
                },
                AgentRowHitChirho {
                    row_start_chirho: 10,
                    row_end_chirho: 12,
                    agent_index_chirho: 1,
                },
            ],
            popup_rect_chirho: None,
            popup_item_rows_chirho: Vec::new(),
        }
    }

    #[test]
    fn normalize_agent_name_appends_suffix_and_lowercases_chirho() {
        assert_eq!(normalize_agent_name_chirho("GPT"), "gpt_chirho");
        assert_eq!(normalize_agent_name_chirho("Claude2"), "claude2_chirho");
        assert_eq!(normalize_agent_name_chirho("gpt_chirho"), "gpt_chirho");
        assert_eq!(normalize_agent_name_chirho("  Gemini  "), "gemini_chirho");
        assert_eq!(normalize_agent_name_chirho("   "), "");
    }

    #[test]
    fn derive_identity_target_matches_fleet_convention_chirho() {
        let form_chirho = AddFormStateChirho {
            session_chirho: "CAIRN_CHIRHO".to_string(),
            window_chirho: "3".to_string(),
            agent_chirho: "GPT".to_string(),
            focus_chirho: AddFieldChirho::SessionChirho,
        };
        let (identity_chirho, target_chirho) = derive_identity_target_chirho(&form_chirho);
        assert_eq!(identity_chirho, "CAIRN_CHIRHO/gpt_chirho");
        assert_eq!(target_chirho, "CAIRN_CHIRHO:3");
    }

    #[test]
    fn agent_index_at_uses_rendered_rows_chirho() {
        let hits_chirho = seeded_hits_chirho();
        assert_eq!(agent_index_at_chirho(&hits_chirho, 85, 7), Some(0));
        assert_eq!(agent_index_at_chirho(&hits_chirho, 85, 12), Some(1));
        assert_eq!(agent_index_at_chirho(&hits_chirho, 85, 9), None);
        assert_eq!(agent_index_at_chirho(&hits_chirho, 5, 7), None);
    }

    #[test]
    fn right_click_then_menu_then_confirm_then_escape_chirho() {
        let mut state_chirho = test_state_chirho();
        state_chirho.hits_chirho = seeded_hits_chirho();
        handle_mouse_event_chirho(&mut state_chirho, mouse_down_chirho(MouseButtonChirho::Right, 85, 11));
        assert_eq!(
            state_chirho.mode_chirho,
            TuiModeChirho::ContextMenuChirho {
                agent_index_chirho: 1,
                selection_chirho: 0
            }
        );
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Enter));
        assert_eq!(
            state_chirho.mode_chirho,
            TuiModeChirho::ConfirmRemoveChirho {
                agent_index_chirho: 1
            }
        );
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Esc));
        assert_eq!(state_chirho.mode_chirho, TuiModeChirho::NormalChirho);
    }

    #[test]
    fn menu_cancel_item_closes_popup_chirho() {
        let mut state_chirho = test_state_chirho();
        state_chirho.mode_chirho = TuiModeChirho::ContextMenuChirho {
            agent_index_chirho: 0,
            selection_chirho: 0,
        };
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Down));
        assert_eq!(
            state_chirho.mode_chirho,
            TuiModeChirho::ContextMenuChirho {
                agent_index_chirho: 0,
                selection_chirho: 1
            }
        );
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Enter));
        assert_eq!(state_chirho.mode_chirho, TuiModeChirho::NormalChirho);
    }

    #[test]
    fn keyboard_path_reaches_remove_and_add_chirho() {
        let mut state_chirho = test_state_chirho();
        toggle_focus_chirho(&mut state_chirho);
        assert_eq!(state_chirho.focus_chirho, TuiFocusChirho::ListenersChirho);
        handle_listeners_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Down));
        assert_eq!(state_chirho.selected_agent_chirho, 1);
        handle_listeners_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Char('x')));
        assert_eq!(
            state_chirho.mode_chirho,
            TuiModeChirho::ConfirmRemoveChirho {
                agent_index_chirho: 1
            }
        );
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Esc));
        handle_listeners_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Char('a')));
        match &state_chirho.mode_chirho {
            TuiModeChirho::AddFormChirho(form_chirho) => {
                assert_eq!(form_chirho.session_chirho, "METROLELUYA");
                assert!(form_chirho.window_chirho.is_empty());
            }
            other_chirho => panic!("expected add form, got {other_chirho:?}"),
        }
    }

    #[test]
    fn left_click_add_button_opens_prefilled_form_chirho() {
        let mut state_chirho = test_state_chirho();
        state_chirho.hits_chirho = seeded_hits_chirho();
        handle_mouse_event_chirho(&mut state_chirho, mouse_down_chirho(MouseButtonChirho::Left, 84, 5));
        match &state_chirho.mode_chirho {
            TuiModeChirho::AddFormChirho(form_chirho) => {
                assert_eq!(form_chirho.session_chirho, "METROLELUYA");
            }
            other_chirho => panic!("expected add form, got {other_chirho:?}"),
        }
    }

    #[test]
    fn add_form_window_field_accepts_digits_only_chirho() {
        let mut state_chirho = test_state_chirho();
        open_add_form_chirho(&mut state_chirho);
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Tab));
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Char('x')));
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Char('3')));
        match &state_chirho.mode_chirho {
            TuiModeChirho::AddFormChirho(form_chirho) => {
                assert_eq!(form_chirho.window_chirho, "3");
                assert_eq!(form_chirho.focus_chirho, AddFieldChirho::WindowChirho);
            }
            other_chirho => panic!("expected add form, got {other_chirho:?}"),
        }
    }

    #[test]
    fn incomplete_add_submit_keeps_form_open_chirho() {
        let mut state_chirho = test_state_chirho();
        open_add_form_chirho(&mut state_chirho);
        handle_modal_key_chirho(&mut state_chirho, key_chirho(KeyCodeChirho::Enter));
        assert!(matches!(
            state_chirho.mode_chirho,
            TuiModeChirho::AddFormChirho(_)
        ));
        assert_eq!(
            state_chirho.status_chirho,
            "add needs session, window#, and name"
        );
    }
}
