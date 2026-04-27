# Popsicle Sync API

This document is the **stable contract** between an open-source Popsicle CLI
(this repo) and any compatible sync server. The reference implementation is
the closed-source [`popsicle-cloud`](https://github.com/popsicle-lab/popsicle-cloud)
SaaS, but anyone may host a server that conforms to this API.

The CLI must not assume any server URL; the endpoint is configured per-project
in `.popsicle/config.toml` under `[sync]`.

## Versioning

Every request and response carries a `schema_version` integer. The CLI refuses
to sync if the server reports a version it does not know how to handle and
prints an upgrade hint.

Current version: **`1`**.

## Transport

- Base URL: configured by user (e.g. `https://api.popsicle.cloud`).
- TLS required in production; plain HTTP allowed only when host is `localhost`.
- All bodies are JSON unless noted.
- Authenticated endpoints expect `Authorization: Bearer <access_token>`.
- Error responses follow the shape:
  ```json
  { "error": { "code": "string_code", "message": "human readable",
               "details": { } } }
  ```
  with appropriate HTTP status (400/401/403/404/409/422/429/500).

Standard error codes: `unauthenticated`, `permission_denied`, `not_found`,
`conflict`, `validation_failed`, `rate_limited`, `schema_version_mismatch`,
`internal`.

## Authentication

### `POST /v1/auth/register`
Body: `{ "email": "...", "password": "..." }`
→ `{ "user": User, "access_token": "...", "refresh_token": "...",
     "access_expires_at": "RFC3339", "refresh_expires_at": "RFC3339" }`

### `POST /v1/auth/login`
Body: `{ "email": "...", "password": "..." }` → same shape as register.

### `POST /v1/auth/refresh`
Body: `{ "refresh_token": "..." }` →
`{ "access_token": "...", "access_expires_at": "RFC3339" }`

### `POST /v1/auth/logout`
Body: `{ "refresh_token": "..." }` (revokes that refresh token) → `204`.

### `GET /v1/me`
→ `User`.

### `POST /v1/me/password`
Body: `{ "current_password": "...", "new_password": "..." }` → `204`.

### `DELETE /v1/me`
Body: `{ "password": "..." }` — hard-delete the account and all data → `204`.

### Types
```jsonc
User = {
  "id": "uuid",
  "email": "string",
  "created_at": "RFC3339",
  "schema_version": 1
}
```

## Sync model

Each authenticated user has one **personal workspace**. Inside that workspace
the entity hierarchy mirrors the local on-disk layout:

`Workspace → Namespace → Spec → Issue → PipelineRun → Document`

Plus standalone entities under a Spec: `Bug`, `UserStory`, `TestCase`.

Every entity has:
- `id` — UUID, stable across sync.
- `version` — monotonically increasing integer assigned by the server.
- `updated_at` — RFC3339 server-side timestamp.
- `deleted` — boolean tombstone flag (entries are soft-deleted on the server
  for at least 30 days so clients can observe deletions).

## Listing & cursors

### `GET /v1/sync/changes?since=<version>&limit=<n>`

Returns all entity changes (any kind) with `version > since`, ordered by
`version` ascending. Used by the CLI daemon for incremental pulls.

```jsonc
{
  "schema_version": 1,
  "changes": [
    {
      "kind": "document",
      "id": "uuid",
      "version": 42,
      "deleted": false,
      "payload": { /* kind-specific body */ }
    }
  ],
  "next_since": 42,
  "has_more": false
}
```

## Push

### `POST /v1/sync/push`

Bulk upsert from the client. Payload:

```jsonc
{
  "schema_version": 1,
  "client_id": "uuid",        // stable per CLI install
  "operations": [
    {
      "kind": "document",
      "id": "uuid",
      "base_version": 41,     // the version the client last synced
      "deleted": false,
      "payload": { ... }
    }
  ]
}
```

Response:

```jsonc
{
  "results": [
    {
      "id": "uuid",
      "status": "applied" | "conflict" | "rejected",
      "version": 42,            // present when applied
      "server_payload": { },    // present when conflict, the current server state
      "error": { "code": "...", "message": "..." } // when rejected
    }
  ]
}
```

Conflict resolution for documents goes through the **CRDT update channel**
below; non-document entities use last-writer-wins by `(version, updated_at)`.

## Document CRDT channel

Documents have free-text bodies and are merged with a CRDT (Yjs/yrs on both
ends). Beyond the entity push above, documents have:

### `POST /v1/sync/documents/{id}/updates`
Body: `{ "schema_version": 1, "updates": ["<base64-yjs-update>", ...] }`
→ `{ "applied": [<base64-yjs-update>, ...], "version": <new_version> }`

The server merges and rebroadcasts updates over WebSocket.

### `GET /v1/sync/documents/{id}/state`
Returns the current Yjs state vector + encoded state for initial
synchronization on a fresh client:
`{ "state_vector": "<base64>", "state": "<base64>", "version": 42 }`

## WebSocket invalidation

### `GET /v1/sync/ws` (Upgrade: websocket)

Authenticated via `Authorization` header on the HTTP upgrade. After the
upgrade, server pushes JSON frames:

```jsonc
{ "type": "changed", "kind": "document", "id": "uuid", "version": 43 }
{ "type": "doc_update", "id": "uuid", "update": "<base64-yjs>" }
{ "type": "deleted", "kind": "issue", "id": "uuid", "version": 50 }
```

Client may send:
```jsonc
{ "type": "subscribe", "kinds": ["document","issue","spec"] }
{ "type": "ping" }
```

## Entity payload shapes (v1)

Each kind's `payload` matches the public DTOs exported by `popsicle-core`
(`crates/popsicle-core/src/dto.rs`), with these additions:
- `parent_id`: the immediate parent entity id (e.g. spec_id for an issue).
- `path`: the relative file path inside `.popsicle/`, when the entity has one.
- `body`: full Markdown body for documents (also synced via CRDT channel).

## Rate limits

- `POST /v1/auth/login` and `/register`: 10 / IP / minute.
- `POST /v1/sync/push`: 60 / user / minute, max 1 MB body.
- `GET /v1/sync/changes`: 120 / user / minute.
- WebSocket: max 4 concurrent connections per user.
