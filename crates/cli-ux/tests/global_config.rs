//! Global multi-project registry tests (isolated via POPSICLE_HOME).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

// POPSICLE_HOME is process-global; parallel tests would race on env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

use cli_ux::global_config::{
    add_project, list_recent_projects, load_global_config, open_project, resolve_workspace_root,
    set_default_project, WorkspaceSource,
};
use cli_ux::workspace::{binary_provenance_for, Workspace};
use cli_ux::{parse_cli, run_command_stateless, WorkspaceDomain};

fn isolated_home() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "popsicle-global-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_workspace(root: &Path) {
    fs::create_dir_all(root.join(".popsicle")).unwrap();
}

#[test]
fn project_add_use_and_default_resolution() {
    let _lock = env_lock();
    let home = isolated_home();
    let proj_a = home.join("proj-a");
    let proj_b = home.join("proj-b");
    write_workspace(&proj_a);
    write_workspace(&proj_b);

    let prev_home = std::env::var("POPSICLE_HOME").ok();
    let prev_proj = std::env::var("POPSICLE_PROJECT").ok();
    std::env::set_var("POPSICLE_HOME", &home);
    std::env::remove_var("POPSICLE_PROJECT");

    add_project(&proj_a.display().to_string(), Some("a")).unwrap();
    add_project(&proj_b.display().to_string(), Some("b")).unwrap();
    set_default_project("a").unwrap();

    let prev_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&home).unwrap();
    let resolved = resolve_workspace_root(None).unwrap();
    std::env::set_current_dir(prev_cwd).unwrap();
    assert_eq!(resolved.source, WorkspaceSource::GlobalDefault);
    assert!(resolved.root.ends_with("proj-a"));

    let cfg = load_global_config().unwrap();
    assert_eq!(cfg.projects.len(), 2);
    assert!(cfg.default_project.as_ref().unwrap().contains("proj-a"));

    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
    if let Some(p) = prev_proj {
        std::env::set_var("POPSICLE_PROJECT", p);
    }
}

#[test]
fn cli_project_flag_overrides_default() {
    let _lock = env_lock();
    let home = isolated_home();
    let proj_a = home.join("proj-a");
    let proj_b = home.join("proj-b");
    write_workspace(&proj_a);
    write_workspace(&proj_b);

    let prev_home = std::env::var("POPSICLE_HOME").ok();
    std::env::set_var("POPSICLE_HOME", &home);
    std::env::remove_var("POPSICLE_PROJECT");

    add_project(&proj_a.display().to_string(), None).unwrap();
    add_project(&proj_b.display().to_string(), None).unwrap();
    set_default_project("proj-a").unwrap();

    let parsed = parse_cli([
        "issue",
        "list",
        "--project",
        proj_b.display().to_string().as_str(),
        "--format",
        "json",
    ])
    .unwrap();
    let domain = WorkspaceDomain::open_with(parsed.globals.project.as_deref()).unwrap();
    assert!(domain.workspace_root().ends_with("proj-b"));
    assert_eq!(domain.workspace_source(), WorkspaceSource::CliFlag);

    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
}

#[test]
fn project_list_stateless_without_cwd_workspace() {
    let _lock = env_lock();
    let home = isolated_home();
    let prev_home = std::env::var("POPSICLE_HOME").ok();
    std::env::set_var("POPSICLE_HOME", &home);

    let resp = run_command_stateless(cli_ux::Command::ProjectList).unwrap();
    assert_eq!(resp.status, "ok");
    assert_eq!(resp.fields.get("count").map(String::as_str), Some("0"));

    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
}

#[test]
fn open_project_records_recent_and_default() {
    let _lock = env_lock();
    let home = isolated_home();
    let proj = home.join("recent-proj");
    write_workspace(&proj);

    let prev_home = std::env::var("POPSICLE_HOME").ok();
    std::env::set_var("POPSICLE_HOME", &home);

    open_project(&proj.display().to_string(), Some("recent")).unwrap();
    let cfg = load_global_config().unwrap();
    assert!(
        cfg.default_project
            .as_deref()
            .is_some_and(|p| p.ends_with("recent-proj")),
        "default should point at recent-proj, got {:?}",
        cfg.default_project
    );
    let entry = cfg.projects.iter().find(|p| p.name == "recent").unwrap();
    assert!(entry.last_opened_at.is_some());

    let recent = list_recent_projects(5).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].name, "recent");

    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
}

#[test]
fn doctor_reports_workspace_source() {
    let _lock = env_lock();
    let home = isolated_home();
    let proj = home.join("dogfood");
    write_workspace(&proj);

    let prev_home = std::env::var("POPSICLE_HOME").ok();
    std::env::set_var("POPSICLE_HOME", &home);
    add_project(&proj.display().to_string(), None).unwrap();
    set_default_project("dogfood").unwrap();

    let prev_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&home).unwrap();
    let domain = WorkspaceDomain::open_with(None).unwrap();
    std::env::set_current_dir(prev_cwd).unwrap();
    let prov = binary_provenance_for(
        &Workspace::at(domain.workspace_root().to_path_buf()),
        domain.workspace_source(),
    )
    .unwrap();
    assert_eq!(prov.workspace_source, "global_default");
    assert!(prov.global_config_path.contains("global.json"));

    if let Some(h) = prev_home {
        std::env::set_var("POPSICLE_HOME", h);
    } else {
        std::env::remove_var("POPSICLE_HOME");
    }
}
