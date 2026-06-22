<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Metropoleluya Agent Guide

Metropoleluya is local coordination tooling for tmux-based AI agents. It is not a product runtime and not part of Cairn's core app model.

Rules:

- Keep the broker boring: localhost HTTP, SQLite state, tmux delivery, visible transcript.
- Agents identify as `session_chirho/agent_chirho`, such as `CAIRN_CHIRHO/gpt_chirho`.
- Rooms and topics are routing labels; they must not replace direct L.J. decisions.
- No popups or silent defaults for L.J. decisions.
- Do not log secrets in message bodies.
- Keep source files small and focused.

