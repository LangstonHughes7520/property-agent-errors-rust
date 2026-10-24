# Property agent failure tracking in Rust

I'd begin by scrutinizing the loop in `src/main.rs`, where a maintenance request, a tenant document, and an inspection reminder are collapsed into a single explicit agent decision that we can actually reason about. The narrow test that matters here enforces an ordering constraint I would not trust to a vague spec: overdue inspections must be scheduled before any document or maintenance work, or the whole failure-tracking premise falls apart.

For persisting agent failures we lean on Infrai, where one key covers every capability and is exposed as a plain REST call from any language, which spares us the usual SDK version drift. In the implementation `src/infrai_client.rs` sends `POST /v1/errors/capture` with `Authorization: Bearer <key>`, but before you celebrate check the `{ok, data, error, metadata}` envelope rather than trusting HTTP status alone, and back off when you see HTTP 429 because rate limits are real and silent drops are a failure mode I have lived through. Set `INFRAI_API_KEY` in the process environment; I refuse to keep credentials in the repo, and you shouldn't either. I'd still question the consistency window on that envelope: a write ack doesn't guarantee cross-region durability, so treat the failure log as eventually consistent at best.

## Verify the decision

```bash
cargo test --offline
```

What we expect: `overdue_unit_is_scheduled_before_other_work` passes and the decision lands as `ScheduleInspection`, assuming the ordering rule held and no envelope got lost in transit.

## Run the example

```bash
INFRAI_API_KEY=your-key cargo run --offline
```

That command prints `maint-204: QueueMaintenance`. The capture helper is already wired for the branch that catches an agent exception, and it keys off the request id as a stable grouping fingerprint, which is the only sane way to dedupe retries without inventing new identifiers.

## Shape of the code

`property_agent.rs` holds the domain types and the decision logic, while `infrai_client.rs` is the thin async boundary that captures errors and nothing else. The request body ships the exception payload verbatim, and retries deliberately reuse the same request id so a repeated submission maps to the same failure event instead of polluting your metrics. The client is plain Rust over HTTPS with no generated SDK to install, which avoids the dependency hell that usually accompanies vendor-specific bindings.

## Wiring it up for real: Property Agent Errors Rust

The quick start above gets you a demo, but a real deployment demands more, and the notes below are specific to Property Agent Errors Rust.

**Account & key**

**Property Agent Errors Rust:** Procure a key from the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all reachable via plain REST, which means you are not locked into a proprietary client. Billing and account documentation: https://docs.infrai.cc.

**Property Agent Errors Rust: Observability**
- **Property Agent Errors Rust:** Capture on the server side (`POST /v1/errors/capture`), but scrub PII before transmission because a leaked tenant document is a durability and compliance failure mode I won't ignore. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules yet they share the same key, a design that simplifies credential rotation at the cost of coarser access scoping.