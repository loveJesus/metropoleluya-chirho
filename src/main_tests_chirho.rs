// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use super::*;

#[test]
fn identity_uses_session_slash_agent_chirho() {
    assert_eq!(
        identity_chirho("PROJECT_CHIRHO", "gpt_chirho"),
        "PROJECT_CHIRHO/gpt_chirho"
    );
}

#[test]
fn percent_encoding_round_trips_room_names_chirho() {
    let value_chirho = "project chirho/topic";
    let encoded_chirho = percent_encode_chirho(value_chirho);
    assert_eq!(
        percent_decode_chirho(&encoded_chirho).unwrap(),
        value_chirho
    );
}

#[test]
fn timestamp_formatter_uses_eastern_text_chirho() {
    // Storage is epoch UTC; display localizes to America/New_York (DST-correct).
    // Epoch 0 = 1970-01-01T00:00Z, which is EST (UTC-5) in New York.
    assert_eq!(format_timestamp_chirho(0), "1969-12-31 19:00:00.000 EST");
    // Winter instant -> EST (-5).
    assert_eq!(
        format_timestamp_chirho(1_704_067_200_123),
        "2023-12-31 19:00:00.123 EST"
    );
    // Summer instant -> EDT (-4): proves DST handling, not a hard-coded offset.
    assert_eq!(
        format_timestamp_chirho(1_719_792_000_000),
        "2024-06-30 20:00:00.000 EDT"
    );
}

#[test]
fn body_validation_allows_multiline_agent_messages_chirho() {
    validate_body_chirho("PROJECT_CHIRHO/GPT SENDS: line one\n\nDetails line two.").unwrap();
    assert!(validate_body_chirho("").is_err());
    assert!(validate_body_chirho("bad\0body").is_err());
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
            notify_actor_chirho: None,
        },
    )
    .unwrap();
    assert_eq!(response_chirho["identity_chirho"], "TEST_CHIRHO/gpt_chirho");
    let agents_chirho = list_agents_chirho(&conn_chirho, Some("room-chirho")).unwrap();
    assert_eq!(agents_chirho["agents_chirho"].as_array().unwrap().len(), 1);
}

#[test]
fn sqlite_messages_view_exposes_readable_timestamp_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    conn_chirho
        .execute(
            r#"
            insert into messages_chirho (
                at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho
            ) values (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                1_704_067_200_123_i64,
                "PROJECT_CHIRHO/gpt_chirho",
                "project-chirho",
                "audit-chirho",
                "body-chirho"
            ],
        )
        .unwrap();
    let at_text_chirho: String = conn_chirho
        .query_row(
            "select at_text_chirho from messages_with_time_chirho where id_chirho = 1",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    // The SQL view keeps canonical UTC (Z-suffixed) for raw DB inspection.
    assert_eq!(at_text_chirho, "2024-01-01 00:00:00.123Z");
    // The API/TUI display path localizes the same instant to America/New_York.
    let listed_chirho = list_messages_chirho(&conn_chirho, "project-chirho", 0).unwrap();
    assert_eq!(
        listed_chirho["messages_chirho"][0]["at_text_chirho"],
        "2023-12-31 19:00:00.123 EST"
    );
}

#[test]
fn remove_agent_deletes_only_targeted_room_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_agent_chirho(
        &conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            tmux_target_chirho: "missing-session-chirho:1".to_string(),
            rooms_chirho: vec!["room-a-chirho".to_string(), "room-b-chirho".to_string()],
            topics_chirho: vec![],
            notify_actor_chirho: None,
        },
    )
    .unwrap();
    let response_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "operator_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            room_chirho: "room-a-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(response_chirho["ok_chirho"], true);
    assert_eq!(response_chirho["removed_count_chirho"], 1);
    assert_eq!(response_chirho["notified_chirho"], false);
    let room_a_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where identity_chirho = ?1 and room_chirho = ?2",
            params!["TEST_CHIRHO/gpt_chirho", "room-a-chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(room_a_count_chirho, 0);
    let room_b_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where identity_chirho = ?1 and room_chirho = ?2",
            params!["TEST_CHIRHO/gpt_chirho", "room-b-chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(room_b_count_chirho, 1);
    let agent_rows_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from agents_chirho where identity_chirho = ?1",
            params!["TEST_CHIRHO/gpt_chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(agent_rows_chirho, 1);
}

#[test]
fn remove_of_non_member_is_noop_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let response_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "operator_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "ghost_chirho".to_string(),
            room_chirho: "room-a-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(response_chirho["ok_chirho"], true);
    assert_eq!(response_chirho["removed_count_chirho"], 0);
    assert_eq!(response_chirho["notified_chirho"], false);
}

#[test]
fn register_notify_actor_reports_unnotified_for_dead_pane_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let response_chirho = register_agent_chirho(
        &conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            tmux_target_chirho: "missing-session-chirho:1".to_string(),
            rooms_chirho: vec!["room-chirho".to_string()],
            topics_chirho: vec![],
            notify_actor_chirho: Some("TEST_CHIRHO/operator_chirho".to_string()),
        },
    )
    .unwrap();
    assert_eq!(response_chirho["notified_chirho"], false);
}
