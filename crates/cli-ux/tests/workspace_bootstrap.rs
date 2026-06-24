//! Runtime workspace auto-bootstrap (UI open + CLI lazy init).

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use cli_ux::global_config::{
    open_project_or_bootstrap, workspace_needs_bootstrap, WorkspaceSource,
};
use cli_ux::{bootstrap_workspace_at, WorkspaceDomain};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("popsicle-bootstrap-{name}-{nanos}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn with_isolated_home<F: FnOnce()>(f: F) {
    let home = temp_dir("home");
    let prev_home = std::env::var("POPSICLE_HOME").ok();
    let prev_proj = std::env::var("POPSICLE_PROJECT").ok();
    std::env::set_var("POPSICLE_HOME", &home);
    std::env::remove_var("POPSICLE_PROJECT");
    f();
    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
    if let Some(p) = prev_proj {
        std::env::set_var("POPSICLE_PROJECT", p);
    } else {
        std::env::remove_var("POPSICLE_PROJECT");
    }
    let _ = fs::remove_dir_all(home);
}

#[test]
fn workspace_needs_bootstrap_detects_missing_popsicle() {
    let root = temp_dir("needs");
    assert!(workspace_needs_bootstrap(root.display().to_string().as_str()).unwrap());
    bootstrap_workspace_at(&root).expect("bootstrap");
    assert!(!workspace_needs_bootstrap(root.display().to_string().as_str()).unwrap());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn bootstrap_workspace_at_creates_popsicle_layout() {
    let root = temp_dir("layout");
    let outcome = bootstrap_workspace_at(&root).expect("bootstrap");
    assert!(outcome.created);
    assert!(root.join(".popsicle").is_dir());
    assert!(root.join(".popsicle/project.yaml").is_file());
    assert!(root.join(".popsicle/pipelines").is_dir());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn open_project_or_bootstrap_registers_new_directory() {
    let _lock = env_lock();
    with_isolated_home(|| {
        let root = temp_dir("ui-open");
        let entry = open_project_or_bootstrap(root.display().to_string().as_str(), Some("ui-test"))
            .expect("open");
        assert!(root.join(".popsicle").is_dir());
        assert_eq!(entry.name, "ui-test");
        assert!(entry.path.contains("popsicle-bootstrap-ui-open"));
        let _ = fs::remove_dir_all(root);
    });
}

#[test]
fn open_with_lazy_bootstraps_cwd_when_no_workspace() {
    let _lock = env_lock();
    with_isolated_home(|| {
        let root = temp_dir("cli-lazy");
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&root).expect("chdir");
        let domain = WorkspaceDomain::open_with_lazy(None).expect("lazy open");
        assert_eq!(domain.workspace_source(), WorkspaceSource::LazyBootstrap);
        assert!(root.join(".popsicle").is_dir());
        std::env::set_current_dir(prev).expect("restore cwd");
        let _ = fs::remove_dir_all(root);
    });
}

#[test]
fn open_with_lazy_bootstraps_explicit_project_path() {
    let _lock = env_lock();
    with_isolated_home(|| {
        let root = temp_dir("cli-flag");
        let path = root.display().to_string();
        let domain = WorkspaceDomain::open_with_lazy(Some(path.as_str())).expect("lazy open");
        assert_eq!(domain.workspace_source(), WorkspaceSource::LazyBootstrap);
        assert!(root.join(".popsicle").is_dir());
        let _ = fs::remove_dir_all(root);
    });
}
