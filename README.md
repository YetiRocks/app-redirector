<p align="center">
  <img src="https://cdn.prod.website-files.com/68e09cef90d613c94c3671c0/697e805a9246c7e090054706_logo_horizontal_grey.png" alt="Yeti" width="200" />
</p>

---

# app-redirector

[![Yeti](https://img.shields.io/badge/Yeti-Application-blue)](https://yetirocks.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **[Yeti](https://yetirocks.com)** - The Performance Platform for Agent-Driven Development.
> Schema-driven APIs, real-time streaming, and vector search. From prompt to production.

**URL redirect management at the edge.** Static paths, regex patterns, host-scoped rules, versioned cutovers, time windows, and bulk import — all in a single yeti application.

App-redirector gives your infrastructure a programmable redirect layer with zero external dependencies. Define rules as static paths or regex patterns, scope them to specific hosts and time windows, group them into versions for atomic activation, and bulk-import thousands of rules from CSV or JSON. The built-in `/r/` endpoint performs actual HTTP redirects with correct status codes and cache headers, ready for edge workers, CDN origin rules, or direct browser use.

---

## Why app-redirector

Managing URL redirects across a real production estate means juggling nginx configs, CDN edge rules, server-side middleware, and spreadsheets of URLs from the SEO team. Every migration, rebrand, or seasonal campaign adds another batch. The typical approach — a flat file or database table with no versioning, no time windows, and no host scoping — breaks down fast.

App-redirector collapses all of that into a single application:

- **Static and regex rules** — exact path matches for simple redirects, regex patterns for dynamic URL structures. Both evaluated in priority order with host and version scoping.
- **Host-specific configurations** — different redirect rules for different domains, each with its own active version, fallback URL, and enable/disable toggle.
- **Versioned rule sets** — group rules into versions and activate them atomically. Test a new set of redirects before going live, roll back instantly.
- **Time-windowed activation** — rules with `utcStartTime` and `utcEndTime` activate and deactivate automatically. Seasonal campaigns, product launches, and temporary promotions without manual intervention.
- **Bulk CSV/JSON upload** — import thousands of rules from a spreadsheet export or JSON array. Composite key deduplication prevents duplicates on re-import.
- **Edge-ready redirect endpoint** — `/r/{path}` performs actual HTTP 301/302/307/308 redirects with correct `Cache-Control` headers. Point a CDN origin rule at it or use it directly.
- **Analytics endpoint** — rule counts by host and status code, request metadata, and multiple serialization formats (JSON, MessagePack, CBOR).
- **React management UI** — built-in web interface for browsing, searching, and managing redirect rules.

---

## Quick Start

### 1. Install

```bash
cd ~/yeti/applications
git clone https://github.com/yetirocks/app-redirector.git
```

Restart yeti. App-redirector compiles automatically on first load (~2 minutes) and is cached for subsequent starts (~10 seconds). Sample rules are loaded from `data/sample-rules.json` on first boot.

### 2. Check a redirect rule

```bash
curl "https://localhost:9996/app-redirector/api/checkredirect/old-page"
```

Response:
```json
{
  "path": "/old-page",
  "host": "",
  "redirectURL": "/new-page",
  "statusCode": 301,
  "version": 0,
  "regex": false,
  "utcStartTime": null,
  "utcEndTime": null
}
```

The check endpoint resolves a path against stored rules and returns the matching rule as JSON — useful for edge worker integration, debugging, and testing.

### 3. Perform an actual redirect

```bash
curl -v "https://localhost:9996/app-redirector/api/r/old-page"
```

Response:
```
< HTTP/2 301
< location: /new-page
< cache-control: public, max-age=31536000
```

The `/r/` endpoint issues a real HTTP redirect. Permanent redirects (301, 308) include a one-year cache header; temporary redirects (302, 307) use `no-cache`.

### 4. Upload rules in bulk

```bash
curl -X POST https://localhost:9996/app-redirector/api/redirectupload \
  -H "Content-Type: application/json" \
  -d '[
    {
      "path": "/legacy/docs",
      "redirectURL": "/documentation",
      "statusCode": 301,
      "host": "",
      "version": 0
    },
    {
      "path": "/legacy/api",
      "redirectURL": "/developers/api",
      "statusCode": 301,
      "host": "",
      "version": 0
    }
  ]'
```

Response:
```json
{
  "message": "Successfully loaded",
  "created": 2,
  "updated": 0,
  "skipped": 0,
  "errors": 0
}
```

Re-uploading the same rules updates existing records (composite key dedup on version + host + path). CSV upload is also supported with `Content-Type: text/csv`.

### 5. Check analytics

```bash
curl "https://localhost:9996/app-redirector/api/redirectmetrics"
```

Response:
```json
{
  "totalRules": 11,
  "activeRules": 11,
  "byHost": {
    "": 10,
    "example.com": 1
  },
  "byStatusCode": {
    "301": 8,
    "302": 3
  },
  "_meta": {
    "requestId": "a1b2c3d4",
    "clientIp": "127.0.0.1"
  }
}
```

### 6. Use host-scoped and versioned lookups

```bash
# Check a host-specific rule
curl "https://localhost:9996/app-redirector/api/checkredirect/contact-us?h=example.com"

# Check a versioned rule
curl "https://localhost:9996/app-redirector/api/checkredirect/versioned-page?v=1"

# Perform a versioned redirect
curl -v "https://localhost:9996/app-redirector/api/r/versioned-page?v=1"
```

### 7. Stream rule changes in real-time

```bash
# SSE stream -- get notified when rules change
curl "https://localhost:9996/app-redirector/api/rule?stream=sse"

# MQTT -- subscribe to rule changes
mosquitto_sub -t "app-redirector/rule" -h localhost -p 8883
```

---

## Architecture

```
CDN / Edge Workers / Browsers / API Clients
    |
    +-- REST / curl --------> app-redirector (schema-driven endpoints)
    +-- SSE ----------------> app-redirector (real-time rule changes)
    +-- MQTT pub/sub -------> app-redirector (native broker)
          |
          v
    +-------------------------------------------------------+
    |                    app-redirector                      |
    |                                                       |
    |  Custom Resources:                                    |
    |  +----------------+  +------------------+             |
    |  | checkredirect  |  | perform redirect |             |
    |  | (GET: resolve) |  | (GET: /r/{path}) |             |
    |  +----------------+  +------------------+             |
    |  +----------------+  +------------------+             |
    |  | redirectupload |  | redirectmetrics  |             |
    |  | (POST: bulk)   |  | (GET/POST/PUT/   |            |
    |  |                |  |  PATCH: analytics)|            |
    |  +----------------+  +------------------+             |
    |                                                       |
    |  Schema Tables (auto-generated CRUD + SSE + MQTT):    |
    |  +--------+  +---------+  +-----------+               |
    |  |  Rule  |  |  Hosts  |  |  Version  |               |
    |  +--------+  +---------+  +-----------+               |
    |                                                       |
    |  check -> normalize -> query rules -> match -> JSON   |
    |  redirect -> normalize -> query rules -> 3xx + cache  |
    |  upload -> parse CSV/JSON -> dedup -> bulk upsert     |
    |  metrics -> scan rules -> aggregate by host/status    |
    +-------------------------------------------------------+
          |
          v
    Yeti (embedded RocksDB, MQTT broker, SSE)
```

**Check path:** Request with URL -> normalize path -> apply query string mode -> scan rules by version + host + path -> return matching rule as JSON (or null).

**Redirect path:** Request to `/r/{path}` -> normalize path -> scan rules by version + host -> issue HTTP redirect with status code + cache header. Permanent (301/308) gets one-year cache; temporary (302/307) gets no-cache.

**Upload path:** CSV or JSON body -> parse into records -> generate composite keys (version + host + path) -> bulk upsert with dedup -> return created/updated/skipped counts.

**Metrics path:** Scan all rules -> aggregate by host and status code -> return summary with request metadata.

---

## Features

### Check Redirect (GET /app-redirector/api/checkredirect/{path})

Resolve a URL path against stored redirect rules and return the matching rule as JSON. Designed for edge worker integration where you need the redirect metadata without performing the actual redirect.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `{path}` or `url` | String (required) | -- | The path to check (as path ID or `url` query param) |
| `h` | String | `""` | Host filter (empty = global rules only) |
| `v` | Integer | `0` | Version number |
| `qs` | String | `"m"` | Query string mode: `"m"` = match (include query string), `"i"` = ignore |

**Resolution logic:**
1. Path is normalized (lowercase, leading slash ensured)
2. Query string mode is applied (strip or preserve query params)
3. Rules are scanned with version and host filters
4. First matching rule is returned (static match first, then regex patterns)
5. Returns `null` if no rule matches

```bash
# Basic check
curl "https://localhost:9996/app-redirector/api/checkredirect/old-page"

# With host filter
curl "https://localhost:9996/app-redirector/api/checkredirect/contact-us?h=example.com"

# Ignore query string during matching
curl "https://localhost:9996/app-redirector/api/checkredirect/page?foo=bar&qs=i"

# Using url query param instead of path
curl "https://localhost:9996/app-redirector/api/checkredirect?url=/old-page"
```

### Perform Redirect (GET /app-redirector/api/r/{path})

Execute an actual HTTP redirect based on stored rules. Returns a 3xx response with `Location` header and appropriate `Cache-Control`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `{path}` | String (required) | -- | The path to redirect (everything after `/r/`) |
| `h` | String | `""` | Host filter |
| `v` | Integer | `0` | Version number |

**Cache behavior:**

| Status Code | Meaning | Cache-Control |
|-------------|---------|---------------|
| 301 | Moved Permanently | `public, max-age=31536000` (1 year) |
| 302 | Found (Temporary) | `no-cache` |
| 307 | Temporary Redirect | `no-cache` |
| 308 | Permanent Redirect | `public, max-age=31536000` (1 year) |

```bash
# Basic redirect
curl -v "https://localhost:9996/app-redirector/api/r/old-page"

# Host-scoped redirect
curl -v "https://localhost:9996/app-redirector/api/r/contact-us?h=example.com"

# Versioned redirect
curl -v "https://localhost:9996/app-redirector/api/r/versioned-page?v=1"
```

Returns 404 if no matching rule is found, or if the matching rule's time window has not started or has expired.

### Redirect Upload (POST /app-redirector/api/redirectupload)

Bulk import redirect rules from CSV or JSON. Uses composite key deduplication (version + host + path) to prevent duplicates on re-import and update existing rules.

| Content-Type | Format | Description |
|-------------|--------|-------------|
| `application/json` | JSON array | Array of rule objects (or single object) |
| `text/csv` | CSV | Standard CSV with header row |

**JSON fields per rule:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `path` | String | Yes | -- | URL path to match |
| `redirectURL` | String | Yes | -- | Target URL |
| `statusCode` | Integer | No | `301` | HTTP status code |
| `host` | String | No | `""` | Host filter (empty = global) |
| `version` | Integer | No | `0` | Version number |
| `regex` | Boolean | No | `false` | Whether path is a regex pattern |

```bash
# JSON upload
curl -X POST https://localhost:9996/app-redirector/api/redirectupload \
  -H "Content-Type: application/json" \
  -d '[
    {"path": "/old", "redirectURL": "/new", "statusCode": 301},
    {"path": "/temp", "redirectURL": "/landing", "statusCode": 302}
  ]'

# CSV upload
curl -X POST https://localhost:9996/app-redirector/api/redirectupload \
  -H "Content-Type: text/csv" \
  -d 'path,redirectURL,statusCode,host,version,regex
/old,/new,301,,0,false
/temp,/landing,302,,0,false'
```

**Deduplication:** Composite key is generated from `version || host || path`. Re-uploading a rule with the same composite key updates the existing record instead of creating a duplicate.

### Redirect Metrics (GET /app-redirector/api/redirectmetrics)

Analytics and diagnostics for redirect rules. Supports multiple HTTP methods and serialization formats.

| Method | Description | Response Format |
|--------|-------------|-----------------|
| GET | Full analytics: rule counts by host and status code | JSON |
| POST | Summary: total rules with request metadata | JSON |
| PUT | Format demo | MessagePack |
| PATCH | Format demo | CBOR (RFC 8949) |

**GET response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `totalRules` | Integer | Total number of redirect rules |
| `activeRules` | Integer | Number of active rules |
| `byHost` | Object | Rule count per host (empty string = global) |
| `byStatusCode` | Object | Rule count per HTTP status code |
| `_meta.requestId` | String | Unique request identifier |
| `_meta.clientIp` | String | Client IP address |

```bash
# Full analytics
curl "https://localhost:9996/app-redirector/api/redirectmetrics"

# Summary
curl -X POST "https://localhost:9996/app-redirector/api/redirectmetrics"

# MessagePack format
curl -X PUT "https://localhost:9996/app-redirector/api/redirectmetrics" --output metrics.msgpack

# CBOR format
curl -X PATCH "https://localhost:9996/app-redirector/api/redirectmetrics" --output metrics.cbor
```

### REST CRUD (auto-generated)

Full CRUD on all tables is auto-generated from the schema:

| Endpoint | Methods | Description |
|----------|---------|-------------|
| `/app-redirector/api/rule` | GET, POST | List/create rules |
| `/app-redirector/api/rule/{id}` | GET, PUT, DELETE | Read/update/delete a rule |
| `/app-redirector/api/hosts` | GET, POST | List/create host configs |
| `/app-redirector/api/hosts/{id}` | GET, PUT, DELETE | Read/update/delete a host config |
| `/app-redirector/api/version` | GET, POST | List/create versions |
| `/app-redirector/api/version/{id}` | GET, PUT, DELETE | Read/update/delete a version |

### Real-Time Streaming (auto-generated)

Real-time updates are built into the platform via `@export(sse: true, mqtt: true)`:

```bash
# SSE -- server-sent events
GET /app-redirector/api/rule?stream=sse
GET /app-redirector/api/hosts?stream=sse
GET /app-redirector/api/version?stream=sse

# MQTT -- subscribe to changes
mosquitto_sub -t "app-redirector/rule" -h localhost -p 8883
mosquitto_sub -t "app-redirector/hosts" -h localhost -p 8883
mosquitto_sub -t "app-redirector/version" -h localhost -p 8883
```

When a rule is created, updated, or deleted via any method (REST, upload, or direct table write), every subscribed client receives the change immediately.

### MCP Tools (auto-generated)

MCP tools for table operations are auto-generated from `@export` schemas. Any MCP-compatible agent (Claude Code, Cursor, Windsurf) can discover and use them via the standard MCP protocol at `POST /app-redirector/api/mcp`.

---

## Data Model

### Rule Table

| Field | Type | Public | Description |
|-------|------|--------|-------------|
| `id` | ID! | Primary key | Composite key: `{version}\|\|{host}\|\|{path}` |
| `staticPath` | String | -- | Static path to match (e.g., `/old-page`) |
| `regexPattern` | String | -- | Regex pattern for dynamic matching |
| `targetUrl` | String! | -- | Target URL for redirect |
| `statusCode` | Int! | -- | HTTP status code (301, 302, 307, 308) |
| `queryStringOp` | String | -- | Query string operation: `preserve`, `ignore`, `filter`, `append` |
| `preserveParams` | String | -- | Query parameters to preserve (when `queryStringOp = filter`) |
| `appendParams` | String | -- | Query parameters to append to target URL |
| `utcStartTime` | String | -- | UTC start time for rule activation (ISO 8601) |
| `utcEndTime` | String | -- | UTC end time for rule deactivation (ISO 8601) |
| `host` | String | -- | Host filter (empty = global rule) |
| `version` | String | -- | Version identifier for mass cutover |
| `priority` | Int | -- | Priority for rule matching (lower = higher priority) |
| `active` | Boolean! | -- | Rule is active |
| `description` | String | -- | Description/notes |
| `createdAt` | String! | -- | Created timestamp |
| `updatedAt` | String! | -- | Last updated timestamp |

Public access: `read` (no authentication required for GET requests).

### Hosts Table

| Field | Type | Public | Description |
|-------|------|--------|-------------|
| `id` | ID! | Primary key | Hostname (e.g., `example.com`) |
| `activeVersion` | String | -- | Active version for this host |
| `enabled` | Boolean! | -- | Host is enabled for redirects |
| `fallbackUrl` | String | -- | Fallback URL for unmatched paths |
| `description` | String | -- | Description/notes |
| `createdAt` | String! | -- | Created timestamp |
| `updatedAt` | String! | -- | Last updated timestamp |

Public access: `read` (no authentication required for GET requests).

### Version Table

| Field | Type | Public | Description |
|-------|------|--------|-------------|
| `id` | ID! | Primary key | Version identifier (e.g., `v1`, `v2`, `prod-2024-01`) |
| `name` | String! | -- | Version name/label |
| `active` | Boolean! | -- | Version is active globally |
| `activatedAt` | String | -- | UTC activation time (ISO 8601) |
| `description` | String | -- | Description/notes |
| `createdAt` | String! | -- | Created timestamp |
| `updatedAt` | String! | -- | Last updated timestamp |

Public access: `read` (no authentication required for GET requests).

---

## Configuration

### Cargo.toml

App configuration lives under `[package.metadata.app]` in `Cargo.toml` (no more `config.yaml` / `services.yaml`):

```toml
[package.metadata.app]
schemas = "schemas/redirect.graphql"
resources = "resources/*.rs"
data_loader = "data/*.json"
static = { path = "web", source = "source", spa = true, build = "npm install && npm run build" }
```

**Key configuration notes:**

| Key | Description |
|-----|-------------|
| `schemas` | GraphQL schema files defining Rule, Hosts, and Version tables |
| `resources` | Rust resource files compiled into native plugins |
| `data_loader` | Seed data loaded on first boot (sample redirect rules) |
| `static.spa` | SPA mode: unmatched routes return `index.html` with 200 |
| `static.build` | Auto-builds frontend from `source/` directory |

### Seed Data

Sample rules are loaded from `data/sample-rules.json` on first boot, including:

- Basic static path redirects (`/old-page` -> `/new-page`)
- Blog migration redirects (`/blog/post-1` -> `/articles/welcome`)
- Host-specific redirects (`/contact-us` on `example.com`)
- Time-windowed rules (seasonal sale with start/end dates)
- Versioned rules (version 1 redirect set)
- External redirects (cross-domain URLs)

---

## Authentication

App-redirector uses yeti's built-in auth system. To require sign-in for write operations, add a `[package.metadata.auth]` block to `Cargo.toml`:

```toml
[package.metadata.auth]
allow_signup = true
default_role = "admin"

[package.metadata.auth.oauth]
providers = [
  { name = "google", client_id = "${GOOGLE_CLIENT_ID}", client_secret = "${GOOGLE_CLIENT_SECRET}" },
]
```

In development mode (no providers configured), all endpoints are accessible without authentication. In production:

- **Rule, Hosts, and Version tables** allow public `read` access (declared via `@export(public: [read])` in the schema)
- **Write operations** (upload, create, update, delete) require authentication
- **Custom resources** (checkredirect, perform redirect) are read-only and publicly accessible
- **JWT**, **Basic Auth**, and **OAuth** are supported (configured via yeti-auth)

**Frontend gate:** the management UI ships a configurable `src/pages/Login.tsx` and a `src/hooks/useAuth.ts` hook. `App.tsx` calls `useAuth()` and renders `<Login/>` when sign-in is required. If no auth providers are configured, `useAuth` returns `true` and the gate is a no-op.

For multi-domain deployments, use the `host` field on rules to scope redirects per domain and the Hosts table to manage per-domain settings.

---

## Project Structure

```
app-redirector/
├── Cargo.toml               # App + [package.metadata.app] manifest
├── schemas/
│   └── redirect.graphql     # Rule, Hosts, Version tables
├── resources/
│   ├── check_redirect.rs    # Test URL resolution against rules
│   ├── perform_redirect.rs  # Execute redirect with status code
│   ├── redirect_upload.rs   # Bulk CSV/JSON rule import
│   └── redirect_metrics.rs  # Analytics and metrics endpoint
├── data/
│   └── sample-rules.json    # Seed redirect rules
└── source/                  # React/Vite frontend (one-page model)
    ├── index.html
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── App.tsx           # Thin shell — auth gate + management UI
        ├── main.tsx          # Entry point
        ├── api.ts            # Fetch helpers
        ├── types.ts          # Shared TypeScript types
        ├── utils.ts          # Pure helpers (formatters, etc.)
        ├── components/       # Reusable UI
        ├── hooks/            # React hooks (includes useAuth.ts)
        ├── pages/            # Page components (includes Login.tsx)
        └── styles/
            ├── _vars.css     # Per-app brand tokens
            ├── yeti.css      # Canonical Yeti stylesheet
            └── index.css     # App-specific overrides
```

---

## Comparison

| | app-redirector | nginx rewrites | CDN edge rules | Custom middleware |
|---|---|---|---|---|
| **Deployment** | Loads with yeti, zero config | Config files, reload required | Vendor-specific UI/API | Custom code in every service |
| **Versioning** | Built-in version sets, atomic cutover | Manual config management | No native versioning | Custom implementation |
| **Time windows** | Native UTC start/end per rule | Requires scripting (Lua/njs) | Limited or unavailable | Custom implementation |
| **Host scoping** | Per-host rules + host config table | Server blocks, manual | Per-domain config | Custom routing |
| **Bulk import** | CSV/JSON upload with dedup | Not supported | Limited import tools | Custom ETL |
| **Regex support** | Native pattern matching | Native | Vendor-specific syntax | Regex library |
| **Analytics** | Built-in metrics endpoint | Log parsing | Vendor dashboard | Custom instrumentation |
| **Real-time sync** | Native SSE + MQTT from schema | Not supported | Webhooks (if available) | Custom pub/sub |
| **Management UI** | Built-in React SPA | Text editor | Vendor UI | Custom admin panel |
| **Edge integration** | JSON check endpoint for workers | Direct execution | Direct execution | Varies |
| **Auth** | Built-in JWT/Basic/OAuth | Basic auth / IP rules | Vendor IAM | Custom auth |

---

Built with [Yeti](https://yetirocks.com) | The Performance Platform for Agent-Driven Development
