// For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV)

//! Private-room and direct-message protocol.
//! Workflow: spec-chirho/workflows-chirho/private-channels-flow-chirho.md.

use crate::{
    arg_value_chirho, deliver_parallel_chirho, format_timestamp_chirho, http_client_chirho,
    identity_chirho, now_ms_chirho, percent_encode_chirho, require_arg_chirho, server_arg_chirho,
    validate_body_chirho, validate_token_chirho, AgentTargetChirho,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_TTL_SECONDS_CHIRHO: u64 = 31_536_000;
const PAIR_SEPARATOR_CHIRHO: char = '\u{1f}';

pub(crate) fn http_error_status_chirho(error_chirho: &str) -> u16 {
    if error_chirho.starts_with("authorization denied") {
        403
    } else if error_chirho == "unknown route" {
        404
    } else {
        400
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DmPostRequestChirho {
    pub(crate) from_session_chirho: String,
    pub(crate) from_agent_chirho: String,
    pub(crate) to_chirho: String,
    pub(crate) body_chirho: String,
    #[serde(default)]
    pub(crate) ttl_seconds_chirho: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CloseRoomRequestChirho {
    pub(crate) actor_identity_chirho: String,
    pub(crate) room_chirho: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CloseDmRequestChirho {
    pub(crate) identity_chirho: String,
    pub(crate) with_chirho: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PurgeRoomRequestChirho {
    pub(crate) actor_identity_chirho: String,
    pub(crate) room_chirho: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PurgeDmRequestChirho {
    pub(crate) identity_chirho: String,
    pub(crate) with_chirho: String,
}

#[derive(Debug)]
struct RoomMetadataChirho {
    private_chirho: bool,
    closed_ms_chirho: Option<i64>,
}

/// Creates additive channel tables and backfills every legacy room as public.
/// Existing room behavior therefore remains unchanged after migration.
pub(crate) fn init_channels_chirho(conn_chirho: &Connection) -> Result<(), String> {
    conn_chirho
        .execute_batch(
            r#"
            create table if not exists rooms_chirho (
                room_chirho text primary key,
                private_chirho integer not null default 0,
                created_by_chirho text not null,
                created_ms_chirho integer not null,
                expires_ms_chirho integer,
                closed_ms_chirho integer
            );
            create table if not exists dm_channels_chirho (
                pair_key_chirho text primary key,
                identity_a_chirho text not null,
                identity_b_chirho text not null,
                created_ms_chirho integer not null,
                last_message_ms_chirho integer,
                expires_ms_chirho integer,
                closed_ms_chirho integer,
                unique(identity_a_chirho, identity_b_chirho)
            );
            create table if not exists direct_messages_chirho (
                id_chirho integer primary key autoincrement,
                pair_key_chirho text not null,
                at_ms_chirho integer not null,
                from_identity_chirho text not null,
                to_identity_chirho text not null,
                body_chirho text not null
            );
            create table if not exists direct_deliveries_chirho (
                id_chirho integer primary key autoincrement,
                direct_message_id_chirho integer not null,
                to_identity_chirho text not null,
                tmux_target_chirho text not null,
                alive_chirho integer not null,
                delivered_chirho integer not null,
                error_chirho text,
                at_ms_chirho integer not null
            );
            create index if not exists direct_messages_pair_id_chirho
                on direct_messages_chirho(pair_key_chirho, id_chirho);
            create index if not exists dm_channels_a_closed_chirho
                on dm_channels_chirho(identity_a_chirho, closed_ms_chirho);
            create index if not exists dm_channels_b_closed_chirho
                on dm_channels_chirho(identity_b_chirho, closed_ms_chirho);
            create index if not exists rooms_expiry_chirho
                on rooms_chirho(expires_ms_chirho)
                where closed_ms_chirho is null and expires_ms_chirho is not null;
            create index if not exists dm_channels_expiry_chirho
                on dm_channels_chirho(expires_ms_chirho)
                where closed_ms_chirho is null and expires_ms_chirho is not null;
            create index if not exists subscriptions_active_room_chirho
                on subscriptions_chirho(active_chirho, room_chirho, identity_chirho);
            create view if not exists direct_messages_with_time_chirho as
                select
                    id_chirho,
                    pair_key_chirho,
                    at_ms_chirho,
                    datetime(at_ms_chirho / 1000, 'unixepoch')
                        || printf('.%03dZ', at_ms_chirho % 1000) as at_text_chirho,
                    from_identity_chirho,
                    to_identity_chirho,
                    body_chirho
                from direct_messages_chirho;
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .execute(
            r#"
            insert or ignore into rooms_chirho (
                room_chirho, private_chirho, created_by_chirho, created_ms_chirho
            )
            select distinct room_chirho, 0, 'legacy-public-chirho', 0
            from subscriptions_chirho
            "#,
            [],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .execute(
            r#"
            insert or ignore into rooms_chirho (
                room_chirho, private_chirho, created_by_chirho, created_ms_chirho
            )
            select distinct room_chirho, 0, 'legacy-public-chirho', 0
            from messages_chirho
            "#,
            [],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    Ok(())
}

fn ttl_to_expires_ms_chirho(
    now_chirho: i64,
    ttl_seconds_chirho: Option<u64>,
) -> Result<Option<i64>, String> {
    let Some(ttl_seconds_chirho) = ttl_seconds_chirho else {
        return Ok(None);
    };
    if ttl_seconds_chirho == 0 || ttl_seconds_chirho > MAX_TTL_SECONDS_CHIRHO {
        return Err(format!(
            "ttl_seconds_chirho must be between 1 and {MAX_TTL_SECONDS_CHIRHO}"
        ));
    }
    let ttl_ms_chirho = i64::try_from(ttl_seconds_chirho)
        .map_err(|_| "ttl_seconds_chirho is too large".to_string())?
        .checked_mul(1_000)
        .ok_or_else(|| "ttl_seconds_chirho overflow".to_string())?;
    now_chirho
        .checked_add(ttl_ms_chirho)
        .map(Some)
        .ok_or_else(|| "expires_ms_chirho overflow".to_string())
}

/// Moves expired open channels to cold/closed state. History remains readable
/// by the former participants and is never silently deleted.
pub(crate) fn retire_expired_channels_chirho(
    conn_chirho: &Connection,
    now_chirho: i64,
) -> Result<(), String> {
    let retirement_needed_chirho = conn_chirho
        .query_row(
            r#"
            select
                exists(
                    select 1 from rooms_chirho
                    where closed_ms_chirho is null
                      and expires_ms_chirho is not null
                      and expires_ms_chirho <= ?1
                    limit 1
                )
                or exists(
                    select 1
                    from subscriptions_chirho s
                    join rooms_chirho r on r.room_chirho = s.room_chirho
                    where s.active_chirho = 1 and r.closed_ms_chirho is not null
                    limit 1
                )
                or exists(
                    select 1 from dm_channels_chirho
                    where closed_ms_chirho is null
                      and expires_ms_chirho is not null
                      and expires_ms_chirho <= ?1
                    limit 1
                )
            "#,
            params![now_chirho],
            |row_chirho| row_chirho.get::<_, i64>(0),
        )
        .map(|value_chirho| value_chirho == 1)
        .map_err(|err_chirho| err_chirho.to_string())?;
    if !retirement_needed_chirho {
        return Ok(());
    }
    conn_chirho
        .execute(
            r#"
            update rooms_chirho
            set closed_ms_chirho = ?1
            where closed_ms_chirho is null
              and expires_ms_chirho is not null
              and expires_ms_chirho <= ?1
            "#,
            params![now_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .execute(
            r#"
            update subscriptions_chirho
            set active_chirho = 0
            where active_chirho = 1
              and exists(
                  select 1 from rooms_chirho r
                  where r.room_chirho = subscriptions_chirho.room_chirho
                    and r.closed_ms_chirho is not null
              )
            "#,
            [],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    conn_chirho
        .execute(
            r#"
            update dm_channels_chirho
            set closed_ms_chirho = ?1
            where closed_ms_chirho is null
              and expires_ms_chirho is not null
              and expires_ms_chirho <= ?1
            "#,
            params![now_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(())
}

fn room_metadata_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
) -> Result<Option<RoomMetadataChirho>, String> {
    conn_chirho
        .query_row(
            r#"
            select private_chirho, closed_ms_chirho
            from rooms_chirho where room_chirho = ?1
            "#,
            params![room_chirho],
            |row_chirho| {
                Ok(RoomMetadataChirho {
                    private_chirho: row_chirho.get::<_, i64>(0)? == 1,
                    closed_ms_chirho: row_chirho.get(1)?,
                })
            },
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())
}

fn room_member_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    identity_chirho: &str,
    active_only_chirho: bool,
) -> Result<bool, String> {
    let sql_chirho = if active_only_chirho {
        r#"
        select exists(
            select 1 from subscriptions_chirho
            where room_chirho = ?1 and identity_chirho = ?2 and active_chirho = 1
        )
        "#
    } else {
        r#"
        select exists(
            select 1 from subscriptions_chirho
            where room_chirho = ?1 and identity_chirho = ?2
        )
        "#
    };
    conn_chirho
        .query_row(
            sql_chirho,
            params![room_chirho, identity_chirho],
            |row_chirho| row_chirho.get::<_, i64>(0),
        )
        .map(|value_chirho| value_chirho == 1)
        .map_err(|err_chirho| err_chirho.to_string())
}

/// Establishes room policy before a subscription is added or reactivated.
/// A new private room is self-created; later additions require an active member.
pub(crate) fn prepare_room_membership_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    target_identity_chirho: &str,
    actor_identity_chirho: Option<&str>,
    private_requested_chirho: bool,
    ttl_seconds_chirho: Option<u64>,
) -> Result<(), String> {
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let actor_identity_chirho = actor_identity_chirho.unwrap_or(target_identity_chirho);
    validate_token_chirho("actor_identity_chirho", actor_identity_chirho)?;
    let now_chirho = now_ms_chirho();
    let expires_ms_chirho = ttl_to_expires_ms_chirho(now_chirho, ttl_seconds_chirho)?;
    match room_metadata_chirho(conn_chirho, room_chirho)? {
        None => {
            if private_requested_chirho && actor_identity_chirho != target_identity_chirho {
                return Err(
                    "authorization denied: a new private room must be created by its first member"
                        .to_string(),
                );
            }
            if !private_requested_chirho && expires_ms_chirho.is_some() {
                return Err("ttl_seconds_chirho is only supported for private rooms".to_string());
            }
            conn_chirho
                .execute(
                    r#"
                    insert into rooms_chirho (
                        room_chirho, private_chirho, created_by_chirho,
                        created_ms_chirho, expires_ms_chirho
                    ) values (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        room_chirho,
                        if private_requested_chirho { 1 } else { 0 },
                        actor_identity_chirho,
                        now_chirho,
                        expires_ms_chirho
                    ],
                )
                .map_err(|err_chirho| err_chirho.to_string())?;
        }
        Some(metadata_chirho) => {
            if metadata_chirho.closed_ms_chirho.is_some() {
                return Err("channel is closed_chirho".to_string());
            }
            if private_requested_chirho && !metadata_chirho.private_chirho {
                return Err(
                    "existing public room cannot be silently converted to private".to_string(),
                );
            }
            if metadata_chirho.private_chirho {
                let actor_is_member_chirho =
                    room_member_chirho(conn_chirho, room_chirho, actor_identity_chirho, true)?;
                if !actor_is_member_chirho {
                    return Err(
                        "authorization denied: an active private-room member must add the target"
                            .to_string(),
                    );
                }
                if let Some(expires_ms_chirho) = expires_ms_chirho {
                    conn_chirho
                        .execute(
                            "update rooms_chirho set expires_ms_chirho = ?1 where room_chirho = ?2",
                            params![expires_ms_chirho, room_chirho],
                        )
                        .map_err(|err_chirho| err_chirho.to_string())?;
                }
            } else if expires_ms_chirho.is_some() {
                return Err("ttl_seconds_chirho is only supported for private rooms".to_string());
            }
        }
    }
    Ok(())
}

/// Private-room read branch in the private-channels workflow.
pub(crate) fn authorize_room_read_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    viewer_identity_chirho: Option<&str>,
) -> Result<(), String> {
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let Some(metadata_chirho) = room_metadata_chirho(conn_chirho, room_chirho)? else {
        return Ok(());
    };
    if !metadata_chirho.private_chirho {
        return Ok(());
    }
    let viewer_identity_chirho = viewer_identity_chirho
        .ok_or_else(|| "authorization denied: private room reads require as_chirho".to_string())?;
    validate_token_chirho("as_chirho", viewer_identity_chirho)?;
    let member_chirho = room_member_chirho(
        conn_chirho,
        room_chirho,
        viewer_identity_chirho,
        metadata_chirho.closed_ms_chirho.is_none(),
    )?;
    if !member_chirho {
        return Err("authorization denied: caller is not a private-room member".to_string());
    }
    Ok(())
}

pub(crate) fn authorize_room_post_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    sender_identity_chirho: &str,
) -> Result<bool, String> {
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let Some(metadata_chirho) = room_metadata_chirho(conn_chirho, room_chirho)? else {
        conn_chirho
            .execute(
                r#"
                insert into rooms_chirho (
                    room_chirho, private_chirho, created_by_chirho, created_ms_chirho
                ) values (?1, 0, ?2, ?3)
                "#,
                params![room_chirho, sender_identity_chirho, now_ms_chirho()],
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        return Ok(false);
    };
    if metadata_chirho.closed_ms_chirho.is_some() {
        return Err("channel is closed_chirho".to_string());
    }
    if metadata_chirho.private_chirho
        && !room_member_chirho(conn_chirho, room_chirho, sender_identity_chirho, true)?
    {
        return Err("authorization denied: sender is not a private-room member".to_string());
    }
    Ok(metadata_chirho.private_chirho)
}

pub(crate) fn authorize_private_target_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    target_identity_chirho: &str,
    private_room_chirho: bool,
) -> Result<(), String> {
    if private_room_chirho
        && !room_member_chirho(conn_chirho, room_chirho, target_identity_chirho, true)?
    {
        return Err(format!(
            "authorization denied: directed target {target_identity_chirho} is not a private-room member"
        ));
    }
    Ok(())
}

pub(crate) fn authorize_membership_change_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    actor_identity_chirho: &str,
) -> Result<(), String> {
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let Some(metadata_chirho) = room_metadata_chirho(conn_chirho, room_chirho)? else {
        return Ok(());
    };
    if !metadata_chirho.private_chirho {
        return Ok(());
    }
    if metadata_chirho.closed_ms_chirho.is_some() {
        return Err("channel is closed_chirho".to_string());
    }
    if !room_member_chirho(conn_chirho, room_chirho, actor_identity_chirho, true)? {
        return Err(
            "authorization denied: an active private-room member must change membership"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn list_room_messages_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
    after_chirho: i64,
    viewer_identity_chirho: Option<&str>,
) -> Result<Value, String> {
    authorize_room_read_chirho(conn_chirho, room_chirho, viewer_identity_chirho)?;
    let limit_chirho = if after_chirho <= 0 { 500 } else { 200 };
    let mut stmt_chirho = conn_chirho
        .prepare(
            r#"
            select id_chirho, at_ms_chirho, from_identity_chirho,
                   room_chirho, topic_chirho, body_chirho
            from messages_chirho
            where room_chirho = ?1 and id_chirho > ?2
            order by id_chirho
            limit ?3
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let rows_chirho = stmt_chirho
        .query_map(
            params![room_chirho, after_chirho, limit_chirho],
            |row_chirho| {
                let at_ms_chirho: i64 = row_chirho.get(1)?;
                Ok(json!({
                    "id_chirho": row_chirho.get::<_, i64>(0)?,
                    "at_ms_chirho": at_ms_chirho,
                    "at_text_chirho": format_timestamp_chirho(at_ms_chirho),
                    "from_identity_chirho": row_chirho.get::<_, String>(2)?,
                    "room_chirho": row_chirho.get::<_, String>(3)?,
                    "topic_chirho": row_chirho.get::<_, String>(4)?,
                    "body_chirho": row_chirho.get::<_, String>(5)?
                }))
            },
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut messages_chirho = Vec::new();
    for row_chirho in rows_chirho {
        messages_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
    }
    Ok(json!({ "ok_chirho": true, "messages_chirho": messages_chirho }))
}

fn ordered_pair_chirho(
    identity_one_chirho: &str,
    identity_two_chirho: &str,
) -> Result<(String, String, String), String> {
    validate_token_chirho("identity_chirho", identity_one_chirho)?;
    validate_token_chirho("with_chirho", identity_two_chirho)?;
    if identity_one_chirho == identity_two_chirho {
        return Err("direct-message participants must be distinct".to_string());
    }
    let (identity_a_chirho, identity_b_chirho) = if identity_one_chirho < identity_two_chirho {
        (identity_one_chirho, identity_two_chirho)
    } else {
        (identity_two_chirho, identity_one_chirho)
    };
    Ok((
        format!("{identity_a_chirho}{PAIR_SEPARATOR_CHIRHO}{identity_b_chirho}"),
        identity_a_chirho.to_string(),
        identity_b_chirho.to_string(),
    ))
}

fn registered_target_chirho(
    conn_chirho: &Connection,
    identity_chirho: &str,
) -> Result<Option<AgentTargetChirho>, String> {
    conn_chirho
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
        .map_err(|err_chirho| err_chirho.to_string())
}

/// First-class DM persistence and delivery branch in the private-channels
/// workflow. Room storage is deliberately never touched.
pub(crate) fn post_dm_chirho(
    conn_chirho: &mut Connection,
    request_chirho: DmPostRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho("from_session_chirho", &request_chirho.from_session_chirho)?;
    validate_token_chirho("from_agent_chirho", &request_chirho.from_agent_chirho)?;
    validate_token_chirho("to_chirho", &request_chirho.to_chirho)?;
    validate_body_chirho(&request_chirho.body_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let from_identity_chirho = identity_chirho(
        &request_chirho.from_session_chirho,
        &request_chirho.from_agent_chirho,
    );
    if registered_target_chirho(conn_chirho, &from_identity_chirho)?.is_none() {
        return Err("direct-message sender must be a registered agent".to_string());
    }
    let target_chirho = registered_target_chirho(conn_chirho, &request_chirho.to_chirho)?
        .ok_or_else(|| "direct-message recipient must be a registered agent".to_string())?;
    let (pair_key_chirho, identity_a_chirho, identity_b_chirho) =
        ordered_pair_chirho(&from_identity_chirho, &request_chirho.to_chirho)?;
    let at_ms_chirho = now_ms_chirho();
    let expires_ms_chirho =
        ttl_to_expires_ms_chirho(at_ms_chirho, request_chirho.ttl_seconds_chirho)?;
    let tx_chirho = conn_chirho
        .transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let existing_closed_ms_chirho: Option<Option<i64>> = tx_chirho
        .query_row(
            "select closed_ms_chirho from dm_channels_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
            |row_chirho| row_chirho.get(0),
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())?;
    if existing_closed_ms_chirho.flatten().is_some() {
        return Err("direct-message channel is closed_chirho".to_string());
    }
    tx_chirho
        .execute(
            r#"
            insert into dm_channels_chirho (
                pair_key_chirho, identity_a_chirho, identity_b_chirho,
                created_ms_chirho, last_message_ms_chirho, expires_ms_chirho
            ) values (?1, ?2, ?3, ?4, ?4, ?5)
            on conflict(pair_key_chirho) do update set
                last_message_ms_chirho = excluded.last_message_ms_chirho,
                expires_ms_chirho = coalesce(excluded.expires_ms_chirho, dm_channels_chirho.expires_ms_chirho)
            "#,
            params![
                pair_key_chirho,
                identity_a_chirho,
                identity_b_chirho,
                at_ms_chirho,
                expires_ms_chirho
            ],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            r#"
            insert into direct_messages_chirho (
                pair_key_chirho, at_ms_chirho, from_identity_chirho,
                to_identity_chirho, body_chirho
            ) values (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                pair_key_chirho,
                at_ms_chirho,
                from_identity_chirho,
                request_chirho.to_chirho,
                request_chirho.body_chirho
            ],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let message_id_chirho = tx_chirho.last_insert_rowid();
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let effective_expires_ms_chirho: Option<i64> = conn_chirho
        .query_row(
            "select expires_ms_chirho from dm_channels_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
            |row_chirho| row_chirho.get(0),
        )
        .map_err(|err_chirho| err_chirho.to_string())?;

    let delivery_text_chirho = format!(
        "METROPOLELUYA_CHIRHO DM #{message_id_chirho} AT {} FROM {} TO {}\n{}",
        format_timestamp_chirho(at_ms_chirho),
        from_identity_chirho,
        request_chirho.to_chirho,
        request_chirho.body_chirho
    );
    let outcomes_chirho =
        deliver_parallel_chirho(std::slice::from_ref(&target_chirho), &delivery_text_chirho);
    let outcome_chirho = outcomes_chirho
        .first()
        .ok_or_else(|| "direct-message delivery produced no outcome".to_string())?;
    conn_chirho
        .execute(
            r#"
            insert into direct_deliveries_chirho (
                direct_message_id_chirho, to_identity_chirho, tmux_target_chirho,
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
                now_ms_chirho()
            ],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(json!({
        "ok_chirho": true,
        "direct_message_id_chirho": message_id_chirho,
        "at_ms_chirho": at_ms_chirho,
        "at_text_chirho": format_timestamp_chirho(at_ms_chirho),
        "from_identity_chirho": from_identity_chirho,
        "to_identity_chirho": request_chirho.to_chirho,
        "delivered_chirho": outcome_chirho.delivered_chirho,
        "expires_ms_chirho": effective_expires_ms_chirho
    }))
}

pub(crate) fn list_dm_messages_chirho(
    conn_chirho: &Connection,
    identity_chirho: &str,
    with_chirho: &str,
    after_chirho: i64,
) -> Result<Value, String> {
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let (pair_key_chirho, _, _) = ordered_pair_chirho(identity_chirho, with_chirho)?;
    let channel_chirho: Option<(Option<i64>, Option<i64>)> = conn_chirho
        .query_row(
            r#"
            select expires_ms_chirho, closed_ms_chirho
            from dm_channels_chirho where pair_key_chirho = ?1
            "#,
            params![pair_key_chirho],
            |row_chirho| Ok((row_chirho.get(0)?, row_chirho.get(1)?)),
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let exists_chirho = channel_chirho.is_some();
    let mut messages_chirho = Vec::new();
    if exists_chirho {
        let limit_chirho = if after_chirho <= 0 { 500 } else { 200 };
        let mut stmt_chirho = conn_chirho
            .prepare(
                r#"
                select id_chirho, at_ms_chirho, from_identity_chirho,
                       to_identity_chirho, body_chirho
                from direct_messages_chirho
                where pair_key_chirho = ?1 and id_chirho > ?2
                order by id_chirho
                limit ?3
                "#,
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        let rows_chirho = stmt_chirho
            .query_map(
                params![pair_key_chirho, after_chirho, limit_chirho],
                |row_chirho| {
                    let at_ms_chirho: i64 = row_chirho.get(1)?;
                    Ok(json!({
                        "id_chirho": row_chirho.get::<_, i64>(0)?,
                        "at_ms_chirho": at_ms_chirho,
                        "at_text_chirho": format_timestamp_chirho(at_ms_chirho),
                        "from_identity_chirho": row_chirho.get::<_, String>(2)?,
                        "to_identity_chirho": row_chirho.get::<_, String>(3)?,
                        "body_chirho": row_chirho.get::<_, String>(4)?
                    }))
                },
            )
            .map_err(|err_chirho| err_chirho.to_string())?;
        for row_chirho in rows_chirho {
            messages_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
        }
    }
    let (expires_ms_chirho, closed_ms_chirho) = channel_chirho.unwrap_or((None, None));
    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": identity_chirho,
        "with_chirho": with_chirho,
        "exists_chirho": exists_chirho,
        "expires_ms_chirho": expires_ms_chirho,
        "closed_ms_chirho": closed_ms_chirho,
        "direct_messages_chirho": messages_chirho
    }))
}

/// `rooms --mine` discovery branch in the private-channels workflow.
pub(crate) fn list_channels_chirho(
    conn_chirho: &Connection,
    identity_chirho: &str,
    include_closed_chirho: bool,
) -> Result<Value, String> {
    validate_token_chirho("identity_chirho", identity_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let mut room_stmt_chirho = conn_chirho
        .prepare(
            r#"
            select s.room_chirho,
                   max(s.active_chirho),
                   coalesce(r.private_chirho, 0),
                   r.expires_ms_chirho,
                   r.closed_ms_chirho,
                   coalesce(group_concat(distinct s.topic_chirho), '')
            from subscriptions_chirho s
            left join rooms_chirho r on r.room_chirho = s.room_chirho
            where s.identity_chirho = ?1
              and (?2 = 1 or s.active_chirho = 1)
            group by s.room_chirho, r.private_chirho, r.expires_ms_chirho, r.closed_ms_chirho
            order by s.room_chirho
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let room_rows_chirho = room_stmt_chirho
        .query_map(
            params![identity_chirho, include_closed_chirho],
            |row_chirho| {
                let topics_text_chirho: String = row_chirho.get(5)?;
                let topics_chirho: Vec<String> = topics_text_chirho
                    .split(',')
                    .filter(|topic_chirho| !topic_chirho.is_empty())
                    .map(str::to_string)
                    .collect();
                Ok(json!({
                    "room_chirho": row_chirho.get::<_, String>(0)?,
                    "active_chirho": row_chirho.get::<_, i64>(1)? == 1,
                    "private_chirho": row_chirho.get::<_, i64>(2)? == 1,
                    "expires_ms_chirho": row_chirho.get::<_, Option<i64>>(3)?,
                    "closed_ms_chirho": row_chirho.get::<_, Option<i64>>(4)?,
                    "topics_chirho": topics_chirho
                }))
            },
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut rooms_chirho = Vec::new();
    for row_chirho in room_rows_chirho {
        rooms_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
    }

    let mut dm_stmt_chirho = conn_chirho
        .prepare(
            r#"
            select identity_a_chirho, identity_b_chirho, created_ms_chirho,
                   last_message_ms_chirho, expires_ms_chirho, closed_ms_chirho
            from dm_channels_chirho
            where (identity_a_chirho = ?1 or identity_b_chirho = ?1)
              and (?2 = 1 or closed_ms_chirho is null)
            order by coalesce(last_message_ms_chirho, created_ms_chirho) desc
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let dm_rows_chirho = dm_stmt_chirho
        .query_map(
            params![identity_chirho, include_closed_chirho],
            |row_chirho| {
                let identity_a_chirho: String = row_chirho.get(0)?;
                let identity_b_chirho: String = row_chirho.get(1)?;
                let counterpart_chirho = if identity_a_chirho == identity_chirho {
                    identity_b_chirho
                } else {
                    identity_a_chirho
                };
                Ok(json!({
                    "with_chirho": counterpart_chirho,
                    "created_ms_chirho": row_chirho.get::<_, i64>(2)?,
                    "last_message_ms_chirho": row_chirho.get::<_, Option<i64>>(3)?,
                    "expires_ms_chirho": row_chirho.get::<_, Option<i64>>(4)?,
                    "closed_ms_chirho": row_chirho.get::<_, Option<i64>>(5)?
                }))
            },
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut direct_messages_chirho = Vec::new();
    for row_chirho in dm_rows_chirho {
        direct_messages_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
    }
    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": identity_chirho,
        "rooms_chirho": rooms_chirho,
        "direct_message_channels_chirho": direct_messages_chirho
    }))
}

fn active_room_targets_chirho(
    conn_chirho: &Connection,
    room_chirho: &str,
) -> Result<Vec<AgentTargetChirho>, String> {
    let mut stmt_chirho = conn_chirho
        .prepare(
            r#"
            select distinct a.identity_chirho, a.tmux_target_chirho
            from agents_chirho a
            join subscriptions_chirho s on s.identity_chirho = a.identity_chirho
            where s.room_chirho = ?1 and s.active_chirho = 1
            order by a.identity_chirho
            "#,
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let rows_chirho = stmt_chirho
        .query_map(params![room_chirho], |row_chirho| {
            Ok(AgentTargetChirho {
                identity_chirho: row_chirho.get(0)?,
                tmux_target_chirho: row_chirho.get(1)?,
            })
        })
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut targets_chirho = Vec::new();
    for row_chirho in rows_chirho {
        targets_chirho.push(row_chirho.map_err(|err_chirho| err_chirho.to_string())?);
    }
    Ok(targets_chirho)
}

/// Explicit cold-retirement branch in the private-channels workflow.
pub(crate) fn close_room_chirho(
    conn_chirho: &Connection,
    request_chirho: CloseRoomRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho(
        "actor_identity_chirho",
        &request_chirho.actor_identity_chirho,
    )?;
    validate_token_chirho("room_chirho", &request_chirho.room_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let metadata_chirho = room_metadata_chirho(conn_chirho, &request_chirho.room_chirho)?
        .ok_or_else(|| "room does not exist".to_string())?;
    if !metadata_chirho.private_chirho {
        return Err("only private rooms have a close lifecycle".to_string());
    }
    if metadata_chirho.closed_ms_chirho.is_some() {
        return Err("channel is already closed_chirho".to_string());
    }
    if !room_member_chirho(
        conn_chirho,
        &request_chirho.room_chirho,
        &request_chirho.actor_identity_chirho,
        true,
    )? {
        return Err("authorization denied: only a private-room member can close it".to_string());
    }
    let targets_chirho = active_room_targets_chirho(conn_chirho, &request_chirho.room_chirho)?;
    let closed_ms_chirho = now_ms_chirho();
    let tx_chirho = conn_chirho
        .unchecked_transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            "update rooms_chirho set closed_ms_chirho = ?1 where room_chirho = ?2",
            params![closed_ms_chirho, request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            "update subscriptions_chirho set active_chirho = 0 where room_chirho = ?1",
            params![request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let notice_chirho = format!(
        "metropoleluya: private room {} was closed by {}.",
        request_chirho.room_chirho, request_chirho.actor_identity_chirho
    );
    let delivered_count_chirho = deliver_parallel_chirho(&targets_chirho, &notice_chirho)
        .iter()
        .filter(|outcome_chirho| outcome_chirho.delivered_chirho)
        .count();
    Ok(json!({
        "ok_chirho": true,
        "room_chirho": request_chirho.room_chirho,
        "closed_ms_chirho": closed_ms_chirho,
        "notification_count_chirho": delivered_count_chirho
    }))
}

pub(crate) fn close_dm_chirho(
    conn_chirho: &Connection,
    request_chirho: CloseDmRequestChirho,
) -> Result<Value, String> {
    let (pair_key_chirho, _, _) =
        ordered_pair_chirho(&request_chirho.identity_chirho, &request_chirho.with_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let existing_chirho: Option<Option<i64>> = conn_chirho
        .query_row(
            "select closed_ms_chirho from dm_channels_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
            |row_chirho| row_chirho.get(0),
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let Some(existing_closed_chirho) = existing_chirho else {
        return Err("direct-message channel does not exist".to_string());
    };
    if existing_closed_chirho.is_some() {
        return Err("direct-message channel is already closed_chirho".to_string());
    }
    let closed_ms_chirho = now_ms_chirho();
    conn_chirho
        .execute(
            "update dm_channels_chirho set closed_ms_chirho = ?1 where pair_key_chirho = ?2",
            params![closed_ms_chirho, pair_key_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let target_chirho = registered_target_chirho(conn_chirho, &request_chirho.with_chirho)?;
    let notification_count_chirho = if let Some(target_chirho) = target_chirho {
        let notice_chirho = format!(
            "metropoleluya: direct-message channel with {} was closed by {}.",
            request_chirho.identity_chirho, request_chirho.identity_chirho
        );
        deliver_parallel_chirho(&[target_chirho], &notice_chirho)
            .iter()
            .filter(|outcome_chirho| outcome_chirho.delivered_chirho)
            .count()
    } else {
        0
    };
    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": request_chirho.identity_chirho,
        "with_chirho": request_chirho.with_chirho,
        "closed_ms_chirho": closed_ms_chirho,
        "notification_count_chirho": notification_count_chirho
    }))
}

/// Explicit destructive retirement after cold close. This is the bounded-
/// growth escape hatch in the private-channels workflow; expiry never calls it.
pub(crate) fn purge_room_chirho(
    conn_chirho: &Connection,
    request_chirho: PurgeRoomRequestChirho,
) -> Result<Value, String> {
    validate_token_chirho(
        "actor_identity_chirho",
        &request_chirho.actor_identity_chirho,
    )?;
    validate_token_chirho("room_chirho", &request_chirho.room_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let metadata_chirho = room_metadata_chirho(conn_chirho, &request_chirho.room_chirho)?
        .ok_or_else(|| "room does not exist".to_string())?;
    if !metadata_chirho.private_chirho {
        return Err("public rooms cannot be purged through private lifecycle".to_string());
    }
    if metadata_chirho.closed_ms_chirho.is_none() {
        return Err("channel must be closed before purge_chirho".to_string());
    }
    if !room_member_chirho(
        conn_chirho,
        &request_chirho.room_chirho,
        &request_chirho.actor_identity_chirho,
        false,
    )? {
        return Err("authorization denied: only a former room member can purge it".to_string());
    }
    let tx_chirho = conn_chirho
        .unchecked_transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            r#"
            delete from deliveries_chirho
            where message_id_chirho in (
                select id_chirho from messages_chirho where room_chirho = ?1
            )
            "#,
            params![request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let purged_message_count_chirho = tx_chirho
        .execute(
            "delete from messages_chirho where room_chirho = ?1",
            params![request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            "delete from subscriptions_chirho where room_chirho = ?1",
            params![request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            "delete from rooms_chirho where room_chirho = ?1",
            params![request_chirho.room_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(json!({
        "ok_chirho": true,
        "room_chirho": request_chirho.room_chirho,
        "purged_message_count_chirho": purged_message_count_chirho
    }))
}

/// Explicit destructive retirement for a closed DM pair channel.
pub(crate) fn purge_dm_chirho(
    conn_chirho: &Connection,
    request_chirho: PurgeDmRequestChirho,
) -> Result<Value, String> {
    let (pair_key_chirho, _, _) =
        ordered_pair_chirho(&request_chirho.identity_chirho, &request_chirho.with_chirho)?;
    retire_expired_channels_chirho(conn_chirho, now_ms_chirho())?;
    let closed_ms_chirho: Option<Option<i64>> = conn_chirho
        .query_row(
            "select closed_ms_chirho from dm_channels_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
            |row_chirho| row_chirho.get(0),
        )
        .optional()
        .map_err(|err_chirho| err_chirho.to_string())?;
    match closed_ms_chirho {
        None => return Err("direct-message channel does not exist".to_string()),
        Some(None) => return Err("channel must be closed before purge_chirho".to_string()),
        Some(Some(_)) => {}
    }
    let tx_chirho = conn_chirho
        .unchecked_transaction()
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            r#"
            delete from direct_deliveries_chirho
            where direct_message_id_chirho in (
                select id_chirho from direct_messages_chirho where pair_key_chirho = ?1
            )
            "#,
            params![pair_key_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    let purged_message_count_chirho = tx_chirho
        .execute(
            "delete from direct_messages_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .execute(
            "delete from dm_channels_chirho where pair_key_chirho = ?1",
            params![pair_key_chirho],
        )
        .map_err(|err_chirho| err_chirho.to_string())?;
    tx_chirho
        .commit()
        .map_err(|err_chirho| err_chirho.to_string())?;
    Ok(json!({
        "ok_chirho": true,
        "identity_chirho": request_chirho.identity_chirho,
        "with_chirho": request_chirho.with_chirho,
        "purged_message_count_chirho": purged_message_count_chirho
    }))
}

fn parse_ttl_cli_chirho(args_chirho: &[String]) -> Result<Option<u64>, String> {
    arg_value_chirho(args_chirho, "--ttl-seconds")
        .map(|value_chirho| {
            value_chirho
                .parse::<u64>()
                .map_err(|_| "--ttl-seconds must be an integer".to_string())
        })
        .transpose()
}

pub(crate) fn run_post_dm_cli_chirho(
    args_chirho: &[String],
    body_text_chirho: String,
) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let body_chirho = json!({
        "from_session_chirho": require_arg_chirho(args_chirho, "--from-session")?,
        "from_agent_chirho": require_arg_chirho(args_chirho, "--from-agent")?,
        "to_chirho": require_arg_chirho(args_chirho, "--dm")?,
        "body_chirho": body_text_chirho,
        "ttl_seconds_chirho": parse_ttl_cli_chirho(args_chirho)?
    });
    println!(
        "{}",
        http_client_chirho(&server_chirho, "POST", "/v1/dm-chirho", Some(&body_chirho))?
    );
    Ok(())
}

pub(crate) fn run_rooms_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let identity_chirho = identity_chirho(
        &require_arg_chirho(args_chirho, "--session")?,
        &require_arg_chirho(args_chirho, "--agent")?,
    );
    if args_chirho.get(1).map(String::as_str) == Some("close") {
        let body_chirho = json!({
            "actor_identity_chirho": identity_chirho,
            "room_chirho": require_arg_chirho(args_chirho, "--room")?
        });
        println!(
            "{}",
            http_client_chirho(
                &server_chirho,
                "POST",
                "/v1/rooms-chirho/close-chirho",
                Some(&body_chirho)
            )?
        );
        return Ok(());
    }
    if args_chirho.get(1).map(String::as_str) == Some("purge") {
        let body_chirho = json!({
            "actor_identity_chirho": identity_chirho,
            "room_chirho": require_arg_chirho(args_chirho, "--room")?
        });
        println!(
            "{}",
            http_client_chirho(
                &server_chirho,
                "POST",
                "/v1/rooms-chirho/purge-chirho",
                Some(&body_chirho)
            )?
        );
        return Ok(());
    }
    if !args_chirho.iter().any(|arg_chirho| arg_chirho == "--mine") {
        return Err("rooms requires --mine or the close subcommand".to_string());
    }
    let path_chirho = format!(
        "/v1/channels-chirho?identity_chirho={}&include_closed_chirho={}",
        percent_encode_chirho(&identity_chirho),
        args_chirho
            .iter()
            .any(|arg_chirho| arg_chirho == "--include-closed")
    );
    println!(
        "{}",
        http_client_chirho(&server_chirho, "GET", &path_chirho, None)?
    );
    Ok(())
}

pub(crate) fn run_dm_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let identity_chirho = identity_chirho(
        &require_arg_chirho(args_chirho, "--session")?,
        &require_arg_chirho(args_chirho, "--agent")?,
    );
    let with_chirho = require_arg_chirho(args_chirho, "--with")?;
    if args_chirho.get(1).map(String::as_str) == Some("close") {
        let body_chirho = json!({
            "identity_chirho": identity_chirho,
            "with_chirho": with_chirho
        });
        println!(
            "{}",
            http_client_chirho(
                &server_chirho,
                "POST",
                "/v1/dms-chirho/close-chirho",
                Some(&body_chirho)
            )?
        );
        return Ok(());
    }
    if args_chirho.get(1).map(String::as_str) == Some("purge") {
        let body_chirho = json!({
            "identity_chirho": identity_chirho,
            "with_chirho": with_chirho
        });
        println!(
            "{}",
            http_client_chirho(
                &server_chirho,
                "POST",
                "/v1/dms-chirho/purge-chirho",
                Some(&body_chirho)
            )?
        );
        return Ok(());
    }
    let after_chirho = arg_value_chirho(args_chirho, "--after").unwrap_or_else(|| "0".to_string());
    let path_chirho = format!(
        "/v1/dms-chirho?identity_chirho={}&with_chirho={}&after_chirho={}",
        percent_encode_chirho(&identity_chirho),
        percent_encode_chirho(&with_chirho),
        percent_encode_chirho(&after_chirho)
    );
    println!(
        "{}",
        http_client_chirho(&server_chirho, "GET", &path_chirho, None)?
    );
    Ok(())
}
