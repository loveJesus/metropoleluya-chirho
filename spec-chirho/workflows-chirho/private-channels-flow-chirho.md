<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Private Channels Flow Chirho

```mermaid
flowchart TD
    register_chirho[register --private] --> prepare_chirho[prepare_room_membership_chirho]
    prepare_chirho -->|new| creator_chirho{actor is first member?}
    prepare_chirho -->|existing| member_admin_chirho{actor is active member?}
    creator_chirho -->|yes| room_tx_chirho[(Atomic room + agent + subscription transaction)]
    member_admin_chirho -->|yes| room_tx_chirho
    creator_chirho -->|no| deny_membership_chirho[Reject without partial state]
    member_admin_chirho -->|no| deny_membership_chirho

    room_read_chirho[Transcript or roster read] --> private_check_chirho{Private room?}
    private_check_chirho -->|no| legacy_open_chirho[Preserve public-room behavior]
    private_check_chirho -->|yes| member_read_chirho{Claimed identity is member?}
    member_read_chirho -->|yes| room_history_chirho[(Return scoped room history)]
    member_read_chirho -->|no| deny_read_chirho[Reject read]

    dm_post_chirho[post --dm counterpart] --> pair_chirho[Sort the two identities]
    pair_chirho --> dm_tx_chirho[(Atomic pair channel + direct message)]
    dm_tx_chirho --> dm_delivery_chirho[Deliver only to counterpart pane]
    dm_tx_chirho --> dm_history_chirho[(Separate DM history, never a room message)]
    dm_history_chirho --> discovery_chirho[rooms --mine discovers counterpart]

    close_chirho[Member close or TTL expiry] --> cold_chirho[(Set closed timestamp)]
    cold_chirho --> deactivate_chirho[Deactivate room delivery subscriptions]
    cold_chirho --> retain_chirho[Retain history for participants]
    retain_chirho --> reject_write_chirho[Reject later writes]
    retain_chirho --> explicit_purge_chirho{Participant requests purge?}
    explicit_purge_chirho -->|yes, channel closed| delete_cold_chirho[Atomically delete channel history and membership]
    explicit_purge_chirho -->|no| keep_cold_chirho[Keep recoverable cold history]
```

Public rooms are deliberately backward compatible. Private rooms and DMs use
broker-level identity membership on a localhost, single-user service. This
prevents accidental cross-room reads through the API, but it is not a
cryptographic boundary against another process running as the same OS user:
that process can claim an identity or open the SQLite database directly.

Lifecycle is cold retirement, not deletion. Explicit close and TTL expiry stop
new writes and delivery, preserve participant history, and allow discovery with
`--include-closed`. An explicit participant-authorized purge is the destructive
bounded-growth escape hatch; open channels and automatic expiry cannot purge.

Implementation anchors:

- `src/channels_chirho.rs`: schema migration, room authorization, DMs,
  discovery, closure, and TTL retirement.
- `src/main.rs`: atomic registration, authorized room posting, HTTP routing,
  and CLI dispatch.
- `src/tui_chirho.rs`: supplies the configured identity on room transcript and
  roster reads.
