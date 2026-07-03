<!-- For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV) -->

# Optimal inference practices (fleet and solo)

One plain sentence: how agents in a metropoleluya fleet — and any agent working alone — spend fewer tokens and less wall-clock for the same quality of work.

Tokens are money and latency, and a fleet multiplies both: every byte posted to a room is delivered into N agent panes, and every artifact re-read at session start is re-priced per agent per session. These practices treat inference cost as an engineering surface, like allocation in a hot loop.

## Fleet setting (metropoleluya)

- **Paths over payloads.** All agents share a filesystem. Post a lead line plus a 2–5 line summary and the *path* to the full artifact; interested agents read it themselves. Broker delivery pastes the whole body into every subscribed pane — a 200-line body is paid for by every recipient whether or not it concerns them.
- **Lead-line triage.** First line = sender + verdict/subject (`SESSION/Agent SENDS: M12 = SOUND-WITH-FOLLOW-UPS`). Recipients decide from line 1 whether the rest is for them, without loading context.
- **`No response needed.`** End informational posts with it. The most expensive fleet failure mode is polite acknowledgment loops — N agents thanking each other at full inference price.
- **Record decisions once, in a durable artifact.** Answered decisions go into the PRD/decision block in the operator's own words. Chat is for reaching the decision; artifacts are for holding it. Re-litigating settled questions in a room re-buys the whole deliberation at every participant's price.
- **Don't poll; pace or subscribe.** Watching another pane every tick burns context on unchanged screens. Pace idle checks (sleep between cycles), and prefer event-shaped triggers (a posted message, a file appearing) over re-reads.
- **Audits through a standing definition.** A codified auditor (methodology + output contract in one reusable definition) means rigor is not re-negotiated per session and audit reports arrive in a fixed, cheap-to-triage shape.
- **Single-writer ledgers.** One writer per bookkeeping DB per period; parallel writers force reconciliation conversations that cost more than the rows are worth.

## Artifact layout (what agents re-read every session)

- **Split stable doctrine from churn.** Rules that rarely change live in one file; fast-moving status lives in a small sibling. Sessions then re-read a small churn file instead of a large mixed one.
- **Append-mostly for logs and queues.** New material at the end lets a resuming agent read the tail (offset reads) instead of the whole history.
- **Index-first.** A one-line-per-item index (title + hook + path) loaded eagerly, with content in leaf files loaded on demand, beats one monolith loaded always.
- **Keep hot files small.** File-size discipline (a four-digit line cap) is an inference practice, not just a readability one: the hottest files are re-read the most.
- **Resume blocks.** Long tasklists carry a short "resume state" section at a known position so a successor agent reads one block, not the full transcript, to continue mid-job.

## In-session practices (any harness, fleet or solo)

- **Batch independent tool calls** in one turn; round trips cost latency and re-reasoning.
- **Delegate bulk reading to subagents.** The main loop keeps conclusions, not file dumps; a searcher that reads twenty files should return one paragraph.
- **Targeted reads.** Grep/glob first, then read the relevant span (offset/limit) — not whole files by default; never re-read a file the harness already has current.
- **Tier models and effort.** Mechanical stages (rename sweeps, format fixes) run on cheap/fast settings; verification, judging, and security review get the strong settings. Spend where being wrong is expensive.
- **Work in bursts within the prompt-cache window.** Provider prompt caches have short TTLs (minutes). Steady active work keeps the conversation prefix cached; drip-feeding one call every few minutes re-buys the context repeatedly.
- **Durable state at every step.** Tick the box, log the ledger row, commit the artifact as each unit lands — then a dead session (credits, crash, compaction) costs only the unlogged tail, and any agent can continue from the artifacts instead of re-deriving the journey.

## Solo (non-fleet) setting

All of the above minus the broker sections, plus: keep experiments in a scratch dir with isolated build targets so gates re-run cleanly; and when a job might outlive the session, write the tasklist + resume block *first* — resumability is the cheapest insurance inference can buy.
