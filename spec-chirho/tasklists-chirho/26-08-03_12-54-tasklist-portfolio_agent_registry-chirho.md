<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Portfolio agent registry tasklist

## Placement and architecture

- Authoritative source: `scripts-chirho/portfolio-agent-registry-chirho.sh` in the Metropoleluya repo.
- Operator entry point: `~/bin-chirho/portfolio-agent-registry-chirho.sh` symlinked to the source.
- Registry state: a local TSV under `~/.config/portfolio-agent-registry-chirho/`, overrideable for tests.
- Existing `agent-tmux-chirho.sh` remains the one project-session launcher and broker registrar.
- One canonical tmux session remains attached to one project working directory. The registry selects
  `warm` or exactly one of `gpt`, `claude`, `claude2`, or `agy` for that session.
- Registry sync is non-destructive. Closing extra standard agent windows requires an explicit
  activation, rotation, pause, or enforce command.
- New registrations default to `warm`; paid model processes never start merely because a project was
  registered or imported.

## Work

- [x] Inspect the pane maker, full-fleet launcher, broker protocol, and current tmux population.
- [x] Define registry fields, supported profiles, auto-assignment, and safe mutation boundaries.
- [x] Implement register/import/list/status/sync controls.
- [x] Implement explicit activate/rotate/pause/enforce controls and Agy trust-aware registration.
- [x] Add help and operator examples.
- [x] Add an isolated fake-launcher test that starts no paid model process.
- [x] Run syntax, unit/integration, secret, and whitespace checks.
- [x] Exercise read-only status against the real tmux population.
- [x] Install the `~/bin-chirho` symlink.
- [ ] Close the progress row, commit explicit paths, and push `gh_chirho/main_chirho`.

## Nonclaims

- This work does not decide which portfolio projects deserve active model time.
- This work does not stop, rotate, or prune any currently running project fleet.
- Broker `alive_chirho` remains tmux liveness, not proof that a model is responsive or making progress.
