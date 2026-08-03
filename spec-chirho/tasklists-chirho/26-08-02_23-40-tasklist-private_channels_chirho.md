<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Private Channels Chirho

## Contract

- [x] Preserve existing public-room reads, posts, registration, and delivery behavior.
- [x] Add private-room metadata with member-only history and posting.
- [x] Require an existing private-room member to add or remove another member; permit self-leave.
- [x] Add unordered-pair direct-message channels with `post --dm SESSION/agent` and participant-only reads.
- [x] Add `rooms --mine` discovery including private rooms and direct-message counterparts.
- [x] Add explicit close and optional TTL expiry; expired channels become cold/closed rather than silently deleting history.
- [x] Add explicit participant-authorized purge for closed channels so bounded cleanup is possible without making expiry destructive.
- [x] Keep the threat model honest: access control prevents accidental/nonmember reads through the broker, but localhost callers running as the same OS user can still impersonate an identity until capability authentication is separately introduced.

## Implementation

- [x] Add focused channel schema/migrations and keep `src/main.rs` below 1.5k lines.
- [x] Route private-room authorization, DM post/read, discovery, close, expiry, and purge APIs.
- [x] Add CLI flags/subcommands and make TUI/watch/console identify their reader on room reads.
- [x] Notify agents when added to private rooms and when a DM or channel closure reaches them.
- [x] Add behavioral tests for nonmember denial, participant access, pair-key symmetry, member-governed membership, rollback, closure, expiry, purge, and public-room compatibility.
- [x] Correct tmux liveness so empty successful probes do not mark nonexistent targets active.
- [x] Hide private-room subscription labels from the global central viewer while preserving agent liveness visibility.
- [x] Keep expiry checks read-only when nothing is due and index work to newly expired channels or active recovery orphans.
- [x] Document API/CLI usage and the private-channel workflow Mermaid DAG.

## Landing Gates

- [x] `cargo fmt --check`.
- [x] `cargo test --release` (40 passed after the request-connection regression test).
- [x] `cargo clippy --release --all-targets -- -D warnings`.
- [x] `cargo build --release`.
- [x] Migrate a copy of the live database and smoke-test end-to-end before restarting the live broker.
- [x] Restart the live broker only after the local and copied-database gates pass; verify public rooms plus private rooms, DMs, close, purge, and zero smoke residue.
