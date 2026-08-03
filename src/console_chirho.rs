// For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV)

//! Polling watch and line-oriented console clients.

use crate::{
    arg_value_chirho, arg_values_chirho, channels_chirho, default_topic_chirho, http_client_chirho,
    identity_chirho, percent_encode_chirho, require_arg_chirho, server_arg_chirho,
};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;

pub(crate) fn run_register_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let rooms_chirho = arg_values_chirho(args_chirho, "--room");
    let topics_chirho = arg_values_chirho(args_chirho, "--topic");
    let body_chirho = json!({
        "session_chirho": require_arg_chirho(args_chirho, "--session")?,
        "agent_chirho": require_arg_chirho(args_chirho, "--agent")?,
        "tmux_target_chirho": require_arg_chirho(args_chirho, "--tmux-target")?,
        "rooms_chirho": rooms_chirho,
        "topics_chirho": topics_chirho,
        "notify_actor_chirho": arg_value_chirho(args_chirho, "--as"),
        "private_chirho": args_chirho.iter().any(|arg_chirho| arg_chirho == "--private"),
        "ttl_seconds_chirho": arg_value_chirho(args_chirho, "--ttl-seconds")
            .map(|value_chirho| value_chirho.parse::<u64>())
            .transpose()
            .map_err(|_| "--ttl-seconds must be an integer".to_string())?
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

pub(crate) fn run_post_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let body_text_chirho = if let Some(file_chirho) = arg_value_chirho(args_chirho, "--body-file") {
        fs::read_to_string(file_chirho).map_err(|err_chirho| err_chirho.to_string())?
    } else {
        require_arg_chirho(args_chirho, "--body")?
    };
    if arg_value_chirho(args_chirho, "--dm").is_some() {
        return channels_chirho::run_post_dm_cli_chirho(args_chirho, body_text_chirho);
    }
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

pub(crate) fn run_watch_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let room_chirho = require_arg_chirho(args_chirho, "--room")?;
    let mut after_chirho = arg_value_chirho(args_chirho, "--after")
        .and_then(|value_chirho| value_chirho.parse::<i64>().ok())
        .unwrap_or(0);
    loop {
        let viewer_query_chirho = arg_value_chirho(args_chirho, "--as")
            .map(|identity_chirho| {
                format!("&as_chirho={}", percent_encode_chirho(&identity_chirho))
            })
            .unwrap_or_default();
        let path_chirho = format!(
            "/v1/messages_chirho?room_chirho={}&after_chirho={after_chirho}{viewer_query_chirho}",
            percent_encode_chirho(&room_chirho),
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
                    "\n#{} {} {} [{}]\n{}\n",
                    id_chirho,
                    message_chirho
                        .get("at_text_chirho")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-time"),
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

pub(crate) fn run_console_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
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
        "--as".to_string(),
        identity_chirho(&session_chirho, &agent_chirho),
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

pub(crate) fn run_agents_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
    let server_chirho = server_arg_chirho(args_chirho);
    let path_chirho = if let Some(room_chirho) = arg_value_chirho(args_chirho, "--room") {
        let viewer_query_chirho = arg_value_chirho(args_chirho, "--as")
            .map(|identity_chirho| {
                format!("&as_chirho={}", percent_encode_chirho(&identity_chirho))
            })
            .unwrap_or_default();
        format!(
            "/v1/agents_chirho?room_chirho={}{viewer_query_chirho}",
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

pub(crate) fn run_refresh_cli_chirho(args_chirho: &[String]) -> Result<(), String> {
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

pub(crate) fn usage_chirho() -> &'static str {
    "usage:
  metropoleluya-chirho server [--bind 127.0.0.1:37371] [--db path]
  metropoleluya-chirho register --session S --agent A --tmux-target T --room R [--topic T] [--private] [--ttl-seconds N] [--as SESSION/agent]
  metropoleluya-chirho remove --from-session S --from-agent A --session TS --agent TA --room R
  metropoleluya-chirho post --from-session S --from-agent A --room R [--topic T] (--body TEXT|--body-file PATH) [--to SESSION/agent]
  metropoleluya-chirho post --from-session S --from-agent A --dm SESSION/agent (--body TEXT|--body-file PATH) [--ttl-seconds N]
  metropoleluya-chirho watch --room R [--as SESSION/agent]
  metropoleluya-chirho console --session S --agent A --room R [--topic T]
  metropoleluya-chirho tui --session S --agent A --room R [--topic T] [--after ID]
  metropoleluya-chirho agents [--room R] [--as SESSION/agent]
  metropoleluya-chirho rooms --mine --session S --agent A [--include-closed]
  metropoleluya-chirho rooms close --session S --agent A --room R
  metropoleluya-chirho rooms purge --session S --agent A --room R
  metropoleluya-chirho dm --session S --agent A --with SESSION/agent [--after ID]
  metropoleluya-chirho dm close --session S --agent A --with SESSION/agent
  metropoleluya-chirho dm purge --session S --agent A --with SESSION/agent
  metropoleluya-chirho refresh"
}
