use intent_core::smt::{verify_vc, VerifyResult};
use intent_core::vcgen::{generate_vcs, VcKind};
use intent_syntax::parse;

fn verify_file(source: &str) -> Vec<(String, VcKind, VerifyResult)> {
    let prog = parse(source).expect("parse failed");
    let vcs = generate_vcs(&prog);
    vcs.iter()
        .filter(|vc| vc.unsupported.is_none())
        .map(|vc| {
            let result = verify_vc(vc, &prog);
            (vc.name.clone(), vc.kind, result)
        })
        .collect()
}

#[test]
fn transfer_safe_verified() {
    let source = std::fs::read_to_string("../../examples/basics/transfer.intent").unwrap();
    let results = verify_file(&source);

    let safe = results
        .iter()
        .find(|(n, _, _)| n == "TransferSafe")
        .unwrap();
    assert!(
        matches!(safe.2, VerifyResult::Verified),
        "TransferSafe should verify"
    );
}

#[test]
fn transfer_buggy_fails() {
    let source = std::fs::read_to_string("../../examples/basics/transfer.intent").unwrap();
    let results = verify_file(&source);

    let buggy = results
        .iter()
        .find(|(n, _, _)| n == "TransferBuggy")
        .unwrap();
    assert!(
        matches!(buggy.2, VerifyResult::Failed { .. }),
        "TransferBuggy should fail"
    );
}

#[test]
fn auth_intents_verified() {
    let source = std::fs::read_to_string("../../examples/basics/auth.intent").unwrap();
    let results = verify_file(&source);

    for (name, kind, result) in &results {
        if *kind == VcKind::Intent {
            assert!(
                matches!(result, VerifyResult::Verified),
                "intent {name} should verify (no invariants)"
            );
        }
    }
}

#[test]
fn inline_withdraw_without_guard_fails() {
    let source = r#"
type Account {
  balance: Int
}

intent Withdraw(acc: Account, amount: Int) {
  require amount > 0
  ensure acc.balance' == acc.balance - amount
  invariant acc.balance' >= 0
}
"#;
    let results = verify_file(source);
    let w = results.iter().find(|(n, _, _)| n == "Withdraw").unwrap();
    // Fails: acc.balance = 0, amount = 1 → acc.balance' = -1
    assert!(
        matches!(w.2, VerifyResult::Failed { .. }),
        "Withdraw without balance guard should fail"
    );
}

#[test]
fn inline_withdraw_with_guard_verified() {
    let source = r#"
type Account {
  balance: Int
}

intent Withdraw(acc: Account, amount: Int) {
  require amount > 0
  require acc.balance >= amount
  ensure acc.balance' == acc.balance - amount
  invariant acc.balance' >= 0
}
"#;
    let results = verify_file(source);
    let w = results.iter().find(|(n, _, _)| n == "Withdraw").unwrap();
    assert!(
        matches!(w.2, VerifyResult::Verified),
        "guarded Withdraw should verify"
    );
}
