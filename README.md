# Nullbox

English | [日本語](README.ja.md)

> ヌルボックス: support, but make it void.

A deliberately small, deliberately silly ticketing system. Customers report problems, workers
answer them, and everyone works toward **Inbox Zero** while the **Backlog Beast** watches.

This is the hackathon's starting point. It is a working product with **no** enterprise features:
no auth, no CI/CD, no observability, no backups. Adding those is your job. The code leaves a clear
seam for each one (see [Extension seams](#extension-seams)).

## Quickstart

Requirements: Rust (pinned in `rust-toolchain.toml`), Node 22 (`.nvmrc`), pnpm via Corepack
(`corepack enable`).

```sh
# API on http://localhost:3000 (creates nullbox.db and runs migrations on start)
cp .env.example .env
cargo run -p nullbox-api

# Web app on http://localhost:5173 (talks to the API cross-origin, no dev proxy)
pnpm install
cp apps/web/.env.example apps/web/.env
pnpm dev
```

Open <http://localhost:5173> to file a ticket, and <http://localhost:5173/inbox> for the worker inbox.

### Dev container

Instead of installing the requirements, open the repo in a [dev container](https://containers.dev/)
(VS Code: **Dev Containers: Reopen in Container**; needs Docker). It brings Rust, Node, pnpm and the
GitHub CLI, installs dependencies and creates both `.env` files. Then start the API and the web app
with `cargo run -p nullbox-api` and `pnpm dev`; ports 3000 and 5173 are forwarded to your machine.

`target/` and `node_modules/` live in Docker volumes, so they're not visible on your machine. A
Rust version change needs `rust-toolchain.toml` only; the version in `devcontainer.json` just avoids
a download. Docker itself isn't included: add the `docker-outside-of-docker` feature if you need it.

GitHub Codespaces works when opened in desktop VS Code. In the browser the app can't reach the
API, because `localhost` means your machine there.

## Checks

```sh
cargo test                 # domain + HTTP tests; also regenerates apps/web/src/api/generated
cargo clippy --all-targets
cargo fmt --check
pnpm test                  # Vitest + Testing Library
pnpm lint                  # Biome
pnpm build                 # typecheck + production bundle in apps/web/dist
```

Nothing enforces these yet. That's one of your upgrades.

## Layout

```
apps/web                     React + Vite SPA (customer portal at /, worker inbox at /inbox)
  src/api/client.ts          the only module that calls fetch
  src/api/generated/         TS types generated from Rust by ts-rs (committed)
  src/copy/                  en.ts, ja.ts and the CopyProvider
  src/router.tsx             every route (switch to createHashRouter for Tauri)
crates/domain                pure types + rules: statuses, transitions, Beast mood. No IO.
crates/api                   axum server, services, sqlx repositories
migrations/                  SQL migrations, embedded and run at startup
.sqlx/                       sqlx offline query cache, so a fresh clone builds without a DB
```

## API

All JSON, under `/api`. Errors are always `{ "code": ErrorCode, "message": string }`: `code` is
stable and shared with TypeScript; `message` is English and for developers only.

| Method | Path | Notes |
|---|---|---|
| POST | `/tickets` | create, returns `{ ticket, access_token, access_link }` |
| GET | `/public/tickets/{id}` | customer view, header `X-Ticket-Token` |
| POST | `/public/tickets/{id}/messages` | customer reply; reopens a resolved ticket |
| GET | `/tickets?status=` | worker inbox; no status = all active tickets |
| GET | `/tickets/{id}` | worker view, includes internal notes |
| POST | `/tickets/{id}/messages` | `{ body, internal? }` |
| PATCH | `/tickets/{id}` | `{ status?, severity?, assignee_name? }` (`""` unassigns) |
| POST | `/tickets/{id}/yeet` | close: sets status `resolved` ("Yeeted") |
| GET | `/stats/backlog` | `{ open_count, beast_mood, oldest_open_age_secs }` |
| GET | `/admin/export` | every ticket, for the CEO's spreadsheet; header `X-Admin-Token` |
| GET | `/healthz` | liveness |
| GET | `/teapot` | 418, as specified by RFC 2324 |

Worker endpoints are **unprotected**: `X-Actor-Name` is whatever the client says it is.

## Configuration

API, from env vars or `.env` (see `.env.example` and `crates/api/src/config.rs`):
`BIND_ADDR`, `DATABASE_URL`, `CORS_ALLOWED_ORIGINS`, `PUBLIC_WEB_URL`, `ADMIN_TOKEN`, `RUST_LOG`.

Web, at build time: `VITE_API_BASE_URL`, the origin of the API without `/api`. Unset, the bundle calls
`http://localhost:3000`, which is only right on your laptop. For a deployment, set it to your site's
address, or to an empty value when one web server serves both the web app and `/api`.

## Extension seams

| Want to add | Start here |
|---|---|
| Login: passwords, SSO, sessions | `Actor` extractor in `crates/api/src/http/extract.rs` (it gets the `AppState`); `authHeaders()` in `apps/web/src/api/client.ts`; for cookies across origins, see `cors()` in `crates/api/src/http/mod.rs` |
| Roles / permissions | `policy::can` in `crates/api/src/policy.rs` (always `true` today) |
| Audit log, webhooks, email, metrics | `EventSink` in `crates/api/src/events.rs` (only logs today); refused requests in the `Actor` extractor |
| Rate limiting, tracing exporters, CORS hardening | `middleware()` in `crates/api/src/http/mod.rs` |
| Postgres, multi-tenancy, backups | `TicketRepository` in `crates/api/src/repo/mod.rs` |
| OpenTelemetry | everything logs through `tracing`; add a layer in `main.rs` |
| Payments | a new route next to the others in `crates/api/src/http/mod.rs`; a page in `apps/web/src/router.tsx`; keys in `Config` |
| MCP server for AI agents | a service at `/mcp`, next to the `/api` nest in `router()`; let its tools call `TicketService`, like the worker handlers do |
| More languages, Serious Mode | add a dictionary of type `Copy` next to `apps/web/src/copy/en.ts` |
| Tauri desktop/mobile | `VITE_API_BASE_URL` + `createHashRouter` in `apps/web/src/router.tsx` |

## Decisions

- **Inbox Zero** means no ticket in `open`, `in_progress` or `waiting_on_customer`. Resolved and
  closed tickets are still reachable through the inbox status filter.
- **Yeet** = close from the worker's point of view: the ticket becomes `resolved` ("Yeeted"). A
  customer reply brings it back to `open` ("IT'S ALIVE!"). `closed` ("Sent to the Shadow Realm")
  is final.
- **Lost secret link**: a known MVP limitation. There is no way to recover it (email would fix that).
- **Name**: "Nullbox" in both languages, with ヌルボックス in the Japanese subtitle.
- **IDs and timestamps** are stored as text (uuid v7, RFC 3339 UTC) and generated in Rust, so the
  schema ports to Postgres with type changes only. Ticket numbers are allocated in the repository.
- `X-Actor-Name` is percent-encoded so non-ASCII names (山田) survive HTTP headers.

## Working on the database

Queries are checked at compile time against `.sqlx/`. After changing a query or migration:

```sh
cargo install sqlx-cli --no-default-features --features sqlite   # once
export DATABASE_URL=sqlite://nullbox.db
sqlx database create && sqlx migrate run
cargo sqlx prepare --workspace     # refresh .sqlx, then commit it
```

## Lore

In the beginning there was the Inbox, and the Inbox was without form, and full.

Every unanswered ticket feeds the **Backlog Beast**. At zero it sleeps, curled up somewhere in
`crates/domain/src/beast.rs`. With a handful of tickets it gets peckish and starts watching you.
Past six it is grumpy and starts to steam. Past twenty it becomes a kaiju, and nobody is safe.
The only known cure is to answer the tickets.

Workers do not close tickets: they **yeet** them. Most yeeted tickets stay gone. Some come back:
a customer replies, and the ticket rises from the dead. Truly finished tickets are sent to the
**Shadow Realm**, from which nothing returns (see `TicketStatus::can_transition_to`).

Ticket #404 exists. Nobody has ever found it.

Some say that if you type ↑ ↑ ↓ ↓ ← → ← → B A, the app turns into Enterprise Software. Those who
have filled in its forms are still waiting for approval.

When the server panics, it logs under the target `EXISTENTIAL`. It is not DNS. It is never DNS.
(It was DNS: see `ApiError::ProbablyDns`.)

### Easter eggs

- Konami code: Enterprise Mode™, with absurd mandatory fields and an eternal "Synergizing…"
- Ticket #404 is shown as "Not Found"
- Typing `sudo` in the inbox search: "This incident will be reported"
- Caps Lock on while replying: "Priority: Loud" (cosmetic, honest)
- Random loading messages, e.g. "Bribing the database…"
- `GET /api/teapot`

## License

[The Unlicense](LICENSE): public domain. Do whatever you want with it.
