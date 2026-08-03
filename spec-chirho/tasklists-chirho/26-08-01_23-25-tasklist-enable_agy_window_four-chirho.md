<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. — John 3:16 (KJV) -->

# Enable Agy In Fleet Window 4 Chirho

- [x] Add default-enabled Agy configuration and stable window-4 startup to the fleet launcher.
- [x] Keep broker registration explicitly gated until the operator accepts Agy's per-workspace trust prompt.
- [x] Verify shell syntax and exercise the launcher twice in a disposable tmux session.

Verification:

- `zsh -n /Users/hallelujah/bin-chirho/agent-tmux-chirho.sh`
- Disposable session produced only shell window 2, `Hallelujah-agy` window 4, and metro window 7.
- A second launcher pass created no duplicate windows.
- While Agy displayed its workspace-trust prompt, the broker room had no registered listeners and the workspace remained untrusted.
