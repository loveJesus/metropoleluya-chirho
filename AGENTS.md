<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Metropoleluya Agent Guide

Metropoleluya is local coordination tooling for tmux-based AI agents. It is not an application runtime.

Rules:

- Keep the broker boring: localhost HTTP, SQLite state, tmux delivery, visible transcript.
- Agents identify as `session_chirho/agent_chirho`, such as `PROJECT_CHIRHO/gpt_chirho`.
- Rooms and topics are routing labels; they must not replace direct operator decisions.
- No popups or silent defaults for operator decisions.
- Do not log secrets in message bodies.
- Keep source files small and focused.
