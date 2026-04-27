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
    crdt::b64_encode, CrdtDoc, Credentials, DocUpdates, HttpSyncClient, LoginRequest,
    PushOperation, PushRequest, RegisterRequest, SyncClient, WsClient, WsEvent, SCHEMA_VERSION,
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
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, email)
        && entry.set_password(&json).is_ok()
    {
        return Ok(());
    }
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
    let (_layout, cfg, db) = open_project()?;
    let dirty = db.list_dirty_sync_state().map_err(|e| anyhow!("{}", e))?;
    let since = db
        .get_sync_meta(META_LAST_SINCE)
        .map_err(|e| anyhow!("{}", e))?
        .unwrap_or_else(|| "0".into());
    let email = db
        .get_sync_meta(META_USER_EMAIL)
        .map_err(|e| anyhow!("{}", e))?
        .unwrap_or_default();
    let v = json!({
        "endpoint": cfg.sync.endpoint,
        "user": email,
        "dirty_count": dirty.len(),
        "last_remote_version": since.parse::<u64>().unwrap_or(0),
        "schema_version": SCHEMA_VERSION,
    });
    print_result(
        format,
        v,
        &format!(
            "endpoint: {}\nuser: {}\ndirty: {}\nlast cursor: {}",
            cfg.sync.endpoint,
            email,
            dirty.len(),
            since
        ),
    );
    Ok(())
}

async fn push(format: &OutputFormat) -> Result<()> {
    let (_layout, cfg, db) = open_project()?;
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
        .filter_map(row_to_push_op)
        .collect();
    let req = PushRequest {
        schema_version: SCHEMA_VERSION,
        client_id,
        operations,
    };
    let resp = client.push(req).await.map_err(|e| anyhow!("{}", e))?;
    let mut applied = 0usize;
    let mut conflicts = 0usize;
    for r in &resp.results {
        match r.status {
            popsicle_sync::PushOutcome::Applied => {
                applied += 1;
                if let Some(row) = dirty.iter().find(|d| d.entity_id == r.id.to_string()) {
                    let mut new_row = row.clone();
                    new_row.dirty = false;
                    new_row.remote_version = r.version.map(|v| v as i64);
                    new_row.last_synced_at = Some(Utc::now().to_rfc3339());
                    db.upsert_sync_state(&new_row).map_err(|e| anyhow!("{}", e))?;
                }
            }
            popsicle_sync::PushOutcome::Conflict => conflicts += 1,
            popsicle_sync::PushOutcome::Rejected => {}
        }
    }
    print_result(
        format,
        json!({
            "pushed": applied,
            "conflicts": conflicts,
            "results": resp.results.len(),
        }),
        &format!(
            "Pushed {} change(s); {} conflict(s); {} total.",
            applied,
            conflicts,
            resp.results.len()
        ),
    );
    Ok(())
}

async fn pull(format: &OutputFormat) -> Result<()> {
    let (_layout, cfg, db) = open_project()?;
    let email = current_email(&db)?;
    let creds = load_creds(&email)?;
    let client = build_client(&cfg, creds)?;
    let mut since: u64 = db
        .get_sync_meta(META_LAST_SINCE)
        .map_err(|e| anyhow!("{}", e))?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut total = 0usize;
    loop {
        let page = client
            .pull_changes(since, 200)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        for ch in &page.changes {
            // For now, mirror remote version into sync_state. Document
            // payload merge happens in M4 (CRDT).
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
        json!({"pulled": total, "cursor": since}),
        &format!("Pulled {} change(s); cursor now {}.", total, since),
    );
    Ok(())
}

fn row_to_push_op(row: &SyncStateRow) -> Option<PushOperation> {
    use popsicle_sync::EntityKind;
    let kind = match row.entity_kind.as_str() {
        "namespace" => EntityKind::Namespace,
        "spec" => EntityKind::Spec,
        "issue" => EntityKind::Issue,
        "pipeline_run" => EntityKind::PipelineRun,
        "document" => EntityKind::Document,
        "bug" => EntityKind::Bug,
        "user_story" => EntityKind::UserStory,
        "test_case" => EntityKind::TestCase,
        _ => return None,
    };
    let id = Uuid::parse_str(&row.entity_id).ok()?;
    Some(PushOperation {
        kind,
        id,
        base_version: row.remote_version.map(|v| v as u64),
        deleted: row.deleted,
        // Real payload assembly will be wired in M4 (read entity from
        // FileStorage / IndexDb and serialize). For now we send a stub
        // marker so the protocol path is exercised end-to-end.
        payload: json!({"_stub": true, "kind": row.entity_kind, "id": row.entity_id}),
    })
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
