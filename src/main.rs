// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_SERVER_CHIRHO: &str = "http://127.0.0.1:37371";
const BUFFER_NAME_CHIRHO: &str = "metropoleluya-chirho";

#[derive(Debug, Deserialize)]
struct RegisterRequestChirho {
    session_chirho: String,
    agent_chirho: String,
    tmux_target_chirho: String,
    #[serde(default)]
    rooms_chirho: Vec<String>,
    #[serde(default)]
    topics_chirho: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PostRequestChirho {
    from_session_chirho: String,
    from_agent_chirho: String,
    room_chirho: String,
    #[serde(default = "default_topic_chirho")]
    topic_chirho: String,
    body_chirho: String,
    #[serde(default)]
    to_chirho: Vec<String>,
    #[serde(default)]
    deliver_to_sender_chirho: bool,
}

#[derive(Debug, Serialize)]
struct MessageViewChirho {
    id_chirho: i64,
    at_ms_chirho: i64,
    from_identity_chirho: String,
    room_chirho: String,
    topic_chirho: String,
    body_chirho: String,
}

#[derive(Debug)]
struct AgentTargetChirho {
    identity_chirho: String,
    tmux_target_chirho: String,
}

#[derive(Debug)]
struct TmuxProbeChirho {
    alive_chirho: bool,
    session_chirho: Option<String>,
    window_index_chirho: Option<String>,
    pane_id_chirho: Option<String>,
    error_chirho: Option<String>,
}

fn default_topic_chirho() -> String {
    "general-chirho".to_string()
}

fn now_ms_chirho() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as i64
}

fn default_db_path_chirho() -> PathBuf {
    let home_chirho = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_chirho)
        .join(".metropoleluya-chirho")
        .join("metropoleluya-chirho.sqlite")
}

fn identity_chirho(session_chirho: &str, agent_chirho: &str) -> String {
    format!("{}/{}", session_chirho, agent_chirho)
}

fn validate_token_chirho(name_chirho: &str, value_chirho: &str) -> Result<(), String> {
    if value_chirho.trim().is_empty() {
        return Err(format!("{name_chirho} is required"));
    }
    if value_chirho
        .chars()
        .any(|char_chirho| char_chirho.is_control())
    {
        return Err(format!("{name_chirho} must not contain control characters"));
    }
    Ok(())
}

fn open_db_chirho(path_chirho: Option<PathBuf>) -> Result<Connection, String> {
    let path_chirho = path_chirho.unwrap_or_else(default_db_path_chirho);
    if let Some(parent_chirho) = path_chirho.parent() {
        fs::create_dir_all(parent_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    }
    let conn_chirho = Connection::open(path_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    init_db_chirho(&conn_chirho)?;
    Ok(conn_chirho)
}

fn init_db_chirho(conn_chirho: &Connection) -> Result<(), String> {
    conn_chirho
        .execute_batch(
            r#"
            create table if not exists agents_chirho (
                identity_chirho text primary key,
                session_chirho text not null,
                agent_chirho text not null,
                tmux_target_chirho text not null,
                window_index_chirho text,
                pane_id_chirho text,
                alive_chirho integer not null default 0,
                created_ms_chirho integer not null,
                last_seen_ms_chirho integer not null
            );
            create table if not exists subscriptions_chirho (
                identity_chirho text not null,
                room_chirho text not null,
                topic_chirho text not null,
                active_chirho integer not null default 1,
                primary key (identity_chirho, room_chirho, topic_chirho)
            );
            create table if not exists messages_chirho (
                id_chirho integer primary key autoincrement,
                at_ms_chirho integer not null,
                from_identity_chirho text not null,
                room_chirho text not null,
                topic_chirho text not null,
                body_chirho text not null
            );
            create table if not exists deliveries_chirho (
                id_chirho integer primary key autoincrement,
                message_id_chirho integer not null,
                to_identity_chirho text not null,
                tmux_target_chirho text not null,
                alive_chirho integer not null,
                delivered_chirho integer not null,
                error_chirho text,
                at_ms_chirho integer not null
            );
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())
}

fn register_agent_chirho(
    conn_chirho: &Connection,
    request_chirho: RegisterRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("session_chirho", &request_chirho.session_chirho)?;
    validate_token_chirho("agent_chirho", &request_chirho.agent_chirho)?;
    validate_token_chirho("tmux_target_chirho", &request_chirho.tmux_target_chirho)?;
    let identity_chirho =
        identity_chirho(&request_chirho.session_chirho, &request_chirho.agent_chirho);
    let probe_chirho = probe_tmux_target_chirho(&request_chirho.tmux_target_chirho);
    let now_chirho = now_ms_chirho();
    conn_chirho
        .execute(
            r#"
            insert into agents_chirho (
                identity_chirho, session_chirho, agent_chirho, tmux_target_chirho,
                window_index_chirho, pane_id_chirho, alive_chirho, created_ms_chirho, last_seen_ms_chirho
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            on conflict(identity_chirho) do update set
                tmux_target_chirho = excluded.tmux_target_chirho,
                window_index_chirho = excluded.window_index_chirho,
                pane_id_chirho = excluded.pane_id_chirho,
                alive_chirho = excluded.alive_chirho,
                last_seen_ms_chirho = excluded.last_seen_ms_chirho
            "#,
            params![
                identity_chirho,
                request_chirho.session_chirho,
                request_chirho.agent_chirho,
                request_chirho.tmux_target_chirho,
                probe_chirho.window_index_chirho,
                probe_chirho.pane_id_chirho,
                if probe_chirho.alive_chirho { 1 } else { 0 },
                now_chirho
            ],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;

    let topics_chirho = if request_chirho.topics_chirho.is_empty() {
        vec!["*".to_string()]
    } else {
        request_chirho.topics_chirho
    };
    for room_chirho in request_chirho.rooms_chirho {
        validate_token_chirho("room_chirho", &room_chirho)?;
        for topic_chirho in &topics_chirho {
            validate_token_chirho("topic_chirho", topic_chirho)?;
            conn_chirho
                .execute(
                    r#"
                    insert into subscriptions_chirho (identity_chirho, room_chirho, topic_chirho, active_chirho)
                    values (?1, ?2, ?3, 1)
                    on conflict(identity_chirho, room_chirho, topic_chirho) do update set active_chirho = 1
                    "#,
                    params![identity_chirho, room_chirho, topic_chirho],
                )
                .map_err(|err_chirho| err_chirho.to_string())?;
        }
    }

    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": identity_chirho,
        "alive_chirho": probe_chirho.alive_chirho,
        "observed_session_chirho": probe_chirho.session_chirho,
        "window_index_chirho": probe_chirho.window_index_chirho,
        "pane_id_chirho": probe_chirho.pane_id_chirho,
        "error_chirho": probe_chirho.error_chirho
    }))
}

fn post_message_chirho(
    conn_chirho: &mut Connection,
    request_chirho: PostRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("from_session_chirho", &request_chirho.from_session_chirho)?;
    validate_token_chirho("from_agent_chirho", &request_chirho.from_agent_chirho)?;
    validate_token_chirho("room_chirho", &request_chirho.room_chirho)?;
    validate_token_chirho("topic_chirho", &request_chirho.topic_chirho)?;
    validate_token_chirho("body_chirho", &request_chirho.body_chirho)?;
    let from_identity_chirho = identity_chirho(
        &request_chirho.from_session_chirho,
        &request_chirho.from_agent_chirho,
    );
    let tx_chirho = conn_chirho
        .transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            r#"
            insert into messages_chirho (at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho)
            values (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                now_ms_chirho(),
                from_identity_chirho,
                request_chirho.room_chirho,
                request_chirho.topic_chirho,
                request_chirho.body_chirho
            ],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let message_id_chirho = tx_chirho.last_insert_rowid();
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;

    let targets_chirho =
        resolve_targets_chirho(conn_chirho, &request_chirho, &from_identity_chirho)?;
    let mut delivered_count_chirho = 0usize;
    for target_chirho in targets_chirho {
        let text_chirho = format_delivery_chirho(
            message_id_chirho,
            &from_identity_chirho,
            &request_chirho.room_chirho,
            &request_chirho.topic_chirho,
            &request_chirho.body_chirho,
        );
        let probe_chirho = probe_tmux_target_chirho(&target_chirho.tmux_target_chirho);
        let result_chirho = if probe_chirho.alive_chirho {
            send_tmux_message_chirho(&target_chirho.tmux_target_chirho, &text_chirho)
        } else {
            Err(probe_chirho
                .error_chirho
                .clone()
                .unwrap_or_else(|| "tmux target is not alive".to_string()))
        };
        let delivered_chirho = result_chirho.is_ok();
        if delivered_chirho {
            delivered_count_chirho += 1;
        }
        conn_chirho
            .execute(
                r#"
                insert into deliveries_chirho (
                    message_id_chirho, to_identity_chirho, tmux_target_chirho,
                    alive_chirho, delivered_chirho, error_chirho, at_ms_chirho
                ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    message_id_chirho,
                    target_chirho.identity_chirho,
                    target_chirho.tmux_target_chirho,
                    if probe_chirho.alive_chirho { 1 } else { 0 },
                    if delivered_chirho { 1 } else { 0 },
                    result_chirho.err(),
                    now_ms_chirho()
                ],
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
    }

    Ok(json!({
        "ok_chirho": true,
        "message_id_chirho": message_id_chirho,
        "delivery_count_chirho": delivered_count_chirho
    }))
}

fn resolve_targets_chirho(
    conn_chirho: &Connection,
    request_chirho: &PostRequestChirho,
    from_identity_chirho: &str,
) -> Result<Vec<AgentTargetChirho>, String> {
    let mut targets_chirho = Vec::new();
    if request_chirho.to_chirho.is_empty() {
        let mut stmt_chirho = conn_chirho
            .prepare(
                r#"
                select distinct a.identity_chirho, a.tmux_target_chirho
                from agents_chirho a
                join subscriptions_chirho s on s.identity_chirho = a.identity_chirho
                where s.room_chirho = ?1
                  and s.active_chirho = 1
                  and (s.topic_chirho = '*' or s.topic_chirho = ?2)
                order by a.identity_chirho
                "#,
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        let rows_chirho = stmt_chirho
            .query_map(
                params![request_chirho.room_chirho, request_chirho.topic_chirho],
                |row_chirho| {
                    Ok(AgentTargetChirho {
                        identity_chirho: row_chirho.get(0)?,
                        tmux_target_chirho: row_chirho.get(1)?,
                    })
                },
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        for row_chirho in rows_chirho {
            let target_chirho = row_chirho.map_err(|err_chirho| err_chirho.to_string())?;
            if request_chirho.deliver_to_sender_chirho
                || target_chirho.identity_chirho != from_identity_chirho
            {
                targets_chirho.push(target_chirho);
            }
        }
    } else {
        for identity_chirho in &request_chirho.to_chirho {
            let target_chirho = conn_chirho
                .query_row(
                    "select identity_chirho, tmux_target_chirho from agents_chirho where identity_chirho = ?1",
                    params![identity_chirho],
                    |row_chirho| {
                        Ok(AgentTargetChirho {
                            identity_chirho: row_chirho.get(0)?,
                            tmux_target_chirho: row_chirho.get(1)?,
                        })
                    },
                )
                .optional()
                .map_err(|err_chirho| err_chirho.to_string())?;
            if let Some(target_chirho) = target_chirho {
                targets_chirho.push(target_chirho);
            }
        }
    }
    Ok(targets_chirho)
}

fn list_messages_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    after_chirho: i64,
) -> Result<Value, String> {
    let mut stmt_chirho = conn_chirho
        .prepare(
            r#"
            select id_chirho, at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho
            from messages_chirho
            where room_chirho = ?1 and id_chirho > ?2
            order by id_chirho
            limit 200
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let rows_chirho = stmt_chirho
        .query_map(params![room_chirho, after_chirho], |row_chirho| {
            Ok(MessageViewChirho {
                id_chirho: row_chirho.get(0)?,
                at_ms_chirho: row_chirho.get(1)?,
                from_identity_chirho: row_chirho.get(2)?,
                room_chirho: row_chirho.get(3)?,
                topic_chirho: row_chirho.get(4)?,
                body_chirho: row_chirho.get(5)?,
            })
        })
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut messages_chirho = Vec::new();
    for row_chirho in rows_chirho {
        messages_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
    }
    Ok(json!({ "ok_chirho": true, "messages_chirho": messages_chirho }))
}

fn list_agents_chirho(
    conn_chirho: &Connection,
    room_chirho: Option<&str>,
) -> Result<Value, String> {
    let sql_chirho = if room_chirho.is_some() {
        r#"
        select distinct a.identity_chirho, a.session_chirho, a.agent_chirho, a.tmux_target_chirho,
               a.window_index_chirho, a.pane_id_chirho, a.alive_chirho, a.last_seen_ms_chirho
        from agents_chirho a
        join subscriptions_chirho s on s.identity_chirho = a.identity_chirho
        where s.room_chirho = ?1 and s.active_chirho = 1
        order by a.identity_chirho
        "#
    } else {
        r#"
        select identity_chirho, session_chirho, agent_chirho, tmux_target_chirho,
               window_index_chirho, pane_id_chirho, alive_chirho, last_seen_ms_chirho
        from agents_chirho
        order by identity_chirho
        "#
    };
    let mut stmt_chirho = conn_chirho
        .prepare(sql_chirho)
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut agents_chirho = Vec::new();
    if let Some(room_chirho) = room_chirho {
        let rows_chirho = stmt_chirho
            .query_map(params![room_chirho], row_agent_json_chirho)
            .map_err(|err_chirho| err_chirho.to_string())?;
        for row_chirho in rows_chirho {
            agents_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
        }
    } else {
        let rows_chirho = stmt_chirho
            .query_map([], row_agent_json_chirho)
            .map_err(|err_chirho| err_chirho.to_string())?;
        for row_chirho in rows_chirho {
            agents_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
        }
    }
    Ok(json!({ "ok_chirho": true, "agents_chirho": agents_chirho }))
}

fn row_agent_json_chirho(row_chirho: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let alive_chirho: i64 = row_chirho.get(6)?;
    Ok(json!({
        "identity_chirho": row_chirho.get::<_, String>(0)?,
        "session_chirho": row_chirho.get::<_, String>(1)?,
        "agent_chirho": row_chirho.get::<_, String>(2)?,
        "tmux_target_chirho": row_chirho.get::<_, String>(3)?,
        "window_index_chirho": row_chirho.get::<_, Option<String>>(4)?,
        "pane_id_chirho": row_chirho.get::<_, Option<String>>(5)?,
        "alive_chirho": alive_chirho == 1,
        "last_seen_ms_chirho": row_chirho.get::<_, i64>(7)?
    }))
}

fn refresh_agents_chirho(conn_chirho: &Connection) -> Result<Value, String> {
    let mut stmt_chirho = conn_chirho
        .prepare("select identity_chirho, tmux_target_chirho from agents_chirho order by identity_chirho")
        .map_err(|err_chirho| err_chirho.to_string())?;
    let rows_chirho = stmt_chirho
        .query_map([], |row_chirho| {
            Ok((
                row_chirho.get::<_, String>(0)?,
                row_chirho.get::<_, String>(1)?,
            ))
        })
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut count_chirho = 0usize;
    for row_chirho in rows_chirho {
        let (identity_chirho, target_chirho) =
            row_chirho.map_err(|err_chirho| err_chirho.to_string())?;
        let probe_chirho = probe_tmux_target_chirho(&target_chirho);
        conn_chirho
            .execute(
                r#"
                update agents_chirho
                set alive_chirho = ?1, window_index_chirho = ?2, pane_id_chirho = ?3, last_seen_ms_chirho = ?4
                where identity_chirho = ?5
                "#,
                params![
                    if probe_chirho.alive_chirho { 1 } else { 0 },
                    probe_chirho.window_index_chirho,
                    probe_chirho.pane_id_chirho,
                    now_ms_chirho(),
                    identity_chirho
                ],
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        count_chirho += 1;
    }
    Ok(json!({ "ok_chirho": true, "refreshed_chirho": count_chirho }))
}

fn probe_tmux_target_chirho(target_chirho: &str) -> TmuxProbeChirho {
    let output_chirho = Command::new("tmux")
        .args([
            "display-message",
            "-p",
            "-t",
            target_chirho,
            "#{session_name}\t#{window_index}\t#{pane_id}",
        ])
        .output();
    match output_chirho {
        Ok(output_chirho) if output_chirho.status.success() => {
            let text_chirho = String::from_utf8_lossy(&output_chirho.stdout);
            let parts_chirho: Vec<&str> = text_chirho.trim().split('\t').collect();
            TmuxProbeChirho {
                alive_chirho: true,
                session_chirho: parts_chirho
                    .first()
                    .map(|value_chirho| value_chirho.to_string()),
                window_index_chirho: parts_chirho
                    .get(1)
                    .map(|value_chirho| value_chirho.to_string()),
                pane_id_chirho: parts_chirho
                    .get(2)
                    .map(|value_chirho| value_chirho.to_string()),
                error_chirho: None,
            }
        }
        Ok(output_chirho) => TmuxProbeChirho {
            alive_chirho: false,
            session_chirho: None,
            window_index_chirho: None,
            pane_id_chirho: None,
            error_chirho: Some(
                String::from_utf8_lossy(&output_chirho.stderr)
                    .trim()
                    .to_string(),
            ),
        },
        Err(err_chirho) => TmuxProbeChirho {
            alive_chirho: false,
            session_chirho: None,
            window_index_chirho: None,
            pane_id_chirho: None,
            error_chirho: Some(err_chirho.to_string()),
        },
    }
}

fn format_delivery_chirho(
    message_id_chirho: i64,
    from_identity_chirho: &str,
    room_chirho: &str,
    topic_chirho: &str,
    body_chirho: &str,
) -> String {
    format!(
        "METROPOLELUYA_CHIRHO MESSAGE #{message_id_chirho} FROM {from_identity_chirho} ROOM {room_chirho} TOPIC {topic_chirho}\n{body_chirho}"
    )
}

fn send_tmux_message_chirho(target_chirho: &str, text_chirho: &str) -> Result<(), String> {
    let mut child_chirho = Command::new("tmux")
        .args(["load-buffer", "-b", BUFFER_NAME_CHIRHO, "-"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err_chirho| err_chirho.to_string())?;
    child_chirho
        .stdin
        .as_mut()
        .ok_or_else(|| "failed to open tmux load-buffer stdin".to_string())?
        .write_all(text_chirho.as_bytes())
        .map_err(|err_chirho| err_chirho.to_string())?;
    let status_chirho = child_chirho
        .wait()
        .map_err(|err_chirho| err_chirho.to_string())?;
    if !status_chirho.success() {
        return Err("tmux load-buffer failed".to_string());
    }
    run_tmux_chirho(&[
        "paste-buffer",
        "-b",
        BUFFER_NAME_CHIRHO,
        "-t",
        target_chirho,
    ])?;
    run_tmux_chirho(&["send-keys", "-t", target_chirho, "Enter"])?;
    thread::sleep(Duration::from_secs(1));
    run_tmux_chirho(&["send-keys", "-t", target_chirho, "Enter"])?;
    Ok(())
}

fn run_tmux_chirho(args_chirho: &[&str]) -> Result<(), String> {
    let output_chirho = Command::new("tmux")
        .args(args_chirho)
        .output()
        .map_err(|err_chirho| err_chirho.to_string())?;
    if output_chirho.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output_chirho.stderr)
            .trim()
            .to_string())
    }
}

fn run_server_chirho(bind_chirho: &str, db_path_chirho: Option<PathBuf>) -> Result<(), String> {
    let listener_chirho =
        TcpListener::bind(bind_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    println!("metropoleluya-chirho listening on http://{bind_chirho}");
    for stream_chirho in listener_chirho.incoming() {
        match stream_chirho {
            Ok(stream_chirho) => {
                let db_path_chirho = db_path_chirho.clone();
                thread::spawn(move || {
                    if let Err(err_chirho) = handle_http_chirho(stream_chirho, db_path_chirho) {
                        eprintln!("request error: {err_chirho}");
                    }
                });
            }
            Err(err_chirho) => eprintln!("accept error: {err_chirho}"),
        }
    }
    Ok(())
}

fn handle_http_chirho(
    mut stream_chirho: TcpStream,
    db_path_chirho: Option<PathBuf>,
) -> Result<(), String> {
    stream_chirho
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|err_chirho| err_chirho.to_string())?;
    let request_chirho = read_http_request_chirho(&mut stream_chirho)?;
    let mut conn_chirho = open_db_chirho(db_path_chirho)?;
    let response_chirho = route_http_chirho(&mut conn_chirho, request_chirho);
    let (status_chirho, body_chirho) = match response_chirho {
        Ok(body_chirho) => (200, body_chirho),
        Err(err_chirho) => (
            400,
            json!({ "ok_chirho": false, "error_chirho": err_chirho }),
        ),
    };
    write_http_response_chirho(&mut stream_chirho, status_chirho, &body_chirho.to_string())
}

struct HttpRequestChirho {
    method_chirho: String,
    path_chirho: String,
    query_chirho: BTreeMap<String, String>,
    body_chirho: Vec<u8>,
}

fn read_http_request_chirho(stream_chirho: &mut TcpStream) -> Result<HttpRequestChirho, String> {
    let mut buffer_chirho = Vec::new();
    let mut temp_chirho = [0u8; 1024];
    let header_end_chirho;
    loop {
        let read_chirho = stream_chirho
            .read(&mut temp_chirho)
            .map_err(|err_chirho| err_chirho.to_string())?;
        if read_chirho == 0 {
            return Err("connection closed before headers".to_string());
        }
        buffer_chirho.extend_from_slice(&temp_chirho[..read_chirho]);
        if let Some(pos_chirho) = find_header_end_chirho(&buffer_chirho) {
            header_end_chirho = pos_chirho;
            break;
        }
        if buffer_chirho.len() > 64 * 1024 {
            return Err("http headers too large".to_string());
        }
    }
    let header_text_chirho = String::from_utf8_lossy(&buffer_chirho[..header_end_chirho]);
    let mut lines_chirho = header_text_chirho.lines();
    let request_line_chirho = lines_chirho
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut request_parts_chirho = request_line_chirho.split_whitespace();
    let method_chirho = request_parts_chirho.next().unwrap_or("").to_string();
    let raw_path_chirho = request_parts_chirho.next().unwrap_or("").to_string();
    let mut content_length_chirho = 0usize;
    for line_chirho in lines_chirho {
        if let Some((name_chirho, value_chirho)) = line_chirho.split_once(':') {
            if name_chirho.eq_ignore_ascii_case("content-length") {
                content_length_chirho = value_chirho
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid content-length".to_string())?;
            }
        }
    }
    let body_start_chirho = header_end_chirho + 4;
    let mut body_chirho = buffer_chirho[body_start_chirho..].to_vec();
    while body_chirho.len() < content_length_chirho {
        let read_chirho = stream_chirho
            .read(&mut temp_chirho)
            .map_err(|err_chirho| err_chirho.to_string())?;
        if read_chirho == 0 {
            break;
        }
        body_chirho.extend_from_slice(&temp_chirho[..read_chirho]);
    }
    body_chirho.truncate(content_length_chirho);
    let (path_chirho, query_chirho) = split_path_query_chirho(&raw_path_chirho)?;
    Ok(HttpRequestChirho {
        method_chirho,
        path_chirho,
        query_chirho,
        body_chirho,
    })
}

fn find_header_end_chirho(buffer_chirho: &[u8]) -> Option<usize> {
    buffer_chirho
        .windows(4)
        .position(|window_chirho| window_chirho == b"\r\n\r\n")
}

fn split_path_query_chirho(
    raw_path_chirho: &str,
) -> Result<(String, BTreeMap<String, String>), String> {
    let (path_chirho, query_text_chirho) = raw_path_chirho
        .split_once('?')
        .unwrap_or((raw_path_chirho, ""));
    let mut query_chirho = BTreeMap::new();
    for pair_chirho in query_text_chirho
        .split('&')
        .filter(|value_chirho| !value_chirho.is_empty())
    {
        let (key_chirho, value_chirho) = pair_chirho.split_once('=').unwrap_or((pair_chirho, ""));
        query_chirho.insert(
            percent_decode_chirho(key_chirho)?,
            percent_decode_chirho(value_chirho)?,
        );
    }
    Ok((path_chirho.to_string(), query_chirho))
}

fn route_http_chirho(
    conn_chirho: &mut Connection,
    request_chirho: HttpRequestChirho,
) -> Result<Value, String> {
    match (
        request_chirho.method_chirho.as_str(),
        request_chirho.path_chirho.as_str(),
    ) {
        ("GET", "/health_chirho") => Ok(json!({ "ok_chirho": true })),
        ("POST", "/v1/register_chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            register_agent_chirho(conn_chirho, request_chirho)
        }
        ("POST", "/v1/post_chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            post_message_chirho(conn_chirho, request_chirho)
        }
        ("GET", "/v1/messages_chirho") => {
            let room_chirho = request_chirho
                .query_chirho
                .get("room_chirho")
                .ok_or_else(|| "room_chirho is required".to_string())?;
            let after_chirho = request_chirho
                .query_chirho
                .get("after_chirho")
                .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
                .unwrap_or(0);
            list_messages_chirho(conn_chirho, room_chirho, after_chirho)
        }
        ("GET", "/v1/agents_chirho") => {
            let room_chirho = request_chirho
                .query_chirho
                .get("room_chirho")
                .map(String::as_str);
            list_agents_chirho(conn_chirho, room_chirho)
        }
        ("POST", "/v1/refresh_chirho") => refresh_agents_chirho(conn_chirho),
        _ => Err("unknown route".to_string()),
    }
}

fn write_http_response_chirho(
    stream_chirho: &mut TcpStream,
    status_chirho: u16,
    body_chirho: &str,
) -> Result<(), String> {
    let label_chirho = if status_chirho == 200 {
        "OK"
    } else {
        "Bad Request"
    };
    let response_chirho = format!(
        "HTTP/1.1 {status_chirho} {label_chirho}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_chirho}",
        body_chirho.len()
    );
    stream_chirho
        .write_all(response_chirho.as_bytes())
        .map_err(|err_chirho| err_chirho.to_string())
}

fn http_client_chirho(
    server_chirho: &str,
    method_chirho: &str,
    path_chirho: &str,
    body_chirho: Option<&Value>,
) -> Result<Value, String> {
    let (host_chirho, port_chirho) = parse_server_chirho(server_chirho)?;
    let mut stream_chirho = TcpStream::connect((host_chirho.as_str(), port_chirho))
        .map_err(|err_chirho| err_chirho.to_string())?;
    let body_text_chirho = body_chirho.map(Value::to_string).unwrap_or_default();
    let request_chirho = format!(
        "{method_chirho} {path_chirho} HTTP/1.1\r\nHost: {host_chirho}:{port_chirho}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_text_chirho}",
        body_text_chirho.len()
    );
    stream_chirho
        .write_all(request_chirho.as_bytes())
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut response_chirho = String::new();
    stream_chirho
        .read_to_string(&mut response_chirho)
        .map_err(|err_chirho| err_chirho.to_string())?;
    let (_, body_chirho) = response_chirho
        .split_once("\r\n\r\n")
        .ok_or_else(|| "bad http response".to_string())?;
    serde_json::from_str(body_chirho).map_err(|err_chirho| err_chirho.to_string())
}

fn parse_server_chirho(server_chirho: &str) -> Result<(String, u16), String> {
    let stripped_chirho = server_chirho
        .strip_prefix("http://")
        .ok_or_else(|| "server must start with http://".to_string())?;
    let host_port_chirho = stripped_chirho.trim_end_matches('/');
    let (host_chirho, port_chirho) = host_port_chirho
        .rsplit_once(':')
        .ok_or_else(|| "server must include a port".to_string())?;
    Ok((
        host_chirho.to_string(),
        port_chirho
            .parse::<u16>()
            .map_err(|_| "invalid server port".to_string())?,
    ))
}

fn percent_encode_chirho(value_chirho: &str) -> String {
    let mut output_chirho = String::new();
    for byte_chirho in value_chirho.bytes() {
        if byte_chirho.is_ascii_alphanumeric() || matches!(byte_chirho, b'-' | b'_' | b'.' | b'~') {
            output_chirho.push(byte_chirho as char);
        } else {
            output_chirho.push_str(&format!("%{byte_chirho:02X}"));
        }
    }
    output_chirho
}

fn percent_decode_chirho(value_chirho: &str) -> Result<String, String> {
    let mut output_chirho = Vec::new();
    let bytes_chirho = value_chirho.as_bytes();
    let mut index_chirho = 0usize;
    while index_chirho < bytes_chirho.len() {
        match bytes_chirho[index_chirho] {
            b'+' => {
                output_chirho.push(b' ');
                index_chirho += 1;
            }
            b'%' if index_chirho + 2 < bytes_chirho.len() => {
                let hex_chirho =
                    std::str::from_utf8(&bytes_chirho[index_chirho + 1..index_chirho + 3])
                        .map_err(|err_chirho| err_chirho.to_string())?;
                let byte_chirho = u8::from_str_radix(hex_chirho, 16)
                    .map_err(|_| "bad percent escape".to_string())?;
                output_chirho.push(byte_chirho);
                index_chirho += 3;
            }
            byte_chirho => {
                output_chirho.push(byte_chirho);
                index_chirho += 1;
            }
        }
    }
    String::from_utf8(output_chirho).map_err(|err_chirho| err_chirho.to_string())
}

fn arg_value_chirho(args_chirho: &[String], name_chirho: &str) -> Option<String> {
    args_chirho
        .windows(2)
        .find(|window_chirho| window_chirho[0] == name_chirho)
        .map(|window_chirho| window_chirho[1].clone())
}

fn arg_values_chirho(args_chirho: &[String], name_chirho: &str) -> Vec<String> {
    let mut values_chirho = Vec::new();
    let mut index_chirho = 0usize;
    while index_chirho + 1 < args_chirho.len() {
        if args_chirho[index_chirho] == name_chirho {
            values_chirho.push(args_chirho[index_chirho + 1].clone());
            index_chirho += 2;
        } else {
            index_chirho += 1;
        }
    }
    values_chirho
}

fn require_arg_chirho(args_chirho: &[String], name_chirho: &str) -> Result<String, String> {
    arg_value_chirho(args_chirho, name_chirho).ok_or_else(|| format!("missing {name_chirho}"))
}

fn server_arg_chirho(args_chirho: &[String]) -> String {
    arg_value_chirho(args_chirho, "--server").unwrap_or_else(|| DEFAULT_SERVER_CHIRHO.to_string())
}

fn run_register_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let rooms_chirho = arg_values_chirho(args_chirho, "--room");
    let topics_chirho = arg_values_chirho(args_chirho, "--topic");
    let body_chirho = json!({
        "session_chirho": require_arg_chirho(args_chirho, "--session")?,
        "agent_chirho": require_arg_chirho(args_chirho, "--agent")?,
        "tmux_target_chirho": require_arg_chirho(args_chirho, "--tmux-target")?,
        "rooms_chirho": rooms_chirho,
        "topics_chirho": topics_chirho
    });
    println!(
        "{}",
        http_client_chirho(
            &server_chirho,
            "POST",
            "/v1/register_chirho",
            Some(&body_chirho)
        )?
    );
    Ok(())
}

fn run_post_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let body_text_chirho = if let Some(file_chirho) = arg_value_chirho(args_chirho, "--body-file") {
        fs::read_to_string(file_chirho).map_err(|err_chirho| err_chirho.to_string())?
    } else {
        require_arg_chirho(args_chirho, "--body")?
    };
    let body_chirho = json!({
        "from_session_chirho": require_arg_chirho(args_chirho, "--from-session")?,
        "from_agent_chirho": require_arg_chirho(args_chirho, "--from-agent")?,
        "room_chirho": require_arg_chirho(args_chirho, "--room")?,
        "topic_chirho": arg_value_chirho(args_chirho, "--topic").unwrap_or_else(default_topic_chirho),
        "body_chirho": body_text_chirho,
        "to_chirho": arg_values_chirho(args_chirho, "--to"),
        "deliver_to_sender_chirho": args_chirho.iter().any(|arg_chirho| arg_chirho == "--deliver-to-sender")
    });
    println!(
        "{}",
        http_client_chirho(
            &server_chirho,
            "POST",
            "/v1/post_chirho",
            Some(&body_chirho)
        )?
    );
    Ok(())
}

fn run_watch_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let room_chirho = require_arg_chirho(args_chirho, "--room")?;
    let mut after_chirho = arg_value_chirho(args_chirho, "--after")
        .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
        .unwrap_or(0);
    loop {
        let path_chirho = format!(
            "/v1/messages_chirho?room_chirho={}&after_chirho={after_chirho}",
            percent_encode_chirho(&room_chirho)
        );
        let response_chirho = http_client_chirho(&server_chirho, "GET", &path_chirho, None)?;
        if let Some(messages_chirho) = response_chirho
            .get("messages_chirho")
            .and_then(Value::as_array)
        {
            for message_chirho in messages_chirho {
                let id_chirho = message_chirho
                    .get("id_chirho")
                    .and_then(Value::as_i64)
                    .unwrap_or(after_chirho);
                after_chirho = after_chirho.max(id_chirho);
                println!(
                    "\n#{} {} [{}]\n{}\n",
                    id_chirho,
                    message_chirho
                        .get("from_identity_chirho")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown"),
                    message_chirho
                        .get("topic_chirho")
                        .and_then(Value::as_str)
                        .unwrap_or("general-chirho"),
                    message_chirho
                        .get("body_chirho")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                );
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn run_console_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let session_chirho = require_arg_chirho(args_chirho, "--session")?;
    let agent_chirho = require_arg_chirho(args_chirho, "--agent")?;
    let room_chirho = require_arg_chirho(args_chirho, "--room")?;
    let topic_chirho =
        arg_value_chirho(args_chirho, "--topic").unwrap_or_else(default_topic_chirho);
    let watch_args_chirho = vec![
        "watch".to_string(),
        "--server".to_string(),
        server_chirho.clone(),
        "--room".to_string(),
        room_chirho.clone(),
    ];
    thread::spawn(move || {
        let _ = run_watch_cli_chirho(&watch_args_chirho);
    });
    println!(
        "typing as {}/{} in room {room_chirho}; enter a line to post",
        session_chirho, agent_chirho
    );
    for line_chirho in io::stdin().lock().lines() {
        let line_chirho = line_chirho.map_err(|err_chirho| err_chirho.to_string())?;
        if line_chirho.trim().is_empty() {
            continue;
        }
        let post_args_chirho = vec![
            "post".to_string(),
            "--server".to_string(),
            server_chirho.clone(),
            "--from-session".to_string(),
            session_chirho.clone(),
            "--from-agent".to_string(),
            agent_chirho.clone(),
            "--room".to_string(),
            room_chirho.clone(),
            "--topic".to_string(),
            topic_chirho.clone(),
            "--body".to_string(),
            line_chirho,
        ];
        run_post_cli_chirho(&post_args_chirho)?;
    }
    Ok(())
}

fn run_agents_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let path_chirho = if let Some(room_chirho) = arg_value_chirho(args_chirho, "--room") {
        format!(
            "/v1/agents_chirho?room_chirho={}",
            percent_encode_chirho(&room_chirho)
        )
    } else {
        "/v1/agents_chirho".to_string()
    };
    println!(
        "{}",
        http_client_chirho(&server_chirho, "GET", &path_chirho, None)?
    );
    Ok(())
}

fn run_refresh_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    println!(
        "{}",
        http_client_chirho(
            &server_chirho,
            "POST",
            "/v1/refresh_chirho",
            Some(&json!({}))
        )?
    );
    Ok(())
}

fn usage_chirho() -> &'static str {
    "usage:
  metropoleluya-chirho server [--bind 127.0.0.1:37371] [--db path]
  metropoleluya-chirho register --session S --agent A --tmux-target T --room R [--topic T]
  metropoleluya-chirho post --from-session S --from-agent A --room R [--topic T] (--body TEXT|--body-file PATH) [--to SESSION/agent]
  metropoleluya-chirho watch --room R
  metropoleluya-chirho console --session S --agent A --room R [--topic T]
  metropoleluya-chirho agents [--room R]
  metropoleluya-chirho refresh"
}

fn main_chirho() -> Result<(), String> {
    let args_chirho: Vec<String> = env::args().skip(1).collect();
    let command_chirho = args_chirho.first().map(String::as_str).unwrap_or("");
    match command_chirho {
        "server" => {
            let bind_chirho = arg_value_chirho(&args_chirho, "--bind")
                .unwrap_or_else(|| "127.0.0.1:37371".to_string());
            let db_path_chirho = arg_value_chirho(&args_chirho, "--db").map(PathBuf::from);
            run_server_chirho(&bind_chirho, db_path_chirho)
        }
        "register" => run_register_cli_chirho(&args_chirho),
        "post" => run_post_cli_chirho(&args_chirho),
        "watch" => run_watch_cli_chirho(&args_chirho),
        "console" => run_console_cli_chirho(&args_chirho),
        "agents" => run_agents_cli_chirho(&args_chirho),
        "refresh" => run_refresh_cli_chirho(&args_chirho),
        _ => Err(usage_chirho().to_string()),
    }
}

fn main() {
    if let Err(err_chirho) = main_chirho() {
        eprintln!("{err_chirho}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn identity_uses_session_slash_agent_chirho() {
        assert_eq!(
            identity_chirho("CAIRN_CHIRHO", "gpt_chirho"),
            "CAIRN_CHIRHO/gpt_chirho"
        );
    }

    #[test]
    fn percent_encoding_round_trips_room_names_chirho() {
        let value_chirho = "cairn chirho/topic";
        let encoded_chirho = percent_encode_chirho(value_chirho);
        assert_eq!(
            percent_decode_chirho(&encoded_chirho).unwrap(),
            value_chirho
        );
    }

    #[test]
    fn in_memory_db_registers_agent_and_room_chirho() {
        let conn_chirho = Connection::open_in_memory().unwrap();
        init_db_chirho(&conn_chirho).unwrap();
        let response_chirho = register_agent_chirho(
            &conn_chirho,
            RegisterRequestChirho {
                session_chirho: "TEST_CHIRHO".to_string(),
                agent_chirho: "gpt_chirho".to_string(),
                tmux_target_chirho: "missing-session-chirho:1".to_string(),
                rooms_chirho: vec!["room-chirho".to_string()],
                topics_chirho: vec!["audit-chirho".to_string()],
            },
        )
        .unwrap();
        assert_eq!(response_chirho["identity_chirho"], "TEST_CHIRHO/gpt_chirho");
        let agents_chirho = list_agents_chirho(&conn_chirho, Some("room-chirho")).unwrap();
        assert_eq!(agents_chirho["agents_chirho"].as_array().unwrap().len(), 1);
    }
}
