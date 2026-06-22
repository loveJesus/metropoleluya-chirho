// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use crate::{
    arg_value_chirho, default_topic_chirho, http_client_chirho, percent_encode_chirho,
    require_arg_chirho, server_arg_chirho,
};
use crossterm::event::{
    self, Event as CrosstermEventChirho, KeyCode as KeyCodeChirho,
    KeyEventKind as KeyEventKindChirho, KeyModifiers as KeyModifiersChirho,
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

const REFRESH_INTERVAL_CHIRHO: Duration = Duration::from_millis(750);
const MAX_MESSAGES_CHIRHO: usize = 300;

#[derive(Debug, Clone)]
struct TuiMessageChirho {
    id_chirho: i64,
    from_identity_chirho: String,
    topic_chirho: String,
    body_chirho: String,
}

#[derive(Debug, Clone)]
struct TuiAgentChirho {
    identity_chirho: String,
    alive_chirho: bool,
    window_index_chirho: Option<String>,
    pane_id_chirho: Option<String>,
    topics_chirho: Vec<String>,
}

#[derive(Debug)]
struct TuiStateChirho {
    server_chirho: String,
    session_chirho: String,
    agent_chirho: String,
    room_chirho: String,
    topic_chirho: String,
    input_chirho: String,
    scroll_offset_chirho: usize,
    status_chirho: String,
    after_chirho: i64,
    messages_chirho: Vec<TuiMessageChirho>,
    agents_chirho: Vec<TuiAgentChirho>,
    last_refresh_chirho: Instant,
}

struct TerminalRestoreChirho;

impl Drop for TerminalRestoreChirho {
    fn drop(&mut self) {
        let _ = disable_raw_mode_chirho();
        let _ = execute!(io::stdout(), LeaveAlternateScreenChirho);
    }
}

pub(crate) fn run_tui_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let mut state_chirho = TuiStateChirho {
        server_chirho: server_arg_chirho(args_chirho),
        session_chirho: require_arg_chirho(args_chirho, "--session")?,
        agent_chirho: require_arg_chirho(args_chirho, "--agent")?,
        room_chirho: require_arg_chirho(args_chirho, "--room")?,
        topic_chirho: arg_value_chirho(args_chirho, "--topic").unwrap_or_else(default_topic_chirho),
        input_chirho: String::new(),
        scroll_offset_chirho: 0,
        status_chirho: "starting room console".to_string(),
        after_chirho: arg_value_chirho(args_chirho, "--after")
            .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
            .unwrap_or(0),
        messages_chirho: Vec::new(),
        agents_chirho: Vec::new(),
        last_refresh_chirho: Instant::now() - REFRESH_INTERVAL_CHIRHO,
    };
    enable_raw_mode_chirho().map_err(|err_chirho| err_chirho.to_string())?;
    let mut stdout_chirho = io::stdout();
    execute!(stdout_chirho, EnterAlternateScreenChirho)
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
    loop {
        refresh_state_if_due_chirho(state_chirho);
        terminal_chirho
            .draw(|frame_chirho| render_tui_chirho(frame_chirho, state_chirho))
            .map_err(|err_chirho| err_chirho.to_string())?;
        if event::poll(Duration::from_millis(100)).map_err(|err_chirho| err_chirho.to_string())? {
            let event_chirho = event::read().map_err(|err_chirho| err_chirho.to_string())?;
            if handle_event_chirho(state_chirho, event_chirho)? {
                break;
            }
        }
    }
    Ok(())
}

fn refresh_state_if_due_chirho(state_chirho: &mut TuiStateChirho) {
    if state_chirho.last_refresh_chirho.elapsed() < REFRESH_INTERVAL_CHIRHO {
        return;
    }
    if let Err(err_chirho) = fetch_messages_chirho(state_chirho) {
        state_chirho.status_chirho = format!("message refresh failed: {err_chirho}");
    }
    if let Err(err_chirho) = fetch_agents_chirho(state_chirho) {
        state_chirho.status_chirho = format!("agent refresh failed: {err_chirho}");
    }
    state_chirho.last_refresh_chirho = Instant::now();
}

fn fetch_messages_chirho(state_chirho: &mut TuiStateChirho) -> Result<(), String> {
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
    for message_chirho in messages_chirho {
        let id_chirho = message_chirho
            .get("id_chirho")
            .and_then(Value::as_i64)
            .unwrap_or(state_chirho.after_chirho);
        state_chirho.after_chirho = state_chirho.after_chirho.max(id_chirho);
        state_chirho.messages_chirho.push(TuiMessageChirho {
            id_chirho,
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
    Ok(())
}

fn fetch_agents_chirho(state_chirho: &mut TuiStateChirho) -> Result<(), String> {
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
    state_chirho.agents_chirho = agents_chirho.iter().map(parse_agent_chirho).collect();
    Ok(())
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
    let CrosstermEventChirho::Key(key_chirho) = event_chirho else {
        return Ok(false);
    };
    if !matches!(
        key_chirho.kind,
        KeyEventKindChirho::Press | KeyEventKindChirho::Repeat
    ) {
        return Ok(false);
    }
    match key_chirho.code {
        KeyCodeChirho::Char('c') if key_chirho.modifiers.contains(KeyModifiersChirho::CONTROL) => {
            Ok(true)
        }
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

fn render_tui_chirho(frame_chirho: &mut Frame<'_>, state_chirho: &TuiStateChirho) {
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

fn render_agents_chirho(
    frame_chirho: &mut Frame<'_>,
    area_chirho: Rect,
    state_chirho: &TuiStateChirho,
) {
    let mut lines_chirho = vec![Line::from(vec![
        Span::styled("Office model: ", Style::default().fg(Color::Yellow)),
        Span::raw("room = open office, topic = workspace"),
    ])];
    for agent_chirho in &state_chirho.agents_chirho {
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
                &agent_chirho.identity_chirho,
                Style::default().fg(Color::Cyan),
            ),
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
    let agents_chirho = Paragraph::new(lines_chirho)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("listeners-chirho"),
        )
        .wrap(Wrap { trim: true });
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
        Line::from(
            "commands: /topic name-chirho, /clear, /quit, PageUp/PageDown/Home/End, Esc or Ctrl-C",
        ),
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
