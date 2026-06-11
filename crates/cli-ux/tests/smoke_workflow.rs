//! End-to-end self-hosting workflow smoke (PDR-002 / T-CU-0008 / PROJ-10).

use std::process::Command;

fn bin() -> &'static str {
    option_env!("CARGO_BIN_EXE_popsicle").unwrap_or("./target/debug/popsicle")
}

fn popsicle(args: &[&str]) -> String {
    let output = Command::new(bin())
        .args(args)
        .output()
        .expect("run popsicle binary");
    assert!(
        output.status.success(),
        "command failed: popsicle {} — stderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn field(stdout: &str, key: &str) -> String {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")).map(str::trim))
        .unwrap_or_else(|| panic!("missing {key} in:\n{stdout}"))
        .to_string()
}

#[test]
fn self_host_workflow_smoke_passes() {
    let doctor = popsicle(&["doctor", "--format", "json"]);
    assert!(doctor.contains("\"current_workspace_binary_match\":\"true\""));
    assert!(doctor.contains("storage_backend"));
    assert!(doctor.contains("PROJ-11"));
    assert!(!doctor.contains("\"json\":"));

    let created = popsicle(&[
        "issue",
        "create",
        "--type",
        "bug",
        "--title",
        "PROJ-10 smoke",
        "--spec",
        "slice-3-cli-ux",
        "--pipeline",
        "tech-decision",
    ]);
    let key = field(&created, "key");

    let started = popsicle(&[
        "issue",
        "start",
        &key,
        "--spec",
        "slice-3-cli-ux",
        "--pipeline",
        "tech-decision",
    ]);
    let run_id = field(&started, "run_id");
    assert_eq!(field(&started, "run_created"), "true");

    let next = popsicle(&["pipeline", "next", "--run", &run_id]);
    assert!(next.contains("next:"));

    popsicle(&[
        "doc",
        "create",
        "shadow-implementer",
        "--title",
        "smoke artifact",
        "--run",
        &run_id,
    ]);

    let stage = field(
        &popsicle(&["pipeline", "status", "--run", &run_id]),
        "current_stage",
    );
    let complete = if next.contains("--confirm") {
        vec![
            "pipeline",
            "stage",
            "complete",
            &stage,
            "--run",
            &run_id,
            "--confirm",
        ]
    } else {
        vec![
            "pipeline",
            "stage",
            "complete",
            &stage,
            "--run",
            &run_id,
        ]
    };
    popsicle(&complete);

    popsicle(&["pipeline", "status", "--run", &run_id]);
    popsicle(&["issue", "show", &key]);
}
