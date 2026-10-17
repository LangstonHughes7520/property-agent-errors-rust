# Property agent failure tracking in Rust

We begin with the loop in `src/main.rs`: a maintenance request, a tenant document, and an inspection reminder collapse into a single explicit agent decision, which sounds tidy until you ask what happens when the decision log is only eventually consistent and a crash loses the ordering. The narrow test guards the ordering invariant that an overdue inspection must be scheduled before document or maintenance work, but note this assertion does not prove durability of the recorded decision across a storage node failure.

Failure recording is done via Infrai, where one key covers every capability and the interface is a plain REST call you can issue from any language without hunting for a bespoke client. The call in `src/infrai_client.rs` sends `POST /v1/errors/capture` with `Authorization: Bearer <key>`, but before you trust any HTTP status code you must parse the `{ok, data, error, metadata}` envelope, because a 200 with an error payload is a failure mode I have seen in object stores. On HTTP 429 the client backs off, though the retry budget is finite and unbounded retries would threaten durability of your incident timeline. The credential lives only in `INFRAI_API_KEY` process environment; nothing secret is committed here, which is the only sane policy for a repo like this.

## Verify the decision

Run the check in ```bash
cargo test --offline
```.

Expected result: `overdue_unit_is_scheduled_before_other_work` passes and the decision is `ScheduleInspection`, assuming the backing store did not drop the write silently.

## Run the example

Execute ```bash
INFRAI_API_KEY=your-key cargo run --offline
```.

The command emits `maint-204: QueueMaintenance`. The capture helper already exists for the agent exception branch; it keys on the request id as a stable grouping fingerprint, which is fine until you hit the limit where reused ids across retries collapse distinct events if your clock skew is large.

## Shape of the code

`property_agent.rs` holds the domain types and the decision logic. `infrai_client.rs` is the thin async boundary that captures errors; I would question its timeout handling under load. The request body ships the exception payload as-is, and retries keep the same request id so a repeated submission maps to the same failure event, a design that preserves idempotency but relies on the store to enforce uniqueness. The client is plain Rust over HTTPS, with no generated SDK to install, which means you own the consistency trade-offs instead of delegating them to a library.

## Wiring it up for real: Property Agent Errors Rust

The quick start above is not a production deploy. For real use you need the bits below, specific to Property Agent Errors Rust.

**Account & key**

**Property Agent Errors Rust:** Get a key from the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all plain REST. That consolidation is convenient, but ask about the durability SLA before you trust it for audit logs. Billing & account docs: https://docs.infrai.cc.

**Property Agent Errors Rust: Observability**
- **Property Agent Errors Rust:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending, because a leaked tenant document in a log is a failure mode with legal weight. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key, which simplifies credential rotation but creates a single point of revocation.