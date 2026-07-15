// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use crate::{
    arg_value_chirho, default_topic_chirho, http_client_chirho, percent_encode_chirho,
    require_arg_chirho, server_arg_chirho,
};
use crate::tui_membership_chirho::{
    handle_listeners_key_chirho, handle_modal_key_chirho, handle_mouse_event_chirho,
    render_membership_overlay_chirho, toggle_focus_chirho, AgentRowHitChirho,
    MembershipHitsChirho, TuiFocusChirho, TuiModeChirho, ADD_BUTTON_LABEL_CHIRHO,
    MENU_GLYPH_LABEL_CHIRHO,
};
use crossterm::event::{
    self, DisableMouseCapture as DisableMouseCaptureChirho,
    EnableMouseCapture as EnableMouseCaptureChirho, Event as CrosstermEventChirho,
    KeyCode as KeyCodeChirho, KeyEvent as KeyEventChirho, KeyEventKind as KeyEventKindChirho,
    KeyModifiers as KeyModifiersChirho,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode as disable_raw_mode_chirho, enable_raw_mode as enable_raw_mode_chirho,
    EnterAlternateScreen as EnterAlternateScreenChirho,
    LeaveAlternateScreen as LeaveAlternateScreenChirho,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use serde_json::{json, Value};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub(crate) const REFRESH_INTERVAL_CHIRHO: Duration = Duration::from_millis(750);
const MAX_MESSAGES_CHIRHO: usize = 300;

#[derive(Debug, Clone)]
pub(crate) struct TuiMessageChirho {
    id_chirho: i64,
    at_text_chirho: String,
    from_identity_chirho: String,
    topic_chirho: String,
    body_chirho: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TuiAgentChirho {
    pub(crate) identity_chirho: String,
    pub(crate) alive_chirho: bool,
    pub(crate) window_index_chirho: Option<String>,
    pub(crate) pane_id_chirho: Option<String>,
    pub(crate) topics_chirho: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct TuiStateChirho {
    pub(crate) server_chirho: String,
    pub(crate) session_chirho: String,
    pub(crate) agent_chirho: String,
    pub(crate) room_chirho: String,
    pub(crate) topic_chirho: String,
    pub(crate) input_chirho: String,
    pub(crate) scroll_offset_chirho: usize,
    pub(crate) status_chirho: String,
    pub(crate) after_chirho: i64,
    pub(crate) messages_chirho: Vec<TuiMessageChirho>,
    pub(crate) agents_chirho: Vec<TuiAgentChirho>,
    pub(crate) last_refresh_chirho: Instant,
    pub(crate) mode_chirho: TuiModeChirho,
    pub(crate) focus_chirho: TuiFocusChirho,
    pub(crate) selected_agent_chirho: usize,
    pub(crate) hits_chirho: MembershipHitsChirho,
}

impl TuiStateChirho {
    /// Fresh console state; also the constructor the membership tests use.
    pub(crate) fn new_chirho(
        server_chirho: String,
        session_chirho: String,
        agent_chirho: String,
        room_chirho: String,
        topic_chirho: String,
        after_chirho: i64,
    ) -> Self {
        TuiStateChirho {
            server_chirho,
            session_chirho,
            agent_chirho,
            room_chirho,
            topic_chirho,
            input_chirho: String::new(),
            scroll_offset_chirho: 0,
            status_chirho: "starting room console".to_string(),
            after_chirho,
            messages_chirho: Vec::new(),
            agents_chirho: Vec::new(),
            last_refresh_chirho: Instant::now() - REFRESH_INTERVAL_CHIRHO,
            mode_chirho: TuiModeChirho::NormalChirho,
            focus_chirho: TuiFocusChirho::ComposeChirho,
            selected_agent_chirho: 0,
            hits_chirho: MembershipHitsChirho::default(),
        }
    }
}

struct TerminalRestoreChirho;

impl Drop for TerminalRestoreChirho {
    fn drop(&mut self) {
        let _ = disable_raw_mode_chirho();
        let _ = execute!(
            io::stdout(),
            DisableMouseCaptureChirho,
            LeaveAlternateScreenChirho
        );
    }
}

pub(crate) fn run_tui_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let mut state_chirho = TuiStateChirho::new_chirho(
        server_arg_chirho(args_chirho),
        require_arg_chirho(args_chirho, "--session")?,
        require_arg_chirho(args_chirho, "--agent")?,
        require_arg_chirho(args_chirho, "--room")?,
        arg_value_chirho(args_chirho, "--topic").unwrap_or_else(default_topic_chirho),
        arg_value_chirho(args_chirho, "--after")
            .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
            .unwrap_or(0),
    );
    enable_raw_mode_chirho().map_err(|err_chirho| err_chirho.to_string())?;
    let mut stdout_chirho = io::stdout();
    execute!(
        stdout_chirho,
        EnterAlternateScreenChirho,
        EnableMouseCaptureChirho
    )
    .map_err(|err_chirho| err_chirho.to_string())?;
    let _restore_chirho = TerminalRestoreChirho;
    let backend_chirho = CrosstermBackend::new(stdout_chirho);
    let mut terminal_chirho =
        Terminal::new(backend_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    terminal_chirho
        .clear()
        .map_err(|err_chirho| err_chirho.to_string())?;
    run_tui_loop_chirho(&mut terminal_chirho, &mut state_chirho)?;
    terminal_chirho
        .show_cursor()
        .map_err(|err_chirho| err_chirho.to_string())
}

fn run_tui_loop_chirho(
    terminal_chirho: &mut Terminal<CrosstermBackend<Stdout>>,
    state_chirho: &mut TuiStateChirho,
) -> Result<(), String> {
    // Idle rooms should cost nothing. We only repaint when a refresh actually
    // brought new data or an input/resize event arrived; otherwise we just block
    // in event::poll. That keeps an untouched console near 0% CPU instead of
    // rebuilding the whole transcript ten times a second on every poll wakeup.
    let mut needs_render_chirho = true;
    loop {
        if refresh_state_if_due_chirho(state_chirho) {
            needs_render_chirho = true;
        }
        if needs_render_chirho {
            terminal_chirho
                .draw(|frame_chirho| render_tui_chirho(frame_chirho, state_chirho))
                .map_err(|err_chirho| err_chirho.to_string())?;
            needs_render_chirho = false;
        }
        if event::poll(Duration::from_millis(100)).map_err(|err_chirho| err_chirho.to_string())? {
            let event_chirho = event::read().map_err(|err_chirho| err_chirho.to_string())?;
            // Any key, resize, or focus event can change what we show; repaint next tick.
            needs_render_chirho = true;
            if handle_event_chirho(state_chirho, event_chirho)? {
                break;
            }
        }
    }
    Ok(())
}

/// Refreshes room state when the interval is due and reports whether anything
/// the operator can see actually changed, so the caller can skip a repaint when
/// nothing did.
fn refresh_state_if_due_chirho(state_chirho: &mut TuiStateChirho) -> bool {
    if state_chirho.last_refresh_chirho.elapsed() < REFRESH_INTERVAL_CHIRHO {
        return false;
    }
    let mut changed_chirho = false;
    match fetch_messages_chirho(state_chirho) {
        Ok(added_chirho) => changed_chirho |= added_chirho,
        Err(err_chirho) => {
            state_chirho.status_chirho = format!("message refresh failed: {err_chirho}");
            changed_chirho = true;
        }
    }
    match fetch_agents_chirho(state_chirho) {
        Ok(agents_changed_chirho) => changed_chirho |= agents_changed_chirho,
        Err(err_chirho) => {
            state_chirho.status_chirho = format!("agent refresh failed: {err_chirho}");
            changed_chirho = true;
        }
    }
    state_chirho.last_refresh_chirho = Instant::now();
    changed_chirho
}

/// Returns `true` when at least one new message was appended to the transcript.
fn fetch_messages_chirho(state_chirho: &mut TuiStateChirho) -> Result<bool, String> {
    let path_chirho = format!(
        "/v1/messages_chirho?room_chirho={}&after_chirho={}",
        percent_encode_chirho(&state_chirho.room_chirho),
        state_chirho.after_chirho
    );
    let response_chirho =
        http_client_chirho(&state_chirho.server_chirho, "GET", &path_chirho, None)?;
    let messages_chirho = response_chirho
        .get("messages_chirho")
        .and_then(Value::as_array)
        .ok_or_else(|| "messages_chirho missing".to_string())?;
    // The server only returns rows past `after_chirho`, so a non-empty batch is
    // exactly "new content the operator hasn't seen".
    let appended_chirho = !messages_chirho.is_empty();
    for message_chirho in messages_chirho {
        let id_chirho = message_chirho
            .get("id_chirho")
            .and_then(Value::as_i64)
            .unwrap_or(state_chirho.after_chirho);
        state_chirho.after_chirho = state_chirho.after_chirho.max(id_chirho);
        state_chirho.messages_chirho.push(TuiMessageChirho {
            id_chirho,
            at_text_chirho: message_chirho
                .get("at_text_chirho")
                .and_then(Value::as_str)
                .unwrap_or("unknown-time")
                .to_string(),
            from_identity_chirho: message_chirho
                .get("from_identity_chirho")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            topic_chirho: message_chirho
                .get("topic_chirho")
                .and_then(Value::as_str)
                .unwrap_or("general-chirho")
                .to_string(),
            body_chirho: message_chirho
                .get("body_chirho")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        });
    }
    if state_chirho.messages_chirho.len() > MAX_MESSAGES_CHIRHO {
        let drop_count_chirho = state_chirho.messages_chirho.len() - MAX_MESSAGES_CHIRHO;
        state_chirho.messages_chirho.drain(0..drop_count_chirho);
    }
    Ok(appended_chirho)
}

/// Returns `true` when the roster differs from what is already on screen.
fn fetch_agents_chirho(state_chirho: &mut TuiStateChirho) -> Result<bool, String> {
    let path_chirho = format!(
        "/v1/agents_chirho?room_chirho={}",
        percent_encode_chirho(&state_chirho.room_chirho)
    );
    let response_chirho =
        http_client_chirho(&state_chirho.server_chirho, "GET", &path_chirho, None)?;
    let agents_chirho = response_chirho
        .get("agents_chirho")
        .and_then(Value::as_array)
        .ok_or_else(|| "agents_chirho missing".to_string())?;
    let parsed_chirho: Vec<TuiAgentChirho> = agents_chirho.iter().map(parse_agent_chirho).collect();
    let changed_chirho = parsed_chirho != state_chirho.agents_chirho;
    state_chirho.agents_chirho = parsed_chirho;
    Ok(changed_chirho)
}

fn parse_agent_chirho(value_chirho: &Value) -> TuiAgentChirho {
    TuiAgentChirho {
        identity_chirho: value_chirho
            .get("identity_chirho")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        alive_chirho: value_chirho
            .get("alive_chirho")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        window_index_chirho: value_chirho
            .get("window_index_chirho")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        pane_id_chirho: value_chirho
            .get("pane_id_chirho")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        topics_chirho: value_chirho
            .get("topics_chirho")
            .and_then(Value::as_array)
            .map(|topics_chirho| {
                topics_chirho
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn handle_event_chirho(
    state_chirho: &mut TuiStateChirho,
    event_chirho: CrosstermEventChirho,
) -> Result<bool, String> {
    match event_chirho {
        CrosstermEventChirho::Key(key_chirho) => handle_key_event_chirho(state_chirho, key_chirho),
        CrosstermEventChirho::Mouse(mouse_chirho) => {
            handle_mouse_event_chirho(state_chirho, mouse_chirho);
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Keyboard routing: open popups first, then the Tab focus toggle, then
/// whichever pane owns the focus (compose box types; listeners list gets the
/// membership-admin keys). Part of
/// spec-chirho/workflows-chirho/room-membership-admin-flow-chirho.md.
fn handle_key_event_chirho(
    state_chirho: &mut TuiStateChirho,
    key_chirho: KeyEventChirho,
) -> Result<bool, String> {
    if !matches!(
        key_chirho.kind,
        KeyEventKindChirho::Press | KeyEventKindChirho::Repeat
    ) {
        return Ok(false);
    }
    if key_chirho.code == KeyCodeChirho::Char('c')
        && key_chirho.modifiers.contains(KeyModifiersChirho::CONTROL)
    {
        return Ok(true);
    }
    if state_chirho.mode_chirho != TuiModeChirho::NormalChirho {
        handle_modal_key_chirho(state_chirho, key_chirho);
        return Ok(false);
    }
    if key_chirho.code == KeyCodeChirho::Tab {
        toggle_focus_chirho(state_chirho);
        return Ok(false);
    }
    if state_chirho.focus_chirho == TuiFocusChirho::ListenersChirho {
        handle_listeners_key_chirho(state_chirho, key_chirho);
        return Ok(false);
    }
    match key_chirho.code {
        KeyCodeChirho::Char(value_chirho) => {
            state_chirho.input_chirho.push(value_chirho);
            Ok(false)
        }
        KeyCodeChirho::Backspace => {
            state_chirho.input_chirho.pop();
            Ok(false)
        }
        KeyCodeChirho::Enter => submit_input_chirho(state_chirho),
        KeyCodeChirho::PageUp => {
            state_chirho.scroll_offset_chirho =
                state_chirho.scroll_offset_chirho.saturating_add(10);
            state_chirho.status_chirho = "scrollback: older messages".to_string();
            Ok(false)
        }
        KeyCodeChirho::PageDown => {
            state_chirho.scroll_offset_chirho =
                state_chirho.scroll_offset_chirho.saturating_sub(10);
            state_chirho.status_chirho = "scrollback: newer messages".to_string();
            Ok(false)
        }
        KeyCodeChirho::Home => {
            state_chirho.scroll_offset_chirho = usize::MAX / 4;
            state_chirho.status_chirho = "scrollback: oldest loaded messages".to_string();
            Ok(false)
        }
        KeyCodeChirho::End => {
            state_chirho.scroll_offset_chirho = 0;
            state_chirho.status_chirho = "scrollback: live tail".to_string();
            Ok(false)
        }
        KeyCodeChirho::Esc => Ok(true),
        _ => Ok(false),
    }
}

fn submit_input_chirho(state_chirho: &mut TuiStateChirho) -> Result<bool, String> {
    let input_chirho = state_chirho.input_chirho.trim().to_string();
    state_chirho.input_chirho.clear();
    if input_chirho.is_empty() {
        return Ok(false);
    }
    if input_chirho == "/quit" {
        return Ok(true);
    }
    if input_chirho == "/clear" {
        state_chirho.messages_chirho.clear();
        state_chirho.status_chirho = "cleared local transcript view".to_string();
        return Ok(false);
    }
    if let Some(topic_chirho) = input_chirho.strip_prefix("/topic ") {
        let topic_chirho = topic_chirho.trim();
        if topic_chirho.is_empty() {
            state_chirho.status_chirho = "topic command needs a value".to_string();
        } else {
            state_chirho.topic_chirho = topic_chirho.to_string();
            state_chirho.status_chirho = format!("speaking at workspace {topic_chirho}");
        }
        return Ok(false);
    }
    post_tui_message_chirho(state_chirho, &input_chirho)?;
    Ok(false)
}

fn post_tui_message_chirho(
    state_chirho: &mut TuiStateChirho,
    input_chirho: &str,
) -> Result<(), String> {
    let body_chirho = json!({
        "from_session_chirho": state_chirho.session_chirho,
        "from_agent_chirho": state_chirho.agent_chirho,
        "room_chirho": state_chirho.room_chirho,
        "topic_chirho": state_chirho.topic_chirho,
        "body_chirho": input_chirho,
        "to_chirho": [],
        "deliver_to_sender_chirho": false
    });
    let response_chirho = http_client_chirho(
        &state_chirho.server_chirho,
        "POST",
        "/v1/post_chirho",
        Some(&body_chirho),
    )?;
    let message_id_chirho = response_chirho
        .get("message_id_chirho")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let delivery_count_chirho = response_chirho
        .get("delivery_count_chirho")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    state_chirho.status_chirho = format!(
        "posted #{message_id_chirho} to {} / {} for {delivery_count_chirho} tmux targets",
        state_chirho.room_chirho, state_chirho.topic_chirho
    );
    state_chirho.last_refresh_chirho = Instant::now() - REFRESH_INTERVAL_CHIRHO;
    Ok(())
}

fn render_tui_chirho(frame_chirho: &mut Frame<'_>, state_chirho: &mut TuiStateChirho) {
    let area_chirho = frame_chirho.area();
    frame_chirho.render_widget(Clear, area_chirho);
    let root_chunks_chirho = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area_chirho);
    render_header_chirho(frame_chirho, root_chunks_chirho[0], state_chirho);
    let body_chunks_chirho = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(root_chunks_chirho[1]);
    render_transcript_chirho(frame_chirho, body_chunks_chirho[0], state_chirho);
    render_agents_chirho(frame_chirho, body_chunks_chirho[1], state_chirho);
    render_input_chirho(frame_chirho, root_chunks_chirho[2], state_chirho);
    render_membership_overlay_chirho(frame_chirho, area_chirho, state_chirho);
}

fn render_header_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &TuiStateChirho,
) {
    let header_chirho = vec![Line::from(vec![
        Span::styled(
            "Metropoleluya ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("office "),
        Span::styled(
            &state_chirho.room_chirho,
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | workspace "),
        Span::styled(
            &state_chirho.topic_chirho,
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | speaking as "),
        Span::styled(
            format!(
                "{}/{}",
                state_chirho.session_chirho, state_chirho.agent_chirho
            ),
            Style::default().fg(Color::Magenta),
        ),
    ])];
    let block_chirho = Paragraph::new(header_chirho)
        .block(Block::default().borders(Borders::ALL).title("room-chirho"))
        .wrap(Wrap { trim: true });
    frame_chirho.render_widget(block_chirho, area_chirho);
}

fn render_transcript_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &TuiStateChirho,
) {
    let mut lines_chirho = Vec::new();
    for message_chirho in &state_chirho.messages_chirho {
        lines_chirho.push(Line::from(vec![
            Span::styled(
                format!("#{} ", message_chirho.id_chirho),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} ", message_chirho.at_text_chirho),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                &message_chirho.from_identity_chirho,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" at "),
            Span::styled(
                &message_chirho.topic_chirho,
                Style::default().fg(Color::Green),
            ),
        ]));
        for body_line_chirho in message_chirho.body_chirho.lines() {
            lines_chirho.push(Line::from(format!("  {body_line_chirho}")));
        }
        lines_chirho.push(Line::from(""));
    }
    if lines_chirho.is_empty() {
        lines_chirho.push(Line::from("No room messages yet."));
    }
    let visible_lines_chirho = visible_transcript_lines_chirho(
        lines_chirho,
        area_chirho.height.saturating_sub(2) as usize,
        state_chirho.scroll_offset_chirho,
    );
    let transcript_chirho = Paragraph::new(visible_lines_chirho)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("transcript-chirho"),
        )
        .wrap(Wrap { trim: false });
    frame_chirho.render_widget(transcript_chirho, area_chirho);
}

fn visible_transcript_lines_chirho<'line_chirho>(
    lines_chirho: Vec<Line<'line_chirho>>,
    max_lines_chirho: usize,
    scroll_offset_chirho: usize,
) -> Vec<Line<'line_chirho>> {
    if max_lines_chirho == 0 || lines_chirho.len() <= max_lines_chirho {
        return lines_chirho;
    }
    let offset_chirho =
        scroll_offset_chirho.min(lines_chirho.len().saturating_sub(max_lines_chirho));
    let start_chirho = lines_chirho
        .len()
        .saturating_sub(max_lines_chirho)
        .saturating_sub(offset_chirho);
    lines_chirho
        .into_iter()
        .skip(start_chirho)
        .take(max_lines_chirho)
        .collect()
}

/// Renders the listeners pane and records the hit-test geometry that the
/// mouse handlers in tui_membership_chirho test against (see
/// spec-chirho/workflows-chirho/room-membership-admin-flow-chirho.md). Wrap
/// stays off so every listener occupies exactly four rows and the row math
/// below matches what is on screen.
fn render_agents_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &mut TuiStateChirho,
) {
    let focused_chirho = state_chirho.focus_chirho == TuiFocusChirho::ListenersChirho;
    let selected_chirho = state_chirho.selected_agent_chirho;
    let content_top_chirho = area_chirho.y.saturating_add(1);
    let content_bottom_chirho = area_chirho.bottom().saturating_sub(2);
    state_chirho.hits_chirho.listeners_rect_chirho = Some(area_chirho);
    state_chirho.hits_chirho.agent_rows_chirho.clear();
    let add_row_chirho = content_top_chirho.saturating_add(1);
    state_chirho.hits_chirho.add_button_rect_chirho = if add_row_chirho <= content_bottom_chirho {
        Some(Rect {
            x: area_chirho.x.saturating_add(1),
            y: add_row_chirho,
            width: (ADD_BUTTON_LABEL_CHIRHO.len() as u16).min(area_chirho.width.saturating_sub(2)),
            height: 1,
        })
    } else {
        None
    };
    let mut lines_chirho = vec![Line::from(vec![
        Span::styled("Office model: ", Style::default().fg(Color::Yellow)),
        Span::raw("room = open office, topic = workspace"),
    ])];
    lines_chirho.push(Line::from(Span::styled(
        ADD_BUTTON_LABEL_CHIRHO,
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    for (index_chirho, agent_chirho) in state_chirho.agents_chirho.iter().enumerate() {
        let row_start_chirho = content_top_chirho.saturating_add(2 + (index_chirho as u16) * 4);
        if row_start_chirho <= content_bottom_chirho {
            state_chirho
                .hits_chirho
                .agent_rows_chirho
                .push(AgentRowHitChirho {
                    row_start_chirho,
                    row_end_chirho: row_start_chirho.saturating_add(2).min(content_bottom_chirho),
                    agent_index_chirho: index_chirho,
                });
        }
        let mut identity_style_chirho = Style::default().fg(Color::Cyan);
        if focused_chirho && index_chirho == selected_chirho {
            identity_style_chirho = identity_style_chirho.add_modifier(Modifier::REVERSED);
        }
        let alive_chirho = if agent_chirho.alive_chirho {
            Span::styled("alive", Style::default().fg(Color::Green))
        } else {
            Span::styled("dead", Style::default().fg(Color::Red))
        };
        let pane_chirho = format!(
            " win={} pane={}",
            agent_chirho.window_index_chirho.as_deref().unwrap_or("?"),
            agent_chirho.pane_id_chirho.as_deref().unwrap_or("?")
        );
        lines_chirho.push(Line::from(vec![
            Span::styled(
                MENU_GLYPH_LABEL_CHIRHO,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(&agent_chirho.identity_chirho, identity_style_chirho),
            Span::raw(" "),
            alive_chirho,
        ]));
        lines_chirho.push(Line::from(Span::styled(
            pane_chirho,
            Style::default().fg(Color::DarkGray),
        )));
        let topics_chirho = if agent_chirho.topics_chirho.is_empty() {
            "topics: none".to_string()
        } else {
            format!("topics: {}", agent_chirho.topics_chirho.join(", "))
        };
        lines_chirho.push(Line::from(Span::styled(
            topics_chirho,
            Style::default().fg(Color::Green),
        )));
        lines_chirho.push(Line::from(""));
    }
    let title_chirho = if focused_chirho {
        "listeners-chirho (focused)"
    } else {
        "listeners-chirho"
    };
    let agents_chirho = Paragraph::new(lines_chirho).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title_chirho),
    );
    frame_chirho.render_widget(agents_chirho, area_chirho);
}

fn render_input_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &TuiStateChirho,
) {
    let input_chirho = vec![
        Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::Yellow)),
            Span::raw(&state_chirho.status_chirho),
        ]),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&state_chirho.input_chirho),
        ]),
        Line::from(match state_chirho.focus_chirho {
            TuiFocusChirho::ComposeChirho => {
                "commands: /topic name-chirho, /clear, /quit | Tab focuses listeners | PageUp/PageDown/Home/End | Esc/Ctrl-C quit"
            }
            TuiFocusChirho::ListenersChirho => {
                "listeners: Up/Down select, Enter menu, x/Delete remove, a/+ add, Esc back | Tab: compose | click [☰] for menu, [ + add ] to add"
            }
        }),
    ];
    let block_chirho = Paragraph::new(input_chirho)
        .block(Block::default().borders(Borders::ALL).title("speak-chirho"))
        .wrap(Wrap { trim: false });
    frame_chirho.render_widget(block_chirho, area_chirho);
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn parse_agent_handles_missing_topics_chirho() {
        let agent_chirho = parse_agent_chirho(&json!({
            "identity_chirho": "PROJECT_CHIRHO/gpt_chirho",
            "alive_chirho": true,
            "window_index_chirho": "3",
            "pane_id_chirho": "%19"
        }));
        assert_eq!(agent_chirho.identity_chirho, "PROJECT_CHIRHO/gpt_chirho");
        assert!(agent_chirho.alive_chirho);
        assert!(agent_chirho.topics_chirho.is_empty());
    }

    #[test]
    fn tui_message_keeps_readable_timestamp_chirho() {
        let message_chirho = TuiMessageChirho {
            id_chirho: 7,
            at_text_chirho: "2024-01-01 00:00:00.123Z".to_string(),
            from_identity_chirho: "PROJECT_CHIRHO/gpt_chirho".to_string(),
            topic_chirho: "audit-chirho".to_string(),
            body_chirho: "body-chirho".to_string(),
        };
        assert_eq!(message_chirho.at_text_chirho, "2024-01-01 00:00:00.123Z");
    }

    #[test]
    fn visible_transcript_returns_tail_by_default_chirho() {
        let lines_chirho = vec![Line::from("one"), Line::from("two"), Line::from("three")];
        let visible_chirho = visible_transcript_lines_chirho(lines_chirho, 2, 0);
        assert_eq!(visible_chirho.len(), 2);
        assert_eq!(visible_chirho[0].to_string(), "two");
    }

    #[test]
    fn visible_transcript_scrolls_to_older_lines_chirho() {
        let lines_chirho = vec![Line::from("one"), Line::from("two"), Line::from("three")];
        let visible_chirho = visible_transcript_lines_chirho(lines_chirho, 2, 1);
        assert_eq!(visible_chirho.len(), 2);
        assert_eq!(visible_chirho[0].to_string(), "one");
    }
}
