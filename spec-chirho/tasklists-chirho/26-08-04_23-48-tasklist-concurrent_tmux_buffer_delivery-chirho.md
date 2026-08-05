<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Concurrent tmux buffer delivery Chirho

- [x] Verify the active pane identity, shared-tree boundary, assignment authority, and live-broker no-restart constraint.
- [x] Claim the transport, focused tests, workflow documentation, tasklist, and progress-log paths with AICEO/GPT.
- [x] Add a deterministic acceptance test that plants the legacy shared-buffer interleaving and proves its cross-target failure.
- [x] Isolate every production delivery in its own tmux buffer and clean buffers on both success and failure.
- [x] Prove concurrent distinct payloads reach only their intended distinct tmux targets while serial and parallel delivery semantics remain intact.
- [x] Update the broker workflow and code linkage for the transport invariant.
- [x] Run formatting, compiler, lint, test, file-size, and dirty-tree gates at the claimed scope.
- [x] Report `READY_FOR_REVIEW` to AICEO/GPT; do not restart or replace the live broker before a reviewed bounded handoff.

## Review follow-up Chirho

- [x] Replace the barrier waits and scoped joins with deadline-backed coordination and result collection.
- [x] Plant missing gate-peer and missing worker-result cases that prove the test harness fails within its bound.
- [x] Serialize each physical pane's complete paste/Enter/settle/Enter transaction with a cold-retiring target lock.
- [x] Plant the unlocked same-pane interleaving and prove repaired concurrent payloads become distinct submissions.
- [x] Prove unrelated physical panes remain independently lockable and the target-lock registry returns to zero.
- [x] Rerun focused and full actual-scope gates while preserving the live broker PID.
- [x] Return a superseding `READY_FOR_REVIEW` to AICEO/GPT without committing or deploying.

## Live handoff receipt Chirho

- [x] Land the reviewed seven-path repair at
  `16bc0e8ade26a7e6e3b85443d2a734ccafc8aa8a`.
- [x] Build the release binary and record SHA-256
  `9e26cbda8d06fda764ae17d76284d3de9629439695c659b021dfa248fd0cfca6`.
- [x] Start the release against a private temporary database on `127.0.0.1:37372`; verify health
  and an empty roster, then stop it and remove the temporary fixture.
- [x] Confirm live PID `98058` owns pane `%2`, has only its listening socket and no established
  request connection immediately before replacement.
- [x] Respawn the same broker pane with the reviewed release; verify healthy listener PID `24725`
  and exact executable path on `127.0.0.1:37371`.
- [x] Prove post-handoff delivery through durable DM `#96`. The public GitHub remote remains
  unchanged; local `main_chirho` is intentionally one reviewed commit ahead.
