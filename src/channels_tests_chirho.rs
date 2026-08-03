// For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV)

use super::*;

fn register_test_agent_chirho(
    conn_chirho: &Connection,
    agent_chirho: &str,
    rooms_chirho: Vec<&str>,
    private_chirho: bool,
    actor_chirho: Option<&str>,
    ttl_seconds_chirho: Option<u64>,
) -> Result<Value, String> {
    register_agent_chirho(
        conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: agent_chirho.to_string(),
            tmux_target_chirho: format!("missing-session-chirho:{agent_chirho}"),
            rooms_chirho: rooms_chirho.into_iter().map(str::to_string).collect(),
            topics_chirho: vec![],
            notify_actor_chirho: actor_chirho.map(str::to_string),
            private_chirho,
            ttl_seconds_chirho,
        },
    )
}

#[test]
fn private_room_requires_membership_for_read_and_post_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(
        &conn_chirho,
        "owner_chirho",
        vec!["private-room-chirho"],
        true,
        None,
        None,
    )
    .unwrap();

    assert!(channels_chirho::list_room_messages_chirho(
        &conn_chirho,
        "private-room-chirho",
        0,
        None,
    )
    .unwrap_err()
    .contains("authorization denied"));
    assert!(channels_chirho::list_room_messages_chirho(
        &conn_chirho,
        "private-room-chirho",
        0,
        Some("TEST_CHIRHO/outsider_chirho"),
    )
    .unwrap_err()
    .contains("authorization denied"));
    assert!(channels_chirho::list_room_messages_chirho(
        &conn_chirho,
        "private-room-chirho",
        0,
        Some("TEST_CHIRHO/owner_chirho"),
    )
    .is_ok());
    let global_agents_chirho = list_agents_chirho(&conn_chirho, None).unwrap();
    assert_eq!(
        global_agents_chirho["agents_chirho"][0]["topics_chirho"],
        json!([])
    );

    let denied_chirho = post_message_chirho(
        &mut conn_chirho,
        PostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "outsider_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
            topic_chirho: default_topic_chirho(),
            body_chirho: "should not persist".to_string(),
            to_chirho: vec![],
            deliver_to_sender_chirho: false,
        },
    )
    .unwrap_err();
    assert!(denied_chirho.contains("authorization denied"));
    let message_count_chirho: i64 = conn_chirho
        .query_row("select count(*) from messages_chirho", [], |row_chirho| {
            row_chirho.get(0)
        })
        .unwrap();
    assert_eq!(message_count_chirho, 0);

    let directed_denied_chirho = post_message_chirho(
        &mut conn_chirho,
        PostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
            topic_chirho: default_topic_chirho(),
            body_chirho: "must roll back".to_string(),
            to_chirho: vec!["TEST_CHIRHO/outsider_chirho".to_string()],
            deliver_to_sender_chirho: false,
        },
    )
    .unwrap_err();
    assert!(directed_denied_chirho.contains("directed target"));
    let message_count_chirho: i64 = conn_chirho
        .query_row("select count(*) from messages_chirho", [], |row_chirho| {
            row_chirho.get(0)
        })
        .unwrap();
    assert_eq!(message_count_chirho, 0);
}

#[test]
fn existing_member_controls_private_membership_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(
        &conn_chirho,
        "owner_chirho",
        vec!["private-room-chirho"],
        true,
        None,
        None,
    )
    .unwrap();
    let denied_chirho = register_test_agent_chirho(
        &conn_chirho,
        "newcomer_chirho",
        vec!["private-room-chirho"],
        false,
        Some("TEST_CHIRHO/outsider_chirho"),
        None,
    )
    .unwrap_err();
    assert!(denied_chirho.contains("authorization denied"));
    let denied_agent_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from agents_chirho where identity_chirho = 'TEST_CHIRHO/newcomer_chirho'",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(denied_agent_count_chirho, 0);
    let multi_room_denied_chirho = register_test_agent_chirho(
        &conn_chirho,
        "partial_chirho",
        vec!["must-rollback-chirho", "private-room-chirho"],
        false,
        Some("TEST_CHIRHO/outsider_chirho"),
        None,
    )
    .unwrap_err();
    assert!(multi_room_denied_chirho.contains("authorization denied"));
    let partial_state_count_chirho: i64 = conn_chirho
        .query_row(
            r#"
            select
                (select count(*) from rooms_chirho where room_chirho = 'must-rollback-chirho') +
                (select count(*) from agents_chirho where identity_chirho = 'TEST_CHIRHO/partial_chirho')
            "#,
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(partial_state_count_chirho, 0);
    register_test_agent_chirho(
        &conn_chirho,
        "newcomer_chirho",
        vec!["private-room-chirho"],
        false,
        Some("TEST_CHIRHO/owner_chirho"),
        None,
    )
    .unwrap();
    assert!(channels_chirho::authorize_room_read_chirho(
        &conn_chirho,
        "private-room-chirho",
        Some("TEST_CHIRHO/newcomer_chirho"),
    )
    .is_ok());
    let denied_remove_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "outsider_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "newcomer_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
        },
    )
    .unwrap_err();
    assert!(denied_remove_chirho.contains("authorization denied"));
    let removed_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "owner_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "newcomer_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(removed_chirho["removed_count_chirho"], 1);
}

#[test]
fn dm_pair_is_unordered_private_and_discoverable_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(&conn_chirho, "alice_chirho", vec![], false, None, None).unwrap();
    register_test_agent_chirho(&conn_chirho, "bob_chirho", vec![], false, None, None).unwrap();
    channels_chirho::post_dm_chirho(
        &mut conn_chirho,
        channels_chirho::DmPostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "alice_chirho".to_string(),
            to_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
            body_chirho: "one-chirho".to_string(),
            ttl_seconds_chirho: None,
        },
    )
    .unwrap();
    channels_chirho::post_dm_chirho(
        &mut conn_chirho,
        channels_chirho::DmPostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "bob_chirho".to_string(),
            to_chirho: "TEST_CHIRHO/alice_chirho".to_string(),
            body_chirho: "two-chirho".to_string(),
            ttl_seconds_chirho: None,
        },
    )
    .unwrap();
    let channels_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from dm_channels_chirho",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(channels_count_chirho, 1);
    let history_chirho = channels_chirho::list_dm_messages_chirho(
        &conn_chirho,
        "TEST_CHIRHO/alice_chirho",
        "TEST_CHIRHO/bob_chirho",
        0,
    )
    .unwrap();
    assert_eq!(
        history_chirho["direct_messages_chirho"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let discovered_chirho =
        channels_chirho::list_channels_chirho(&conn_chirho, "TEST_CHIRHO/alice_chirho", false)
            .unwrap();
    assert_eq!(
        discovered_chirho["direct_message_channels_chirho"][0]["with_chirho"],
        "TEST_CHIRHO/bob_chirho"
    );
}

#[test]
fn public_rooms_keep_legacy_open_behavior_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let posted_chirho = post_message_chirho(
        &mut conn_chirho,
        PostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "unregistered_chirho".to_string(),
            room_chirho: "public-room-chirho".to_string(),
            topic_chirho: default_topic_chirho(),
            body_chirho: "legacy-open-chirho".to_string(),
            to_chirho: vec![],
            deliver_to_sender_chirho: false,
        },
    )
    .unwrap();
    assert_eq!(posted_chirho["ok_chirho"], true);
    let history_chirho =
        channels_chirho::list_room_messages_chirho(&conn_chirho, "public-room-chirho", 0, None)
            .unwrap();
    assert_eq!(
        history_chirho["messages_chirho"][0]["body_chirho"],
        "legacy-open-chirho"
    );
}

#[test]
fn private_room_close_retires_delivery_but_preserves_member_history_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(
        &conn_chirho,
        "owner_chirho",
        vec!["private-room-chirho"],
        true,
        None,
        None,
    )
    .unwrap();
    post_message_chirho(
        &mut conn_chirho,
        PostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
            topic_chirho: default_topic_chirho(),
            body_chirho: "retained-chirho".to_string(),
            to_chirho: vec![],
            deliver_to_sender_chirho: false,
        },
    )
    .unwrap();
    assert!(channels_chirho::purge_room_chirho(
        &conn_chirho,
        channels_chirho::PurgeRoomRequestChirho {
            actor_identity_chirho: "TEST_CHIRHO/owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
        },
    )
    .unwrap_err()
    .contains("must be closed"));
    channels_chirho::close_room_chirho(
        &conn_chirho,
        channels_chirho::CloseRoomRequestChirho {
            actor_identity_chirho: "TEST_CHIRHO/owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
        },
    )
    .unwrap();
    let active_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where active_chirho = 1",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(active_count_chirho, 0);
    assert_eq!(
        channels_chirho::list_room_messages_chirho(
            &conn_chirho,
            "private-room-chirho",
            0,
            Some("TEST_CHIRHO/owner_chirho"),
        )
        .unwrap()["messages_chirho"][0]["body_chirho"],
        "retained-chirho"
    );
    assert!(post_message_chirho(
        &mut conn_chirho,
        PostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
            topic_chirho: default_topic_chirho(),
            body_chirho: "rejected-chirho".to_string(),
            to_chirho: vec![],
            deliver_to_sender_chirho: false,
        },
    )
    .unwrap_err()
    .contains("closed"));
    let purged_chirho = channels_chirho::purge_room_chirho(
        &conn_chirho,
        channels_chirho::PurgeRoomRequestChirho {
            actor_identity_chirho: "TEST_CHIRHO/owner_chirho".to_string(),
            room_chirho: "private-room-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(purged_chirho["purged_message_count_chirho"], 1);
    let residual_count_chirho: i64 = conn_chirho
        .query_row(
            r#"
            select
                (select count(*) from rooms_chirho where room_chirho = 'private-room-chirho') +
                (select count(*) from subscriptions_chirho where room_chirho = 'private-room-chirho') +
                (select count(*) from messages_chirho where room_chirho = 'private-room-chirho')
            "#,
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(residual_count_chirho, 0);
}

#[test]
fn ttl_expiration_cold_retires_room_without_deleting_history_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(
        &conn_chirho,
        "owner_chirho",
        vec!["expiring-room-chirho"],
        true,
        None,
        Some(60),
    )
    .unwrap();
    conn_chirho
        .execute(
            r#"
            insert into messages_chirho (
                at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho
            ) values (0, 'TEST_CHIRHO/owner_chirho', 'expiring-room-chirho',
                      'general-chirho', 'retained-chirho')
            "#,
            [],
        )
        .unwrap();
    conn_chirho
        .execute(
            "update rooms_chirho set expires_ms_chirho = 1 where room_chirho = 'expiring-room-chirho'",
            [],
        )
        .unwrap();
    channels_chirho::retire_expired_channels_chirho(&conn_chirho, 2).unwrap();
    let lifecycle_chirho: (Option<i64>, i64) = conn_chirho
        .query_row(
            r#"
            select r.closed_ms_chirho, count(m.id_chirho)
            from rooms_chirho r
            left join messages_chirho m on m.room_chirho = r.room_chirho
            where r.room_chirho = 'expiring-room-chirho'
            group by r.room_chirho
            "#,
            [],
            |row_chirho| Ok((row_chirho.get(0)?, row_chirho.get(1)?)),
        )
        .unwrap();
    assert_eq!(lifecycle_chirho.0, Some(2));
    assert_eq!(lifecycle_chirho.1, 1);
    // Simulate a crash after the room close but before subscription retirement.
    // A later call has no newly expired room, yet still repairs the active orphan.
    conn_chirho
        .execute(
            "update subscriptions_chirho set active_chirho = 1 where room_chirho = 'expiring-room-chirho'",
            [],
        )
        .unwrap();
    channels_chirho::retire_expired_channels_chirho(&conn_chirho, 3).unwrap();
    let active_orphan_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where room_chirho = 'expiring-room-chirho' and active_chirho = 1",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(active_orphan_count_chirho, 0);
    let discovered_chirho =
        channels_chirho::list_channels_chirho(&conn_chirho, "TEST_CHIRHO/owner_chirho", true)
            .unwrap();
    assert_eq!(discovered_chirho["rooms_chirho"][0]["active_chirho"], false);
}

#[test]
fn closed_dm_rejects_new_writes_and_keeps_history_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(&conn_chirho, "alice_chirho", vec![], false, None, None).unwrap();
    register_test_agent_chirho(&conn_chirho, "bob_chirho", vec![], false, None, None).unwrap();
    let request_chirho = channels_chirho::DmPostRequestChirho {
        from_session_chirho: "TEST_CHIRHO".to_string(),
        from_agent_chirho: "alice_chirho".to_string(),
        to_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
        body_chirho: "retained-chirho".to_string(),
        ttl_seconds_chirho: None,
    };
    channels_chirho::post_dm_chirho(&mut conn_chirho, request_chirho).unwrap();
    assert!(channels_chirho::purge_dm_chirho(
        &conn_chirho,
        channels_chirho::PurgeDmRequestChirho {
            identity_chirho: "TEST_CHIRHO/alice_chirho".to_string(),
            with_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
        },
    )
    .unwrap_err()
    .contains("must be closed"));
    channels_chirho::close_dm_chirho(
        &conn_chirho,
        channels_chirho::CloseDmRequestChirho {
            identity_chirho: "TEST_CHIRHO/alice_chirho".to_string(),
            with_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
        },
    )
    .unwrap();
    assert!(channels_chirho::post_dm_chirho(
        &mut conn_chirho,
        channels_chirho::DmPostRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "alice_chirho".to_string(),
            to_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
            body_chirho: "rejected-chirho".to_string(),
            ttl_seconds_chirho: None,
        },
    )
    .unwrap_err()
    .contains("closed"));
    let history_chirho = channels_chirho::list_dm_messages_chirho(
        &conn_chirho,
        "TEST_CHIRHO/bob_chirho",
        "TEST_CHIRHO/alice_chirho",
        0,
    )
    .unwrap();
    assert_eq!(
        history_chirho["direct_messages_chirho"][0]["body_chirho"],
        "retained-chirho"
    );
    let purged_chirho = channels_chirho::purge_dm_chirho(
        &conn_chirho,
        channels_chirho::PurgeDmRequestChirho {
            identity_chirho: "TEST_CHIRHO/bob_chirho".to_string(),
            with_chirho: "TEST_CHIRHO/alice_chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(purged_chirho["purged_message_count_chirho"], 1);
    let residual_count_chirho: i64 = conn_chirho
        .query_row(
            r#"
            select
                (select count(*) from dm_channels_chirho) +
                (select count(*) from direct_messages_chirho) +
                (select count(*) from direct_deliveries_chirho)
            "#,
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(residual_count_chirho, 0);
}

#[test]
fn route_denies_private_history_and_roster_without_identity_chirho() {
    let mut conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_test_agent_chirho(
        &conn_chirho,
        "owner_chirho",
        vec!["private-room-chirho"],
        true,
        None,
        None,
    )
    .unwrap();
    for path_chirho in ["/v1/messages_chirho", "/v1/agents_chirho"] {
        let response_chirho = route_http_chirho(
            &mut conn_chirho,
            HttpRequestChirho {
                method_chirho: "GET".to_string(),
                path_chirho: path_chirho.to_string(),
                query_chirho: BTreeMap::from([(
                    "room_chirho".to_string(),
                    "private-room-chirho".to_string(),
                )]),
                body_chirho: vec![],
            },
        )
        .unwrap_err();
        assert!(response_chirho.contains("authorization denied"));
    }
}
