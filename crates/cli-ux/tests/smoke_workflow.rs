//! End-to-end self-hosting workflow smoke (PDR-002 / T-CU-0008 / PROJ-10).
//!
//! PROJ-24 (O-102): the workflow runs in an isolated temp workspace with an
//! isolated `POPSICLE_HOME` so `cargo test` never accretes smoke issues/runs in
//! the real repository and does not depend on the developer's global registry.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    option_env!("CARGO_BIN_EXE_popsicle").unwrap_or("./target/debug/popsicle")
}

fn temp_workspace() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("popsicle-smoke-{nanos}"));
    std::fs::create_dir_all(root.join("products/smoke-spec")).expect("product dir");
    root
}

fn isolated_home() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home = std::env::temp_dir().join(format!("popsicle-smoke-home-{nanos}"));
    std::fs::create_dir_all(&home).expect("isolated POPSICLE_HOME");
    home
}

fn popsicle_in(dir: &PathBuf, home: &PathBuf, args: &[&str]) -> (String, String, i32) {
    let output = Command::new(bin())
        .args(args)
        .current_dir(dir)
        .env("POPSICLE_HOME", home)
        .env_remove("POPSICLE_PROJECT")
        .output()
        .expect("run popsicle binary");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

fn ok_in(dir: &PathBuf, home: &PathBuf, args: &[&str]) -> String {
    let (stdout, stderr, code) = popsicle_in(dir, home, args);
    assert_eq!(
        code,
        0,
        "command failed: popsicle {} — stderr: {stderr}",
        args.join(" ")
    );
    stdout
}

fn field(stdout: &str, key: &str) -> String {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")).map(str::trim))
        .unwrap_or_else(|| panic!("missing {key} in:\n{stdout}"))
        .to_string()
}

#[test]
fn workspace_workflow_smoke_passes() {
    let home = isolated_home();
    let ws = temp_workspace();
    ok_in(&ws, &home, &["init"]);

    let doctor = ok_in(&ws, &home, &["doctor", "--format", "json"]);
    // Binary-match only holds when the test binary IS the workspace binary;
    // sandboxed/redirected CARGO_TARGET_DIR runs use a different path. The
    // unconditional check lives in the doctor-provenance golden script.
    let repo = std::env::current_dir().expect("cwd");
    if repo.join("target/debug/popsicle") == std::path::Path::new(bin()) {
        assert!(doctor.contains("\"current_workspace_binary_match\":\"true\""));
    }
    assert!(doctor.contains("storage_backend"));
    assert!(doctor.contains("PROJ-11"));

    // `--type bug` exercises the bundled default pipeline mapping (D-101):
    // no --pipeline flag anywhere below.
    let created = ok_in(
        &ws,
        &home,
        &[
            "issue",
            "create",
            "--type",
            "bug",
            "--title",
            "smoke bug",
            "--product",
            "smoke-spec",
        ],
    );
    let key = field(&created, "key");

    let started = ok_in(&ws, &home, &["issue", "start", &key]);
    let run_id = field(&started, "run_id");
    assert_eq!(field(&started, "run_created"), "true");

    // Closing while a run is active must fail actionably.
    let (_, close_err, close_code) = popsicle_in(&ws, &home, &["issue", "close", &key]);
    assert_ne!(close_code, 0);
    assert!(close_err.contains("active"), "stderr: {close_err}");

    let next = ok_in(&ws, &home, &["pipeline", "next", "--run", &run_id]);
    assert!(next.contains("next:"));

    let doc = ok_in(
        &ws,
        &home,
        &[
            "doc",
            "create",
            "shadow-implementer",
            "--title",
            "smoke artifact",
            "--run",
            &run_id,
        ],
    );
    let doc_id = field(&doc, "id");
    let doc_path = field(&doc, "file_path");

    // Fresh stub must fail `doc check`, filled doc must pass.
    let (check_out, _, check_code) = popsicle_in(&ws, &home, &["doc", "check", &doc_id]);
    assert_eq!(check_code, 1, "stub doc should fail check:\n{check_out}");
    assert_eq!(field(&check_out, "body_filled"), "false");

    let abs = ws.join(&doc_path);
    let mut content = std::fs::read_to_string(&abs).expect("read doc");
    content.push_str(
        "\n## Fix\n\nReproduced, fixed, regression test added.\n\n- [x] regression test\n",
    );
    std::fs::write(&abs, content).expect("fill doc");
    let check_ok = ok_in(&ws, &home, &["doc", "check", &doc_id]);
    assert_eq!(field(&check_ok, "passed"), "true");
    assert_eq!(field(&check_ok, "checkboxes_checked"), "1");

    // bugfix pipeline: implement → verify, no approvals.
    ok_in(
        &ws,
        &home,
        &[
            "pipeline",
            "stage",
            "complete",
            "implement",
            "--run",
            &run_id,
        ],
    );
    ok_in(
        &ws,
        &home,
        &["pipeline", "stage", "complete", "verify", "--run", &run_id],
    );
    let status = ok_in(&ws, &home, &["pipeline", "status", "--run", &run_id]);
    assert_eq!(field(&status, "run_status"), "completed");

    // Run completed → issue close now succeeds.
    let closed = ok_in(&ws, &home, &["issue", "close", &key]);
    assert_eq!(field(&closed, "issue_status"), "done");

    let _ = std::fs::remove_dir_all(&ws);
    let _ = std::fs::remove_dir_all(&home);
}
