// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use jiff::Timestamp;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod channels_chirho;
mod console_chirho;
mod tmux_transport_chirho;
mod tui_chirho;
mod tui_membership_chirho;

use tmux_transport_chirho::{probe_tmux_target_chirho, send_tmux_message_chirho};

#[cfg(test)]
mod channels_tests_chirho;
#[cfg(test)]
mod main_tests_chirho;

const DEFAULT_SERVER_CHIRHO: &str = "http://127.0.0.1:37371";

#[derive(Debug, Deserialize)]
struct RegisterRequestChirho {
    session_chirho: String,
    agent_chirho: String,
    tmux_target_chirho: String,
    #[serde(default)]
    rooms_chirho: Vec<String>,
    #[serde(default)]
    topics_chirho: Vec<String>,
    #[serde(default)]
    notify_actor_chirho: Option<String>,
    #[serde(default)]
    private_chirho: bool,
    #[serde(default)]
    ttl_seconds_chirho: Option<u64>,
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

#[derive(Debug, Deserialize)]
struct RemoveRequestChirho {
    from_session_chirho: String,
    from_agent_chirho: String,
    session_chirho: String,
    agent_chirho: String,
    room_chirho: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentTargetChirho {
    pub(crate) identity_chirho: String,
    pub(crate) tmux_target_chirho: String,
}

pub(crate) fn default_topic_chirho() -> String {
    "general-chirho".to_string()
}

pub(crate) fn now_ms_chirho() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as i64
}

/// America/New_York (US Eastern) IANA zone. Display timestamps are localized
/// here so operators read wall-clock ET; storage stays epoch UTC.
const EASTERN_TZ_CHIRHO: &str = "America/New_York";

/// Formats an epoch-millisecond instant as America/New_York wall-clock text,
/// e.g. "2026-07-17 15:25:02.661 EDT". DST-correct via jiff's tz database (the
/// zone abbreviation flips EST/EDT with the season). Storage stays epoch UTC in
/// `at_ms_chirho`; only display is localized.
pub(crate) fn format_timestamp_chirho(at_ms_chirho: i64) -> String {
    Timestamp::from_millisecond(at_ms_chirho)
        .and_then(|instant_chirho| instant_chirho.in_tz(EASTERN_TZ_CHIRHO))
        .map(|zoned_chirho| {
            zoned_chirho
                .strftime("%Y-%m-%d %H:%M:%S.%3f %Z")
                .to_string()
        })
        .unwrap_or_else(|_| format!("{at_ms_chirho}ms-epoch"))
}

fn default_db_path_chirho() -> PathBuf {
    let home_chirho = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home_chirho)
        .join(".metropoleluya-chirho")
        .join("metropoleluya-chirho.sqlite")
}

pub(crate) fn identity_chirho(session_chirho: &str, agent_chirho: &str) -> String {
    format!("{}/{}", session_chirho, agent_chirho)
}

pub(crate) fn validate_token_chirho(name_chirho: &str, value_chirho: &str) -> Result<(), String> {
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

pub(crate) fn validate_body_chirho(body_chirho: &str) -> Result<(), String> {
    if body_chirho.trim().is_empty() {
        return Err("body_chirho is required".to_string());
    }
    if body_chirho.chars().any(|char_chirho| char_chirho == '\0') {
        return Err("body_chirho must not contain NUL characters".to_string());
    }
    Ok(())
}

fn open_db_chirho(path_chirho: Option<PathBuf>) -> Result<Connection, String> {
    let path_chirho = path_chirho.unwrap_or_else(default_db_path_chirho);
    if let Some(parent_chirho) = path_chirho.parent() {
        fs::create_dir_all(parent_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    }
    let conn_chirho = Connection::open(path_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .busy_timeout(Duration::from_secs(5))
        .map_err(|err_chirho| err_chirho.to_string())?;
    init_db_chirho(&conn_chirho)?;
    Ok(conn_chirho)
}

fn open_request_db_chirho(path_chirho: Option<PathBuf>) -> Result<Connection, String> {
    let path_chirho = path_chirho.unwrap_or_else(default_db_path_chirho);
    let conn_chirho = Connection::open(path_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .busy_timeout(Duration::from_secs(5))
        .map_err(|err_chirho| err_chirho.to_string())?;
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
            -- at_text_chirho here is canonical UTC (Z-suffixed) for raw DB
            -- inspection; the API/TUI display path localizes to America/New_York
            -- via format_timestamp_chirho. These views stay UTC so they remain
            -- hermetic across machine time zones.
            create view if not exists messages_with_time_chirho as
                select
                    id_chirho,
                    at_ms_chirho,
                    datetime(at_ms_chirho / 1000, 'unixepoch')
                        || printf('.%03dZ', at_ms_chirho % 1000) as at_text_chirho,
                    from_identity_chirho,
                    room_chirho,
                    topic_chirho,
                    body_chirho
                from messages_chirho;
            create view if not exists deliveries_with_time_chirho as
                select
                    id_chirho,
                    message_id_chirho,
                    to_identity_chirho,
                    tmux_target_chirho,
                    alive_chirho,
                    delivered_chirho,
                    error_chirho,
                    at_ms_chirho,
                    datetime(at_ms_chirho / 1000, 'unixepoch')
                        || printf('.%03dZ', at_ms_chirho % 1000) as at_text_chirho
                from deliveries_chirho;
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    channels_chirho::init_channels_chirho(conn_chirho)
}

/// Registration also serves the TUI "add to room" flow
/// (spec-chirho/workflows-chirho/room-membership-admin-flow-chirho.md): when
/// notify_actor_chirho is set, the newcomer's pane gets a best-effort nudge.
fn register_agent_chirho(
    conn_chirho: &Connection,
    request_chirho: RegisterRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("session_chirho", &request_chirho.session_chirho)?;
    validate_token_chirho("agent_chirho", &request_chirho.agent_chirho)?;
    validate_token_chirho("tmux_target_chirho", &request_chirho.tmux_target_chirho)?;
    if let Some(actor_chirho) = request_chirho.notify_actor_chirho.as_deref() {
        validate_token_chirho("notify_actor_chirho", actor_chirho)?;
    }
    let identity_chirho =
        identity_chirho(&request_chirho.session_chirho, &request_chirho.agent_chirho);
    let probe_chirho = probe_tmux_target_chirho(&request_chirho.tmux_target_chirho);
    let now_chirho = now_ms_chirho();
    let topics_chirho = if request_chirho.topics_chirho.is_empty() {
        vec!["*".to_string()]
    } else {
        request_chirho.topics_chirho.clone()
    };
    for room_chirho in &request_chirho.rooms_chirho {
        validate_token_chirho("room_chirho", room_chirho)?;
    }
    for topic_chirho in &topics_chirho {
        validate_token_chirho("topic_chirho", topic_chirho)?;
    }

    // Membership policy, registration, and all requested subscriptions land
    // together. A denied room cannot leave behind a mutated agent or a
    // half-created private channel.
    let tx_chirho = conn_chirho
        .unchecked_transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    for room_chirho in &request_chirho.rooms_chirho {
        channels_chirho::prepare_room_membership_chirho(
            &tx_chirho,
            room_chirho,
            &identity_chirho,
            request_chirho.notify_actor_chirho.as_deref(),
            request_chirho.private_chirho,
            request_chirho.ttl_seconds_chirho,
        )?;
    }
    tx_chirho
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
    for room_chirho in &request_chirho.rooms_chirho {
        for topic_chirho in &topics_chirho {
            tx_chirho
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
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;

    let notified_chirho = match request_chirho.notify_actor_chirho.as_deref() {
        Some(actor_chirho) if !request_chirho.rooms_chirho.is_empty() => {
            let notice_chirho = format!(
                "metropoleluya: you were added to room {} by {}.",
                request_chirho.rooms_chirho.join(", "),
                actor_chirho
            );
            probe_chirho.alive_chirho
                && send_tmux_message_chirho(&request_chirho.tmux_target_chirho, &notice_chirho)
                    .is_ok()
        }
        _ => false,
    };

    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": identity_chirho,
        "alive_chirho": probe_chirho.alive_chirho,
        "observed_session_chirho": probe_chirho.session_chirho,
        "window_index_chirho": probe_chirho.window_index_chirho,
        "pane_id_chirho": probe_chirho.pane_id_chirho,
        "notified_chirho": notified_chirho,
        "error_chirho": probe_chirho.error_chirho
    }))
}

/// Part of the room-membership admin flow
/// (spec-chirho/workflows-chirho/room-membership-admin-flow-chirho.md): notify
/// the target pane first (best effort, while it is still subscribed), then
/// delete only this room's subscription rows. The agents_chirho row and any
/// other room subscriptions stay intact, so the add flow can reverse this.
fn remove_agent_chirho(
    conn_chirho: &Connection,
    request_chirho: RemoveRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("from_session_chirho", &request_chirho.from_session_chirho)?;
    validate_token_chirho("from_agent_chirho", &request_chirho.from_agent_chirho)?;
    validate_token_chirho("session_chirho", &request_chirho.session_chirho)?;
    validate_token_chirho("agent_chirho", &request_chirho.agent_chirho)?;
    validate_token_chirho("room_chirho", &request_chirho.room_chirho)?;
    let from_identity_chirho = identity_chirho(
        &request_chirho.from_session_chirho,
        &request_chirho.from_agent_chirho,
    );
    let identity_chirho =
        identity_chirho(&request_chirho.session_chirho, &request_chirho.agent_chirho);
    channels_chirho::authorize_membership_change_chirho(
        conn_chirho,
        &request_chirho.room_chirho,
        &from_identity_chirho,
    )?;
    let tmux_target_chirho: Option<String> = conn_chirho
        .query_row(
            "select tmux_target_chirho from agents_chirho where identity_chirho = ?1",
            params![identity_chirho],
            |row_chirho| row_chirho.get(0),
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let notice_chirho = format!(
        "metropoleluya: you were removed from room {} by {}.",
        request_chirho.room_chirho, from_identity_chirho
    );
    let notified_chirho = tmux_target_chirho.as_deref().is_some_and(|target_chirho| {
        probe_tmux_target_chirho(target_chirho).alive_chirho
            && send_tmux_message_chirho(target_chirho, &notice_chirho).is_ok()
    });
    let removed_count_chirho = conn_chirho
        .execute(
            "delete from subscriptions_chirho where identity_chirho = ?1 and room_chirho = ?2",
            params![identity_chirho, request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": identity_chirho,
        "removed_count_chirho": removed_count_chirho,
        "notified_chirho": notified_chirho
    }))
}

/// Outcome of one delivery attempt, produced off-thread (tmux I/O only) so the
/// caller can persist results and prune afterward with the DB handle.
pub(crate) struct DeliveryOutcomeChirho {
    pub(crate) alive_chirho: bool,
    pub(crate) delivered_chirho: bool,
    pub(crate) error_chirho: Option<String>,
}

fn deliver_one_chirho(tmux_target_chirho: &str, text_chirho: &str) -> DeliveryOutcomeChirho {
    let probe_chirho = probe_tmux_target_chirho(tmux_target_chirho);
    if !probe_chirho.alive_chirho {
        return DeliveryOutcomeChirho {
            alive_chirho: false,
            delivered_chirho: false,
            error_chirho: Some(
                probe_chirho
                    .error_chirho
                    .unwrap_or_else(|| "tmux target is not alive".to_string()),
            ),
        };
    }
    match send_tmux_message_chirho(tmux_target_chirho, text_chirho) {
        Ok(()) => DeliveryOutcomeChirho {
            alive_chirho: true,
            delivered_chirho: true,
            error_chirho: None,
        },
        Err(err_chirho) => DeliveryOutcomeChirho {
            alive_chirho: true,
            delivered_chirho: false,
            error_chirho: Some(err_chirho),
        },
    }
}

/// Delivers to every target with bounded concurrency, so a post's wall-clock is
/// roughly one delivery's settle delay regardless of subscriber count (instead
/// of the sum). tmux I/O only — no DB handle crosses a thread boundary.
pub(crate) fn deliver_parallel_chirho(
    targets_chirho: &[AgentTargetChirho],
    text_chirho: &str,
) -> Vec<DeliveryOutcomeChirho> {
    const MAX_DELIVERY_CONCURRENCY_CHIRHO: usize = 16;
    let mut outcomes_chirho = Vec::with_capacity(targets_chirho.len());
    for chunk_chirho in targets_chirho.chunks(MAX_DELIVERY_CONCURRENCY_CHIRHO) {
        let chunk_outcomes_chirho: Vec<DeliveryOutcomeChirho> = thread::scope(|scope_chirho| {
            chunk_chirho
                .iter()
                .map(|target_chirho| {
                    scope_chirho.spawn(|| {
                        deliver_one_chirho(&target_chirho.tmux_target_chirho, text_chirho)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle_chirho| {
                    handle_chirho.join().unwrap_or(DeliveryOutcomeChirho {
                        alive_chirho: false,
                        delivered_chirho: false,
                        error_chirho: Some("delivery thread panicked".to_string()),
                    })
                })
                .collect()
        });
        outcomes_chirho.extend(chunk_outcomes_chirho);
    }
    outcomes_chirho
}

/// Persists and delivers a room post after the private-channel authorization
/// branch in `spec-chirho/workflows-chirho/private-channels-flow-chirho.md`.
fn post_message_chirho(
    conn_chirho: &mut Connection,
    request_chirho: PostRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("from_session_chirho", &request_chirho.from_session_chirho)?;
    validate_token_chirho("from_agent_chirho", &request_chirho.from_agent_chirho)?;
    validate_token_chirho("room_chirho", &request_chirho.room_chirho)?;
    validate_token_chirho("topic_chirho", &request_chirho.topic_chirho)?;
    validate_body_chirho(&request_chirho.body_chirho)?;
    let from_identity_chirho = identity_chirho(
        &request_chirho.from_session_chirho,
        &request_chirho.from_agent_chirho,
    );
    let tx_chirho = conn_chirho
        .transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let private_room_chirho = channels_chirho::authorize_room_post_chirho(
        &tx_chirho,
        &request_chirho.room_chirho,
        &from_identity_chirho,
    )?;
    let targets_chirho = resolve_targets_chirho(
        &tx_chirho,
        &request_chirho,
        &from_identity_chirho,
        private_room_chirho,
    )?;
    let message_at_ms_chirho = now_ms_chirho();
    tx_chirho
        .execute(
            r#"
            insert into messages_chirho (at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho)
            values (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                message_at_ms_chirho,
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

    let text_chirho = format_delivery_chirho(
        message_id_chirho,
        message_at_ms_chirho,
        &from_identity_chirho,
        &request_chirho.room_chirho,
        &request_chirho.topic_chirho,
        &request_chirho.body_chirho,
    );
    let outcomes_chirho = deliver_parallel_chirho(&targets_chirho, &text_chirho);
    let recorded_at_ms_chirho = now_ms_chirho();
    let mut delivered_count_chirho = 0usize;
    for (target_chirho, outcome_chirho) in targets_chirho.iter().zip(outcomes_chirho.iter()) {
        if outcome_chirho.delivered_chirho {
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
                    if outcome_chirho.alive_chirho { 1 } else { 0 },
                    if outcome_chirho.delivered_chirho {
                        1
                    } else {
                        0
                    },
                    outcome_chirho.error_chirho.clone(),
                    recorded_at_ms_chirho
                ],
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        // Prune stale routing: a dead target's subscription in this room is
        // deactivated so later posts skip it (self-heals — the agent's next
        // register reactivates it, and humans watch by polling, not delivery).
        if !outcome_chirho.alive_chirho {
            conn_chirho
                .execute(
                    "update subscriptions_chirho set active_chirho = 0 \
                     where identity_chirho = ?1 and room_chirho = ?2",
                    params![target_chirho.identity_chirho, request_chirho.room_chirho],
                )
                .map_err(|err_chirho| err_chirho.to_string())?;
            conn_chirho
                .execute(
                    "update agents_chirho set alive_chirho = 0 where identity_chirho = ?1",
                    params![target_chirho.identity_chirho],
                )
                .map_err(|err_chirho| err_chirho.to_string())?;
        }
    }

    Ok(json!({
        "ok_chirho": true,
        "message_id_chirho": message_id_chirho,
        "at_ms_chirho": message_at_ms_chirho,
        "at_text_chirho": format_timestamp_chirho(message_at_ms_chirho),
        "delivery_count_chirho": delivered_count_chirho
    }))
}

fn resolve_targets_chirho(
    conn_chirho: &Connection,
    request_chirho: &PostRequestChirho,
    from_identity_chirho: &str,
    private_room_chirho: bool,
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
            channels_chirho::authorize_private_target_chirho(
                conn_chirho,
                &request_chirho.room_chirho,
                identity_chirho,
                private_room_chirho,
            )?;
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

fn list_agents_chirho(
    conn_chirho: &Connection,
    room_chirho: Option<&str>,
) -> Result<Value, String> {
    let sql_chirho = if room_chirho.is_some() {
        r#"
        select a.identity_chirho, a.session_chirho, a.agent_chirho, a.tmux_target_chirho,
               a.window_index_chirho, a.pane_id_chirho, a.alive_chirho, a.last_seen_ms_chirho,
               coalesce(group_concat(distinct s.topic_chirho), '') as topics_chirho
        from agents_chirho a
        join subscriptions_chirho s on s.identity_chirho = a.identity_chirho
        where s.room_chirho = ?1 and s.active_chirho = 1
        group by a.identity_chirho, a.session_chirho, a.agent_chirho, a.tmux_target_chirho,
                 a.window_index_chirho, a.pane_id_chirho, a.alive_chirho, a.last_seen_ms_chirho
        order by a.identity_chirho
        "#
    } else {
        r#"
        select a.identity_chirho, a.session_chirho, a.agent_chirho, a.tmux_target_chirho,
               a.window_index_chirho, a.pane_id_chirho, a.alive_chirho, a.last_seen_ms_chirho,
               coalesce(group_concat(distinct s.room_chirho || ':' || s.topic_chirho), '') as topics_chirho
        from agents_chirho a
        left join subscriptions_chirho s
          on s.identity_chirho = a.identity_chirho
         and s.active_chirho = 1
         and s.room_chirho not in (
             select room_chirho from rooms_chirho where private_chirho = 1
         )
        group by a.identity_chirho, a.session_chirho, a.agent_chirho, a.tmux_target_chirho,
                 a.window_index_chirho, a.pane_id_chirho, a.alive_chirho, a.last_seen_ms_chirho
        order by a.identity_chirho
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
    let identity_chirho = row_chirho.get::<_, String>(0)?;
    let session_chirho = row_chirho.get::<_, String>(1)?;
    let agent_chirho = row_chirho.get::<_, String>(2)?;
    let tmux_target_chirho = row_chirho.get::<_, String>(3)?;
    let window_index_chirho = row_chirho.get::<_, Option<String>>(4)?;
    let pane_id_chirho = row_chirho.get::<_, Option<String>>(5)?;
    let last_seen_ms_chirho = row_chirho.get::<_, i64>(7)?;
    let topics_text_chirho = row_chirho.get::<_, String>(8)?;
    let topics_chirho: Vec<String> = topics_text_chirho
        .split(',')
        .filter(|topic_chirho| !topic_chirho.is_empty())
        .map(|topic_chirho| topic_chirho.to_string())
        .collect();
    Ok(json!({
        "identity_chirho": identity_chirho,
        "session_chirho": session_chirho,
        "agent_chirho": agent_chirho,
        "tmux_target_chirho": tmux_target_chirho,
        "window_index_chirho": window_index_chirho,
        "pane_id_chirho": pane_id_chirho,
        "alive_chirho": alive_chirho == 1,
        "last_seen_ms_chirho": last_seen_ms_chirho,
        "topics_chirho": topics_chirho,
        "delivery_mode_chirho": "tmux-sendkeys-chirho"
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

fn format_delivery_chirho(
    message_id_chirho: i64,
    at_ms_chirho: i64,
    from_identity_chirho: &str,
    room_chirho: &str,
    topic_chirho: &str,
    body_chirho: &str,
) -> String {
    let at_text_chirho = format_timestamp_chirho(at_ms_chirho);
    format!(
        "METROPOLELUYA_CHIRHO MESSAGE #{message_id_chirho} AT {at_text_chirho} FROM {from_identity_chirho} ROOM {room_chirho} TOPIC {topic_chirho}\n{body_chirho}"
    )
}

/// What the supervisor does after the broker process exits: wait then restart,
/// or give up. The give-up cap is the guard against a "restart bomb" — a broker
/// that dies instantly on every start is never relaunched forever.
#[derive(Debug, PartialEq)]
enum SupervisorActionChirho {
    RestartAfterChirho(Duration),
    GiveUpChirho,
}

/// Decides the next supervisor step from how long the broker ran. A run of at
/// least MIN_HEALTHY_RUN_CHIRHO resets the rapid-crash counter (it was healthy,
/// just exited); shorter runs are rapid crashes that grow the backoff. After
/// MAX_RAPID_CRASHES_CHIRHO in a row we give up instead of spinning the CPU.
fn next_supervisor_action_chirho(
    ran_chirho: Duration,
    rapid_crashes_chirho: &mut u32,
) -> SupervisorActionChirho {
    const MIN_HEALTHY_RUN_CHIRHO: Duration = Duration::from_secs(10);
    const MAX_RAPID_CRASHES_CHIRHO: u32 = 5;
    const BACKOFF_CAP_CHIRHO: Duration = Duration::from_secs(30);
    if ran_chirho >= MIN_HEALTHY_RUN_CHIRHO {
        *rapid_crashes_chirho = 0;
    } else {
        *rapid_crashes_chirho = rapid_crashes_chirho.saturating_add(1);
    }
    if *rapid_crashes_chirho >= MAX_RAPID_CRASHES_CHIRHO {
        return SupervisorActionChirho::GiveUpChirho;
    }
    let backoff_secs_chirho = 1u64 << (*rapid_crashes_chirho).min(5);
    SupervisorActionChirho::RestartAfterChirho(
        Duration::from_secs(backoff_secs_chirho).min(BACKOFF_CAP_CHIRHO),
    )
}

/// Runs the broker under supervision: (re)spawns `server` as a child and, on
/// exit, backs off and restarts — with the crash-loop cap above so a broker
/// that cannot start never becomes a CPU-pegging restart bomb. Restart a
/// supervised broker with `tmux kill-session` (SIGHUP reaches the child too),
/// not a hard SIGKILL of the supervisor alone.
fn run_supervisor_chirho(bind_chirho: &str, db_path_chirho: Option<PathBuf>) -> Result<(), String> {
    let exe_chirho = env::current_exe().map_err(|err_chirho| err_chirho.to_string())?;
    let mut rapid_crashes_chirho = 0u32;
    println!("metropoleluya-chirho supervising broker on {bind_chirho}");
    loop {
        let started_chirho = Instant::now();
        let mut command_chirho = Command::new(&exe_chirho);
        command_chirho.arg("server").arg("--bind").arg(bind_chirho);
        if let Some(path_chirho) = &db_path_chirho {
            command_chirho.arg("--db").arg(path_chirho);
        }
        match command_chirho.status() {
            Ok(status_chirho) => eprintln!(
                "broker exited ({status_chirho}) after {:?}",
                started_chirho.elapsed()
            ),
            Err(err_chirho) => eprintln!("broker failed to start: {err_chirho}"),
        }
        match next_supervisor_action_chirho(started_chirho.elapsed(), &mut rapid_crashes_chirho) {
            SupervisorActionChirho::GiveUpChirho => {
                return Err(format!(
                    "broker crash-looped {rapid_crashes_chirho}x; supervisor stopping to avoid a restart bomb"
                ));
            }
            SupervisorActionChirho::RestartAfterChirho(backoff_chirho) => {
                eprintln!("restarting broker in {backoff_chirho:?}");
                thread::sleep(backoff_chirho);
            }
        }
    }
}

/// Builds the broker's listening socket with SO_REUSEADDR set, so a freshly
/// launched broker can rebind (e.g. 127.0.0.1:37371) immediately over sockets
/// the previous process left in TIME_WAIT — clean restarts instead of a
/// ~minute "address already in use" stall while the fleet has no broker.
fn bind_listener_chirho(bind_chirho: &str) -> Result<TcpListener, String> {
    let addr_chirho: SocketAddr = bind_chirho
        .parse()
        .map_err(|err_chirho: std::net::AddrParseError| err_chirho.to_string())?;
    let socket_chirho = Socket::new(
        Domain::for_address(addr_chirho),
        Type::STREAM,
        Some(Protocol::TCP),
    )
    .map_err(|err_chirho| err_chirho.to_string())?;
    socket_chirho
        .set_reuse_address(true)
        .map_err(|err_chirho| err_chirho.to_string())?;
    socket_chirho
        .bind(&addr_chirho.into())
        .map_err(|err_chirho| err_chirho.to_string())?;
    socket_chirho
        .listen(128)
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(socket_chirho.into())
}

fn run_server_chirho(bind_chirho: &str, db_path_chirho: Option<PathBuf>) -> Result<(), String> {
    // Schema initialization and legacy backfill are startup work, not request
    // work. Per-request connections below only set a bounded busy timeout.
    drop(open_db_chirho(db_path_chirho.clone())?);
    let listener_chirho = bind_listener_chirho(bind_chirho)?;
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
    let mut conn_chirho = open_request_db_chirho(db_path_chirho)?;
    let response_chirho = route_http_chirho(&mut conn_chirho, request_chirho);
    let (status_chirho, body_chirho) = match response_chirho {
        Ok(body_chirho) => (200, body_chirho),
        Err(err_chirho) => {
            let status_chirho = channels_chirho::http_error_status_chirho(&err_chirho);
            (
                status_chirho,
                json!({ "ok_chirho": false, "error_chirho": err_chirho }),
            )
        }
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

/// HTTP dispatch for both public-room compatibility and the authenticated-by-
/// identity branches documented in the private-channels workflow.
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
        ("POST", "/v1/dm-chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            channels_chirho::post_dm_chirho(conn_chirho, request_chirho)
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
            let viewer_identity_chirho = request_chirho
                .query_chirho
                .get("as_chirho")
                .map(String::as_str);
            channels_chirho::list_room_messages_chirho(
                conn_chirho,
                room_chirho,
                after_chirho,
                viewer_identity_chirho,
            )
        }
        ("GET", "/v1/agents_chirho") => {
            let room_chirho = request_chirho
                .query_chirho
                .get("room_chirho")
                .map(String::as_str);
            if let Some(room_chirho) = room_chirho {
                channels_chirho::authorize_room_read_chirho(
                    conn_chirho,
                    room_chirho,
                    request_chirho
                        .query_chirho
                        .get("as_chirho")
                        .map(String::as_str),
                )?;
            }
            list_agents_chirho(conn_chirho, room_chirho)
        }
        ("GET", "/v1/dms-chirho") => {
            let identity_chirho = request_chirho
                .query_chirho
                .get("identity_chirho")
                .ok_or_else(|| "identity_chirho is required".to_string())?;
            let with_chirho = request_chirho
                .query_chirho
                .get("with_chirho")
                .ok_or_else(|| "with_chirho is required".to_string())?;
            let after_chirho = request_chirho
                .query_chirho
                .get("after_chirho")
                .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
                .unwrap_or(0);
            channels_chirho::list_dm_messages_chirho(
                conn_chirho,
                identity_chirho,
                with_chirho,
                after_chirho,
            )
        }
        ("GET", "/v1/channels-chirho") => {
            let identity_chirho = request_chirho
                .query_chirho
                .get("identity_chirho")
                .ok_or_else(|| "identity_chirho is required".to_string())?;
            let include_closed_chirho = request_chirho
                .query_chirho
                .get("include_closed_chirho")
                .is_some_and(|value_chirho| value_chirho == "true");
            channels_chirho::list_channels_chirho(
                conn_chirho,
                identity_chirho,
                include_closed_chirho,
            )
        }
        ("POST", "/v1/rooms-chirho/close-chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            channels_chirho::close_room_chirho(conn_chirho, request_chirho)
        }
        ("POST", "/v1/dms-chirho/close-chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            channels_chirho::close_dm_chirho(conn_chirho, request_chirho)
        }
        ("POST", "/v1/rooms-chirho/purge-chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            channels_chirho::purge_room_chirho(conn_chirho, request_chirho)
        }
        ("POST", "/v1/dms-chirho/purge-chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            channels_chirho::purge_dm_chirho(conn_chirho, request_chirho)
        }
        ("POST", "/v1/remove_chirho") => {
            let request_chirho = serde_json::from_slice(&request_chirho.body_chirho)
                .map_err(|err_chirho| err_chirho.to_string())?;
            remove_agent_chirho(conn_chirho, request_chirho)
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
    let label_chirho = match status_chirho {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let response_chirho = format!(
        "HTTP/1.1 {status_chirho} {label_chirho}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_chirho}",
        body_chirho.len()
    );
    stream_chirho
        .write_all(response_chirho.as_bytes())
        .map_err(|err_chirho| err_chirho.to_string())
}

pub(crate) fn http_client_chirho(
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
    let value_chirho: Value =
        serde_json::from_str(body_chirho).map_err(|err_chirho| err_chirho.to_string())?;
    if value_chirho.get("ok_chirho").and_then(Value::as_bool) == Some(false) {
        return Err(value_chirho
            .get("error_chirho")
            .and_then(Value::as_str)
            .unwrap_or("broker request failed_chirho")
            .to_string());
    }
    Ok(value_chirho)
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

pub(crate) fn percent_encode_chirho(value_chirho: &str) -> String {
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

pub(crate) fn arg_value_chirho(args_chirho: &[String], name_chirho: &str) -> Option<String> {
    args_chirho
        .windows(2)
        .find(|window_chirho| window_chirho[0] == name_chirho)
        .map(|window_chirho| window_chirho[1].clone())
}

pub(crate) fn arg_values_chirho(args_chirho: &[String], name_chirho: &str) -> Vec<String> {
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

pub(crate) fn require_arg_chirho(
    args_chirho: &[String],
    name_chirho: &str,
) -> Result<String, String> {
    arg_value_chirho(args_chirho, name_chirho).ok_or_else(|| format!("missing {name_chirho}"))
}

pub(crate) fn server_arg_chirho(args_chirho: &[String]) -> String {
    arg_value_chirho(args_chirho, "--server").unwrap_or_else(|| DEFAULT_SERVER_CHIRHO.to_string())
}

fn run_remove_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let body_chirho = json!({
        "from_session_chirho": require_arg_chirho(args_chirho, "--from-session")?,
        "from_agent_chirho": require_arg_chirho(args_chirho, "--from-agent")?,
        "session_chirho": require_arg_chirho(args_chirho, "--session")?,
        "agent_chirho": require_arg_chirho(args_chirho, "--agent")?,
        "room_chirho": require_arg_chirho(args_chirho, "--room")?
    });
    println!(
        "{}",
        http_client_chirho(
            &server_chirho,
            "POST",
            "/v1/remove_chirho",
            Some(&body_chirho)
        )?
    );
    Ok(())
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
        "supervise" => {
            let bind_chirho = arg_value_chirho(&args_chirho, "--bind")
                .unwrap_or_else(|| "127.0.0.1:37371".to_string());
            let db_path_chirho = arg_value_chirho(&args_chirho, "--db").map(PathBuf::from);
            run_supervisor_chirho(&bind_chirho, db_path_chirho)
        }
        "register" => console_chirho::run_register_cli_chirho(&args_chirho),
        "remove" => run_remove_cli_chirho(&args_chirho),
        "post" => console_chirho::run_post_cli_chirho(&args_chirho),
        "watch" => console_chirho::run_watch_cli_chirho(&args_chirho),
        "console" => console_chirho::run_console_cli_chirho(&args_chirho),
        "tui" => tui_chirho::run_tui_cli_chirho(&args_chirho),
        "agents" => console_chirho::run_agents_cli_chirho(&args_chirho),
        "rooms" => channels_chirho::run_rooms_cli_chirho(&args_chirho),
        "dm" => channels_chirho::run_dm_cli_chirho(&args_chirho),
        "refresh" => console_chirho::run_refresh_cli_chirho(&args_chirho),
        _ => Err(console_chirho::usage_chirho().to_string()),
    }
}

fn main() {
    if let Err(err_chirho) = main_chirho() {
        eprintln!("{err_chirho}");
        std::process::exit(1);
    }
}
