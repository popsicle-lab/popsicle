//! `popsicle sync` — cloud sync (login, logout, whoami, status, push, pull,
//! and one-shot reconcile). Talks to any popsicle-cloud-compatible server
//! configured via `[sync]` in `.popsicle/config.toml`.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::Subcommand;
use popsicle_core::storage::{IndexDb, ProjectConfig, ProjectLayout, SyncStateRow};
use popsicle_sync::{
    conflict, crdt::b64_encode, path::canonical_path, CrdtDoc, Credentials, DocUpdates,
    EntityKind, HttpSyncClient, LoginRequest, PushOperation, PushOutcome, PushRequest,
    RegisterRequest, SyncClient, WsClient, WsEvent, SCHEMA_VERSION,
};
use serde_json::json;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::OutputFormat;

const KEYRING_SERVICE: &str = "popsicle-cli";
const META_LAST_SINCE: &str = "last_remote_version";
const META_USER_EMAIL: &str = "user_email";
const META_CLIENT_ID: &str = "client_id";

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Log in to the configured cloud endpoint
    Login {
        /// Email address (prompted if not provided)
        #[arg(short, long)]
        email: Option<String>,
        /// Register a new account instead of logging in
        #[arg(long)]
        register: bool,
    },
    /// Log out and clear stored credentials
    Logout,
    /// Show the currently logged-in user
    Whoami,
    /// Print local sync status (dirty rows, last cursor, endpoint)
    Status,
    /// Push local dirty changes to the server
    Push,
    /// Pull remote changes since the last cursor
    Pull,
    /// One-shot reconcile (push then pull)
    Run,
    /// Push a document's text content as a CRDT update
    DocPush {
        /// Document ID (UUID)
        doc_id: Uuid,
        /// Path to the text file (default: stdin)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Pull a document's authoritative state and write its text content
    DocPull {
        /// Document ID (UUID)
        doc_id: Uuid,
        /// Output file (default: stdout)
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Subscribe to live invalidation events over WebSocket
    Watch,
    /// Run the sync daemon (foreground): file-watch + WS + periodic reconcile
    Daemon {
        /// Override reconcile interval (seconds; defaults to config)
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Bootstrap a fresh `.popsicle/` workspace from server state
    Clone {
        /// Pull all entities from the start (default: true)
        #[arg(long, default_value_t = true)]
        full: bool,
    },
    /// Mark all existing local entities as dirty so they are included in the next push
    Seed,
}

pub fn execute(cmd: SyncCommand, format: &OutputFormat) -> Result<()> {
    let rt = Runtime::new().context("create tokio runtime")?;
    rt.block_on(async move {
        match cmd {
            SyncCommand::Login { email, register } => login(email, register, format).await,
            SyncCommand::Logout => logout(format),
            SyncCommand::Whoami => whoami(format).await,
            SyncCommand::Status => status(format),
            SyncCommand::Push => push(format).await,
            SyncCommand::Pull => pull(format).await,
            SyncCommand::Run => {
                push(format).await?;
                pull(format).await
            }
            SyncCommand::DocPush { doc_id, file } => doc_push(doc_id, file, format).await,
            SyncCommand::DocPull { doc_id, out } => doc_pull(doc_id, out, format).await,
            SyncCommand::Watch => watch(format).await,
            SyncCommand::Daemon { interval } => daemon(interval, format).await,
            SyncCommand::Clone { full } => clone_workspace(full, format).await,
            SyncCommand::Seed => seed(format),
        }
    })
}

// ---- credentials store --------------------------------------------------

fn fallback_creds_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no home directory"))?;
    Ok(home.join(".popsicle").join("credentials.json"))
}

fn load_creds(email: &str) -> Result<Credentials> {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, email)
        && let Ok(secret) = entry.get_password()
        && let Ok(c) = serde_json::from_str::<StoredCreds>(&secret)
    {
        return Ok(c.into());
    }
    let path = fallback_creds_path()?;
    if path.exists() {
        let text = fs::read_to_string(&path)?;
        let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text)?;
        if let Some(v) = map.get(email) {
            let c: StoredCreds = serde_json::from_value(v.clone())?;
            return Ok(c.into());
        }
    }
    Ok(Credentials::default())
}

fn save_creds(email: &str, creds: &Credentials) -> Result<()> {
    let stored: StoredCreds = creds.clone().into();
    let json = serde_json::to_string(&stored)?;

    // Always write to the file store first — this is the authoritative path.
    // keyring access may silently fail on unsigned binaries or locked keychains.
    let path = fallback_creds_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut map: serde_json::Map<String, serde_json::Value> = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default()
    } else {
        Default::default()
    };
    map.insert(email.to_string(), serde_json::to_value(stored)?);
    write_secure(&path, serde_json::to_string_pretty(&map)?.as_bytes())?;

    // Best-effort keyring write so the credential is available system-wide.
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, email) {
        let _ = entry.set_password(&json);
    }

    Ok(())
}

fn clear_creds(email: &str) -> Result<()> {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, email) {
        let _ = entry.delete_credential();
    }
    let path = fallback_creds_path()?;
    if path.exists() {
        let mut map: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or_default();
        map.remove(email);
        write_secure(&path, serde_json::to_string_pretty(&map)?.as_bytes())?;
    }
    Ok(())
}

#[cfg(unix)]
fn write_secure(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_secure(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredCreds {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl From<Credentials> for StoredCreds {
    fn from(c: Credentials) -> Self {
        Self {
            access_token: c.access_token,
            refresh_token: c.refresh_token,
        }
    }
}

impl From<StoredCreds> for Credentials {
    fn from(s: StoredCreds) -> Self {
        Self {
            access_token: s.access_token,
            refresh_token: s.refresh_token,
        }
    }
}

// ---- helpers ------------------------------------------------------------

fn project_layout() -> Result<ProjectLayout> {
    let cwd = std::env::current_dir()?;
    popsicle_core::helpers::project_layout(&cwd).map_err(|e| anyhow::anyhow!("{}", e))
}

fn open_project() -> Result<(ProjectLayout, ProjectConfig, IndexDb)> {
    let layout = project_layout()?;
    let cfg = ProjectConfig::load(&layout.config_path())
        .map_err(|e| anyhow!("failed to load config: {}", e))?;
    if !cfg.sync.is_active() {
        return Err(anyhow!(
            "sync is not enabled. Set [sync] endpoint and enabled=true in .popsicle/config.toml"
        ));
    }
    let db = IndexDb::open(&layout.db_path()).map_err(|e| anyhow!("{}", e))?;
    Ok((layout, cfg, db))
}

fn ensure_client_id(db: &IndexDb) -> Result<Uuid> {
    if let Some(s) = db.get_sync_meta(META_CLIENT_ID).map_err(|e| anyhow!("{}", e))?
        && let Ok(u) = Uuid::parse_str(&s)
    {
        return Ok(u);
    }
    let u = Uuid::new_v4();
    db.set_sync_meta(META_CLIENT_ID, &u.to_string())
        .map_err(|e| anyhow!("{}", e))?;
    Ok(u)
}

fn current_email(db: &IndexDb) -> Result<String> {
    db.get_sync_meta(META_USER_EMAIL)
        .map_err(|e| anyhow!("{}", e))?
        .ok_or_else(|| anyhow!("not logged in. Run `popsicle sync login`."))
}

fn build_client(cfg: &ProjectConfig, creds: Credentials) -> Result<HttpSyncClient> {
    HttpSyncClient::new(&cfg.sync.endpoint, creds).map_err(|e| anyhow!("{}", e))
}

fn print_result(format: &OutputFormat, value: serde_json::Value, text: &str) {
    match format {
        OutputFormat::Json => println!("{}", value),
        OutputFormat::Text => println!("{}", text),
    }
}

// ---- commands -----------------------------------------------------------

async fn login(email_arg: Option<String>, register: bool, format: &OutputFormat) -> Result<()> {
    let (_layout, cfg, db) = open_project()?;
    let email = match email_arg {
        Some(e) => e,
        None => {
            print!("Email: ");
            std::io::stdout().flush()?;
            let mut s = String::new();
            std::io::stdin().read_line(&mut s)?;
            s.trim().to_string()
        }
    };
    if email.is_empty() {
        return Err(anyhow!("email is required"));
    }
    let password = rpassword::prompt_password("Password: ")?;
    let client = build_client(&cfg, Credentials::default())?;
    let tokens = if register {
        client
            .register(RegisterRequest {
                email: email.clone(),
                password,
            })
            .await
    } else {
        client
            .login(LoginRequest {
                email: email.clone(),
                password,
            })
            .await
    }
    .map_err(|e| anyhow!("{}", e))?;

    let creds = Credentials {
        access_token: Some(tokens.access_token),
        refresh_token: Some(tokens.refresh_token),
    };
    save_creds(&email, &creds)?;
    db.set_sync_meta(META_USER_EMAIL, &email)
        .map_err(|e| anyhow!("{}", e))?;
    print_result(
        format,
        json!({"email": tokens.user.email, "user_id": tokens.user.id}),
        &format!("Logged in as {}", tokens.user.email),
    );
    Ok(())
}

fn logout(format: &OutputFormat) -> Result<()> {
    let (_layout, _cfg, db) = open_project()?;
    if let Some(email) = db.get_sync_meta(META_USER_EMAIL).map_err(|e| anyhow!("{}", e))? {
        clear_creds(&email)?;
    }
    db.set_sync_meta(META_USER_EMAIL, "")
        .map_err(|e| anyhow!("{}", e))?;
    print_result(format, json!({"ok": true}), "Logged out.");
    Ok(())
}

async fn whoami(format: &OutputFormat) -> Result<()> {
    let (_layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    if creds.access_token.is_none() {
        return Err(anyhow!("no access token; please run `popsicle sync login`"));
    }
    let client = build_client(&cfg, creds)?;
    let user = client.me().await.map_err(|e| anyhow!("{}", e))?;
    print_result(
        format,
        json!({"email": user.email, "user_id": user.id}),
        &format!("{} ({})", user.email, user.id),
    );
    Ok(())
}

fn status(format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let dirty = db.list_dirty_sync_state().map_err(|e| anyhow!("{}", e))?;
    let since = db
        .get_sync_meta(META_LAST_SINCE)
        .map_err(|e| anyhow!("{}", e))?
        .unwrap_or_else(|| "0".into());
    let email = db
        .get_sync_meta(META_USER_EMAIL)
        .map_err(|e| anyhow!("{}", e))?
        .unwrap_or_default();
    let conflicts_log = layout.sync_conflicts_log();
    let (conflicts_total, conflicts_recent) = if conflicts_log.exists() {
        let txt = std::fs::read_to_string(&conflicts_log).unwrap_or_default();
        let lines: Vec<&str> = txt.lines().filter(|l| !l.trim().is_empty()).collect();
        let recent: Vec<String> =
            lines.iter().rev().take(5).map(|s| s.to_string()).collect();
        (lines.len(), recent)
    } else {
        (0, vec![])
    };
    let v = json!({
        "endpoint": cfg.sync.endpoint,
        "user": email,
        "dirty_count": dirty.len(),
        "last_remote_version": since.parse::<u64>().unwrap_or(0),
        "schema_version": SCHEMA_VERSION,
        "conflicts_total": conflicts_total,
        "conflicts_log": conflicts_log.display().to_string(),
    });
    let mut text = format!(
        "endpoint: {}\nuser: {}\ndirty: {}\nlast cursor: {}\nconflicts: {}",
        cfg.sync.endpoint,
        email,
        dirty.len(),
        since,
        conflicts_total
    );
    if !conflicts_recent.is_empty() {
        text.push_str("\nrecent:");
        for line in conflicts_recent.iter().rev() {
            text.push_str(&format!("\n  {}", line));
        }
    }
    print_result(format, v, &text);
    Ok(())
}

async fn push(format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;
    let client_id = ensure_client_id(&db)?;
    let dirty = db.list_dirty_sync_state().map_err(|e| anyhow!("{}", e))?;
    if dirty.is_empty() {
        print_result(format, json!({"pushed": 0}), "Nothing to push.");
        return Ok(());
    }
    let operations: Vec<PushOperation> = dirty
        .iter()
        .filter_map(|row| row_to_push_op(&db, row))
        .collect();
    let req = PushRequest {
        schema_version: SCHEMA_VERSION,
        client_id,
        operations,
    };
    let resp = client.push(req).await.map_err(|e| anyhow!("{}", e))?;
    let mut applied = 0usize;
    let mut conflicts = 0usize;
    let mut lost_total = 0usize;
    for r in &resp.results {
        match r.status {
            PushOutcome::Applied => {
                applied += 1;
                if let Some(row) = dirty.iter().find(|d| d.entity_id == r.id.to_string()) {
                    let mut new_row = row.clone();
                    new_row.dirty = false;
                    new_row.remote_version = r.version.map(|v| v as i64);
                    new_row.last_synced_at = Some(Utc::now().to_rfc3339());
                    db.upsert_sync_state(&new_row).map_err(|e| anyhow!("{}", e))?;
                }
            }
            PushOutcome::Conflict => {
                conflicts += 1;
                let lost = handle_conflict(&layout, &db, &dirty, r)?;
                lost_total += lost;
            }
            PushOutcome::Rejected => {}
        }
    }
    print_result(
        format,
        json!({
            "pushed": applied,
            "conflicts": conflicts,
            "lost_fields": lost_total,
            "results": resp.results.len(),
        }),
        &format!(
            "Pushed {} change(s); {} conflict(s) ({} field(s) overwritten); {} total.",
            applied,
            conflicts,
            lost_total,
            resp.results.len()
        ),
    );
    Ok(())
}

/// On conflict: take server payload as truth, merge any local-only fields,
/// log overwritten fields to `.sync/conflicts.log`, and mark the entity as
/// no-longer-dirty with the new server version. The user can re-push by
/// editing again; we don't auto-retry to keep the protocol simple.
fn handle_conflict(
    layout: &ProjectLayout,
    db: &IndexDb,
    dirty: &[SyncStateRow],
    result: &popsicle_sync::PushResult,
) -> Result<usize> {
    let Some(row) = dirty.iter().find(|d| d.entity_id == result.id.to_string()) else {
        return Ok(0);
    };
    let server_payload = result
        .server_payload
        .clone()
        .unwrap_or(serde_json::Value::Null);
    // Local payload reconstruction: we currently only know the stub. Without
    // a stored last-pushed payload, we treat every non-equal field as a
    // potential local edit. Document bodies are not affected (they go via
    // the CRDT log), so this conservatively logs metadata divergences.
    let local_stub = serde_json::json!({"_stub": true, "kind": row.entity_kind, "id": row.entity_id});
    let report = conflict::merge(&local_stub, &serde_json::Value::Null, &server_payload);
    let kind = EntityKind::parse(&row.entity_kind).unwrap_or(EntityKind::Namespace);
    if let Ok(id) = Uuid::parse_str(&row.entity_id) {
        let _ = conflict::append_log(&layout.sync_conflicts_log(), kind, id, &report);
    }
    let mut new_row = row.clone();
    new_row.dirty = false;
    new_row.remote_version = result.version.map(|v| v as i64);
    new_row.last_synced_at = Some(Utc::now().to_rfc3339());
    db.upsert_sync_state(&new_row).map_err(|e| anyhow!("{}", e))?;
    Ok(report.lost.len())
}

async fn pull(format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;
    let mut since: u64 = db
        .get_sync_meta(META_LAST_SINCE)
        .map_err(|e| anyhow!("{}", e))?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut total = 0usize;
    let mut materialised = 0usize;
    let mut recovered = 0usize;
    loop {
        let page = client
            .pull_changes(since, 200)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        for ch in &page.changes {
            // Materialise to canonical path. Document bodies are intentionally
            // skipped here — they ride the CRDT channel via doc_pull.
            if ch.kind != EntityKind::Document
                && let Some(rel) = canonical_path(ch.kind, ch.id, &ch.payload)
            {
                let abs = layout.dot_dir().join(&rel);
                if ch.deleted {
                    if abs.exists() {
                        // Preserve any local-only edits before deletion.
                        let recovered_path = layout
                            .sync_recovered_dir()
                            .join(format!("{}-{}.bak", ch.kind.as_str(), ch.id));
                        if let Some(parent) = recovered_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::copy(&abs, &recovered_path);
                        let _ = std::fs::remove_file(&abs);
                        recovered += 1;
                    }
                } else if let Err(e) = write_entity_file(&abs, ch.kind, &ch.payload) {
                    eprintln!("warn: failed to materialise {}: {}", abs.display(), e);
                } else {
                    materialised += 1;
                }
            }
            let row = SyncStateRow {
                entity_kind: ch.kind.as_str().to_string(),
                entity_id: ch.id.to_string(),
                local_hash: None,
                remote_version: Some(ch.version as i64),
                last_synced_at: Some(Utc::now().to_rfc3339()),
                dirty: false,
                deleted: ch.deleted,
            };
            db.upsert_sync_state(&row).map_err(|e| anyhow!("{}", e))?;
            total += 1;
        }
        since = page.next_since;
        db.set_sync_meta(META_LAST_SINCE, &since.to_string())
            .map_err(|e| anyhow!("{}", e))?;
        if !page.has_more {
            break;
        }
    }
    print_result(
        format,
        json!({"pulled": total, "cursor": since, "materialised": materialised, "recovered": recovered}),
        &format!(
            "Pulled {} change(s); cursor now {}; wrote {} file(s); preserved {} deleted local file(s).",
            total, since, materialised, recovered
        ),
    );
    Ok(())
}

/// Render an entity payload to disk. For Skill/Pipeline, the payload is
/// expected to carry a `body` string with the YAML source. For everything
/// else, we serialise the payload as YAML frontmatter so a re-clone can be
/// re-indexed by `popsicle reindex` later.
fn write_entity_file(
    path: &std::path::Path,
    kind: EntityKind,
    payload: &serde_json::Value,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body: String = match kind {
        EntityKind::Skill | EntityKind::Pipeline => payload
            .get("body")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                serde_yaml_ng::to_string(payload).unwrap_or_else(|_| "{}".into())
            }),
        _ => {
            let yaml = serde_yaml_ng::to_string(payload)
                .unwrap_or_else(|_| "{}".into());
            let body_md = payload
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("---\n{}---\n\n{}", yaml, body_md)
        }
    };
    // Atomic-ish: write to tmp + rename. Skip if identical content already
    // exists (avoid retriggering the file watcher).
    if let Ok(existing) = std::fs::read_to_string(path)
        && existing == body
    {
        return Ok(());
    }
    std::fs::write(path, body)
}

fn row_to_push_op(db: &IndexDb, row: &SyncStateRow) -> Option<PushOperation> {
    let kind = EntityKind::parse(row.entity_kind.as_str())?;
    let id = Uuid::parse_str(&row.entity_id).ok()?;
    let payload = if row.deleted {
        serde_json::json!({"id": row.entity_id, "deleted": true})
    } else {
        build_entity_payload(db, kind, &row.entity_id)?
    };
    Some(PushOperation {
        kind,
        id,
        base_version: row.remote_version.map(|v| v as u64),
        deleted: row.deleted,
        payload,
    })
}

fn build_entity_payload(db: &IndexDb, kind: EntityKind, id: &str) -> Option<serde_json::Value> {
    match kind {
        EntityKind::Namespace => db.get_namespace(id).ok().flatten().and_then(|n| serde_json::to_value(n).ok()),
        EntityKind::Spec => db.get_spec(id).ok().flatten().and_then(|s| serde_json::to_value(s).ok()),
        EntityKind::Issue => db.get_issue(id).ok().flatten().and_then(|i| serde_json::to_value(i).ok()),
        EntityKind::PipelineRun => db.get_pipeline_run(id).ok().flatten().and_then(|p| serde_json::to_value(p).ok()),
        EntityKind::Document => db.get_document_row(id).ok().flatten().and_then(|d| serde_json::to_value(d).ok()),
        EntityKind::Bug => db.get_bug(id).ok().flatten().and_then(|b| serde_json::to_value(b).ok()),
        EntityKind::UserStory => db.get_user_story(id).ok().flatten().and_then(|u| serde_json::to_value(u).ok()),
        EntityKind::TestCase => db.get_test_case(id).ok().flatten().and_then(|t| serde_json::to_value(t).ok()),
        EntityKind::Skill | EntityKind::Pipeline => None,
    }
}

// ---- document CRDT commands --------------------------------------------

async fn doc_push(doc_id: Uuid, file: Option<PathBuf>, format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;

    // Read new text content (file or stdin).
    let new_text = match file {
        Some(p) => std::fs::read_to_string(&p)
            .with_context(|| format!("read {}", p.display()))?,
        None => {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            s
        }
    };

    // Load (or initialise) the local CRDT cache.
    let crdt_path = layout.sync_doc_path(&doc_id.to_string());
    let mut local =
        CrdtDoc::load(&crdt_path).map_err(|e| anyhow!("load crdt: {}", e))?;

    // Sync with the server's current state first so our diff lands cleanly
    // even if another client raced us.
    let remote_state = client.doc_state(doc_id).await.map_err(|e| anyhow!("{}", e))?;
    if !remote_state.state.is_empty() {
        local
            .apply_update_b64(&remote_state.state)
            .map_err(|e| anyhow!("merge remote: {}", e))?;
    }

    // Apply the local edit and capture the diff.
    let update_bytes = local
        .replace_text(&new_text)
        .map_err(|e| anyhow!("apply local edit: {}", e))?;
    if update_bytes.is_empty() {
        local.save(&crdt_path).map_err(|e| anyhow!("{}", e))?;
        print_result(format, json!({"version": remote_state.version, "noop": true}), "No changes.");
        return Ok(());
    }
    let payload = DocUpdates {
        schema_version: SCHEMA_VERSION,
        updates: vec![b64_encode(&update_bytes)],
    };
    let resp = client
        .doc_apply_updates(doc_id, payload)
        .await
        .map_err(|e| anyhow!("{}", e))?;
    local.save(&crdt_path).map_err(|e| anyhow!("{}", e))?;
    print_result(
        format,
        json!({"version": resp.version, "applied": resp.applied.len()}),
        &format!("Pushed doc update; version now {}.", resp.version),
    );
    Ok(())
}

async fn doc_pull(doc_id: Uuid, out: Option<PathBuf>, format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;

    let remote = client.doc_state(doc_id).await.map_err(|e| anyhow!("{}", e))?;
    let crdt_path = layout.sync_doc_path(&doc_id.to_string());
    let mut local = CrdtDoc::load(&crdt_path).map_err(|e| anyhow!("{}", e))?;
    if !remote.state.is_empty() {
        local
            .apply_update_b64(&remote.state)
            .map_err(|e| anyhow!("merge remote: {}", e))?;
    }
    local.save(&crdt_path).map_err(|e| anyhow!("{}", e))?;
    let text = local.text();

    match out {
        Some(p) => {
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&p, &text)?;
            print_result(
                format,
                json!({"version": remote.version, "bytes": text.len(), "path": p.display().to_string()}),
                &format!("Wrote {} bytes to {} (version {}).", text.len(), p.display(), remote.version),
            );
        }
        None => {
            match format {
                OutputFormat::Json => println!(
                    "{}",
                    json!({"version": remote.version, "text": text})
                ),
                OutputFormat::Text => print!("{text}"),
            }
        }
    }
    Ok(())
}

async fn watch(format: &OutputFormat) -> Result<()> {
    let (_layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let access = creds
        .access_token
        .clone()
        .ok_or_else(|| anyhow!("no access token; run `popsicle sync login`"))?;
    let mut rx = WsClient::connect(&cfg.sync.endpoint, &access)
        .await
        .map_err(|e| anyhow!("ws connect: {}", e))?;
    eprintln!("Connected to {} (Ctrl-C to stop).", cfg.sync.endpoint);
    while let Some(ev) = rx.recv().await {
        match format {
            OutputFormat::Json => match &ev {
                WsEvent::Changed { kind, id, version, deleted } => {
                    println!(
                        "{}",
                        json!({"type":"changed","kind":kind,"id":id,"version":version,"deleted":deleted})
                    );
                }
                WsEvent::DocUpdate { id, update } => {
                    println!("{}", json!({"type":"doc_update","id":id,"update_bytes":update.len()}));
                }
            },
            OutputFormat::Text => match &ev {
                WsEvent::Changed { kind, id, version, deleted } => {
                    println!(
                        "[changed] {} {} v{}{}",
                        kind,
                        id,
                        version,
                        if *deleted { " (deleted)" } else { "" }
                    );
                }
                WsEvent::DocUpdate { id, update } => {
                    println!("[doc_update] {} ({} bytes)", id, update.len());
                }
            },
        }
    }
    Ok(())
}

// ---- daemon ------------------------------------------------------------

async fn daemon(interval_override: Option<u64>, _format: &OutputFormat) -> Result<()> {
    use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc as stdmpsc;
    use std::time::Duration;
    use tokio::sync::mpsc;

    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let access = creds
        .access_token
        .clone()
        .ok_or_else(|| anyhow!("no access token; run `popsicle sync login`"))?;

    // Write PID file for `popsicle sync daemon stop` (future) and human ops.
    let pid_path = layout.sync_daemon_pid();
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pid_path, std::process::id().to_string())?;

    let interval = Duration::from_secs(
        interval_override.unwrap_or(cfg.sync.interval_secs.max(10)),
    );
    eprintln!(
        "popsicle daemon started (pid {}, interval {}s, endpoint {})",
        std::process::id(),
        interval.as_secs(),
        cfg.sync.endpoint
    );

    // ---- file watcher (notify is sync; bridge into a tokio channel) ----
    let (fs_tx, mut fs_rx) = mpsc::channel::<()>(16);
    let (raw_tx, raw_rx) = stdmpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = RecommendedWatcher::new(raw_tx, notify::Config::default())
        .map_err(|e| anyhow!("watcher: {}", e))?;
    let watch_root = layout.dot_dir().to_path_buf();
    watcher
        .watch(&watch_root, RecursiveMode::Recursive)
        .map_err(|e| anyhow!("watch: {}", e))?;
    let pid_path_for_filter = pid_path.clone();
    let sync_dir_for_filter = layout.sync_dir();
    std::thread::spawn(move || {
        for ev in raw_rx {
            let Ok(ev) = ev else { continue };
            // Filter out our own .sync/ writes (PID file, .crdt cache).
            let touched_self = ev.paths.iter().any(|p| {
                p == &pid_path_for_filter || p.starts_with(&sync_dir_for_filter)
            });
            if touched_self {
                continue;
            }
            if matches!(
                ev.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) && fs_tx.try_send(()).is_err()
            {
                // Channel full — coalesce; another tick already pending.
            }
        }
    });

    // ---- WS subscription ----
    let mut ws_rx = match WsClient::connect(&cfg.sync.endpoint, &access).await {
        Ok(rx) => Some(rx),
        Err(e) => {
            eprintln!("ws connect failed: {} (continuing with poll only)", e);
            None
        }
    };

    let mut tick = tokio::time::interval(interval);
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    let format = OutputFormat::Text;
    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => {
                eprintln!("\nshutting down...");
                break;
            }
            _ = tick.tick() => {
                if let Err(e) = run_reconcile(&format).await {
                    eprintln!("reconcile error: {}", e);
                }
            }
            Some(_) = fs_rx.recv() => {
                // Debounce: drain any further events queued in the channel.
                while fs_rx.try_recv().is_ok() {}
                if let Err(e) = run_reconcile(&format).await {
                    eprintln!("fs-triggered sync error: {}", e);
                }
            }
            ev = async {
                match ws_rx.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => None,
                }
            }, if ws_rx.is_some() => {
                match ev {
                    Some(WsEvent::Changed { kind, id, version, .. }) => {
                        eprintln!("[ws] changed {} {} v{}; pulling", kind, id, version);
                        if let Err(e) = pull(&format).await {
                            eprintln!("ws-triggered pull error: {}", e);
                        }
                    }
                    Some(WsEvent::DocUpdate { id, .. }) => {
                        eprintln!("[ws] doc update {}; refreshing local cache", id);
                        // Refresh the doc CRDT cache by pulling its state.
                        if let Err(e) = doc_pull(id, None, &OutputFormat::Json).await {
                            eprintln!("doc refresh error: {}", e);
                        }
                    }
                    None => {
                        eprintln!("[ws] disconnected; falling back to polling");
                        ws_rx = None;
                    }
                }
            }
        }
    }

    let _ = std::fs::remove_file(&pid_path);
    Ok(())
}

async fn run_reconcile(format: &OutputFormat) -> Result<()> {
    push(format).await?;
    pull(format).await
}

/// Bootstrap a fresh `.popsicle/` workspace by pulling everything from the
/// server (since=0) and materialising entities + document bodies on disk.
async fn clone_workspace(_full: bool, format: &OutputFormat) -> Result<()> {
    let (layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;

    // Reset cursor so pull() walks history from the beginning.
    db.set_sync_meta(META_LAST_SINCE, "0")
        .map_err(|e| anyhow!("{}", e))?;

    // First pass: pull metadata for every entity, materialising all non-document files.
    pull(format).await?;

    // Second pass: hydrate document bodies via CRDT for every Document we now know about.
    let mut doc_count = 0usize;
    let mut since: u64 = 0;
    loop {
        let page = client
            .pull_changes(since, 200)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        for ch in &page.changes {
            if ch.kind != EntityKind::Document || ch.deleted {
                continue;
            }
            // Resolve canonical path; fall back to artifacts/<id>.md.
            let rel = canonical_path(EntityKind::Document, ch.id, &ch.payload)
                .unwrap_or_else(|| PathBuf::from("artifacts").join(format!("{}.md", ch.id)));
            let abs = layout.dot_dir().join(&rel);
            if let Some(parent) = abs.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Pull CRDT state, render text.
            let remote = client.doc_state(ch.id).await.map_err(|e| anyhow!("{}", e))?;
            let crdt_path = layout.sync_doc_path(&ch.id.to_string());
            let mut local = CrdtDoc::load(&crdt_path).map_err(|e| anyhow!("{}", e))?;
            if !remote.state.is_empty() {
                local
                    .apply_update_b64(&remote.state)
                    .map_err(|e| anyhow!("merge remote: {}", e))?;
            }
            local.save(&crdt_path).map_err(|e| anyhow!("{}", e))?;
            std::fs::write(&abs, local.text())?;
            doc_count += 1;
        }
        since = page.next_since;
        if !page.has_more {
            break;
        }
    }

    print_result(
        format,
        json!({"documents_hydrated": doc_count}),
        &format!(
            "Workspace cloned. Hydrated {} document body(ies). Run `popsicle reindex` to rebuild the local index.",
            doc_count
        ),
    );
    Ok(())
}

fn seed(format: &OutputFormat) -> Result<()> {
    let (_layout, _cfg, db) = open_project()?;
    let n = db.seed_sync_state().map_err(|e| anyhow!("{}", e))?;
    print_result(
        format,
        json!({"seeded": n}),
        &format!("Seeded {n} entities into sync state. Run `popsicle sync push` to upload."),
    );
    Ok(())
}
